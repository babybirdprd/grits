use crate::fs::FileSystem;
use crate::models::Issue;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::{Comment, Dependency};
use anyhow::Result;
use std::path::Path;

pub trait Store {
    fn get_config(&self, key: &str) -> Result<Option<String>>;
    fn set_config(&self, key: &str, value: &str) -> Result<()>;
    fn update_issue(&self, issue: &Issue) -> Result<()>;
    fn list_config(&self) -> Result<Vec<(String, String)>>;
    fn execute_raw(&self, sql: &str) -> Result<()>;
    fn get_issue(&self, id: &str) -> Result<Option<Issue>>;
    fn list_issues(
        &self,
        status: Option<&str>,
        assignee: Option<&str>,
        priority: Option<i32>,
        issue_type: Option<&str>,
        label: Option<&str>,
        sort_by: Option<&str>,
    ) -> Result<Vec<Issue>>;
    fn import_from_jsonl(&mut self, jsonl_path: &Path, fs: &dyn FileSystem) -> Result<()>;
    fn generate_unique_id(
        &self,
        prefix: &str,
        title: &str,
        description: &str,
        creator: &str,
    ) -> Result<String>;
    fn create_issue(&self, issue: &Issue) -> Result<()>;
    fn export_to_jsonl(&self, jsonl_path: &Path, fs: &dyn FileSystem) -> Result<()>;
}

#[cfg(not(target_arch = "wasm32"))]
pub use sqlite_impl::SqliteStore;

#[cfg(not(target_arch = "wasm32"))]
pub mod sqlite_impl {
    use super::*;
    use crate::util;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use rusqlite::{params, Connection};
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;
    use std::io::{BufRead, BufWriter, Write};

    pub struct SqliteStore {
        conn: Connection,
    }

    impl SqliteStore {
        pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
            let conn = Connection::open(&path)?;

            conn.execute_batch(
                "
                CREATE TABLE IF NOT EXISTS issues (
                    id TEXT PRIMARY KEY,
                    content_hash TEXT DEFAULT '',
                    title TEXT,
                    description TEXT,
                    design TEXT DEFAULT '',
                    acceptance_criteria TEXT DEFAULT '',
                    notes TEXT DEFAULT '',
                    status TEXT,
                    priority INTEGER,
                    issue_type TEXT,
                    assignee TEXT,
                    estimated_minutes INTEGER,
                    created_at TEXT,
                    updated_at TEXT,
                    closed_at TEXT,
                    external_ref TEXT,
                    sender TEXT DEFAULT '',
                    ephemeral BOOLEAN DEFAULT 0,
                    replies_to TEXT DEFAULT '',
                    relates_to TEXT DEFAULT '',
                    duplicate_of TEXT DEFAULT '',
                    superseded_by TEXT DEFAULT '',
                    deleted_at TEXT,
                    deleted_by TEXT DEFAULT '',
                    delete_reason TEXT DEFAULT '',
                    original_type TEXT DEFAULT '',
                    affected_symbols TEXT DEFAULT '',
                    solid_volume TEXT,
                    topology_hash TEXT DEFAULT '',
                    is_solid BOOLEAN DEFAULT 0
                );

                CREATE TABLE IF NOT EXISTS labels (
                    issue_id TEXT,
                    label TEXT,
                    PRIMARY KEY (issue_id, label)
                );

                CREATE TABLE IF NOT EXISTS dependencies (
                    issue_id TEXT,
                    depends_on_id TEXT,
                    type TEXT,
                    created_at TEXT,
                    created_by TEXT,
                    PRIMARY KEY (issue_id, depends_on_id, type)
                );

                CREATE TABLE IF NOT EXISTS comments (
                    id TEXT PRIMARY KEY,
                    issue_id TEXT,
                    author TEXT,
                    text TEXT,
                    created_at TEXT
                );

                CREATE TABLE IF NOT EXISTS dirty_issues (
                    issue_id TEXT PRIMARY KEY
                );

                CREATE TABLE IF NOT EXISTS metadata (
                    key TEXT PRIMARY KEY,
                    value TEXT
                );

                CREATE TABLE IF NOT EXISTS config (
                    key TEXT PRIMARY KEY,
                    value TEXT
                );
                ",
            )?;

            Ok(SqliteStore { conn })
        }

        fn export_all_issues(&self) -> Result<Vec<Issue>> {
            // Fetch all related data first (bulk)
            let labels_map = self.get_all_labels()?;
            let deps_map = self.get_all_dependencies()?;
            let comments_map = self.get_all_comments()?;

            let mut stmt = self.conn.prepare(
                "SELECT
                    id, content_hash, title, description, design, acceptance_criteria, notes,
                    status, priority, issue_type, assignee, estimated_minutes,
                    created_at, updated_at, closed_at, external_ref,
                    sender, ephemeral, replies_to, relates_to, duplicate_of, superseded_by,
                    deleted_at, deleted_by, delete_reason, original_type,
                    affected_symbols, solid_volume, topology_hash, is_solid
                FROM issues
                ORDER BY id",
            )?;

            let issue_iter = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let created_at_s: String = row.get(12)?;
                let updated_at_s: String = row.get(13)?;
                let closed_at_s: Option<String> = row.get(14)?;
                let deleted_at_s: Option<String> = row.get(22)?;

                let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
                let updated_at = parse_timestamp(&updated_at_s).unwrap_or_else(|| Utc::now());
                let closed_at = closed_at_s.and_then(|s| parse_timestamp(&s));
                let deleted_at = deleted_at_s.and_then(|s| parse_timestamp(&s));

                let relates_to_s: String = row.get(19).unwrap_or_default();
                let relates_to = if relates_to_s.is_empty() {
                    Vec::new()
                } else {
                    serde_json::from_str(&relates_to_s).unwrap_or_default()
                };

                let affected_symbols_s: String = row.get(26).unwrap_or_default();
                let affected_symbols = if affected_symbols_s.is_empty() {
                    Vec::new()
                } else {
                    serde_json::from_str(&affected_symbols_s).unwrap_or_default()
                };

                Ok(Issue {
                    id: id.clone(),
                    content_hash: row.get(1).unwrap_or_default(),
                    title: row.get(2).unwrap_or_default(),
                    description: row.get(3).unwrap_or_default(),
                    design: row.get(4).unwrap_or_default(),
                    acceptance_criteria: row.get(5).unwrap_or_default(),
                    notes: row.get(6).unwrap_or_default(),
                    status: row.get(7).unwrap_or_default(),
                    priority: row.get(8).unwrap_or_default(),
                    issue_type: row.get(9).unwrap_or_default(),
                    assignee: row.get(10)?,
                    estimated_minutes: row.get(11)?,
                    created_at,
                    updated_at,
                    closed_at,
                    external_ref: row.get(15)?,
                    sender: row.get(16).unwrap_or_default(),
                    ephemeral: row.get(17).unwrap_or(false),
                    replies_to: row.get(18).unwrap_or_default(),
                    relates_to,
                    duplicate_of: row.get(20).unwrap_or_default(),
                    superseded_by: row.get(21).unwrap_or_default(),

                    deleted_at,
                    deleted_by: row.get(23).unwrap_or_default(),
                    delete_reason: row.get(24).unwrap_or_default(),
                    original_type: row.get(25).unwrap_or_default(),

                    labels: labels_map.get(&id).cloned().unwrap_or_default(),
                    dependencies: deps_map.get(&id).cloned().unwrap_or_default(),
                    comments: comments_map.get(&id).cloned().unwrap_or_default(),

                    affected_symbols,
                    solid_volume: row.get(27).ok(),
                    topology_hash: row.get(28).unwrap_or_default(),
                    is_solid: row.get(29).unwrap_or(false),
                })
            })?;

            let mut issues = Vec::new();
            for issue in issue_iter {
                issues.push(issue?);
            }
            Ok(issues)
        }

        fn get_all_labels(&self) -> Result<HashMap<String, Vec<String>>> {
            let mut stmt = self.conn.prepare("SELECT issue_id, label FROM labels")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            let mut map: HashMap<String, Vec<String>> = HashMap::new();
            for row in rows {
                let (id, label) = row?;
                map.entry(id).or_default().push(label);
            }
            Ok(map)
        }

        fn get_all_dependencies(&self) -> Result<HashMap<String, Vec<Dependency>>> {
            let mut stmt = self.conn.prepare(
                "SELECT issue_id, depends_on_id, type, created_at, created_by FROM dependencies",
            )?;
            let rows = stmt.query_map([], |row| {
                let created_at_s: String = row.get(3)?;
                let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());

                Ok(Dependency {
                    issue_id: row.get(0)?,
                    depends_on_id: row.get(1)?,
                    type_: row.get(2)?,
                    created_at,
                    created_by: row.get(4)?,
                })
            })?;

            let mut map: HashMap<String, Vec<Dependency>> = HashMap::new();
            for row in rows {
                let dep = row?;
                map.entry(dep.issue_id.clone()).or_default().push(dep);
            }
            Ok(map)
        }

        fn get_all_comments(&self) -> Result<HashMap<String, Vec<Comment>>> {
            let mut stmt = self
                .conn
                .prepare("SELECT id, issue_id, author, text, created_at FROM comments")?;
            let rows = stmt.query_map([], |row| {
                let created_at_s: String = row.get(4)?;
                let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());

                Ok(Comment {
                    id: row.get(0)?,
                    issue_id: row.get(1)?,
                    author: row.get(2)?,
                    text: row.get(3)?,
                    created_at,
                })
            })?;

            let mut map: HashMap<String, Vec<Comment>> = HashMap::new();
            for row in rows {
                let comment = row?;
                map.entry(comment.issue_id.clone())
                    .or_default()
                    .push(comment);
            }
            Ok(map)
        }
    }

    impl Store for SqliteStore {
        fn get_config(&self, key: &str) -> Result<Option<String>> {
            let mut stmt = self
                .conn
                .prepare("SELECT value FROM config WHERE key = ?1")?;
            let mut rows = stmt.query([key])?;
            if let Some(row) = rows.next()? {
                Ok(Some(row.get(0)?))
            } else {
                Ok(None)
            }
        }

        fn set_config(&self, key: &str, value: &str) -> Result<()> {
            self.conn.execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
            Ok(())
        }

        fn update_issue(&self, issue: &Issue) -> Result<()> {
            // Serialize nested fields
            let relates_to_json = serde_json::to_string(&issue.relates_to).unwrap_or_default();
            let affected_symbols_json =
                serde_json::to_string(&issue.affected_symbols).unwrap_or_default();

            // Update main issue record
            self.conn.execute(
                "UPDATE issues SET
                    content_hash = ?2, title = ?3, description = ?4, design = ?5, acceptance_criteria = ?6, notes = ?7,
                    status = ?8, priority = ?9, issue_type = ?10, assignee = ?11, estimated_minutes = ?12,
                    created_at = ?13, updated_at = ?14, closed_at = ?15, external_ref = ?16,
                    sender = ?17, ephemeral = ?18, replies_to = ?19, relates_to = ?20,
                    duplicate_of = ?21, superseded_by = ?22,
                    deleted_at = ?23, deleted_by = ?24, delete_reason = ?25, original_type = ?26,
                    affected_symbols = ?27, solid_volume = ?28, topology_hash = ?29, is_solid = ?30
                WHERE id = ?1",
                params![
                    &issue.id,
                    &issue.content_hash,
                    &issue.title,
                    &issue.description,
                    &issue.design,
                    &issue.acceptance_criteria,
                    &issue.notes,
                    &issue.status,
                    &issue.priority,
                    &issue.issue_type,
                    &issue.assignee,
                    &issue.estimated_minutes,
                    issue.created_at.to_rfc3339(),
                    issue.updated_at.to_rfc3339(),
                    issue.closed_at.map(|t| t.to_rfc3339()),
                    &issue.external_ref,
                    &issue.sender,
                    issue.ephemeral,
                    &issue.replies_to,
                    relates_to_json,
                    &issue.duplicate_of,
                    &issue.superseded_by,
                    issue.deleted_at.map(|t| t.to_rfc3339()),
                    &issue.deleted_by,
                    &issue.delete_reason,
                    &issue.original_type,
                    affected_symbols_json,
                    &issue.solid_volume,
                    &issue.topology_hash,
                    issue.is_solid,
                ],
            )?;

            // Replace labels
            self.conn
                .execute("DELETE FROM labels WHERE issue_id = ?1", params![&issue.id])?;
            for label in &issue.labels {
                self.conn.execute(
                    "INSERT INTO labels (issue_id, label) VALUES (?1, ?2)",
                    params![&issue.id, label],
                )?;
            }

            // Replace dependencies
            self.conn.execute(
                "DELETE FROM dependencies WHERE issue_id = ?1",
                params![&issue.id],
            )?;
            for dep in &issue.dependencies {
                self.conn.execute(
                    "INSERT INTO dependencies (issue_id, depends_on_id, type, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![&dep.issue_id, &dep.depends_on_id, &dep.type_, dep.created_at.to_rfc3339(), &dep.created_by],
                )?;
            }

            // Mark as dirty
            self.conn.execute(
                "INSERT OR IGNORE INTO dirty_issues (issue_id) VALUES (?1)",
                params![&issue.id],
            )?;
            Ok(())
        }

        fn list_config(&self) -> Result<Vec<(String, String)>> {
            let mut stmt = self
                .conn
                .prepare("SELECT key, value FROM config ORDER BY key")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok(result)
        }

        fn execute_raw(&self, sql: &str) -> Result<()> {
            self.conn.execute_batch(sql)?;
            Ok(())
        }

        fn get_issue(&self, id: &str) -> Result<Option<Issue>> {
            // Handle short ID (prefix match)
            let query_id = if id.len() < 36 {
                format!("{}%", id)
            } else {
                id.to_string()
            };

            let mut stmt = self.conn.prepare(
                "SELECT
                    id, content_hash, title, description, design, acceptance_criteria, notes,
                    status, priority, issue_type, assignee, estimated_minutes,
                    created_at, updated_at, closed_at, external_ref,
                    sender, ephemeral, replies_to, relates_to, duplicate_of, superseded_by,
                    deleted_at, deleted_by, delete_reason, original_type,
                    affected_symbols, solid_volume, topology_hash, is_solid
                FROM issues
                WHERE id LIKE ?1
                LIMIT 1",
            )?;

            let mut rows = stmt.query([&query_id])?;

            let row = if let Some(row) = rows.next()? {
                row
            } else {
                return Ok(None);
            };

            let id: String = row.get(0)?;

            // Fetch children
            let mut labels = Vec::new();
            let mut labels_stmt = self
                .conn
                .prepare("SELECT label FROM labels WHERE issue_id = ?1")?;
            let labels_rows = labels_stmt.query_map([&id], |r| r.get(0))?;
            for l in labels_rows {
                labels.push(l?);
            }

            let mut deps = Vec::new();
            let mut deps_stmt = self.conn.prepare("SELECT depends_on_id, type, created_at, created_by FROM dependencies WHERE issue_id = ?1")?;
            let deps_rows = deps_stmt.query_map([&id], |r| {
                let created_at_s: String = r.get(2)?;
                let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
                Ok(Dependency {
                    issue_id: id.clone(),
                    depends_on_id: r.get(0)?,
                    type_: r.get(1)?,
                    created_at,
                    created_by: r.get(3)?,
                })
            })?;
            for d in deps_rows {
                deps.push(d?);
            }

            let mut comments = Vec::new();
            let mut comments_stmt = self.conn.prepare("SELECT id, author, text, created_at FROM comments WHERE issue_id = ?1 ORDER BY created_at")?;
            let comments_rows = comments_stmt.query_map([&id], |r| {
                let created_at_s: String = r.get(3)?;
                let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
                Ok(Comment {
                    id: r.get(0)?,
                    issue_id: id.clone(),
                    author: r.get(1)?,
                    text: r.get(2)?,
                    created_at,
                })
            })?;
            for c in comments_rows {
                comments.push(c?);
            }

            let created_at_s: String = row.get(12)?;
            let updated_at_s: String = row.get(13)?;
            let closed_at_s: Option<String> = row.get(14)?;
            let deleted_at_s: Option<String> = row.get(22)?;

            let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
            let updated_at = parse_timestamp(&updated_at_s).unwrap_or_else(|| Utc::now());
            let closed_at = closed_at_s.and_then(|s| parse_timestamp(&s));
            let deleted_at = deleted_at_s.and_then(|s| parse_timestamp(&s));

            let relates_to_s: String = row.get(19).unwrap_or_default();
            let relates_to = if relates_to_s.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&relates_to_s).unwrap_or_default()
            };

            let affected_symbols_s: String = row.get(26).unwrap_or_default();
            let affected_symbols = if affected_symbols_s.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&affected_symbols_s).unwrap_or_default()
            };

            Ok(Some(Issue {
                id,
                content_hash: row.get(1).unwrap_or_default(),
                title: row.get(2).unwrap_or_default(),
                description: row.get(3).unwrap_or_default(),
                design: row.get(4).unwrap_or_default(),
                acceptance_criteria: row.get(5).unwrap_or_default(),
                notes: row.get(6).unwrap_or_default(),
                status: row.get(7).unwrap_or_default(),
                priority: row.get(8).unwrap_or_default(),
                issue_type: row.get(9).unwrap_or_default(),
                assignee: row.get(10)?,
                estimated_minutes: row.get(11)?,
                created_at,
                updated_at,
                closed_at,
                external_ref: row.get(15)?,
                sender: row.get(16).unwrap_or_default(),
                ephemeral: row.get(17).unwrap_or(false),
                replies_to: row.get(18).unwrap_or_default(),
                relates_to,
                duplicate_of: row.get(20).unwrap_or_default(),
                superseded_by: row.get(21).unwrap_or_default(),
                deleted_at,
                deleted_by: row.get(23).unwrap_or_default(),
                delete_reason: row.get(24).unwrap_or_default(),
                original_type: row.get(25).unwrap_or_default(),
                labels,
                dependencies: deps,
                comments,
                affected_symbols,
                solid_volume: row.get(27).ok(),
                topology_hash: row.get(28).unwrap_or_default(),
                is_solid: row.get(29).unwrap_or(false),
            }))
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
            let mut conditions = Vec::new();
            let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(s) = status {
                conditions.push("issues.status = ?");
                args.push(Box::new(s.to_string()));
            }

            if let Some(a) = assignee {
                if a == "unassigned" {
                    conditions.push("(issues.assignee IS NULL OR issues.assignee = '')");
                } else {
                    conditions.push("issues.assignee = ?");
                    args.push(Box::new(a.to_string()));
                }
            }

            if let Some(p) = priority {
                conditions.push("issues.priority = ?");
                args.push(Box::new(p));
            }

            if let Some(t) = issue_type {
                conditions.push("issues.issue_type = ?");
                args.push(Box::new(t.to_string()));
            }

            let mut sql = "SELECT issues.id, issues.title, issues.description, issues.status, issues.priority, issues.issue_type, issues.created_at, issues.updated_at, issues.assignee, issues.affected_symbols FROM issues".to_string();

            if let Some(l) = label {
                sql.push_str(" INNER JOIN labels ON issues.id = labels.issue_id");
                conditions.push("labels.label = ?");
                args.push(Box::new(l.to_string()));
            }

            if !conditions.is_empty() {
                sql.push_str(" WHERE ");
                sql.push_str(&conditions.join(" AND "));
            }

            let order_clause = match sort_by {
                Some("updated") => "ORDER BY issues.updated_at DESC",
                Some("created") => "ORDER BY issues.created_at DESC",
                Some("priority") => "ORDER BY issues.priority ASC, issues.created_at DESC",
                _ => "ORDER BY issues.created_at DESC",
            };
            sql.push_str(" ");
            sql.push_str(order_clause);

            let mut stmt = self.conn.prepare(&sql)?;

            let issue_iter = stmt.query_map(rusqlite::params_from_iter(args.iter()), |row| {
                let created_at_s: String = row.get(6)?;
                let updated_at_s: String = row.get(7)?;

                let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
                let updated_at = parse_timestamp(&updated_at_s).unwrap_or_else(|| Utc::now());

                let affected_symbols_s: String = row.get(9).unwrap_or_default();
                let affected_symbols = if affected_symbols_s.is_empty() {
                    Vec::new()
                } else {
                    serde_json::from_str(&affected_symbols_s).unwrap_or_default()
                };

                Ok(Issue {
                    id: row.get(0)?,
                    content_hash: String::new(),
                    title: row.get(1)?,
                    description: row.get(2)?,
                    design: String::new(),
                    acceptance_criteria: String::new(),
                    notes: String::new(),
                    status: row.get(3)?,
                    priority: row.get(4)?,
                    issue_type: row.get(5)?,
                    assignee: row.get(8)?,
                    estimated_minutes: None,
                    created_at,
                    updated_at,
                    closed_at: None,
                    external_ref: None,
                    sender: String::new(),
                    ephemeral: false,
                    replies_to: String::new(),
                    relates_to: Vec::new(),
                    duplicate_of: String::new(),
                    superseded_by: String::new(),

                    deleted_at: None,
                    deleted_by: String::new(),
                    delete_reason: String::new(),
                    original_type: String::new(),

                    labels: Vec::new(),
                    dependencies: Vec::new(),
                    comments: Vec::new(),
                    affected_symbols,
                    solid_volume: None,
                    topology_hash: String::new(),
                    is_solid: false,
                })
            })?;

            let mut issues = Vec::new();
            for issue in issue_iter {
                issues.push(issue?);
            }
            Ok(issues)
        }

        fn import_from_jsonl(&mut self, jsonl_path: &Path, fs: &dyn FileSystem) -> Result<()> {
            if !fs.exists(jsonl_path) {
                return Ok(());
            }

            let reader = fs.open_read(jsonl_path)?;

            let tx = self.conn.transaction()?;

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let issue: Issue = serde_json::from_str(&line)?;

                let relates_to_json = serde_json::to_string(&issue.relates_to).unwrap_or_default();
                let affected_symbols_json =
                    serde_json::to_string(&issue.affected_symbols).unwrap_or_default();

                tx.execute(
                    "INSERT OR REPLACE INTO issues (
                        id, content_hash, title, description, design, acceptance_criteria, notes,
                        status, priority, issue_type, assignee, estimated_minutes,
                        created_at, updated_at, closed_at, external_ref,
                        sender, ephemeral, replies_to, relates_to, duplicate_of, superseded_by,
                        deleted_at, deleted_by, delete_reason, original_type,
                        affected_symbols, solid_volume, topology_hash, is_solid
                    )
                    VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                        ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22,
                        ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30
                    )",
                    params![
                        &issue.id,
                        &issue.content_hash,
                        &issue.title,
                        &issue.description,
                        &issue.design,
                        &issue.acceptance_criteria,
                        &issue.notes,
                        &issue.status,
                        &issue.priority,
                        &issue.issue_type,
                        &issue.assignee,
                        &issue.estimated_minutes,
                        issue.created_at.to_rfc3339(),
                        issue.updated_at.to_rfc3339(),
                        issue.closed_at.map(|t| t.to_rfc3339()),
                        &issue.external_ref,
                        &issue.sender,
                        issue.ephemeral,
                        &issue.replies_to,
                        relates_to_json,
                        &issue.duplicate_of,
                        &issue.superseded_by,
                        issue.deleted_at.map(|t| t.to_rfc3339()),
                        &issue.deleted_by,
                        &issue.delete_reason,
                        &issue.original_type,
                        affected_symbols_json,
                        &issue.solid_volume,
                        &issue.topology_hash,
                        issue.is_solid,
                    ],
                )?;

                tx.execute("DELETE FROM labels WHERE issue_id = ?1", params![&issue.id])?;
                for label in &issue.labels {
                    tx.execute(
                        "INSERT INTO labels (issue_id, label) VALUES (?1, ?2)",
                        params![&issue.id, label],
                    )?;
                }

                tx.execute(
                    "DELETE FROM dependencies WHERE issue_id = ?1",
                    params![&issue.id],
                )?;
                for dep in &issue.dependencies {
                    tx.execute(
                        "INSERT INTO dependencies (issue_id, depends_on_id, type, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![&dep.issue_id, &dep.depends_on_id, &dep.type_, dep.created_at.to_rfc3339(), &dep.created_by],
                    )?;
                }

                let mut stmt =
                    tx.prepare_cached("SELECT author, text FROM comments WHERE issue_id = ?1")?;
                let existing_comments: std::collections::HashSet<(String, String)> = stmt
                    .query_map([&issue.id], |row| Ok((row.get(0)?, row.get(1)?)))?
                    .filter_map(Result::ok)
                    .collect();
                drop(stmt);

                for comment in &issue.comments {
                    if !existing_comments.contains(&(comment.author.clone(), comment.text.clone()))
                    {
                        tx.execute(
                            "INSERT INTO comments (id, issue_id, author, text, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![&comment.id, &issue.id, &comment.author, &comment.text, comment.created_at.to_rfc3339()],
                        )?;
                    }
                }
            }

            tx.commit()?;
            Ok(())
        }

        fn generate_unique_id(
            &self,
            prefix: &str,
            title: &str,
            description: &str,
            creator: &str,
        ) -> Result<String> {
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
                    let count: i64 = self.conn.query_row(
                        "SELECT COUNT(*) FROM issues WHERE id = ?1",
                        params![&candidate],
                        |row| row.get(0),
                    )?;

                    if count == 0 {
                        return Ok(candidate);
                    }
                }
            }
            Err(anyhow::anyhow!(
                "Failed to generate unique ID after retries"
            ))
        }

        fn create_issue(&self, issue: &Issue) -> Result<()> {
            let relates_to_json = serde_json::to_string(&issue.relates_to).unwrap_or_default();
            let affected_symbols_json =
                serde_json::to_string(&issue.affected_symbols).unwrap_or_default();

            self.conn.execute(
                "INSERT INTO issues (
                    id, content_hash, title, description, design, acceptance_criteria, notes,
                    status, priority, issue_type, assignee, estimated_minutes,
                    created_at, updated_at, closed_at, external_ref,
                    sender, ephemeral, replies_to, relates_to, duplicate_of, superseded_by,
                    deleted_at, deleted_by, delete_reason, original_type,
                    affected_symbols, solid_volume, topology_hash, is_solid
                )
                VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                    ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22,
                    ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30
                )",
                params![
                    &issue.id,
                    &issue.content_hash,
                    &issue.title,
                    &issue.description,
                    &issue.design,
                    &issue.acceptance_criteria,
                    &issue.notes,
                    &issue.status,
                    &issue.priority,
                    &issue.issue_type,
                    &issue.assignee,
                    &issue.estimated_minutes,
                    issue.created_at.to_rfc3339(),
                    issue.updated_at.to_rfc3339(),
                    issue.closed_at.map(|t| t.to_rfc3339()),
                    &issue.external_ref,
                    &issue.sender,
                    issue.ephemeral,
                    &issue.replies_to,
                    relates_to_json,
                    &issue.duplicate_of,
                    &issue.superseded_by,
                    issue.deleted_at.map(|t| t.to_rfc3339()),
                    &issue.deleted_by,
                    &issue.delete_reason,
                    &issue.original_type,
                    affected_symbols_json,
                    &issue.solid_volume,
                    &issue.topology_hash,
                    issue.is_solid,
                ],
            )?;

            for label in &issue.labels {
                self.conn.execute(
                    "INSERT INTO labels (issue_id, label) VALUES (?1, ?2)",
                    params![&issue.id, label],
                )?;
            }

            for dep in &issue.dependencies {
                self.conn.execute(
                    "INSERT INTO dependencies (issue_id, depends_on_id, type, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![&dep.issue_id, &dep.depends_on_id, &dep.type_, dep.created_at.to_rfc3339(), &dep.created_by],
                )?;
            }

            for comment in &issue.comments {
                self.conn.execute(
                    "INSERT INTO comments (id, issue_id, author, text, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![&comment.id, &comment.issue_id, &comment.author, &comment.text, comment.created_at.to_rfc3339()],
                )?;
            }

            self.conn.execute(
                "INSERT OR IGNORE INTO dirty_issues (issue_id) VALUES (?1)",
                params![&issue.id],
            )?;
            Ok(())
        }

        fn export_to_jsonl(&self, jsonl_path: &Path, fs: &dyn FileSystem) -> Result<()> {
            let issues = self.export_all_issues()?;

            let dir = jsonl_path.parent().unwrap_or_else(|| Path::new("."));
            let file_name = jsonl_path.file_name().unwrap_or_default();
            let temp_path = dir.join(format!(".{}.tmp", file_name.to_string_lossy()));

            {
                let file = fs.open_write(&temp_path)?;
                let mut writer = BufWriter::new(file);

                for issue in &issues {
                    let json = serde_json::to_string(issue)?;
                    writeln!(writer, "{}", json)?;
                }
                writer.flush()?;
            }

            fs.rename(&temp_path, jsonl_path)?;

            let mut reader = fs.open_read(jsonl_path)?;
            let mut hasher = Sha256::new();
            std::io::copy(&mut reader, &mut hasher)?;
            let hash = hex::encode(hasher.finalize());

            self.conn.execute(
                "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
                params!["jsonl_content_hash", &hash],
            )?;

            self.conn.execute("DELETE FROM dirty_issues", [])?;

            Ok(())
        }
    }

    fn parse_timestamp(s: &str) -> Option<DateTime<Utc>> {
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.with_timezone(&Utc));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
        }
        None
    }
}
