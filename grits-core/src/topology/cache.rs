use super::{scanner::DirectoryScanner, SymbolGraph};
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
}

impl TopologyCache {
    pub fn new() -> Self {
        Self {
            graph: SymbolGraph::new(),
            file_hashes: HashMap::new(),
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

    pub fn update_from_dir(&mut self, dir: &Path, scanner: &DirectoryScanner) -> Result<()> {
        let new_graph = scanner.scan(dir)?;

        // In Phase 4 we will implement incremental updates here.
        self.graph = new_graph;

        Ok(())
    }

    pub fn hash_file(path: &Path) -> Result<String> {
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(content);
        Ok(hex::encode(hasher.finalize()))
    }
}
