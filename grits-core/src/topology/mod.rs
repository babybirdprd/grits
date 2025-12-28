use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub language: String,
    pub kind: String, // "function", "class", "struct", etc.
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
        self.edges.push((
            from.to_string(),
            to.to_string(),
            DependencyEdge {
                relation: relation.to_string(),
                strength: 1.0,
            },
        ));
    }
}

pub mod analysis;

#[cfg(not(target_arch = "wasm32"))]
pub mod parser;
