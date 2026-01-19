use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

/// Star neighborhood - all nodes connected to a center node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarNeighborhood {
    pub center: String,
    pub neighbors: Vec<String>,
    pub edges: Vec<(String, String, String)>, // (from, to, relation)
    pub depth: usize,
}

pub fn get_star(graph_data: &SymbolGraph, node_id: &str, depth: usize) -> StarNeighborhood {
    let mut neighbors = HashSet::new();
    let mut edges = Vec::new();
    let mut current_level = HashSet::new();
    current_level.insert(node_id.to_string());
    let mut visited = HashSet::new();

    // Mark start node as visited so we don't include it in neighbors
    visited.insert(node_id.to_string());

    for _ in 0..depth {
        let mut next_level = HashSet::new();

        for current in &current_level {
            for (from, to, edge) in &graph_data.edges {
                if from == current && !visited.contains(to) {
                    neighbors.insert(to.clone());
                    next_level.insert(to.clone());
                    visited.insert(to.clone());
                    edges.push((from.clone(), to.clone(), edge.relation.clone()));
                }
                if to == current && !visited.contains(from) {
                    neighbors.insert(from.clone());
                    next_level.insert(from.clone());
                    visited.insert(from.clone());
                    edges.push((from.clone(), to.clone(), edge.relation.clone()));
                }
            }
        }
        current_level = next_level;
    }

    StarNeighborhood {
        center: node_id.to_string(),
        neighbors: neighbors.into_iter().collect(),
        edges,
        depth,
    }
}

// Keeping parser, scanner, cache, workspace as they are useful for basic graph building
#[cfg(not(target_arch = "wasm32"))]
pub mod parser;

#[cfg(not(target_arch = "wasm32"))]
pub mod scanner;

#[cfg(not(target_arch = "wasm32"))]
pub mod cache;

#[cfg(not(target_arch = "wasm32"))]
pub mod workspace;