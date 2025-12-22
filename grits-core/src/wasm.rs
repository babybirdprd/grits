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

use crate::models::Issue;
use serde_json::Value;

/// WASM bridge for the VS Code extension UI.
/// Provides pure functions for parsing and updating issue data.
#[wasm_bindgen]
pub struct GritsWasm;

#[wasm_bindgen]
impl GritsWasm {
    /// Parse JSONL content into a JSON array of issues.
    /// Input: Raw JSONL string from VS Code (one issue per line)
    /// Output: JSON array string for React UI
    #[wasm_bindgen]
    pub fn parse_issues(content: &str) -> Result<String, JsValue> {
        let mut issues: Vec<Issue> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let issue: Issue = serde_json::from_str(trimmed)
                .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
            issues.push(issue);
        }

        serde_json::to_string(&issues)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    /// Generic field updater with type validation.
    ///
    /// Arguments:
    /// * `content`: The full JSONL file content
    /// * `id`: The issue ID to find
    /// * `field`: The name of the field to update (e.g., "status", "priority", "title")
    /// * `value_json`: The new value as a JSON string (e.g., "1", "\"done\"", "\"New Title\"")
    ///
    /// Returns: New JSONL string to save to disk
    #[wasm_bindgen]
    pub fn update_field(
        content: &str,
        id: &str,
        field: &str,
        value_json: &str,
    ) -> Result<String, JsValue> {
        // Parse the new value from JSON string (preserves types)
        let new_value: Value = serde_json::from_str(value_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid JSON value: {}", e)))?;

        let mut output = String::new();
        let mut found = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Parse line as dynamic JSON
            let mut doc: Value = serde_json::from_str(trimmed)
                .map_err(|e| JsValue::from_str(&format!("Corrupt DB line: {}", e)))?;

            // Check if this is our target
            if let Some(doc_id) = doc.get("id").and_then(|v| v.as_str()) {
                if doc_id == id {
                    // Apply the update dynamically
                    if let Some(obj) = doc.as_object_mut() {
                        obj.insert(field.to_string(), new_value.clone());

                        // Update the updated_at timestamp
                        obj.insert(
                            "updated_at".to_string(),
                            Value::String(chrono::Utc::now().to_rfc3339()),
                        );
                    }

                    // Validation: Try to convert back to strict Issue struct
                    let _valid_issue: Issue = serde_json::from_value(doc.clone()).map_err(|e| {
                        JsValue::from_str(&format!("Type Error: Field '{}' invalid - {}", field, e))
                    })?;

                    found = true;
                }
            }

            // Write back to string
            let line_str = serde_json::to_string(&doc)
                .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;
            output.push_str(&line_str);
            output.push('\n');
        }

        if !found {
            return Err(JsValue::from_str(&format!("Issue ID not found: {}", id)));
        }

        Ok(output)
    }

    /// Serialize issues array back to JSONL format.
    /// Input: JSON array string of issues
    /// Output: JSONL string (one issue per line)
    #[wasm_bindgen]
    pub fn serialize_issues(issues_json: &str) -> Result<String, JsValue> {
        let issues: Vec<Issue> = serde_json::from_str(issues_json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        let mut output = String::new();
        for issue in issues {
            let line = serde_json::to_string(&issue)
                .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;
            output.push_str(&line);
            output.push('\n');
        }

        Ok(output)
    }

    /// Create a new issue and return the updated JSONL content.
    /// This is a convenience method for the UI.
    #[wasm_bindgen]
    pub fn create_issue(
        content: &str,
        title: &str,
        description: &str,
        issue_type: &str,
        priority: i32,
    ) -> Result<String, JsValue> {
        use chrono::Utc;
        use sha2::{Digest, Sha256};

        // Generate a simple ID
        let now = Utc::now();
        let hash_input = format!("{}{}{}", title, description, now.timestamp_millis());
        let mut hasher = Sha256::new();
        hasher.update(hash_input.as_bytes());
        let hash = hasher.finalize();
        let id = format!("gr-{}", hex::encode(&hash[..4]));

        let issue = Issue {
            id,
            title: title.to_string(),
            description: description.to_string(),
            status: "open".to_string(),
            priority,
            issue_type: issue_type.to_string(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        };

        let new_line = serde_json::to_string(&issue)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;

        let mut output = content.to_string();
        if !output.ends_with('\n') && !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&new_line);
        output.push('\n');

        Ok(output)
    }
}
