use super::{
    scanner::{DirectoryScanner, ScanProgress},
    SymbolGraph,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct TopologyCache {
    pub graph: SymbolGraph,
    pub file_hashes: HashMap<String, String>, // Path -> content hash
    /// Git commit hash when this cache was last built
    #[serde(default)]
    pub git_commit_hash: Option<String>,
}

impl TopologyCache {
    pub fn new() -> Self {
        Self {
            graph: SymbolGraph::new(),
            file_hashes: HashMap::new(),
            git_commit_hash: None,
        }
    }

    pub fn from_graph(graph: SymbolGraph) -> Self {
        Self {
            graph,
            file_hashes: HashMap::new(),
            git_commit_hash: None,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let cache = serde_json::from_str(&content)?;
        Ok(cache)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Check if the cache is stale compared to the current git HEAD
    pub fn is_stale(&self, current_head: &str) -> bool {
        match &self.git_commit_hash {
            Some(cached_hash) => cached_hash != current_head,
            None => true, // No cached hash means stale
        }
    }

    /// Get the list of files changed since the cached commit
    pub fn get_changed_files(&self, repo_root: &Path) -> Result<Vec<String>> {
        let cached_hash = match &self.git_commit_hash {
            Some(h) => h.clone(),
            None => return Ok(Vec::new()), // Full rebuild needed
        };

        // Run git diff to get changed files
        let output = std::process::Command::new("git")
            .args(["diff", "--name-only", &cached_hash, "HEAD"])
            .current_dir(repo_root)
            .output()?;

        if !output.status.success() {
            // If git fails (e.g., commit doesn't exist), full rebuild
            return Ok(Vec::new());
        }

        let files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
        Ok(files)
    }

    /// Get current git HEAD commit hash
    pub fn get_current_head(repo_root: &Path) -> Result<String> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_root)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to get git HEAD");
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn update_from_dir(&mut self, dir: &Path, scanner: &DirectoryScanner) -> Result<()> {
        let new_graph = scanner.scan(dir)?;
        self.graph = new_graph;
        // Update git commit hash
        if let Ok(head) = Self::get_current_head(dir) {
            self.git_commit_hash = Some(head);
        }
        Ok(())
    }

    /// Update from directory with progress callback
    pub fn update_from_dir_with_progress<F>(
        &mut self,
        dir: &Path,
        scanner: &DirectoryScanner,
        on_progress: F,
    ) -> Result<()>
    where
        F: Fn(ScanProgress),
    {
        let new_graph = scanner.scan_with_progress(dir, on_progress)?;
        self.graph = new_graph;
        // Update git commit hash
        if let Ok(head) = Self::get_current_head(dir) {
            self.git_commit_hash = Some(head);
        }
        Ok(())
    }

    pub fn hash_file(path: &Path) -> Result<String> {
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(content);
        Ok(hex::encode(hasher.finalize()))
    }
}
