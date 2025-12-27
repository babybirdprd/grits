use super::SymbolGraph;
use petgraph::graph::DiGraph;
use petgraph::algo::connected_components;

pub struct TopologicalAnalysis {
    pub betti_0: usize, // Connected components
    pub betti_1: usize, // Cycles (approximate via Euler characteristic)
    pub solid_volume: usize, // Filled volumes (3-cliques)
}

impl TopologicalAnalysis {
    pub fn analyze(graph_data: &SymbolGraph) -> Self {
        let mut graph = DiGraph::<(), ()>::new();
        let mut node_indices = std::collections::HashMap::new();

        // Iterate edges first to find ALL nodes (even implicit ones not in graph_data.nodes)
        // Then populate map
        let mut all_node_ids = std::collections::HashSet::new();
        for (id, _) in &graph_data.nodes {
            all_node_ids.insert(id.clone());
        }
        for (from, to, _) in &graph_data.edges {
            all_node_ids.insert(from.clone());
            all_node_ids.insert(to.clone());
        }

        for id in all_node_ids {
             let idx = graph.add_node(());
             node_indices.insert(id, idx);
        }

        for (from, to, _) in &graph_data.edges {
            if let (Some(&from_idx), Some(&to_idx)) = (node_indices.get(from), node_indices.get(to)) {
                graph.add_edge(from_idx, to_idx, ());
            }
        }

        let components = connected_components(&graph);
        let nodes = graph.node_count();
        let edges = graph.edge_count();

        // Euler characteristic for 1-complex (graph): X = V - E
        // Betti_0 = components
        // Betti_1 = E - V + components

        // We use usize, so we need to be careful with subtraction
        let cycles = if edges + components >= nodes {
            edges + components - nodes
        } else {
            0
        };

        // Simplified volume calculation (stub for now, as finding all 3-cliques is expensive)
        // For now, we assume 0 volume unless we implement clique finding
        let solid_volume = 0;

        Self {
            betti_0: components,
            betti_1: cycles,
            solid_volume,
        }
    }
}
