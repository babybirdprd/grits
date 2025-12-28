//! MCP Server for Grits - Agent Engine
//!
//! Provides structured tool access for AI agents via the Model Context Protocol.

use chrono::Utc;
use git2::Repository;
use grits_core::{Issue, SqliteStore, StdFileSystem, Store};
use rmcp::{
    handler::server::tool::ToolRouter,
    model::{CallToolResult, Content},
    schemars::JsonSchema,
    tool, tool_router, ErrorData as McpError,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

// Tool parameter types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListIssuesParams {
    /// Filter by status (e.g., "open", "closed", "in-progress")
    #[serde(default)]
    pub status: Option<String>,
    /// Filter by assignee
    #[serde(default)]
    pub assignee: Option<String>,
    /// Filter by priority (1 = highest)
    #[serde(default)]
    pub priority: Option<i32>,
    /// Filter by issue type (e.g., "bug", "feature", "epic")
    #[serde(default)]
    pub issue_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateIssueParams {
    /// Title of the issue (required)
    pub title: String,
    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,
    /// Issue type: "bug", "feature", "task", "epic"
    #[serde(default = "default_issue_type")]
    pub issue_type: String,
    /// Priority: 1 (highest) to 5 (lowest)
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_issue_type() -> String {
    "task".to_string()
}

fn default_priority() -> i32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateIssueParams {
    /// Issue ID
    pub id: String,
    /// New title
    #[serde(default)]
    pub title: Option<String>,
    /// New description
    #[serde(default)]
    pub description: Option<String>,
    /// New status
    #[serde(default)]
    pub status: Option<String>,
    /// New priority
    #[serde(default)]
    pub priority: Option<i32>,
    /// New assignee
    #[serde(default)]
    pub assignee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueIdParam {
    /// Issue ID
    pub id: String,
}

// Tool result types
#[derive(Debug, Serialize, Deserialize)]
pub struct IssueSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: i32,
    pub issue_type: String,
    pub assignee: Option<String>,
}

impl From<&Issue> for IssueSummary {
    fn from(issue: &Issue) -> Self {
        Self {
            id: issue.id.clone(),
            title: issue.title.clone(),
            status: issue.status.clone(),
            priority: issue.priority,
            issue_type: issue.issue_type.clone(),
            assignee: issue.assignee.clone(),
        }
    }
}

// ============================================================
// STRATEGIC MCP TOOL PARAMETERS
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct EmptyParams {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetNextTaskParams {
    /// Optional current file path to boost context-relevant issues
    #[serde(default)]
    pub current_file: Option<String>,
    /// Optional assignee to filter by
    #[serde(default)]
    pub assignee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchIssuesParams {
    /// Natural language search query
    pub query: String,
    /// Maximum results to return (default: 10)
    #[serde(default)]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CleanupStaleParams {
    /// Days of inactivity to consider stale (default: 30)
    #[serde(default)]
    pub days_inactive: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SummarizeSprintParams {
    /// Number of days to summarize (default: 7)
    #[serde(default)]
    pub days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindRelatedIssuesParams {
    /// File path to find related issues for
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SuggestIssueForErrorParams {
    /// Error message to match against issues
    pub error_message: String,
    /// Maximum results (default: 5)
    #[serde(default)]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InferIssueFromDiffParams {
    /// Git diff content (optional - uses HEAD~1..HEAD if not provided)
    #[serde(default)]
    pub diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BulkTriageParams {
    /// Issue IDs to triage
    pub issue_ids: Vec<String>,
    /// New status to apply (optional)
    #[serde(default)]
    pub status: Option<String>,
    /// New priority to apply (optional)
    #[serde(default)]
    pub priority: Option<i32>,
    /// New assignee to apply (optional)
    #[serde(default)]
    pub assignee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LinkCommitToIssuesParams {
    /// Commit message to parse for issue references
    pub commit_message: String,
    /// Optional commit hash for the link
    #[serde(default)]
    pub commit_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateIssueFromTodoParams {
    /// File path to scan for TODOs
    pub file_path: String,
    /// Optional line number to target specific TODO
    #[serde(default)]
    pub line_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetIssueTopologyParams {
    pub issue_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateArchitecturalChangeParams {
    pub file_path: String,
    pub content: String,
}

// Results types for strategic tools are now imported from grits_core::strategic

/// MCP Server for Grits issue tracking
#[derive(Clone)]
pub struct GritsServer {
    db_path: Arc<PathBuf>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl GritsServer {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path: Arc::new(db_path),
            tool_router: Self::tool_router(),
        }
    }

    fn get_store(&self) -> Result<SqliteStore, McpError> {
        // Auto-create database if it doesn't exist (auto-onboard for AI agents)
        if !self.db_path.exists() {
            if let Some(parent) = self.db_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    McpError::internal_error(format!("Failed to create .grits dir: {}", e), None)
                })?;
            }
        }
        let mut store = SqliteStore::open(&*self.db_path)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // AUTO-IMPORT: Bring in any changes from the Human Engine (UI)
        let jsonl_path = self.db_path.parent().unwrap().join("issues.jsonl");
        if jsonl_path.exists() {
            let fs = StdFileSystem;
            let _ = store.import_from_jsonl(&jsonl_path, &fs);
        }

        Ok(store)
    }

    fn auto_export(&self, store: &SqliteStore) -> Result<(), McpError> {
        let jsonl_path = self.db_path.parent().unwrap().join("issues.jsonl");
        let fs = StdFileSystem;
        store
            .export_to_jsonl(&jsonl_path, &fs)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "List issues with optional filters. Returns issue summaries as JSON.")]
    async fn list_issues(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ListIssuesParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;

        let issues = store
            .list_issues(
                params.0.status.as_deref(),
                params.0.assignee.as_deref(),
                params.0.priority,
                params.0.issue_type.as_deref(),
                None,
                None,
            )
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let summaries: Vec<IssueSummary> = issues.iter().map(IssueSummary::from).collect();
        let json = serde_json::to_string_pretty(&summaries)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Create a new issue. Returns the created issue ID.")]
    async fn create_issue(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CreateIssueParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;

        let user = store
            .get_config("user.name")
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .unwrap_or_else(|| "unknown".to_string());

        let prefix = store
            .get_config("issue_id_prefix")
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .unwrap_or_else(|| "gr".to_string());

        let description = params.0.description.clone().unwrap_or_default();

        let id = store
            .generate_unique_id(&prefix, &params.0.title, &description, &user)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let now = Utc::now();
        let issue = Issue {
            id: id.clone(),
            title: params.0.title.clone(),
            description,
            status: "open".to_string(),
            priority: params.0.priority,
            issue_type: params.0.issue_type.clone(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        };

        store
            .create_issue(&issue)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // AUTO-EXPORT: Push changes to Human Engine (UI)
        self.auto_export(&store)?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created issue: {}",
            id
        ))]))
    }

    #[tool(description = "Update an existing issue by ID. Only provided fields are updated.")]
    async fn update_issue(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<UpdateIssueParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;

        let issue = store
            .get_issue(&params.0.id)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .ok_or_else(|| {
                McpError::invalid_params(format!("Issue not found: {}", params.0.id), None)
            })?;

        let mut updated = issue.clone();

        if let Some(ref title) = params.0.title {
            updated.title = title.clone();
        }
        if let Some(ref description) = params.0.description {
            updated.description = description.clone();
        }
        if let Some(ref status) = params.0.status {
            updated.status = status.clone();
        }
        if let Some(priority) = params.0.priority {
            updated.priority = priority;
        }
        if let Some(ref assignee) = params.0.assignee {
            updated.assignee = Some(assignee.clone());
        }
        updated.updated_at = Utc::now();

        store
            .update_issue(&updated)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // AUTO-EXPORT: Push changes to Human Engine (UI)
        self.auto_export(&store)?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Updated issue: {}",
            updated.id
        ))]))
    }

    #[tool(description = "Close an issue by ID.")]
    async fn close_issue(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<IssueIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;

        let issue = store
            .get_issue(&params.0.id)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .ok_or_else(|| {
                McpError::invalid_params(format!("Issue not found: {}", params.0.id), None)
            })?;

        let mut closed = issue.clone();
        closed.status = "closed".to_string();
        closed.closed_at = Some(Utc::now());
        closed.updated_at = Utc::now();

        store
            .update_issue(&closed)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // AUTO-EXPORT: Push changes to Human Engine (UI)
        self.auto_export(&store)?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Closed issue: {} ({})",
            closed.id, closed.title
        ))]))
    }

    #[tool(description = "Get detailed information about a specific issue.")]
    async fn get_issue(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<IssueIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;

        let issue = store
            .get_issue(&params.0.id)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .ok_or_else(|| {
                McpError::invalid_params(format!("Issue not found: {}", params.0.id), None)
            })?;

        let json = serde_json::to_string_pretty(&issue)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ============================================================
    // STRATEGIC MCP TOOLS - AI-Native Operations
    // ============================================================

    #[tool(
        description = "Suggest the next task to work on based on priority, blockers, and optionally current file context. Returns a ranked list of actionable issues."
    )]
    async fn get_next_task(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<GetNextTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results = grits_core::strategic::advisor::get_next_task(
            &store,
            params.0.current_file.as_deref(),
            params.0.assignee.as_deref(),
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Search issues using natural language. Matches against title, description, and labels."
    )]
    async fn search_issues(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<SearchIssuesParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results = grits_core::strategic::analysis::search_issues(
            &store,
            &params.0.query,
            params.0.limit.unwrap_or(10),
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Find issues that might be duplicates based on similar titles and descriptions."
    )]
    async fn detect_duplicates(
        &self,
        _params: rmcp::handler::server::wrapper::Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results = grits_core::strategic::analysis::detect_duplicates(&store)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Get the full dependency graph of all issues for planning and visualization."
    )]
    async fn get_issue_graph(
        &self,
        _params: rmcp::handler::server::wrapper::Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results = grits_core::strategic::analysis::get_issue_graph(&store)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Find stale issues that haven't been updated recently. Returns issues for review/closure."
    )]
    async fn cleanup_stale(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CleanupStaleParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results = grits_core::strategic::workflow::cleanup_stale(
            &store,
            params.0.days_inactive.unwrap_or(30),
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Summarize sprint progress: issues closed, opened, and in progress during a time period."
    )]
    async fn summarize_sprint(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<SummarizeSprintParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results =
            grits_core::strategic::advisor::summarize_sprint(&store, params.0.days.unwrap_or(7))
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ============================================================
    // CONTEXTUAL OPERATIONS - Require understanding codebase
    // ============================================================

    #[tool(
        description = "Find issues related to a specific file path. Matches file name against issue titles and descriptions."
    )]
    async fn find_related_issues(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<FindRelatedIssuesParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results =
            grits_core::strategic::analysis::find_related_issues(&store, &params.0.file_path)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Match an error message against existing issues to suggest which issue might be related."
    )]
    async fn suggest_issue_for_error(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<SuggestIssueForErrorParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results = grits_core::strategic::context::suggest_issue_for_error(
            &store,
            &params.0.error_message,
            params.0.limit.unwrap_or(5),
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Analyze git diff and suggest a new issue based on the changes. Uses HEAD~1..HEAD if no diff provided."
    )]
    async fn infer_issue_from_diff(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<InferIssueFromDiffParams>,
    ) -> Result<CallToolResult, McpError> {
        let diff_text = if let Some(ref diff) = params.0.diff {
            diff.clone()
        } else {
            // Try to get diff from git
            let repo = Repository::open(".").map_err(|e| {
                McpError::internal_error(format!("Failed to open git repo: {}", e), None)
            })?;

            let head = repo.head().map_err(|e| {
                McpError::internal_error(format!("Failed to get HEAD: {}", e), None)
            })?;

            let head_commit = head.peel_to_commit().map_err(|e| {
                McpError::internal_error(format!("Failed to get commit: {}", e), None)
            })?;

            let parent = head_commit.parent(0).map_err(|e| {
                McpError::internal_error(format!("Failed to get parent commit: {}", e), None)
            })?;

            let diff = repo
                .diff_tree_to_tree(
                    Some(&parent.tree().unwrap()),
                    Some(&head_commit.tree().unwrap()),
                    None,
                )
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to get diff: {}", e), None)
                })?;

            let mut diff_str = String::new();
            diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                if let Ok(content) = std::str::from_utf8(line.content()) {
                    diff_str.push_str(content);
                }
                true
            })
            .ok();

            diff_str
        };

        let result = grits_core::strategic::context::infer_issue_from_diff(&diff_text)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ============================================================
    // BULK OPERATIONS - Batch processing
    // ============================================================

    #[tool(
        description = "Batch update multiple issues at once. Apply same status, priority, or assignee to multiple issues."
    )]
    async fn bulk_triage(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<BulkTriageParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut store = self.get_store()?;

        let result = grits_core::strategic::workflow::bulk_triage(
            &mut store,
            params.0.issue_ids.clone(),
            params.0.status.clone(),
            params.0.priority.clone(),
            params.0.assignee.clone(),
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // AUTO-EXPORT: Push changes to Human Engine (UI)
        self.auto_export(&store)?;

        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ============================================================
    // WORKFLOW ASSISTANCE - Help with development flow
    // ============================================================

    #[tool(
        description = "Parse a commit message and find issues it references (fixes #123, refs gr-abc, closes #456)."
    )]
    async fn link_commit_to_issues(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<LinkCommitToIssuesParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let results =
            grits_core::strategic::context::link_commit_to_issues(&store, &params.0.commit_message)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Scan a file for TODO/FIXME comments and generate issue suggestions.")]
    async fn generate_issue_from_todo(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<GenerateIssueFromTodoParams>,
    ) -> Result<CallToolResult, McpError> {
        let file_path = &params.0.file_path;

        let content = std::fs::read_to_string(file_path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let results = grits_core::strategic::context::generate_issue_from_todo(
            file_path,
            &content,
            params.0.line_number,
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ============================================================
    // ARCHITECTURAL TOOLS - Solid Graph
    // ============================================================

    #[tool(description = "Get the topological skeleton of code affected by an issue.")]
    async fn get_issue_topology(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<GetIssueTopologyParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.get_store()?;
        let issue = store
            .get_issue(&params.0.issue_id)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .ok_or_else(|| McpError::invalid_params("Issue not found".to_string(), None))?;

        if let Some(volume_json) = issue.solid_volume {
            Ok(CallToolResult::success(vec![Content::text(volume_json)]))
        } else {
            // If no volume yet, maybe return empty graph or try to calculate one?
            // For now, return empty graph structure
            Ok(CallToolResult::success(vec![Content::text(
                "{\"nodes\": {}, \"edges\": []}",
            )]))
        }
    }

    #[tool(
        description = "Validate if a code change violates architectural topology (creates cycles or voids)."
    )]
    async fn validate_architectural_change(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ValidateArchitecturalChangeParams>,
    ) -> Result<CallToolResult, McpError> {
        // Write content to a temp file if provided, otherwise use file directly
        let file_path = if !params.0.content.is_empty() {
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!(
                "grits_validate_{}",
                std::path::Path::new(&params.0.file_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "temp.rs".to_string())
            ));
            std::fs::write(&temp_file, &params.0.content).map_err(|e| {
                McpError::internal_error(format!("Failed to write temp file: {}", e), None)
            })?;
            temp_file.to_string_lossy().to_string()
        } else {
            params.0.file_path.clone()
        };

        // Call CLI command
        let output = std::process::Command::new("gr")
            .args(["analysis", "validate-topology", &file_path])
            .output()
            .map_err(|e| McpError::internal_error(format!("Failed to run CLI: {}", e), None))?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stderr.is_empty() && result.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(stderr)]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(result)]))
        }
    }
}

// Implement ServerHandler to enable the Service trait
#[rmcp::tool_handler]
impl rmcp::ServerHandler for GritsServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::V_2024_11_05,
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability::default()),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: "grits".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Run the MCP server over stdio
pub async fn run_server(db_path: PathBuf) -> anyhow::Result<()> {
    use rmcp::{transport::stdio, ServiceExt};

    let server = GritsServer::new(db_path);

    // Use stdio transport and serve
    let transport = stdio();
    let service = server.serve(transport).await?;

    // Wait for the service to complete
    service.waiting().await?;

    Ok(())
}
