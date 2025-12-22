use crate::fs::FileSystem;
use crate::merge::merge3way;
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

    Ok(())
}
