use super::SymbolGraph;
use petgraph::algo::connected_components;
use petgraph::graph::{DiGraph, NodeIndex, UnGraph};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Full topological analysis including simplicial complex features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologicalAnalysis {
    pub betti_0: usize, // Connected components
    pub betti_1: usize, // 1-cycles (circular dependencies)
    pub betti_2: usize, // 2-voids (higher dimensional holes)
    pub node_count: usize,
    pub edge_count: usize,
    pub triangle_count: usize,    // 2-simplexes (filled triangles)
    pub triangles: Vec<Triangle>, // All detected 3-cliques
    pub feature_volumes: Vec<FeatureVolume>, // Grouped cliques as "solid" regions
}

/// A 2-simplex (triangle) in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triangle {
    pub nodes: [String; 3],
    pub edge_types: [String; 3], // "calls", "imports", etc.
}

/// A feature volume is a connected set of triangles (a "solid" region)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVolume {
    pub id: String,
    pub nodes: Vec<String>,
    pub cohesion_score: f32, // How tightly coupled (0.0 - 1.0)
}

/// Star neighborhood - all nodes connected to a center node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarNeighborhood {
    pub center: String,
    pub neighbors: Vec<String>,
    pub edges: Vec<(String, String, String)>, // (from, to, relation)
    pub depth: usize,
}

impl TopologicalAnalysis {
    /// Perform full simplicial complex analysis on the graph
    pub fn analyze(graph_data: &SymbolGraph) -> Self {
        // Build petgraph structures
        let mut digraph = DiGraph::<String, String>::new();
        let mut undirected = UnGraph::<String, ()>::new_undirected();
        let mut node_to_di: HashMap<String, NodeIndex> = HashMap::new();
        let mut node_to_un: HashMap<String, NodeIndex> = HashMap::new();

        // Collect all node IDs (including implicit ones from edges)
        let mut all_node_ids = HashSet::new();
        for (id, _) in &graph_data.nodes {
            all_node_ids.insert(id.clone());
        }
        for (from, to, _) in &graph_data.edges {
            all_node_ids.insert(from.clone());
            all_node_ids.insert(to.clone());
        }

        // Add nodes to both graphs
        for id in &all_node_ids {
            let di_idx = digraph.add_node(id.clone());
            let un_idx = undirected.add_node(id.clone());
            node_to_di.insert(id.clone(), di_idx);
            node_to_un.insert(id.clone(), un_idx);
        }

        // Add edges
        let mut edge_map: HashMap<(String, String), String> = HashMap::new();
        for (from, to, edge) in &graph_data.edges {
            if let (Some(&from_di), Some(&to_di)) = (node_to_di.get(from), node_to_di.get(to)) {
                digraph.add_edge(from_di, to_di, edge.relation.clone());
            }
            if let (Some(&from_un), Some(&to_un)) = (node_to_un.get(from), node_to_un.get(to)) {
                // Only add edge once for undirected
                if !undirected.find_edge(from_un, to_un).is_some() {
                    undirected.add_edge(from_un, to_un, ());
                }
            }
            edge_map.insert((from.clone(), to.clone()), edge.relation.clone());
            edge_map.insert((to.clone(), from.clone()), edge.relation.clone());
        }

        // Calculate Betti_0 (connected components)
        let betti_0 = connected_components(&digraph);

        // Find all triangles (3-cliques) for 2-simplexes
        let triangles = Self::find_triangles(&undirected, &node_to_un, &edge_map);
        let triangle_count = triangles.len();

        // Calculate Betti numbers using Euler characteristic
        // For a simplicial complex: χ = V - E + F (faces/triangles)
        // Betti_0 = components
        // Betti_1 = E - V + Betti_0 - triangles (adjusted for filled triangles)
        // Betti_2 = triangles that form "voids" (simplified: 0 for now)

        let node_count = all_node_ids.len();
        let edge_count = digraph.edge_count();

        let betti_1 = if edge_count + betti_0 >= node_count + triangle_count {
            edge_count + betti_0 - node_count - triangle_count
        } else {
            0
        };

        // Betti_2 would require finding tetrahedra (4-cliques) which is expensive
        // For now, we approximate based on triangle clustering
        let betti_2 = 0;

        // Group triangles into feature volumes
        let feature_volumes = Self::compute_feature_volumes(&triangles, graph_data);

        Self {
            betti_0,
            betti_1,
            betti_2,
            node_count,
            edge_count,
            triangle_count,
            triangles,
            feature_volumes,
        }
    }

    /// Find all triangles (3-cliques) in an undirected graph
    fn find_triangles(
        graph: &UnGraph<String, ()>,
        node_map: &HashMap<String, NodeIndex>,
        edge_map: &HashMap<(String, String), String>,
    ) -> Vec<Triangle> {
        let mut triangles = Vec::new();
        let mut seen = HashSet::new();

        // Get reverse map
        let idx_to_node: HashMap<NodeIndex, String> =
            node_map.iter().map(|(k, v)| (*v, k.clone())).collect();

        // For each edge (u, v), find common neighbors
        for edge in graph.edge_references() {
            let u = edge.source();
            let v = edge.target();

            let u_neighbors: HashSet<NodeIndex> = graph.neighbors(u).collect();
            let v_neighbors: HashSet<NodeIndex> = graph.neighbors(v).collect();

            // Common neighbors form triangles with u and v
            for &w in u_neighbors.intersection(&v_neighbors) {
                let mut nodes = [
                    idx_to_node.get(&u).cloned().unwrap_or_default(),
                    idx_to_node.get(&v).cloned().unwrap_or_default(),
                    idx_to_node.get(&w).cloned().unwrap_or_default(),
                ];
                nodes.sort();

                let key = format!("{}-{}-{}", nodes[0], nodes[1], nodes[2]);
                if !seen.contains(&key) {
                    seen.insert(key);

                    let edge_types = [
                        edge_map
                            .get(&(nodes[0].clone(), nodes[1].clone()))
                            .cloned()
                            .unwrap_or_else(|| "unknown".to_string()),
                        edge_map
                            .get(&(nodes[1].clone(), nodes[2].clone()))
                            .cloned()
                            .unwrap_or_else(|| "unknown".to_string()),
                        edge_map
                            .get(&(nodes[0].clone(), nodes[2].clone()))
                            .cloned()
                            .unwrap_or_else(|| "unknown".to_string()),
                    ];

                    triangles.push(Triangle { nodes, edge_types });
                }
            }
        }

        triangles
    }

    /// Group triangles into feature volumes (connected clique regions)
    fn compute_feature_volumes(triangles: &[Triangle], _graph: &SymbolGraph) -> Vec<FeatureVolume> {
        if triangles.is_empty() {
            return Vec::new();
        }

        // Build a graph of triangles (connected if they share an edge)
        let mut triangle_graph: HashMap<usize, HashSet<usize>> = HashMap::new();

        for i in 0..triangles.len() {
            triangle_graph.insert(i, HashSet::new());
        }

        for i in 0..triangles.len() {
            for j in (i + 1)..triangles.len() {
                // Two triangles share an edge if they have 2 common nodes
                let nodes_i: HashSet<_> = triangles[i].nodes.iter().collect();
                let nodes_j: HashSet<_> = triangles[j].nodes.iter().collect();
                let common: Vec<_> = nodes_i.intersection(&nodes_j).collect();

                if common.len() >= 2 {
                    triangle_graph.get_mut(&i).unwrap().insert(j);
                    triangle_graph.get_mut(&j).unwrap().insert(i);
                }
            }
        }

        // Find connected components of triangles
        let mut visited = HashSet::new();
        let mut volumes = Vec::new();

        for start in 0..triangles.len() {
            if visited.contains(&start) {
                continue;
            }

            let mut component = Vec::new();
            let mut stack = vec![start];

            while let Some(idx) = stack.pop() {
                if visited.contains(&idx) {
                    continue;
                }
                visited.insert(idx);
                component.push(idx);

                if let Some(neighbors) = triangle_graph.get(&idx) {
                    for &neighbor in neighbors {
                        if !visited.contains(&neighbor) {
                            stack.push(neighbor);
                        }
                    }
                }
            }

            // Collect all unique nodes in this volume
            let mut volume_nodes: HashSet<String> = HashSet::new();
            for &tri_idx in &component {
                for node in &triangles[tri_idx].nodes {
                    volume_nodes.insert(node.clone());
                }
            }

            let node_count = volume_nodes.len();
            let edge_count = component.len() * 3 / 2; // Approximate
            let cohesion = if node_count > 0 {
                (edge_count as f32) / (node_count as f32 * (node_count as f32 - 1.0) / 2.0).max(1.0)
            } else {
                0.0
            };

            volumes.push(FeatureVolume {
                id: format!("volume_{}", volumes.len()),
                nodes: volume_nodes.into_iter().collect(),
                cohesion_score: cohesion.min(1.0),
            });
        }

        volumes
    }

    /// Get star neighborhood for a node (all directly connected nodes)
    pub fn get_star(graph_data: &SymbolGraph, node_id: &str, depth: usize) -> StarNeighborhood {
        let mut neighbors = HashSet::new();
        let mut edges = Vec::new();
        let mut current_level = HashSet::new();
        current_level.insert(node_id.to_string());
        let mut visited = HashSet::new();

        for _ in 0..depth {
            let mut next_level = HashSet::new();

            for current in &current_level {
                if visited.contains(current) {
                    continue;
                }
                visited.insert(current.clone());

                for (from, to, edge) in &graph_data.edges {
                    if from == current && !visited.contains(to) {
                        neighbors.insert(to.clone());
                        next_level.insert(to.clone());
                        edges.push((from.clone(), to.clone(), edge.relation.clone()));
                    }
                    if to == current && !visited.contains(from) {
                        neighbors.insert(from.clone());
                        next_level.insert(from.clone());
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
}

/// Layer configuration for invariant checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub patterns: Vec<String>, // File/module patterns that belong to this layer
    pub allowed_deps: Vec<String>, // Layer names this layer can depend on
}

/// Result of invariant checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantResult {
    pub is_valid: bool,
    pub layer_violations: Vec<LayerViolation>,
    pub orphaned_nodes: Vec<String>,
    pub component_increase: bool, // Did adding something break connectivity?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerViolation {
    pub from_node: String,
    pub from_layer: String,
    pub to_node: String,
    pub to_layer: String,
    pub violation_type: String, // "upstream_dependency", "cycle", etc.
}

impl InvariantResult {
    /// Check invariants on a graph given layer configuration
    pub fn check(graph: &SymbolGraph, config: &LayerConfig) -> Self {
        let mut violations = Vec::new();
        let mut orphaned = Vec::new();

        // Build layer membership map
        let mut node_to_layer: HashMap<String, String> = HashMap::new();
        for (node_id, symbol) in &graph.nodes {
            for layer in &config.layers {
                for pattern in &layer.patterns {
                    if symbol.file_path.contains(pattern) || symbol.name.contains(pattern) {
                        node_to_layer.insert(node_id.clone(), layer.name.clone());
                        break;
                    }
                }
            }
        }

        // Check each edge for layer violations
        for (from, to, _edge) in &graph.edges {
            let from_layer = node_to_layer.get(from);
            let to_layer = node_to_layer.get(to);

            if let (Some(from_l), Some(to_l)) = (from_layer, to_layer) {
                // Find the layer config for from_layer
                if let Some(layer_cfg) = config.layers.iter().find(|l| &l.name == from_l) {
                    if from_l != to_l && !layer_cfg.allowed_deps.contains(to_l) {
                        violations.push(LayerViolation {
                            from_node: from.clone(),
                            from_layer: from_l.clone(),
                            to_node: to.clone(),
                            to_layer: to_l.clone(),
                            violation_type: "disallowed_dependency".to_string(),
                        });
                    }
                }
            }
        }

        // Find orphaned nodes (nodes with no edges)
        let mut has_edges: HashSet<String> = HashSet::new();
        for (from, to, _) in &graph.edges {
            has_edges.insert(from.clone());
            has_edges.insert(to.clone());
        }
        for (node_id, _) in &graph.nodes {
            if !has_edges.contains(node_id) {
                orphaned.push(node_id.clone());
            }
        }

        InvariantResult {
            is_valid: violations.is_empty(),
            layer_violations: violations,
            orphaned_nodes: orphaned,
            component_increase: false,
        }
    }
}
