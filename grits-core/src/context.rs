//! Mini Codebase / Context Assembly
//!
//! "Semantic Tree-Shaking" for AI agents: extract only the topologically-relevant
//! code from a large repository into a minimal, focused context.
//!
//! ## Concept
//! When an agent needs context for an issue, we don't give it whole files.
//! Instead, we use the Star Neighborhood (topology) to identify exactly which
//! symbols are relevant, then extract only those code blocks.
//!
//! ## Usage
//! ```ignore
//! gr context assemble --issue <ID> --format mini-repo
//! ```

use crate::topology::{analysis::TopologicalAnalysis, SymbolGraph};
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

    /// PageRank score (importance)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagerank: Option<f32>,

    /// Whether this symbol is in a cycle
    #[serde(default)]
    pub in_cycle: bool,
}

/// Architectural invariants to preserve
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextInvariants {
    /// Number of cycles (Betti_1) - should not increase
    pub betti_1: usize,

    /// Symbols that should NOT be called/imported
    pub forbidden_dependencies: Vec<String>,

    /// Layer violations to avoid
    pub layer_constraints: Vec<String>,

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

    /// Strength threshold for inclusion
    pub strength_threshold: f32,

    /// Number of symbols in full graph
    pub total_symbols_in_graph: usize,

    /// Solid Score at time of extraction
    pub solid_score: f32,
}

impl MiniCodebase {
    /// Assemble a mini codebase from seed symbols
    pub fn assemble(
        graph: &SymbolGraph,
        seed_symbols: Vec<String>,
        depth: usize,
        strength_threshold: f32,
        issue_id: Option<String>,
    ) -> Self {
        // Collect all relevant symbol IDs using star neighborhood
        let mut relevant_ids = HashSet::new();
        let mut all_edges = Vec::new();

        for seed in &seed_symbols {
            let star = TopologicalAnalysis::get_star(graph, seed, depth);
            relevant_ids.insert(star.center.clone());
            for neighbor in star.neighbors {
                relevant_ids.insert(neighbor);
            }
            all_edges.extend(star.edges);
        }

        // Compute PageRank for importance ranking
        let pagerank = TopologicalAnalysis::weighted_pagerank(graph, 0.85, 20);

        // Detect cycle membership via triangles (nodes in triangles are tightly coupled)
        let analysis = TopologicalAnalysis::analyze(graph);
        let cycle_nodes: HashSet<String> = analysis
            .triangles
            .iter()
            .flat_map(|tri| tri.nodes.iter().cloned())
            .collect();

        // Build symbol entries
        let mut symbols: Vec<SymbolEntry> = Vec::new();
        let mut files = HashSet::new();

        for id in &relevant_ids {
            if let Some(symbol) = graph.nodes.get(id) {
                // Check if this symbol meets the strength threshold
                let rank = pagerank.get(id).copied().unwrap_or(0.0);

                // Include if it's a seed or meets threshold
                let is_seed = seed_symbols.contains(id);
                if is_seed || rank >= strength_threshold {
                    files.insert(symbol.file_path.clone());

                    symbols.push(SymbolEntry {
                        id: id.clone(),
                        name: symbol.name.clone(),
                        file_path: symbol.file_path.clone(),
                        kind: symbol.kind.clone(),
                        code: None, // To be filled by hydrate_code
                        byte_range: symbol.byte_range,
                        pagerank: Some(rank),
                        in_cycle: cycle_nodes.contains(id),
                    });
                }
            }
        }

        // Sort by PageRank (most important first)
        symbols.sort_by(|a, b| {
            b.pagerank
                .unwrap_or(0.0)
                .partial_cmp(&a.pagerank.unwrap_or(0.0))
                .unwrap()
        });

        // Build invariants
        let invariants = ContextInvariants {
            betti_1: analysis.betti_1,
            forbidden_dependencies: Vec::new(), // TODO: Extract from layer analysis
            layer_constraints: Vec::new(),
            notes: if analysis.betti_1 > 0 {
                vec![format!(
                    "⚠️ {} cycles detected. Avoid adding new dependencies that create cycles.",
                    analysis.betti_1
                )]
            } else {
                vec![
                    "✓ No cycles. Maintain this by following unidirectional dependencies."
                        .to_string(),
                ]
            },
        };

        // Build metadata
        let solid_score = analysis.solid_score();
        let metadata = ContextMetadata {
            assembled_at: chrono::Utc::now().to_rfc3339(),
            neighborhood_depth: depth,
            strength_threshold,
            total_symbols_in_graph: graph.nodes.len(),
            solid_score: solid_score.normalized,
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

                // Extract substring based on byte indices
                // Safety: We use byte ranges from tree-sitter, which are valid byte offsets.
                // However, we should be careful with UTF-8 boundaries if we were using character offsets.
                // tree_sitter::Node::byte_range() returns valid byte offsets.
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
            md.push_str(&format!("> Assembled for issue: `{}`\n\n", issue));
        }

        // Stats
        md.push_str(&format!(
            "**Symbols**: {} | **Files**: {} | **Solid Score**: {:.0}%\n\n",
            self.symbols.len(),
            self.files.len(),
            self.metadata.solid_score * 100.0
        ));

        // Invariants
        md.push_str("## Architectural Invariants\n\n");
        for note in &self.invariants.notes {
            md.push_str(&format!("- {}\n", note));
        }
        if self.invariants.betti_1 > 0 {
            md.push_str(&format!(
                "- Betti₁ = {} (cycles to avoid increasing)\n",
                self.invariants.betti_1
            ));
        }
        md.push('\n');

        // Files
        md.push_str("## Files\n\n");
        for file in &self.files {
            md.push_str(&format!("- `{}`\n", file));
        }
        md.push('\n');

        // Symbols
        md.push_str("## Symbols\n\n");
        for symbol in &self.symbols {
            let cycle_marker = if symbol.in_cycle { " ⚠️ CYCLE" } else { "" };
            let rank = symbol
                .pagerank
                .map(|r| format!(" (rank: {:.2})", r))
                .unwrap_or_default();

            md.push_str(&format!("### `{}`{}{}\n\n", symbol.id, cycle_marker, rank));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::Symbol;
    use std::collections::HashMap;

    #[test]
    fn test_mini_codebase_assembly() {
        let mut graph = SymbolGraph::new();

        // Add some test symbols
        graph.add_symbol(Symbol {
            id: "main.rs::main".to_string(),
            name: "main".to_string(),
            file_path: "main.rs".to_string(),
            package: None,
            language: "rust".to_string(),
            kind: "function".to_string(),
            byte_range: None,
            metadata: HashMap::new(),
        });

        graph.add_symbol(Symbol {
            id: "store.rs::Store".to_string(),
            name: "Store".to_string(),
            file_path: "store.rs".to_string(),
            package: None,
            language: "rust".to_string(),
            kind: "struct".to_string(),
            byte_range: None,
            metadata: HashMap::new(),
        });

        graph.add_dependency("main.rs::main", "store.rs::Store", "calls");

        let mini = MiniCodebase::assemble(
            &graph,
            vec!["main.rs::main".to_string()],
            1,
            0.0,
            Some("test-issue".to_string()),
        );

        assert_eq!(mini.symbols.len(), 2);
        assert!(mini.files.contains(&"main.rs".to_string()));
        assert!(mini.files.contains(&"store.rs".to_string()));
    }
}
