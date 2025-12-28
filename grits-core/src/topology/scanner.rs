use super::{parser::CodeParser, SymbolGraph};
use anyhow::Result;
use glob::Pattern;
use std::path::Path;
use walkdir::WalkDir;

pub struct DirectoryScanner {
    extensions: Vec<String>,
    max_depth: Option<usize>,
    exclude_patterns: Vec<Pattern>,
}

impl DirectoryScanner {
    pub fn new() -> Self {
        Self {
            extensions: vec!["rs".to_string(), "ts".to_string(), "js".to_string()],
            max_depth: None,
            exclude_patterns: Vec::new(),
        }
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_excludes(mut self, excludes: Vec<String>) -> Self {
        self.exclude_patterns = excludes
            .into_iter()
            .filter_map(|s| Pattern::new(&s).ok())
            .collect();
        self
    }

    pub fn scan(&self, dir: &Path) -> Result<SymbolGraph> {
        let mut graph = SymbolGraph::new();
        let mut walker = WalkDir::new(dir);

        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Check extensions
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if !self.extensions.contains(&ext.to_string()) {
                continue;
            }

            // Check excludes
            let relative_path = path.strip_prefix(dir).unwrap_or(path);
            if self
                .exclude_patterns
                .iter()
                .any(|p| p.matches_path(relative_path))
            {
                continue;
            }

            // Parse file
            let lang = match ext {
                "rs" => "rust",
                "ts" => "typescript",
                "js" => "javascript",
                _ => continue,
            };

            if let Ok(content) = std::fs::read_to_string(path) {
                let mut parser = CodeParser::new(lang)?;
                let file_id = relative_path.to_string_lossy().to_string();
                let _ = parser.parse_file(&file_id, &content, &mut graph);
            }
        }

        Ok(graph)
    }
}
