use super::{parser::CodeParser, SymbolGraph};
use anyhow::Result;
use glob::Pattern;
use std::path::Path;
use walkdir::WalkDir;

/// Default patterns to exclude from scanning (common build/dependency directories)
const DEFAULT_EXCLUDES: &[&str] = &[
    "**/node_modules/**",
    "**/target/**",
    "**/.git/**",
    "**/dist/**",
    "**/build/**",
    "**/*.min.js",
    "**/*.bundle.js",
    "**/.next/**",
    "**/coverage/**",
    "**/vendor/**",
];

/// Progress information during scanning
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub files_scanned: usize,
    pub total_files: Option<usize>,
    pub current_file: String,
}

/// Cache for parsers to avoid re-initialization overhead
struct ParserCache {
    rust: Option<CodeParser>,
    typescript: Option<CodeParser>,
    javascript: Option<CodeParser>,
    python: Option<CodeParser>,
    go: Option<CodeParser>,
}

impl ParserCache {
    fn new() -> Self {
        Self {
            rust: None,
            typescript: None,
            javascript: None,
            python: None,
            go: None,
        }
    }

    fn get_or_create(&mut self, lang: &str) -> Result<&mut CodeParser> {
        match lang {
            "rust" => {
                if self.rust.is_none() {
                    self.rust = Some(CodeParser::new("rust")?);
                }
                Ok(self.rust.as_mut().unwrap())
            }
            "typescript" => {
                if self.typescript.is_none() {
                    self.typescript = Some(CodeParser::new("typescript")?);
                }
                Ok(self.typescript.as_mut().unwrap())
            }
            "javascript" => {
                if self.javascript.is_none() {
                    self.javascript = Some(CodeParser::new("javascript")?);
                }
                Ok(self.javascript.as_mut().unwrap())
            }
            "python" => {
                if self.python.is_none() {
                    self.python = Some(CodeParser::new("python")?);
                }
                Ok(self.python.as_mut().unwrap())
            }
            "go" => {
                if self.go.is_none() {
                    self.go = Some(CodeParser::new("go")?);
                }
                Ok(self.go.as_mut().unwrap())
            }
            _ => Err(anyhow::anyhow!("Unsupported language: {}", lang)),
        }
    }
}

pub struct DirectoryScanner {
    extensions: Vec<String>,
    max_depth: Option<usize>,
    exclude_patterns: Vec<Pattern>,
    use_default_excludes: bool,
}

impl DirectoryScanner {
    pub fn new() -> Self {
        Self {
            extensions: vec![
                "rs".to_string(),
                "ts".to_string(),
                "js".to_string(),
                "py".to_string(),
                "go".to_string(),
            ],
            max_depth: None,
            exclude_patterns: Vec::new(),
            use_default_excludes: true,
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

    pub fn without_default_excludes(mut self) -> Self {
        self.use_default_excludes = false;
        self
    }

    fn get_all_exclude_patterns(&self) -> Vec<Pattern> {
        let mut patterns = self.exclude_patterns.clone();
        if self.use_default_excludes {
            for p in DEFAULT_EXCLUDES {
                if let Ok(pattern) = Pattern::new(p) {
                    patterns.push(pattern);
                }
            }
        }
        patterns
    }

    fn is_excluded(&self, path: &Path, base_dir: &Path, patterns: &[Pattern]) -> bool {
        let relative = path.strip_prefix(base_dir).unwrap_or(path);
        let rel_str = relative.to_string_lossy();

        // Quick path-based checks (faster than glob matching)
        for component in relative.components() {
            let name = component.as_os_str().to_string_lossy();
            if name == "node_modules" || name == "target" || name == ".git" {
                return true;
            }
        }

        // Glob pattern matching
        patterns
            .iter()
            .any(|p| p.matches(&rel_str) || p.matches_path(relative))
    }

    /// Scan a directory with progress callback
    pub fn scan_with_progress<F>(&self, dir: &Path, on_progress: F) -> Result<SymbolGraph>
    where
        F: Fn(ScanProgress),
    {
        let mut graph = SymbolGraph::new();
        let mut cache = ParserCache::new();
        let exclude_patterns = self.get_all_exclude_patterns();

        // First pass: collect all file paths (fast)
        let mut walker = WalkDir::new(dir);
        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        let files: Vec<_> = walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                let path = entry.path();
                if !path.is_file() {
                    return false;
                }

                // Check extension
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                if !self.extensions.contains(&ext.to_string()) {
                    return false;
                }

                // Check excludes
                !self.is_excluded(path, dir, &exclude_patterns)
            })
            .collect();

        let total = files.len();

        // Second pass: parse files (slower, but with progress)
        for (i, entry) in files.iter().enumerate() {
            let path = entry.path();
            let relative_path = path.strip_prefix(dir).unwrap_or(path);

            on_progress(ScanProgress {
                files_scanned: i + 1,
                total_files: Some(total),
                current_file: relative_path.to_string_lossy().to_string(),
            });

            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let lang = match ext {
                "rs" => "rust",
                "ts" => "typescript",
                "js" => "javascript",
                "py" => "python",
                "go" => "go",
                _ => continue,
            };

            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(parser) = cache.get_or_create(lang) {
                    // Normalize to forward slashes for cross-platform compatibility
                    let file_id = relative_path.to_string_lossy().replace('\\', "/");
                    let _ = parser.parse_file(&file_id, &content, &mut graph);
                }
            }
        }

        Ok(graph)
    }

    /// Original scan method (for backwards compatibility)
    pub fn scan(&self, dir: &Path) -> Result<SymbolGraph> {
        self.scan_with_progress(dir, |_| {})
    }
}
