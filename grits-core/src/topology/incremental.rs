use super::{cache::TopologyCache, parser::CodeParser, scanner::ScanProgress, SymbolGraph};
use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

/// Result of an incremental scan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalScanResult {
    /// Nodes added since last scan
    pub added_nodes: Vec<String>,
    /// Nodes removed since last scan  
    pub removed_nodes: Vec<String>,
    /// Edges that were modified
    pub modified_edges: Vec<(String, String)>,
    /// Number of files re-parsed
    pub files_parsed: usize,
    /// Whether a full rebuild was performed
    pub was_full_rebuild: bool,
}

/// Incremental scanner that only re-parses changed files
pub struct IncrementalScanner {
    extensions: Vec<String>,
}

impl IncrementalScanner {
    pub fn new() -> Self {
        Self {
            extensions: vec![
                "rs".to_string(),
                "ts".to_string(),
                "js".to_string(),
                "py".to_string(),
                "go".to_string(),
            ],
        }
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn scan_incremental<F>(
        &self,
        dir: &Path,
        cache: &mut TopologyCache,
        on_progress: F,
    ) -> Result<IncrementalScanResult>
    where
        F: Fn(ScanProgress) + Sync,
    {
        tracing::info!("Incremental scan starting...");

        // Get current HEAD
        let current_head = match TopologyCache::get_current_head(dir) {
            Ok(h) => {
                tracing::info!("Current HEAD: {}", h);
                h
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to get current HEAD: {:?}. Falling back to full rebuild.",
                    e
                );
                // Not a git repo or error - do full rebuild
                return self.full_rebuild(dir, cache, on_progress);
            }
        };

        // Check if cache is stale
        if !cache.is_stale(&current_head) {
            tracing::info!("Cache is up to date (HEAD: {}).", current_head);
            // Cache is up to date
            return Ok(IncrementalScanResult {
                added_nodes: vec![],
                removed_nodes: vec![],
                modified_edges: vec![],
                files_parsed: 0,
                was_full_rebuild: false,
            });
        }

        tracing::info!(
            "Cache is stale (cached: {:?}, current: {}).",
            cache.git_commit_hash,
            current_head
        );

        // Get changed files
        let changed_files = cache.get_changed_files(dir)?;
        tracing::info!(
            "Changed files from git: {} files found.",
            changed_files.len()
        );

        if changed_files.is_empty() {
            if cache.git_commit_hash.is_none() {
                tracing::info!("No cached hash found. Falling back to full rebuild.");
                return self.full_rebuild(dir, cache, on_progress);
            } else {
                tracing::info!(
                    "No changed files detected by git. Updating hash to {}.",
                    current_head
                );
                cache.git_commit_hash = Some(current_head);
                return Ok(IncrementalScanResult {
                    added_nodes: vec![],
                    removed_nodes: vec![],
                    modified_edges: vec![],
                    files_parsed: 0,
                    was_full_rebuild: false,
                });
            }
        }

        // Filter to only supported extensions
        let relevant_files: Vec<String> = changed_files
            .into_iter()
            .filter(|f| {
                self.extensions
                    .iter()
                    .any(|ext| f.ends_with(&format!(".{}", ext)))
            })
            .collect();

        if relevant_files.is_empty() {
            tracing::info!(
                "No relevant (supported extension) files changed. Updating hash to {}.",
                current_head
            );
            // No relevant files changed, just update the hash
            cache.git_commit_hash = Some(current_head);
            return Ok(IncrementalScanResult {
                added_nodes: vec![],
                removed_nodes: vec![],
                modified_edges: vec![],
                files_parsed: 0,
                was_full_rebuild: false,
            });
        }

        tracing::info!("Re-parsing {} relevant files.", relevant_files.len());

        // Track nodes before update
        let old_nodes: HashSet<String> = cache.graph.nodes.keys().cloned().collect();

        // Remove nodes from changed files
        let files_to_remove: HashSet<String> = relevant_files.iter().cloned().collect();
        cache
            .graph
            .nodes
            .retain(|id, _| !files_to_remove.iter().any(|f| id.starts_with(f)));
        cache.graph.edges.retain(|(from, to, _)| {
            !files_to_remove
                .iter()
                .any(|f| from.starts_with(f) || to.starts_with(f))
        });

        // Parse changed files in parallel using rayon
        let new_symbols: Mutex<Vec<(String, super::Symbol, Vec<(String, String, String)>)>> =
            Mutex::new(Vec::new());
        let total = relevant_files.len();

        relevant_files
            .par_iter()
            .enumerate()
            .for_each(|(i, file_path)| {
                let full_path = dir.join(file_path);
                if !full_path.exists() {
                    return; // File was deleted
                }

                on_progress(ScanProgress {
                    files_scanned: i + 1,
                    total_files: Some(total),
                    current_file: file_path.clone(),
                });

                let ext = full_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();

                let lang = match ext {
                    "rs" => "rust",
                    "ts" => "typescript",
                    "js" => "javascript",
                    "py" => "python",
                    "go" => "go",
                    _ => return,
                };

                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    if let Ok(mut parser) = CodeParser::new(lang) {
                        let mut temp_graph = SymbolGraph::new();
                        let file_id = file_path.replace('\\', "/");
                        if parser
                            .parse_file(&file_id, &content, &mut temp_graph)
                            .is_ok()
                        {
                            let mut results = new_symbols.lock().unwrap();
                            for (id, symbol) in temp_graph.nodes {
                                let edges: Vec<_> = temp_graph
                                    .edges
                                    .iter()
                                    .filter(|(from, _, _)| from == &id)
                                    .map(|(from, to, edge)| {
                                        (from.clone(), to.clone(), edge.relation.clone())
                                    })
                                    .collect();
                                results.push((id, symbol, edges));
                            }
                        }
                    }
                }
            });

        // Merge results into cache graph
        let results = new_symbols.into_inner().unwrap();
        let mut added_nodes = Vec::new();
        let mut modified_edges = Vec::new();

        for (id, symbol, edges) in results {
            if !old_nodes.contains(&id) {
                added_nodes.push(id.clone());
            }
            cache.graph.nodes.insert(id.clone(), symbol);
            for (from, to, relation) in edges {
                modified_edges.push((from.clone(), to.clone()));
                cache.graph.add_dependency(&from, &to, &relation);
            }
        }

        // Find removed nodes
        let new_nodes: HashSet<String> = cache.graph.nodes.keys().cloned().collect();
        let removed_nodes: Vec<String> = old_nodes.difference(&new_nodes).cloned().collect();

        // Update git commit hash
        cache.git_commit_hash = Some(current_head);

        Ok(IncrementalScanResult {
            added_nodes,
            removed_nodes,
            modified_edges,
            files_parsed: total,
            was_full_rebuild: false,
        })
    }

    /// Perform a full rebuild of the topology cache
    fn full_rebuild<F>(
        &self,
        dir: &Path,
        cache: &mut TopologyCache,
        on_progress: F,
    ) -> Result<IncrementalScanResult>
    where
        F: Fn(ScanProgress),
    {
        let _old_node_count = cache.graph.nodes.len();

        let scanner =
            super::scanner::DirectoryScanner::new().with_extensions(self.extensions.clone());

        cache.update_from_dir_with_progress(dir, &scanner, on_progress)?;

        let new_nodes: Vec<String> = cache.graph.nodes.keys().cloned().collect();

        Ok(IncrementalScanResult {
            added_nodes: new_nodes,
            removed_nodes: vec![], // Can't determine on full rebuild
            modified_edges: vec![],
            files_parsed: cache.graph.nodes.len(),
            was_full_rebuild: true,
        })
    }
}

impl Default for IncrementalScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_scan_result_serialization() {
        let result = IncrementalScanResult {
            added_nodes: vec!["foo".to_string()],
            removed_nodes: vec![],
            modified_edges: vec![],
            files_parsed: 1,
            was_full_rebuild: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("foo"));
    }
}
