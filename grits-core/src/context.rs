//! Mini Codebase / Context Assembly
//!
//! "Semantic Tree-Shaking" for AI agents: extract only the topologically-relevant
//! code from a large repository into a minimal, focused context.

use crate::topology::{get_star, SymbolGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A mini codebase is a "Simplicial Shadow Workspace" - the distilled essence
/// of the repository tailored for a specific task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniCodebase {
    /// The issue or task this context was assembled for
    pub seed_issue: Option<String>,

    /// The seed symbols that started the expansion
    pub seed_symbols: Vec<String>,

    /// Extracted symbol entries with their code
    pub symbols: Vec<SymbolEntry>,

    /// Files involved (unique, for reference)
    pub files: Vec<String>,

    /// Architectural invariants and constraints
    pub invariants: ContextInvariants,

    /// Metadata about the extraction
    pub metadata: ContextMetadata,
}

/// An extracted symbol with its source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    /// Fully qualified symbol ID (e.g., "store.rs::SqliteStore")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// File path where this symbol lives
    pub file_path: String,

    /// Symbol kind (function, struct, class, etc.)
    pub kind: String,

    /// The actual source code (if extracted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Byte range in the file (start, end)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_range: Option<(usize, usize)>,
}

/// Architectural invariants to preserve
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextInvariants {
    /// Symbols that should NOT be called/imported
    pub forbidden_dependencies: Vec<String>,

    /// Free-form architectural notes
    pub notes: Vec<String>,
}

/// Metadata about the context extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// When this context was assembled
    pub assembled_at: String,

    /// Depth of star neighborhood used
    pub neighborhood_depth: usize,

    /// Number of symbols in full graph
    pub total_symbols_in_graph: usize,
}

impl MiniCodebase {
    /// Assemble a mini codebase from seed symbols
    pub fn assemble(
        graph: &SymbolGraph,
        seed_symbols: Vec<String>,
        depth: usize,
        _strength_threshold: f32, // Unused now
        issue_id: Option<String>,
    ) -> Self {
        // Collect all relevant symbol IDs using star neighborhood
        let mut relevant_ids = HashSet::new();

        for seed in &seed_symbols {
            let star = get_star(graph, seed, depth);
            relevant_ids.insert(star.center.clone());
            for neighbor in star.neighbors {
                relevant_ids.insert(neighbor);
            }
        }

        // Build symbol entries
        let mut symbols: Vec<SymbolEntry> = Vec::new();
        let mut files = HashSet::new();

        for id in &relevant_ids {
            if let Some(symbol) = graph.nodes.get(id) {
                files.insert(symbol.file_path.clone());

                symbols.push(SymbolEntry {
                    id: id.clone(),
                    name: symbol.name.clone(),
                    file_path: symbol.file_path.clone(),
                    kind: symbol.kind.clone(),
                    code: None, // To be filled by hydrate_code
                    byte_range: symbol.byte_range,
                });
            }
        }

        // Simple sort by file path then ID
        symbols.sort_by(|a, b| {
            a.file_path.cmp(&b.file_path).then(a.id.cmp(&b.id))
        });

        // Build invariants
        let invariants = ContextInvariants {
            forbidden_dependencies: Vec::new(),
            notes: vec![
                "Focus on the provided symbols.".to_string(),
            ],
        };

        // Build metadata
        let metadata = ContextMetadata {
            assembled_at: chrono::Utc::now().to_rfc3339(),
            neighborhood_depth: depth,
            total_symbols_in_graph: graph.nodes.len(),
        };

        MiniCodebase {
            seed_issue: issue_id,
            seed_symbols,
            symbols,
            files: files.into_iter().collect(),
            invariants,
            metadata,
        }
    }

    /// Extract code snippets for symbols using their byte ranges
    pub fn hydrate_code(&mut self, base_path: &std::path::Path) {
        use std::collections::HashMap;
        use std::fs;

        let mut file_cache: HashMap<String, String> = HashMap::new();

        for symbol in &mut self.symbols {
            if let Some((start, end)) = symbol.byte_range {
                let content = if let Some(c) = file_cache.get(&symbol.file_path) {
                    c
                } else {
                    let full_path = base_path.join(&symbol.file_path);
                    if let Ok(c) = fs::read_to_string(full_path) {
                        file_cache.insert(symbol.file_path.clone(), c);
                        file_cache.get(&symbol.file_path).unwrap()
                    } else {
                        continue;
                    }
                };

                if start < content.len() && end <= content.len() && start <= end {
                    symbol.code = Some(content[start..end].to_string());
                }
            }
        }
    }

    /// Render as Markdown for agent consumption
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Header
        md.push_str("# Mini Codebase\n\n");

        if let Some(issue) = &self.seed_issue {
            md.push_str(&format!( "> Assembled for issue: `{}`\n\n", issue));
        }

        // Stats
        md.push_str(&format!(
            "**Symbols**: {} | **Files**: {}\n\n",
            self.symbols.len(),
            self.files.len()
        ));

        // Files
        md.push_str("## Files\n\n");
        for file in &self.files {
            md.push_str(&format!("- `{}`\n", file));
        }
        md.push('\n');

        // Symbols
        md.push_str("## Symbols\n\n");
        for symbol in &self.symbols {
            md.push_str(&format!("### `{}`\n\n", symbol.id));
            md.push_str(&format!("- **File**: `{}`\n", symbol.file_path));
            md.push_str(&format!("- **Kind**: {}\n", symbol.kind));

            if let Some(code) = &symbol.code {
                md.push_str("\n```\n");
                md.push_str(code);
                md.push_str("\n```\n");
            }
            md.push('\n');
        }

        md
    }
}