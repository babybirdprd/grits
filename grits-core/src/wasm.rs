use crate::fs::FileSystem;
use crate::git::GitOps;
use crate::memory_store::MemoryStore;
use crate::models::Issue;
use crate::search::SearchIndex;
use crate::store::Store;
use anyhow::{bail, Result};
use std::io::{BufRead, Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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
        let content = self.read_to_string(path)?;
        Ok(Box::new(Cursor::new(content.into_bytes())))
    }

    fn open_write(&self, path: &Path) -> Result<Box<dyn Write>> {
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
// WasmStore - Stateful Store for VS Code Extension UI
// =============================================================================

#[wasm_bindgen]
pub struct WasmStore {
    store: Arc<MemoryStore>,
    search_index: Arc<Mutex<SearchIndex>>,
}

#[wasm_bindgen]
impl WasmStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmStore {
            store: Arc::new(MemoryStore::new()),
            search_index: Arc::new(Mutex::new(SearchIndex::new())),
        }
    }

    /// Load issues from a JSONL string content.
    /// This populates the memory store and rebuilds the search index.
    #[wasm_bindgen]
    pub fn load_from_jsonl(&self, content: &str) -> Result<(), JsValue> {
        // Clear existing issues to avoid stale data when reloading
        // MemoryStore doesn't expose clear(), but since we are refilling, we might want to?
        // Actually MemoryStore persists in memory. If we load new content, we probably want to replace or merge.
        // "Twin Engine" implies syncing with file. If we load file content, it should reflect file state.
        // But for multi-repo, we call this multiple times?
        // Let's assume single-file mode clears first if we could, but we can't easily.
        // But `update_issue` overwrites by ID.
        // If an issue was deleted in file, it won't be deleted here if we just update.
        // This is a limitation of this simple WasmStore.
        // For now, we assume this loads the "world state".

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let issue: Issue = serde_json::from_str(line)
                .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
            self.store
                .update_issue(&issue)
                .map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?;
        }

        // Rebuild search index
        self.rebuild_search_index()?;

        Ok(())
    }

    /// Load multiple JSONL contents (Workspace Mode).
    /// Concatenates content and loads them.
    #[wasm_bindgen]
    pub fn load_workspace(&self, contents: Box<[JsValue]>) -> Result<(), JsValue> {
        // Ideally we clear first.

        for content_val in contents.iter() {
            if let Some(content) = content_val.as_string() {
                self.load_from_jsonl(&content)?;
            }
        }
        Ok(())
    }

    fn rebuild_search_index(&self) -> Result<(), JsValue> {
        let issues = self.store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| JsValue::from_str(&format!("List error: {}", e)))?;

        let mut index = self.search_index.lock().unwrap();
        index.index_issues(&issues);
        Ok(())
    }

    /// Get all issues as a JSON array string, optionally filtered.
    #[wasm_bindgen]
    pub fn list_issues(&self, filter_json: &str) -> Result<String, JsValue> {
        // filter_json could be { status: "open", assignee: "me", ... }
        // For now, let's keep it simple and just use the args that MemoryStore::list_issues accepts if possible,
        // or parse the JSON.

        #[derive(serde::Deserialize)]
        struct Filters {
            status: Option<String>,
            assignee: Option<String>,
            priority: Option<i32>,
            issue_type: Option<String>,
            label: Option<String>,
            sort_by: Option<String>,
        }

        let filters: Filters = if filter_json.is_empty() {
            Filters { status: None, assignee: None, priority: None, issue_type: None, label: None, sort_by: None }
        } else {
            serde_json::from_str(filter_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid filter JSON: {}", e)))?
        };

        let issues = self.store.list_issues(
            filters.status.as_deref(),
            filters.assignee.as_deref(),
            filters.priority,
            filters.issue_type.as_deref(),
            filters.label.as_deref(),
            filters.sort_by.as_deref(),
        ).map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?;

        serde_json::to_string(&issues)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Get a single issue by ID.
    #[wasm_bindgen]
    pub fn get_issue(&self, id: &str) -> Result<String, JsValue> {
        let issue = self.store.get_issue(id)
            .map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?;

        match issue {
            Some(i) => serde_json::to_string(&i)
                .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e))),
            None => Ok("null".to_string()),
        }
    }

    /// Update an issue from a JSON string.
    #[wasm_bindgen]
    pub fn update_issue(&self, issue_json: &str) -> Result<(), JsValue> {
        let mut issue: Issue = serde_json::from_str(issue_json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        // Ensure updated_at is set
        issue.updated_at = chrono::Utc::now();

        self.store.update_issue(&issue)
            .map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?;

        // Update index (incremental update not implemented, so full rebuild for now or optimize later)
        // For now, let's just update the index for this one issue if we could, but index_issues takes a slice.
        // We'll rebuild lazily or on specific actions if performance is an issue.
        // Given < 1000 issues, full rebuild is fast.
        self.rebuild_search_index()?;

        Ok(())
    }

    /// Create a new issue from a JSON string.
    #[wasm_bindgen]
    pub fn create_issue(&self, issue_json: &str) -> Result<(), JsValue> {
        let mut issue: Issue = serde_json::from_str(issue_json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        // Ensure created_at/updated_at are set if missing (though serde might fail if strict,
        // usually we pass a partial object from JS and fill the rest here, but Issue struct has required fields).
        // Better: JS passes a full object, or we use a separate method for creation.
        // Assuming JS constructs the object with all required fields except maybe ID if it's new.

        if issue.id.is_empty() {
             issue.id = self.store.generate_unique_id("gr", &issue.title, &issue.description, &issue.sender)
                .map_err(|e| JsValue::from_str(&format!("ID generation error: {}", e)))?;
        }

        let now = chrono::Utc::now();
        if issue.created_at.timestamp() == 0 {
            issue.created_at = now;
        }
        issue.updated_at = now;

        self.store.create_issue(&issue)
            .map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?;

        self.rebuild_search_index()?;

        Ok(())
    }

    /// Add a comment to an issue.
    /// Returns the updated issue JSON.
    #[wasm_bindgen]
    pub fn add_comment(&self, issue_id: &str, author: &str, text: &str) -> Result<String, JsValue> {
        let mut issue = self.store.get_issue(issue_id)
            .map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?
            .ok_or_else(|| JsValue::from_str("Issue not found"))?;

        use crate::models::Comment;

        let comment = Comment {
            id: chrono::Utc::now().timestamp(), // Simple numeric ID for now, or could use UUID
            issue_id: issue_id.to_string(),
            author: author.to_string(),
            text: text.to_string(),
            created_at: chrono::Utc::now(),
        };

        issue.comments.push(comment);
        issue.updated_at = chrono::Utc::now();

        self.store.update_issue(&issue)
            .map_err(|e| JsValue::from_str(&format!("Update error: {}", e)))?;

        serde_json::to_string(&issue)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Add a label to an issue.
    /// Returns the updated issue JSON.
    #[wasm_bindgen]
    pub fn add_label(&self, issue_id: &str, label: &str) -> Result<String, JsValue> {
        let mut issue = self.store.get_issue(issue_id)
            .map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?
            .ok_or_else(|| JsValue::from_str("Issue not found"))?;

        if !issue.labels.contains(&label.to_string()) {
            issue.labels.push(label.to_string());
            issue.updated_at = chrono::Utc::now();

            self.store.update_issue(&issue)
                .map_err(|e| JsValue::from_str(&format!("Update error: {}", e)))?;
        }

        serde_json::to_string(&issue)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Remove a label from an issue.
    /// Returns the updated issue JSON.
    #[wasm_bindgen]
    pub fn remove_label(&self, issue_id: &str, label: &str) -> Result<String, JsValue> {
        let mut issue = self.store.get_issue(issue_id)
            .map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?
            .ok_or_else(|| JsValue::from_str("Issue not found"))?;

        if let Some(pos) = issue.labels.iter().position(|l| l == label) {
            issue.labels.remove(pos);
            issue.updated_at = chrono::Utc::now();

            self.store.update_issue(&issue)
                .map_err(|e| JsValue::from_str(&format!("Update error: {}", e)))?;
        }

        serde_json::to_string(&issue)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Search issues using BM25/TF-IDF.
    #[wasm_bindgen]
    pub fn search(&self, query: &str) -> Result<String, JsValue> {
        let index = self.search_index.lock().unwrap();
        let results = index.search(query);

        // Results are (id, score). We want to return the full issues sorted by score.
        let mut issues = Vec::new();
        for (id, _score) in results {
            if let Some(issue) = self.store.get_issue(&id).unwrap_or(None) {
                issues.push(issue);
            }
        }

        serde_json::to_string(&issues)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Export the store state to a JSONL string.
    #[wasm_bindgen]
    pub fn save_to_jsonl(&self) -> Result<String, JsValue> {
        let issues = self.store.list_issues(None, None, None, None, None, None)
             .map_err(|e| JsValue::from_str(&format!("Store error: {}", e)))?;

        let mut output = String::new();
        for issue in issues {
            let line = serde_json::to_string(&issue)
                .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;
            output.push_str(&line);
            output.push('\n');
        }
        Ok(output)
    }
}
