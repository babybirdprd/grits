use crate::fs::FileSystem;
use crate::models::Issue;
use crate::store::Store;
use crate::util;
use anyhow::Result;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct MemoryStore {
    // We use Arc<RwLock> to allow interior mutability and cloning
    inner: Arc<RwLock<MemoryStoreInner>>,
}

#[derive(Default)]
struct MemoryStoreInner {
    issues: HashMap<String, Issue>,
    config: HashMap<String, String>,
    #[allow(dead_code)]
    metadata: HashMap<String, String>,
    dirty_issues: HashSet<String>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn import_from_string(&mut self, content: &str) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        inner.issues.clear();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let issue: Issue = serde_json::from_str(trimmed)?;
            inner.issues.insert(issue.id.clone(), issue);
        }
        Ok(())
    }

    pub fn export_to_string(&self) -> Result<String> {
        let inner = self.inner.read().unwrap();
        let mut issues: Vec<_> = inner.issues.values().cloned().collect();
        issues.sort_by(|a, b| a.id.cmp(&b.id));
        let mut output = String::new();
        for issue in issues {
            let json = serde_json::to_string(&issue)?;
            output.push_str(&json);
            output.push('\n');
        }
        Ok(output)
    }
}

impl Store for MemoryStore {
    fn get_config(&self, key: &str) -> Result<Option<String>> {
        let inner = self.inner.read().unwrap();
        Ok(inner.config.get(key).cloned())
    }

    fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        inner.config.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn update_issue(&self, issue: &Issue) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        inner.issues.insert(issue.id.clone(), issue.clone());
        inner.dirty_issues.insert(issue.id.clone());
        Ok(())
    }

    fn list_config(&self) -> Result<Vec<(String, String)>> {
        let inner = self.inner.read().unwrap();
        let mut list: Vec<_> = inner
            .config
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(list)
    }

    fn execute_raw(&self, _sql: &str) -> Result<()> {
        // No-op for memory store
        Ok(())
    }

    fn get_issue(&self, id: &str) -> Result<Option<Issue>> {
        let inner = self.inner.read().unwrap();
        if let Some(issue) = inner.issues.get(id) {
            return Ok(Some(issue.clone()));
        }
        // Handle prefix match
        if id.len() < 36 {
            for (key, issue) in &inner.issues {
                if key.starts_with(id) {
                    return Ok(Some(issue.clone()));
                }
            }
        }
        Ok(None)
    }

    fn list_issues(
        &self,
        status: Option<&str>,
        assignee: Option<&str>,
        priority: Option<i32>,
        issue_type: Option<&str>,
        label: Option<&str>,
        sort_by: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let inner = self.inner.read().unwrap();
        let mut issues: Vec<Issue> = inner.issues.values().cloned().collect();

        // Filtering
        issues.retain(|issue| {
            if let Some(s) = status {
                if issue.status != s {
                    return false;
                }
            }
            if let Some(a) = assignee {
                if a == "unassigned" {
                    if issue.assignee.is_some() && !issue.assignee.as_ref().unwrap().is_empty() {
                        return false;
                    }
                } else if issue.assignee.as_deref() != Some(a) {
                    return false;
                }
            }
            if let Some(p) = priority {
                if issue.priority != p {
                    return false;
                }
            }
            if let Some(t) = issue_type {
                if issue.issue_type != t {
                    return false;
                }
            }
            if let Some(l) = label {
                if !issue.labels.contains(&l.to_string()) {
                    return false;
                }
            }
            true
        });

        // Sorting
        match sort_by {
            Some("updated") => issues.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
            Some("created") => issues.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            Some("priority") => issues.sort_by(|a, b| {
                a.priority
                    .cmp(&b.priority)
                    .then_with(|| b.created_at.cmp(&a.created_at))
            }),
            _ => issues.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        }

        Ok(issues)
    }

    fn import_from_jsonl(&mut self, jsonl_path: &Path, fs: &dyn FileSystem) -> Result<()> {
        if !fs.exists(jsonl_path) {
            return Ok(());
        }
        let reader = fs.open_read(jsonl_path)?;
        let mut inner = self.inner.write().unwrap();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let issue: Issue = serde_json::from_str(&line)?;
            inner.issues.insert(issue.id.clone(), issue);
        }
        Ok(())
    }

    fn generate_unique_id(
        &self,
        prefix: &str,
        title: &str,
        description: &str,
        creator: &str,
    ) -> Result<String> {
        let inner = self.inner.read().unwrap();
        let created_at = Utc::now();
        let base_length = 6;
        let max_length = 8;

        for length in base_length..=max_length {
            for nonce in 0..10 {
                let candidate = util::generate_hash_id(
                    prefix,
                    title,
                    description,
                    creator,
                    created_at,
                    length,
                    nonce,
                );
                if !inner.issues.contains_key(&candidate) {
                    return Ok(candidate);
                }
            }
        }
        Err(anyhow::anyhow!(
            "Failed to generate unique ID after retries"
        ))
    }

    fn create_issue(&self, issue: &Issue) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        inner.issues.insert(issue.id.clone(), issue.clone());
        inner.dirty_issues.insert(issue.id.clone());
        Ok(())
    }

    fn export_to_jsonl(&self, jsonl_path: &Path, fs: &dyn FileSystem) -> Result<()> {
        let mut inner = self.inner.write().unwrap();

        let mut issues: Vec<_> = inner.issues.values().cloned().collect();
        issues.sort_by(|a, b| a.id.cmp(&b.id));

        // Use a temp path or string buffer? The fs trait works with paths.
        // Assuming we can write to a temp file in the same dir.
        let dir = jsonl_path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = jsonl_path.file_name().unwrap_or_default();
        let temp_path = dir.join(format!(".{}.tmp", file_name.to_string_lossy()));

        {
            let file = fs.open_write(&temp_path)?;
            let mut writer = std::io::BufWriter::new(file);

            for issue in &issues {
                let json = serde_json::to_string(issue)?;
                writeln!(writer, "{}", json)?;
            }
            // flush handled by drop usually, but explicit flush is safer if we read immediately
            writer.flush()?;
        }

        fs.rename(&temp_path, jsonl_path)?;

        // Update metadata hash (simplified for memory store)
        // In real SqliteStore we read back and hash. Here we can probably skip or emulate.
        // But to be consistent with Store behavior:

        // Clear dirty issues
        inner.dirty_issues.clear();

        Ok(())
    }
}
