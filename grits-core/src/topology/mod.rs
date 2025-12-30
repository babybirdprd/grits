use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: String,
    pub name: String,
    pub file_path: String,
    #[serde(default)]
    pub package: Option<String>, // Workspace package name (monorepo)
    pub language: String,
    pub kind: String, // "function", "class", "struct", etc.
    #[serde(default)]
    pub byte_range: Option<(usize, usize)>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub relation: String, // "calls", "imports", "inherits"
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolGraph {
    pub nodes: HashMap<String, Symbol>, // Map ID to Symbol
    pub edges: Vec<(String, String, DependencyEdge)>, // Source ID, Target ID, Edge Data
}

impl SymbolGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.nodes.insert(symbol.id.clone(), symbol);
    }

    pub fn add_dependency(&mut self, from: &str, to: &str, relation: &str) {
        self.add_weighted_dependency(from, to, relation, 1.0);
    }

    pub fn add_weighted_dependency(&mut self, from: &str, to: &str, relation: &str, weight: f32) {
        self.edges.push((
            from.to_string(),
            to.to_string(),
            DependencyEdge {
                relation: relation.to_string(),
                strength: weight,
            },
        ));
    }
}

pub mod analysis;

#[cfg(not(target_arch = "wasm32"))]
pub mod parser;

#[cfg(not(target_arch = "wasm32"))]
pub mod scanner;

#[cfg(not(target_arch = "wasm32"))]
pub mod cache;

#[cfg(not(target_arch = "wasm32"))]
pub mod incremental;

#[cfg(not(target_arch = "wasm32"))]
pub mod workspace;

#[cfg(not(target_arch = "wasm32"))]
pub mod refactor;
