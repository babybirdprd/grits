use crate::fs::FileSystem;
use crate::merge::merge3way;
#[cfg(not(target_arch = "wasm32"))]
use crate::topology::{SymbolGraph, parser::CodeParser};
use crate::{GitOps, Store};
use anyhow::{bail, Context, Result};
use std::path::Path;

pub fn run_sync(
    store: &mut impl Store,
    git: &impl GitOps,
    git_root: &Path,
    jsonl_path: &Path,
    fs: &impl FileSystem,
    squash: bool,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        // 1. Export to temp file
        let temp_dir = std::env::temp_dir();
        let temp_jsonl = temp_dir.join("issues.jsonl.dry-run");
        store
            .export_to_jsonl(&temp_jsonl, fs)
            .context("Dry-run export failed")?;

        tracing::info!("Dry run: Exported pending changes to {:?}", temp_jsonl);
        tracing::info!("Dry run: Git operations and import skipped.");
        return Ok(());
    }

    // 1. Export
    store
        .export_to_jsonl(jsonl_path, fs)
        .context("Export failed")?;

    if squash {
        tracing::info!("Squash mode: Changes exported to JSONL. Git operations skipped.");
        return Ok(());
    }

    // 2. Git Add
    git.add(jsonl_path).context("Git add failed")?;

    // 3. Git Commit
    // Check if changes to commit? git commit will fail/no-op if clean.
    // Our Git wrapper returns Ok if clean.
    git.commit("sync: update issues")
        .context("Git commit failed")?;

    // 4. Pull Rebase
    if git.has_remote()? {
        if let Err(e) = git.pull_rebase() {
            // Check for conflict
            let status = git.status()?;

            // Calculate relative path for git show
            let rel_path = match jsonl_path.strip_prefix(git_root) {
                // Git expects forward slashes even on Windows
                Ok(p) => p.to_string_lossy().replace('\\', "/"),
                Err(_) => {
                    // If jsonl_path is not under git_root, we can't easily resolve the git path.
                    // Assuming jsonl_path is absolute or relative to CWD, and git_root is also.
                    // If they are separate trees, we have a problem.
                    // Let's try to canonicalize both first?
                    // For now, if strip_prefix fails, we assume the user provided path is what git knows
                    // if they are running from git root.
                    // But safer is to bail or try just the filename if it's in root.
                    bail!(
                        "Could not determine relative path of {:?} to git root {:?}",
                        jsonl_path,
                        git_root
                    );
                }
            };

            // Check if issues.jsonl is in conflict (UU = both modified)
            if status.contains("UU") && status.contains(&rel_path) {
                tracing::info!("Conflict detected on {}. Attempting merge...", rel_path);

                let base_content = git.show(&format!(":1:{}", rel_path))?;
                let left_content = git.show(&format!(":2:{}", rel_path))?;
                let right_content = git.show(&format!(":3:{}", rel_path))?;

                // Write to temp files
                let temp_dir = std::env::temp_dir();
                let base_path = temp_dir.join("base.jsonl");
                let left_path = temp_dir.join("left.jsonl");
                let right_path = temp_dir.join("right.jsonl");

                fs.write(&base_path, base_content.as_bytes())?;
                fs.write(&left_path, left_content.as_bytes())?;
                fs.write(&right_path, right_content.as_bytes())?;

                // Run merge
                // Output directly to jsonl_path (overwriting conflict markers)
                merge3way(
                    jsonl_path.to_str().unwrap(),
                    base_path.to_str().unwrap(),
                    left_path.to_str().unwrap(),
                    right_path.to_str().unwrap(),
                    false,
                    fs,
                )?;

                // Add and continue
                git.add(jsonl_path)?;
                git.rebase_continue()?;

                tracing::info!("Merge resolved.");
            } else {
                // Re-throw original error if not our specific conflict
                return Err(e);
            }
        }
    }

    // 5. Push
    if git.has_remote()? {
        git.push().context("Git push failed")?;
    }

    // 6. Import changes from JSONL back to DB
    store
        .import_from_jsonl(jsonl_path, fs)
        .context("Import failed")?;

    // 7. Topology Update (Solid Graph)
    // We run this after import so we have the latest issues.
    #[cfg(not(target_arch = "wasm32"))]
    {
        update_topology(store, git_root)?;
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn update_topology(store: &mut impl Store, git_root: &Path) -> Result<()> {
    tracing::info!("Updating code topology...");
    // 1. Scan files
    // For simplicity, we scan all tracked files (or a subset)
    // In a real incremental system, we'd use git diff or dirty_issues
    // For this implementation, let's just scan files referenced by issues or all small source files
    // But since we want "affected_symbols", we should scan everything or at least what issues touch.
    // Let's iterate issues, see if they claim files (via frontmatter or references?)
    // The requirement says "Issue is a Simplex... that encapsulates a specific subgraph".
    // Currently, Issue doesn't explicitly list files except maybe in description.
    // BUT, we added `affected_symbols`.
    // Let's assume we want to populate `affected_symbols` and `solid_volume` for all issues.
    // Wait, how does an issue KNOW which files it owns?
    // "An Issue is... a Simplex (a volume) that encapsulates a specific subgraph of code."
    // This implies we need to infer it or the user sets it.
    // If the user didn't set it, maybe we use "Context" tools to find related files?
    // For this "Solid Graph" MVP, let's focus on calculating the global graph and checking if issues define a volume.
    // If an issue has no volume, we try to find one?
    // Or maybe we just build the graph of the whole repo so `validate_architectural_change` is fast?
    // The prompt says: "Hook SymbolGraph updates into sync.rs (update graph when files change)."

    // Let's build a global graph of the repo.
    let mut graph = SymbolGraph::new();

    // Walk directory
    for entry in walkdir::WalkDir::new(git_root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let lang = match ext {
                "rs" => "rust",
                "ts" | "tsx" => "typescript",
                _ => continue,
            };

            if let Ok(content) = std::fs::read_to_string(path) {
                // Determine relative path for ID
                let rel_path = path.strip_prefix(git_root).unwrap_or(path).to_string_lossy();
                if let Ok(mut parser) = CodeParser::new(lang) {
                    let _ = parser.parse_file(&rel_path, &content, &mut graph);
                }
            }
        }
    }

    // Now we have a global graph.
    // We should update issues that claim to cover certain symbols or files.
    // If an issue has `affected_symbols`, we can compute its `solid_volume`.
    // If it has none, we leave it.

    let issues = store.list_issues(None, None, None, None, None, None)?;
    for mut issue in issues {
        if !issue.affected_symbols.is_empty() {
             // Build subgraph
             // Simplified: just include the nodes and edges between them?
             // Or include neighbors?
             // "encapsulates a specific subgraph"
             // Let's take the induced subgraph on affected_symbols + immediate neighbors
             // For now, strict induced subgraph on affected_symbols to avoid explosion

             let mut sub_nodes = std::collections::HashMap::new();
             let mut sub_edges = Vec::new();

             for sym_id in &issue.affected_symbols {
                 if let Some(sym) = graph.nodes.get(sym_id) {
                     sub_nodes.insert(sym_id.clone(), sym.clone());
                 }
             }

             for (from, to, edge) in &graph.edges {
                 if sub_nodes.contains_key(from) && sub_nodes.contains_key(to) {
                     sub_edges.push((from.clone(), to.clone(), edge.clone()));
                 }
             }

             let volume = SymbolGraph {
                 nodes: sub_nodes,
                 edges: sub_edges,
             };

             if let Ok(json) = serde_json::to_string(&volume) {
                 issue.solid_volume = Some(json);
                 issue.is_solid = true;
                 let _ = store.update_issue(&issue);
             }
        }
    }

    Ok(())
}
