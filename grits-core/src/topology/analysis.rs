use super::SymbolGraph;
use petgraph::algo::{astar, connected_components};
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

/// Edge persistence based on filtration value (CoPHo-inspired)
/// Lower lifetime = weaker link = better candidate for refactoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgePersistence {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub birth: f32,      // Filtration value when edge was added
    pub death: f32,      // When cycle containing edge was destroyed (f32::MAX if never)
    pub lifetime: f32,   // death - birth (lower = weaker)
    pub cycle_id: usize, // Which cycle this edge participates in
}

/// Euler-inspired health metric for code architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolidScore {
    pub raw_euler: i64,        // χ = V - E + F - T
    pub normalized: f32,       // 0.0 (spaghetti) to 1.0 (crystal)
    pub betti_components: f32, // B0 penalty (disconnected = bad)
    pub betti_cycles: f32,     // B1 penalty (cycles = bad)
    pub cohesion_bonus: f32,   // Feature volume density (good)
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

        // Find all tetrahedra (4-cliques) for 3-simplexes
        let tetra_count = Self::find_tetrahedra(&undirected, &node_to_un, &triangles);

        // Calculate Betti numbers using Euler characteristic
        // χ = V - E + F - T + ...
        // χ = β0 - β1 + β2 - β3 + ...
        let node_count = all_node_ids.len();
        let edge_count = undirected.edge_count();

        // β0 is connected components
        // β1 = (E - V + β0) - (independent triangles)
        // We approximate independent triangles as triangle_count,
        // but we must not exceed the cycle basis rank (E - V + β0).
        let total_cycles = if edge_count + betti_0 >= node_count {
            edge_count + betti_0 - node_count
        } else {
            0
        };

        let independent_triangles = triangle_count.min(total_cycles);
        let betti_1 = total_cycles - independent_triangles;

        // β2 = χ - β0 + β1 = (V - E + F - T) - β0 + β1
        // Putting it all together:
        // β2 = (V - E + F - T) - β0 + (E - V + β0 - F_indep)
        // If F_indep = F: β2 = -T (which means we have voids if T < something)
        // Actually, the real formula for β2 is Rank(Z2) - Rank(B2).
        // A simpler way: β2 is the number of "hollow" shells.
        // We approximate β2 by looking for clusters of triangles that aren't filled by tetrahedra.
        let chi = (node_count as i32) - (edge_count as i32) + (triangle_count as i32)
            - (tetra_count as i32);
        let betti_2 = (chi - (betti_0 as i32) + (betti_1 as i32)).max(0) as usize;

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

    /// Find all tetrahedra (4-cliques) in an undirected graph
    fn find_tetrahedra(
        graph: &UnGraph<String, ()>,
        node_map: &HashMap<String, NodeIndex>,
        triangles: &[Triangle],
    ) -> usize {
        let mut tetra_count = 0;
        let mut seen = HashSet::new();

        // A tetrahedron is a 4-clique.
        // We can find it by taking a triangle (u, v, w) and finding a common neighbor x.
        for tri in triangles {
            let u = match node_map.get(&tri.nodes[0]) {
                Some(&idx) => idx,
                None => continue,
            };
            let v = match node_map.get(&tri.nodes[1]) {
                Some(&idx) => idx,
                None => continue,
            };
            let w = match node_map.get(&tri.nodes[2]) {
                Some(&idx) => idx,
                None => continue,
            };

            let u_neighbors: HashSet<NodeIndex> = graph.neighbors(u).collect();
            let v_neighbors: HashSet<NodeIndex> = graph.neighbors(v).collect();
            let w_neighbors: HashSet<NodeIndex> = graph.neighbors(w).collect();

            let common_uv: HashSet<_> = u_neighbors.intersection(&v_neighbors).copied().collect();
            let common_uvw: HashSet<_> = common_uv.intersection(&w_neighbors).copied().collect();

            for x in common_uvw {
                let mut nodes = [
                    tri.nodes[0].clone(),
                    tri.nodes[1].clone(),
                    tri.nodes[2].clone(),
                    graph[x].clone(),
                ];
                nodes.sort();

                let key = format!("{}-{}-{}-{}", nodes[0], nodes[1], nodes[2], nodes[3]);
                if !seen.contains(&key) {
                    seen.insert(key);
                    tetra_count += 1;
                }
            }
        }

        tetra_count
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

    /// Compute weighted PageRank for nodes based on edge strengths.
    /// Higher strength edges contribute more to rank propagation.
    /// Returns a map of node_id -> rank (0.0 to 1.0, higher = more important)
    pub fn weighted_pagerank(
        graph_data: &SymbolGraph,
        damping: f32,
        iterations: usize,
    ) -> HashMap<String, f32> {
        // Collect all node ids
        let mut all_nodes: HashSet<String> = HashSet::new();
        for (id, _) in &graph_data.nodes {
            all_nodes.insert(id.clone());
        }
        for (from, to, _) in &graph_data.edges {
            all_nodes.insert(from.clone());
            all_nodes.insert(to.clone());
        }

        let n = all_nodes.len() as f32;
        if n == 0.0 {
            return HashMap::new();
        }

        // Initialize ranks
        let initial_rank = 1.0 / n;
        let mut ranks: HashMap<String, f32> = all_nodes
            .iter()
            .map(|id| (id.clone(), initial_rank))
            .collect();

        // Build outgoing edge weights per node
        let mut out_weights: HashMap<String, f32> = HashMap::new();
        for (from, _, edge) in &graph_data.edges {
            *out_weights.entry(from.clone()).or_insert(0.0) += edge.strength;
        }

        // Iterate
        for _ in 0..iterations {
            let mut new_ranks: HashMap<String, f32> = all_nodes
                .iter()
                .map(|id| (id.clone(), (1.0 - damping) / n))
                .collect();

            for (from, to, edge) in &graph_data.edges {
                let from_rank = *ranks.get(from).unwrap_or(&0.0);
                let total_out = *out_weights.get(from).unwrap_or(&1.0);

                if total_out > 0.0 {
                    // Weighted contribution: (rank * edge_weight / total_out_weight) * damping
                    let contribution = damping * from_rank * (edge.strength / total_out);
                    *new_ranks.entry(to.clone()).or_insert(0.0) += contribution;
                }
            }

            ranks = new_ranks;
        }

        // Normalize to 0-1 range
        let max_rank = ranks.values().cloned().fold(0.0f32, f32::max);
        if max_rank > 0.0 {
            for rank in ranks.values_mut() {
                *rank /= max_rank;
            }
        }

        ranks
    }

    /// Compute edge persistence for all edges participating in cycles
    /// Uses a filtration based on PageRank - edges connecting important nodes are added first
    pub fn compute_edge_persistence(graph_data: &SymbolGraph) -> Vec<EdgePersistence> {
        let mut persistence = Vec::new();

        // Get PageRank to order edges by importance
        let ranks = Self::weighted_pagerank(graph_data, 0.85, 20);

        // Build edge list with filtration values
        let mut edges_with_filtration: Vec<(String, String, String, f32)> = graph_data
            .edges
            .iter()
            .map(|(from, to, edge)| {
                // Filtration value = max rank of endpoints (higher = added earlier)
                let from_rank = ranks.get(from).copied().unwrap_or(0.0);
                let to_rank = ranks.get(to).copied().unwrap_or(0.0);
                let filtration = from_rank.max(to_rank);
                (from.clone(), to.clone(), edge.relation.clone(), filtration)
            })
            .collect();

        // Sort by filtration (descending - high rank edges first)
        edges_with_filtration
            .sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

        // Use union-find to detect when edges create cycles
        let mut all_nodes: HashSet<String> = HashSet::new();
        for (from, to, _, _) in &edges_with_filtration {
            all_nodes.insert(from.clone());
            all_nodes.insert(to.clone());
        }

        let node_list: Vec<String> = all_nodes.into_iter().collect();
        let node_to_idx: HashMap<String, usize> = node_list
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();

        // Simple union-find
        let mut parent: Vec<usize> = (0..node_list.len()).collect();

        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], i: usize, j: usize) {
            let pi = find(parent, i);
            let pj = find(parent, j);
            if pi != pj {
                parent[pi] = pj;
            }
        }

        let mut cycle_id = 0;

        for (from, to, relation, filtration) in edges_with_filtration {
            let from_idx = match node_to_idx.get(&from) {
                Some(&i) => i,
                None => continue,
            };
            let to_idx = match node_to_idx.get(&to) {
                Some(&i) => i,
                None => continue,
            };

            let from_parent = find(&mut parent, from_idx);
            let to_parent = find(&mut parent, to_idx);

            if from_parent == to_parent {
                // This edge creates a cycle - it has low lifetime (born and dies at same point)
                persistence.push(EdgePersistence {
                    source: from,
                    target: to,
                    relation,
                    birth: filtration,
                    death: filtration, // Dies immediately
                    lifetime: 0.0,     // Minimum lifetime = weakest link
                    cycle_id,
                });
                cycle_id += 1;
            } else {
                // This edge connects components - it persists longer
                union(&mut parent, from_idx, to_idx);
                persistence.push(EdgePersistence {
                    source: from,
                    target: to,
                    relation,
                    birth: filtration,
                    death: f32::MAX,
                    lifetime: f32::MAX,
                    cycle_id: usize::MAX, // Not part of a cycle initially
                });
            }
        }

        // Sort by lifetime (ascending) so weakest links are first
        persistence.sort_by(|a, b| {
            a.lifetime
                .partial_cmp(&b.lifetime)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        persistence
    }

    /// Suggest which edge to remove to break a specific cycle
    /// Returns the edge with lowest lifetime in that cycle
    pub fn suggest_refactor(graph_data: &SymbolGraph, cycle_id: usize) -> Option<EdgePersistence> {
        let persistence = Self::compute_edge_persistence(graph_data);
        persistence
            .into_iter()
            .filter(|e| e.cycle_id == cycle_id)
            .min_by(|a, b| {
                a.lifetime
                    .partial_cmp(&b.lifetime)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Compute the Solid Score - a unified health metric
    pub fn solid_score(&self) -> SolidScore {
        // Euler characteristic: χ = V - E + F - T
        let raw_euler =
            (self.node_count as i64) - (self.edge_count as i64) + (self.triangle_count as i64);

        // B0 penalty: 1.0 if single component, decreases with more components
        let betti_components = if self.betti_0 == 0 {
            0.0
        } else {
            1.0 / (self.betti_0 as f32)
        };

        // B1 penalty: 1.0 if no cycles, decreases exponentially with cycles
        let betti_cycles = (-0.5 * self.betti_1 as f32).exp();

        // Cohesion bonus: average cohesion of feature volumes
        let cohesion_bonus = if self.feature_volumes.is_empty() {
            0.5 // Neutral if no volumes
        } else {
            self.feature_volumes
                .iter()
                .map(|v| v.cohesion_score)
                .sum::<f32>()
                / self.feature_volumes.len() as f32
        };

        // Weighted combination: connectivity and cycles matter most
        let normalized = (
            0.3 * betti_components +  // Want single connected component
            0.5 * betti_cycles +      // Want no cycles
            0.2 * cohesion_bonus
            // Bonus for well-structured volumes
        )
            .clamp(0.0, 1.0);

        SolidScore {
            raw_euler,
            normalized,
            betti_components,
            betti_cycles,
            cohesion_bonus,
        }
    }

    /// Find shortest path between two symbols using A*
    pub fn get_path(graph_data: &SymbolGraph, start: &str, end: &str) -> Option<Vec<String>> {
        let mut digraph = DiGraph::<String, ()>::new();
        let mut node_to_idx = HashMap::new();

        // Ensure all nodes from edges are also included
        let mut all_node_ids = HashSet::new();
        for (id, _) in &graph_data.nodes {
            all_node_ids.insert(id.clone());
        }
        for (from, to, _) in &graph_data.edges {
            all_node_ids.insert(from.clone());
            all_node_ids.insert(to.clone());
        }

        for id in all_node_ids {
            let idx = digraph.add_node(id.clone());
            node_to_idx.insert(id.clone(), idx);
        }

        for (from, to, _) in &graph_data.edges {
            if let (Some(&u), Some(&v)) = (node_to_idx.get(from), node_to_idx.get(to)) {
                digraph.add_edge(u, v, ());
            }
        }

        let start_idx = *node_to_idx.get(start)?;
        let end_idx = *node_to_idx.get(end)?;

        let path = astar(&digraph, start_idx, |n| n == end_idx, |_| 1, |_| 0)?;

        Some(path.1.into_iter().map(|idx| digraph[idx].clone()).collect())
    }

    /// Find a symbol ID using fuzzy matching.
    /// Tries exact match first, then suffix match (e.g., "::SqliteStore" matches "store::SqliteStore").
    /// Returns the full symbol ID if found.
    pub fn find_symbol_fuzzy(graph_data: &SymbolGraph, query: &str) -> Option<String> {
        // Collect all node IDs
        let mut all_ids: HashSet<String> = HashSet::new();
        for (id, _) in &graph_data.nodes {
            all_ids.insert(id.clone());
        }
        for (from, to, _) in &graph_data.edges {
            all_ids.insert(from.clone());
            all_ids.insert(to.clone());
        }

        // 1. Try exact match first
        if all_ids.contains(query) {
            return Some(query.to_string());
        }

        // 2. Try suffix match (ends with "::query" or "/query")
        let suffix_patterns = [
            format!("::{}", query),
            format!("/{}", query),
            format!(".{}", query),
        ];

        for id in &all_ids {
            for suffix in &suffix_patterns {
                if id.ends_with(suffix) {
                    return Some(id.clone());
                }
            }
        }

        // 3. Try substring match (contains the query as a component)
        for id in &all_ids {
            // Extract the final component of the symbol path
            let final_component = id
                .rsplit("::")
                .next()
                .or_else(|| id.rsplit('/').next())
                .or_else(|| id.rsplit('.').next())
                .unwrap_or(id);

            if final_component == query {
                return Some(id.clone());
            }
        }

        // 4. Try case-insensitive match on the final component
        let query_lower = query.to_lowercase();
        for id in &all_ids {
            let final_component = id
                .rsplit("::")
                .next()
                .or_else(|| id.rsplit('/').next())
                .or_else(|| id.rsplit('.').next())
                .unwrap_or(id);

            if final_component.to_lowercase() == query_lower {
                return Some(id.clone());
            }
        }

        None
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
