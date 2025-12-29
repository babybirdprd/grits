use crate::fs::FileSystem;
use crate::git::GitOps;
use anyhow::{bail, Result};
use std::io::{BufRead, Cursor, Write};
use std::path::Path;
use wasm_bindgen::prelude::*;

// JS bindings for grits-core
#[wasm_bindgen(module = "/js/grits_fs.js")]
extern "C" {
    fn fs_read_to_string(path: &str) -> String;
    fn fs_write(path: &str, content: &[u8]);
    fn fs_create_dir_all(path: &str);
    fn fs_rename(from: &str, to: &str);
    fn fs_exists(path: &str) -> bool;
}

#[wasm_bindgen(module = "/js/grits_git.js")]
extern "C" {
    fn git_init() -> String;
    fn git_add(path: &str) -> String;
    fn git_commit(message: &str) -> String;
    fn git_pull_rebase() -> String;
    fn git_push() -> String;
    fn git_status() -> String;
    fn git_show(revision: &str) -> String;
    fn git_rebase_continue() -> String;
    fn git_has_remote() -> bool;
    fn git_config(key: &str, value: &str) -> String;
}

/// FileSystem implementation that delegates to JavaScript.
#[wasm_bindgen]
pub struct WasmFileSystem;

#[wasm_bindgen]
impl WasmFileSystem {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmFileSystem
    }
}

impl FileSystem for WasmFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        // In a real implementation, we'd handle errors from JS, possibly returning Result<String, JsValue>
        // and mapping it. For now assuming the JS binding throws or returns a string.
        // But wasm_bindgen extern functions usually match the signature.
        // If JS can fail, we should use catch.
        Ok(fs_read_to_string(path_str))
    }

    fn write(&self, path: &Path, contents: &[u8]) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        fs_write(path_str, contents);
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        fs_create_dir_all(path_str);
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let from_str = from
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        let to_str = to.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        fs_rename(from_str, to_str);
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        if let Some(path_str) = path.to_str() {
            fs_exists(path_str)
        } else {
            false
        }
    }

    fn open_read(&self, path: &Path) -> Result<Box<dyn BufRead>> {
        // For now, read entire file into memory and return a Cursor
        let content = self.read_to_string(path)?;
        Ok(Box::new(Cursor::new(content.into_bytes())))
    }

    fn open_write(&self, path: &Path) -> Result<Box<dyn Write>> {
        // This is tricky because we need a writer that writes back to JS on flush/drop.
        // For simple WASM usage, we might buffer in memory.
        Ok(Box::new(WasmFileWriter {
            path: path.to_path_buf(),
            buffer: Vec::new(),
            fs: WasmFileSystem,
        }))
    }
}

struct WasmFileWriter {
    path: std::path::PathBuf,
    buffer: Vec<u8>,
    fs: WasmFileSystem,
}

impl Write for WasmFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.fs
            .write(&self.path, &self.buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }
}

#[wasm_bindgen]
pub struct WasmGit;

#[wasm_bindgen]
impl WasmGit {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmGit
    }
}

impl GitOps for WasmGit {
    fn init(&self) -> Result<()> {
        let res = git_init();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn add(&self, path: &Path) -> Result<()> {
        let path_str = path.to_str().unwrap_or("");
        let res = git_add(path_str);
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn commit(&self, message: &str) -> Result<()> {
        let res = git_commit(message);
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn pull_rebase(&self) -> Result<()> {
        let res = git_pull_rebase();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn push(&self) -> Result<()> {
        let res = git_push();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn status(&self) -> Result<String> {
        let res = git_status();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(res)
    }

    fn show(&self, revision: &str) -> Result<String> {
        let res = git_show(revision);
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(res)
    }

    fn rebase_continue(&self) -> Result<()> {
        let res = git_rebase_continue();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn has_remote(&self) -> Result<bool> {
        Ok(git_has_remote())
    }

    fn config(&self, key: &str, value: &str) -> Result<()> {
        let res = git_config(key, value);
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }
}

// =============================================================================
// GritsWasm - WASM Bridge for VS Code Extension UI
// =============================================================================

use crate::memory_store::MemoryStore;
use crate::models::{Comment, Issue};
use crate::search::SearchIndex;
use crate::store::Store;
use serde_json::Value;

fn extract_graph(json: &str) -> Result<crate::topology::SymbolGraph, JsValue> {
    let v: Value = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse JSON failed: {}", e)))?;

    if v.get("graph").is_some() {
        serde_json::from_value(v["graph"].clone())
            .map_err(|e| JsValue::from_str(&format!("Parse nested graph failed: {}", e)))
    } else {
        serde_json::from_value(v)
            .map_err(|e| JsValue::from_str(&format!("Parse raw graph failed: {}", e)))
    }
}

/// Stateful WASM store wrapping MemoryStore.
#[wasm_bindgen]
pub struct WasmStore {
    inner: MemoryStore,
}

#[wasm_bindgen]
impl WasmStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmStore {
            inner: MemoryStore::new(),
        }
    }

    #[wasm_bindgen]
    pub fn search(&self, query: &str) -> Result<String, JsValue> {
        let issues = self
            .inner
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| JsValue::from_str(&format!("List failed: {}", e)))?;

        let index = SearchIndex::new(&issues);
        let results = index.search(query, &issues);

        let found_issues: Vec<Issue> = results.into_iter().map(|(i, _)| i).collect();

        serde_json::to_string(&found_issues)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Load issues from JSONL content string
    #[wasm_bindgen]
    pub fn load(&mut self, content: &str) -> Result<(), JsValue> {
        self.inner
            .import_from_string(content)
            .map_err(|e| JsValue::from_str(&format!("Failed to load content: {}", e)))
    }

    /// Export issues to JSONL content string
    #[wasm_bindgen]
    pub fn export(&self) -> Result<String, JsValue> {
        self.inner
            .export_to_string()
            .map_err(|e| JsValue::from_str(&format!("Failed to export content: {}", e)))
    }

    /// List issues with optional filters (passed as JSON object)
    /// { "status": "open", "sort_by": "updated" }
    #[wasm_bindgen]
    pub fn list_issues(&self, filter_json: Option<String>) -> Result<String, JsValue> {
        let (status, assignee, priority, issue_type, label, sort_by) =
            if let Some(json) = filter_json {
                let v: Value = serde_json::from_str(&json)
                    .map_err(|e| JsValue::from_str(&format!("Invalid filter JSON: {}", e)))?;
                (
                    v.get("status")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    v.get("assignee")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    v.get("priority").and_then(|v| v.as_i64()).map(|i| i as i32),
                    v.get("type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    v.get("label")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    v.get("sort_by")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                )
            } else {
                (None, None, None, None, None, None)
            };

        let issues = self
            .inner
            .list_issues(
                status.as_deref(),
                assignee.as_deref(),
                priority,
                issue_type.as_deref(),
                label.as_deref(),
                sort_by.as_deref(),
            )
            .map_err(|e| JsValue::from_str(&format!("List issues failed: {}", e)))?;

        serde_json::to_string(&issues)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    #[wasm_bindgen]
    pub fn update_issue(&self, id: &str, field: &str, value_json: &str) -> Result<(), JsValue> {
        let mut issue = self
            .inner
            .get_issue(id)
            .map_err(|e| JsValue::from_str(&format!("DB error: {}", e)))?
            .ok_or_else(|| JsValue::from_str(&format!("Issue not found: {}", id)))?;

        let value: Value = serde_json::from_str(value_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid value JSON: {}", e)))?;

        match field {
            "title" => issue.title = value.as_str().unwrap_or(&issue.title).to_string(),
            "description" => {
                issue.description = value.as_str().unwrap_or(&issue.description).to_string()
            }
            "status" => issue.status = value.as_str().unwrap_or(&issue.status).to_string(),
            "priority" => issue.priority = value.as_i64().unwrap_or(issue.priority as i64) as i32,
            "issue_type" => {
                issue.issue_type = value.as_str().unwrap_or(&issue.issue_type).to_string()
            }
            "assignee" => issue.assignee = value.as_str().map(|s| s.to_string()), // Null clears it
            _ => return Err(JsValue::from_str(&format!("Unknown field: {}", field))),
        }

        issue.updated_at = chrono::Utc::now();

        self.inner
            .update_issue(&issue)
            .map_err(|e| JsValue::from_str(&format!("Update failed: {}", e)))?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn bulk_update(
        &self,
        ids_json: &str,
        field: &str,
        value_json: &str,
    ) -> Result<(), JsValue> {
        let ids: Vec<String> = serde_json::from_str(ids_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid IDs JSON: {}", e)))?;
        let value: Value = serde_json::from_str(value_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid value JSON: {}", e)))?;

        for id in ids {
            if let Ok(Some(mut issue)) = self.inner.get_issue(&id) {
                match field {
                    "status" => issue.status = value.as_str().unwrap_or(&issue.status).to_string(),
                    "priority" => {
                        issue.priority = value.as_i64().unwrap_or(issue.priority as i64) as i32
                    }
                    "issue_type" => {
                        issue.issue_type = value.as_str().unwrap_or(&issue.issue_type).to_string()
                    }
                    "assignee" => issue.assignee = value.as_str().map(|s| s.to_string()),
                    _ => continue, // Skip unknown fields
                }
                issue.updated_at = chrono::Utc::now();
                let _ = self.inner.update_issue(&issue);
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn add_label(&self, id: &str, label: &str) -> Result<(), JsValue> {
        let mut issue = self
            .inner
            .get_issue(id)
            .map_err(|e| JsValue::from_str(&format!("DB error: {}", e)))?
            .ok_or_else(|| JsValue::from_str(&format!("Issue not found: {}", id)))?;

        if !issue.labels.contains(&label.to_string()) {
            issue.labels.push(label.to_string());
            issue.updated_at = chrono::Utc::now();
            self.inner
                .update_issue(&issue)
                .map_err(|e| JsValue::from_str(&format!("Update failed: {}", e)))?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn remove_label(&self, id: &str, label: &str) -> Result<(), JsValue> {
        let mut issue = self
            .inner
            .get_issue(id)
            .map_err(|e| JsValue::from_str(&format!("DB error: {}", e)))?
            .ok_or_else(|| JsValue::from_str(&format!("Issue not found: {}", id)))?;

        if let Some(pos) = issue.labels.iter().position(|l| l == label) {
            issue.labels.remove(pos);
            issue.updated_at = chrono::Utc::now();
            self.inner
                .update_issue(&issue)
                .map_err(|e| JsValue::from_str(&format!("Update failed: {}", e)))?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn add_comment(&self, id: &str, text: &str, author: &str) -> Result<(), JsValue> {
        let mut issue = self
            .inner
            .get_issue(id)
            .map_err(|e| JsValue::from_str(&format!("DB error: {}", e)))?
            .ok_or_else(|| JsValue::from_str(&format!("Issue not found: {}", id)))?;

        let comment = Comment {
            id: uuid::Uuid::new_v4().to_string(),
            issue_id: issue.id.clone(),
            text: text.to_string(),
            author: author.to_string(),
            created_at: chrono::Utc::now(),
        };

        issue.comments.push(comment);
        issue.updated_at = chrono::Utc::now();
        self.inner
            .update_issue(&issue)
            .map_err(|e| JsValue::from_str(&format!("Update failed: {}", e)))?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn create_issue(
        &self,
        title: &str,
        description: &str,
        issue_type: &str,
        priority: i32,
    ) -> Result<String, JsValue> {
        use chrono::Utc;

        // Generate ID (simplified version of Store logic or reuse Store::generate_unique_id if exposed)
        // Store::generate_unique_id requires prefix and creator, which we might not have perfectly here.
        // We'll mimic the logic for now or update MemoryStore to expose it better.
        // Actually MemoryStore implements Store which has generate_unique_id.
        // We need a config for prefix/user though.
        let prefix = match self.inner.get_config("issue_id_prefix") {
            Ok(Some(p)) => p,
            _ => "gr".to_string(),
        };
        let user = match self.inner.get_config("user.name") {
            Ok(Some(u)) => u,
            _ => "unknown".to_string(),
        };

        let id = self
            .inner
            .generate_unique_id(&prefix, title, description, &user)
            .map_err(|e| JsValue::from_str(&format!("ID generation failed: {}", e)))?;

        let now = Utc::now();
        let issue = Issue {
            id: id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            status: "open".to_string(),
            priority,
            issue_type: issue_type.to_string(),
            created_at: now,
            updated_at: now,
            assignee: None,
            comments: Vec::new(),
            labels: Vec::new(),
            dependencies: Vec::new(),
            ..Default::default()
        };

        self.inner
            .create_issue(&issue)
            .map_err(|e| JsValue::from_str(&format!("Create failed: {}", e)))?;

        Ok(id)
    }

    #[wasm_bindgen]
    pub fn get_all_labels(&self) -> Result<String, JsValue> {
        // Collect all unique labels
        let issues = self
            .inner
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| JsValue::from_str(&format!("List failed: {}", e)))?;

        let mut labels = std::collections::HashSet::new();
        for issue in issues {
            for label in issue.labels {
                labels.insert(label);
            }
        }
        let mut list: Vec<_> = labels.into_iter().collect();
        list.sort();
        serde_json::to_string(&list)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    // =========================================================================
    // Topology Visualization Exports
    // =========================================================================

    /// Get topology graph data for 3D visualization
    /// Returns JSON with {nodes: {id: {name, kind, file_path, pageRank, inCycle}}, edges: [[src, dst, {relation, strength}]]}
    #[wasm_bindgen]
    pub fn get_topology_for_viz(&self, topology_json: &str) -> Result<String, JsValue> {
        use crate::topology::analysis::TopologicalAnalysis;
        use crate::topology::SymbolGraph;

        // Parse the cached topology JSON
        let graph = extract_graph(topology_json)?;

        let analysis = TopologicalAnalysis::analyze(&graph);
        let pagerank = TopologicalAnalysis::weighted_pagerank(&graph, 0.85, 50);

        // Get cycle count from betti_1
        let cycle_count = analysis.betti_1;

        // Build nodes array
        let mut nodes: Vec<serde_json::Value> = Vec::new();
        for (id, sym) in &graph.nodes {
            let rank = pagerank.get(id).copied().unwrap_or(0.0);
            nodes.push(serde_json::json!({
                "id": id,
                "name": sym.name,
                "kind": sym.kind,
                "file_path": sym.file_path,
                "package": sym.package,
                "pageRank": rank,
                "inCycle": false  // Would need cycle detection to set this
            }));
        }

        // Build edges array
        let mut edges: Vec<serde_json::Value> = Vec::new();
        for (from, to, edge) in &graph.edges {
            edges.push(serde_json::json!({
                "source": from,
                "target": to,
                "relation": edge.relation,
                "strength": edge.strength
            }));
        }

        let result = serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "stats": {
                "nodeCount": graph.nodes.len(),
                "edgeCount": graph.edges.len(),
                "cycleCount": cycle_count
            }
        });

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Compute solid score from topology JSON
    #[wasm_bindgen]
    pub fn compute_solid_score(&self, topology_json: &str) -> Result<f64, JsValue> {
        use crate::topology::analysis::TopologicalAnalysis;
        use crate::topology::SymbolGraph;

        let graph = extract_graph(topology_json)?;

        let analysis = TopologicalAnalysis::analyze(&graph);
        let score = analysis.solid_score();

        Ok(score.normalized as f64)
    }

    /// Get PageRank hotspots (top N most connected symbols)
    #[wasm_bindgen]
    pub fn get_pagerank_hotspots(
        &self,
        topology_json: &str,
        limit: usize,
    ) -> Result<String, JsValue> {
        use crate::topology::analysis::TopologicalAnalysis;
        use crate::topology::SymbolGraph;

        let graph = extract_graph(topology_json)?;

        let pagerank = TopologicalAnalysis::weighted_pagerank(&graph, 0.85, 50);

        let mut ranked: Vec<_> = pagerank.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let hotspots: Vec<_> = ranked
            .into_iter()
            .take(limit)
            .map(|(name, score)| serde_json::json!({ "name": name, "score": score }))
            .collect();

        serde_json::to_string(&hotspots)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Get vitals summary for dashboard
    #[wasm_bindgen]
    pub fn get_vitals(&self, topology_json: Option<String>) -> Result<String, JsValue> {
        use crate::topology::analysis::TopologicalAnalysis;
        use crate::topology::SymbolGraph;

        let issues = self
            .inner
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| JsValue::from_str(&format!("List failed: {}", e)))?;

        let in_progress = issues.iter().filter(|i| i.status == "in-progress").count();

        let (solid_score, betti0, betti1, hotspots) = if let Some(topo_json) = topology_json {
            let graph = extract_graph(&topo_json)?;

            let analysis = TopologicalAnalysis::analyze(&graph);
            let score = analysis.solid_score();
            let pagerank = TopologicalAnalysis::weighted_pagerank(&graph, 0.85, 50);

            let mut ranked: Vec<_> = pagerank.into_iter().collect();
            ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let top3: Vec<_> = ranked
                .into_iter()
                .take(3)
                .map(|(n, s)| serde_json::json!({"name": n, "score": s}))
                .collect();

            (
                score.normalized as f64,
                analysis.betti_0,
                analysis.betti_1,
                top3,
            )
        } else {
            (0.0, 0, 0, vec![])
        };

        let result = serde_json::json!({
            "solidScore": solid_score,
            "betti0": betti0,
            "betti1": betti1,
            "inProgressCount": in_progress,
            "totalIssues": issues.len(),
            "hotspots": hotspots
        });

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }
}

// =============================================================================
// WasmTopologyStore - In-Memory Graph for Linear-Fast Performance (v2.1)
// =============================================================================

use std::collections::HashMap;

/// In-memory topology store with pre-computed PageRank and instant search.
/// This eliminates CLI round-trips for <10ms response times.
#[wasm_bindgen]
pub struct WasmTopologyStore {
    graph: Option<crate::topology::SymbolGraph>,
    pagerank_cache: HashMap<String, f32>,
    solid_score_cache: Option<f64>,
    betti_cache: (usize, usize, usize), // (b0, b1, b2)
}

#[wasm_bindgen]
impl WasmTopologyStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmTopologyStore {
            graph: None,
            pagerank_cache: HashMap::new(),
            solid_score_cache: None,
            betti_cache: (0, 0, 0),
        }
    }

    /// Load topology from JSON and pre-compute PageRank + Solid Score.
    /// Call this once on dashboard open for instant subsequent queries.
    #[wasm_bindgen]
    pub fn load_topology(&mut self, topology_json: &str) -> Result<String, JsValue> {
        use crate::topology::analysis::TopologicalAnalysis;
        use crate::topology::SymbolGraph;
        use serde_json::Value;

        let graph = extract_graph(topology_json)?;

        // Pre-compute analysis (this is the expensive part, do it once)
        let analysis = TopologicalAnalysis::analyze(&graph);
        let score = analysis.solid_score();

        // Cache PageRank
        self.pagerank_cache = TopologicalAnalysis::weighted_pagerank(&graph, 0.85, 50);
        self.solid_score_cache = Some(score.normalized as f64);
        self.betti_cache = (analysis.betti_0, analysis.betti_1, analysis.betti_2);

        let stats = serde_json::json!({
            "nodes": graph.nodes.len(),
            "edges": graph.edges.len(),
            "solidScore": score.normalized,
            "betti0": analysis.betti_0,
            "betti1": analysis.betti_1,
            "betti2": analysis.betti_2,
            "triangles": analysis.triangle_count
        });

        self.graph = Some(graph);

        serde_json::to_string(&stats)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Instant search for symbols by name (fuzzy match).
    /// Returns in <10ms from pre-loaded graph.
    #[wasm_bindgen]
    pub fn search_symbols(&self, query: &str, limit: usize) -> Result<String, JsValue> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Topology not loaded. Call load_topology first."))?;

        let query_lower = query.to_lowercase();
        let mut matches: Vec<_> = graph
            .nodes
            .values()
            .filter(|sym| {
                sym.name.to_lowercase().contains(&query_lower)
                    || sym.file_path.to_lowercase().contains(&query_lower)
            })
            .map(|sym| {
                let rank = self.pagerank_cache.get(&sym.id).copied().unwrap_or(0.0);
                serde_json::json!({
                    "id": sym.id,
                    "name": sym.name,
                    "file": sym.file_path,
                    "kind": sym.kind,
                    "pagerank": rank
                })
            })
            .collect();

        // Sort by PageRank (most important first)
        matches.sort_by(|a, b| {
            let ra = a["pagerank"].as_f64().unwrap_or(0.0);
            let rb = b["pagerank"].as_f64().unwrap_or(0.0);
            rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
        });

        let results: Vec<_> = matches.into_iter().take(limit).collect();

        serde_json::to_string(&results)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Get cached solid score (instant, no computation).
    #[wasm_bindgen]
    pub fn get_solid_score(&self) -> f64 {
        self.solid_score_cache.unwrap_or(0.0)
    }

    /// Get cached Betti numbers as JSON.
    #[wasm_bindgen]
    pub fn get_betti(&self) -> String {
        serde_json::json!({
            "b0": self.betti_cache.0,
            "b1": self.betti_cache.1,
            "b2": self.betti_cache.2
        })
        .to_string()
    }

    /// Get top N symbols by PageRank (cached).
    #[wasm_bindgen]
    pub fn get_hotspots(&self, limit: usize) -> Result<String, JsValue> {
        let mut ranked: Vec<_> = self
            .pagerank_cache
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let hotspots: Vec<_> = ranked
            .into_iter()
            .take(limit)
            .map(|(name, score)| serde_json::json!({"name": name, "score": score}))
            .collect();

        serde_json::to_string(&hotspots)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Get node data for 3D visualization (pre-computed layout data).
    #[wasm_bindgen]
    pub fn get_nodes_for_viz(&self) -> Result<String, JsValue> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Topology not loaded"))?;

        let nodes: Vec<_> = graph
            .nodes
            .values()
            .map(|sym| {
                let rank = self.pagerank_cache.get(&sym.id).copied().unwrap_or(0.0);
                serde_json::json!({
                    "id": sym.id,
                    "name": sym.name,
                    "file": sym.file_path,
                    "kind": sym.kind,
                    "package": sym.package,
                    "pagerank": rank,
                    // Scale size by PageRank
                    "size": 0.1 + rank * 2.0
                })
            })
            .collect();

        serde_json::to_string(&nodes)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Get edges for visualization.
    #[wasm_bindgen]
    pub fn get_edges_for_viz(&self) -> Result<String, JsValue> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Topology not loaded"))?;

        let edges: Vec<_> = graph
            .edges
            .iter()
            .map(|(from, to, edge)| {
                serde_json::json!({
                    "source": from,
                    "target": to,
                    "relation": edge.relation,
                    "strength": edge.strength
                })
            })
            .collect();

        serde_json::to_string(&edges)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Check if topology is loaded.
    #[wasm_bindgen]
    pub fn is_loaded(&self) -> bool {
        self.graph.is_some()
    }

    /// Get node count.
    #[wasm_bindgen]
    pub fn node_count(&self) -> usize {
        self.graph.as_ref().map(|g| g.nodes.len()).unwrap_or(0)
    }

    /// Get edge count.
    #[wasm_bindgen]
    pub fn edge_count(&self) -> usize {
        self.graph.as_ref().map(|g| g.edges.len()).unwrap_or(0)
    }
}
