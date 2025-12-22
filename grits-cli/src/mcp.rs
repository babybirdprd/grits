//! MCP Server for Grits - Agent Engine
//!
//! Provides structured tool access for AI agents via the Model Context Protocol.

use chrono::Utc;
use git2::Repository;
use grits_core::{Issue, SqliteStore, Store};
use regex::Regex;
use rmcp::{
    handler::server::tool::ToolRouter,
    model::{CallToolResult, Content},
    schemars::JsonSchema,
    tool, tool_router, ErrorData as McpError,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;

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

// ============================================================
// STRATEGIC MCP TOOL RESULTS
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSuggestion {
    pub rank: usize,
    pub id: String,
    pub title: String,
    pub priority: i32,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub relevance_score: i32,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicatePair {
    pub issue_a: String,
    pub title_a: String,
    pub issue_b: String,
    pub title_b: String,
    pub similarity_percent: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StaleIssue {
    pub id: String,
    pub title: String,
    pub status: String,
    pub last_updated: String,
    pub days_inactive: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SprintSummary {
    pub period_days: i32,
    pub issues_closed: i32,
    pub issues_created: i32,
    pub issues_in_progress: i32,
    pub closed_titles: Vec<String>,
    pub in_progress_titles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedIssue {
    pub id: String,
    pub title: String,
    pub relevance: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMatch {
    pub id: String,
    pub title: String,
    pub match_score: i32,
    pub matched_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferredIssue {
    pub suggested_title: String,
    pub suggested_description: String,
    pub suggested_type: String,
    pub files_changed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkTriageResult {
    pub updated_count: i32,
    pub updated_ids: Vec<String>,
    pub failed_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitIssueLink {
    pub issue_id: String,
    pub issue_title: String,
    pub link_type: String, // "fixes", "refs", "closes"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoItem {
    pub line: i32,
    pub text: String,
    pub suggested_title: String,
    pub context: String,
}

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
        SqliteStore::open(&*self.db_path).map_err(|e| McpError::internal_error(e.to_string(), None))
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

        // Get all open/in-progress issues
        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Filter to actionable (not closed, not blocked)
        let mut actionable: Vec<&Issue> = issues
            .iter()
            .filter(|i| i.status != "closed" && i.status != "blocked")
            .filter(|i| {
                // Check if any of its blockers are still open
                !i.dependencies.iter().any(|d| {
                    d.type_ == "blocking"
                        && issues
                            .iter()
                            .any(|bi| bi.id == d.depends_on_id && bi.status != "closed")
                })
            })
            .collect();

        // Sort by priority (lower = higher priority), then by updated_at
        actionable.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });

        // If current_file provided, boost issues mentioning that file
        if let Some(ref file) = params.0.current_file {
            let file_lower = file.to_lowercase();
            actionable.sort_by(|a, b| {
                let a_mentions = a.description.to_lowercase().contains(&file_lower)
                    || a.title.to_lowercase().contains(&file_lower);
                let b_mentions = b.description.to_lowercase().contains(&file_lower)
                    || b.title.to_lowercase().contains(&file_lower);
                b_mentions
                    .cmp(&a_mentions)
                    .then_with(|| a.priority.cmp(&b.priority))
            });
        }

        // Take top 5
        let suggestions: Vec<TaskSuggestion> = actionable
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, issue)| TaskSuggestion {
                rank: i + 1,
                id: issue.id.clone(),
                title: issue.title.clone(),
                priority: issue.priority,
                reason: if i == 0 {
                    "Highest priority actionable issue".to_string()
                } else {
                    format!("Priority {} issue", issue.priority)
                },
            })
            .collect();

        let json = serde_json::to_string_pretty(&suggestions)
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

        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let query_lower = params.0.query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        // Score each issue by how many query words it matches
        let mut scored: Vec<(i32, &Issue)> = issues
            .iter()
            .map(|issue| {
                let text = format!(
                    "{} {} {} {}",
                    issue.title,
                    issue.description,
                    issue.labels.join(" "),
                    issue.issue_type
                )
                .to_lowercase();

                let score = query_words
                    .iter()
                    .filter(|word| text.contains(*word))
                    .count() as i32;

                (score, issue)
            })
            .filter(|(score, _)| *score > 0)
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        let results: Vec<SearchResult> = scored
            .iter()
            .take(params.0.limit.unwrap_or(10) as usize)
            .map(|(score, issue)| SearchResult {
                id: issue.id.clone(),
                title: issue.title.clone(),
                status: issue.status.clone(),
                relevance_score: *score,
                snippet: issue.description.chars().take(100).collect::<String>()
                    + if issue.description.len() > 100 {
                        "..."
                    } else {
                        ""
                    },
            })
            .collect();

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

        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let mut duplicates: Vec<DuplicatePair> = Vec::new();

        // Simple n^2 comparison - find issues with similar titles
        for i in 0..issues.len() {
            for j in (i + 1)..issues.len() {
                let a = &issues[i];
                let b = &issues[j];

                // Skip if either is closed
                if a.status == "closed" || b.status == "closed" {
                    continue;
                }

                // Check title similarity (simple word overlap)
                let a_title_lower = a.title.to_lowercase();
                let b_title_lower = b.title.to_lowercase();
                let a_words: std::collections::HashSet<&str> =
                    a_title_lower.split_whitespace().collect();
                let b_words: std::collections::HashSet<&str> =
                    b_title_lower.split_whitespace().collect();

                let intersection = a_words.intersection(&b_words).count();
                let union = a_words.union(&b_words).count();

                if union > 0 {
                    let similarity = (intersection as f64 / union as f64 * 100.0) as i32;
                    if similarity >= 50 {
                        duplicates.push(DuplicatePair {
                            issue_a: a.id.clone(),
                            title_a: a.title.clone(),
                            issue_b: b.id.clone(),
                            title_b: b.title.clone(),
                            similarity_percent: similarity,
                        });
                    }
                }
            }
        }

        duplicates.sort_by(|a, b| b.similarity_percent.cmp(&a.similarity_percent));

        let json = serde_json::to_string_pretty(&duplicates)
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

        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let nodes: Vec<GraphNode> = issues
            .iter()
            .map(|i| GraphNode {
                id: i.id.clone(),
                title: i.title.clone(),
                status: i.status.clone(),
                priority: i.priority,
            })
            .collect();

        let edges: Vec<GraphEdge> = issues
            .iter()
            .flat_map(|i| {
                i.dependencies.iter().map(move |d| GraphEdge {
                    from: i.id.clone(),
                    to: d.depends_on_id.clone(),
                    edge_type: d.type_.clone(),
                })
            })
            .collect();

        let graph = IssueGraph { nodes, edges };

        let json = serde_json::to_string_pretty(&graph)
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

        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let days_threshold = params.0.days_inactive.unwrap_or(30);
        let cutoff = Utc::now() - chrono::Duration::days(days_threshold as i64);

        let stale: Vec<StaleIssue> = issues
            .iter()
            .filter(|i| i.status != "closed" && i.updated_at < cutoff)
            .map(|i| StaleIssue {
                id: i.id.clone(),
                title: i.title.clone(),
                status: i.status.clone(),
                last_updated: i.updated_at.to_rfc3339(),
                days_inactive: (Utc::now() - i.updated_at).num_days() as i32,
            })
            .collect();

        let json = serde_json::to_string_pretty(&stale)
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

        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let days = params.0.days.unwrap_or(7);
        let since = Utc::now() - chrono::Duration::days(days as i64);

        let closed_recently: Vec<&Issue> = issues
            .iter()
            .filter(|i| i.status == "closed" && i.closed_at.map(|c| c > since).unwrap_or(false))
            .collect();

        let created_recently: Vec<&Issue> =
            issues.iter().filter(|i| i.created_at > since).collect();

        let in_progress: Vec<&Issue> = issues
            .iter()
            .filter(|i| i.status == "in-progress")
            .collect();

        let summary = SprintSummary {
            period_days: days,
            issues_closed: closed_recently.len() as i32,
            issues_created: created_recently.len() as i32,
            issues_in_progress: in_progress.len() as i32,
            closed_titles: closed_recently.iter().map(|i| i.title.clone()).collect(),
            in_progress_titles: in_progress.iter().map(|i| i.title.clone()).collect(),
        };

        let json = serde_json::to_string_pretty(&summary)
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

        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let file_path = &params.0.file_path;

        // Extract filename and key parts from path
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);

        let file_stem = std::path::Path::new(file_name)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or(file_name);

        let mut related: Vec<RelatedIssue> = Vec::new();

        for issue in &issues {
            let text = format!("{} {}", issue.title, issue.description).to_lowercase();
            let file_lower = file_name.to_lowercase();
            let stem_lower = file_stem.to_lowercase();

            if text.contains(&file_lower) {
                related.push(RelatedIssue {
                    id: issue.id.clone(),
                    title: issue.title.clone(),
                    relevance: format!("Mentions file '{}'", file_name),
                });
            } else if text.contains(&stem_lower) && stem_lower.len() > 3 {
                related.push(RelatedIssue {
                    id: issue.id.clone(),
                    title: issue.title.clone(),
                    relevance: format!("Mentions '{}' (file stem)", file_stem),
                });
            }
        }

        let json = serde_json::to_string_pretty(&related)
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

        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let error_lower = params.0.error_message.to_lowercase();
        let error_words: Vec<&str> = error_lower
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        let mut matches: Vec<ErrorMatch> = Vec::new();

        for issue in &issues {
            let issue_text = format!("{} {}", issue.title, issue.description).to_lowercase();

            // Count matching significant words
            let match_count = error_words
                .iter()
                .filter(|word| issue_text.contains(*word))
                .count();

            if match_count > 0 {
                // Find the actual matched portion
                let matched: Vec<String> = error_words
                    .iter()
                    .filter(|word| issue_text.contains(*word))
                    .map(|s| s.to_string())
                    .collect();

                matches.push(ErrorMatch {
                    id: issue.id.clone(),
                    title: issue.title.clone(),
                    match_score: match_count as i32,
                    matched_text: matched.join(", "),
                });
            }
        }

        matches.sort_by(|a, b| b.match_score.cmp(&a.match_score));
        let limit = params.0.limit.unwrap_or(5) as usize;
        matches.truncate(limit);

        let json = serde_json::to_string_pretty(&matches)
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

        // Parse the diff to extract files changed
        let file_regex = Regex::new(r"(?m)^diff --git a/(.+?) b/").unwrap();
        let files_changed: Vec<String> = file_regex
            .captures_iter(&diff_text)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        // Determine issue type based on changes
        let issue_type = if diff_text.contains("fix") || diff_text.contains("bug") {
            "bug"
        } else if diff_text.contains("test") {
            "task"
        } else if diff_text.contains("feat") || diff_text.contains("add") {
            "feature"
        } else {
            "task"
        };

        // Generate title from files changed
        let suggested_title = if files_changed.len() == 1 {
            format!("Changes to {}", files_changed[0])
        } else if files_changed.len() <= 3 {
            format!("Update {}", files_changed.join(", "))
        } else {
            format!("Update {} files", files_changed.len())
        };

        let inferred = InferredIssue {
            suggested_title,
            suggested_description: format!(
                "Auto-generated from git diff.\n\nFiles changed:\n{}",
                files_changed
                    .iter()
                    .map(|f| format!("- {}", f))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            suggested_type: issue_type.to_string(),
            files_changed,
        };

        let json = serde_json::to_string_pretty(&inferred)
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
        let store = self.get_store()?;

        let mut updated_ids: Vec<String> = Vec::new();
        let mut failed_ids: Vec<String> = Vec::new();

        for id in &params.0.issue_ids {
            match store.get_issue(id) {
                Ok(Some(mut issue)) => {
                    if let Some(ref status) = params.0.status {
                        issue.status = status.clone();
                    }
                    if let Some(priority) = params.0.priority {
                        issue.priority = priority;
                    }
                    if let Some(ref assignee) = params.0.assignee {
                        issue.assignee = Some(assignee.clone());
                    }
                    issue.updated_at = Utc::now();

                    match store.update_issue(&issue) {
                        Ok(_) => updated_ids.push(id.clone()),
                        Err(_) => failed_ids.push(id.clone()),
                    }
                }
                _ => failed_ids.push(id.clone()),
            }
        }

        let result = BulkTriageResult {
            updated_count: updated_ids.len() as i32,
            updated_ids,
            failed_ids,
        };

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

        let issues = store
            .list_issues(None, None, None, None, None, None)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let commit_msg = &params.0.commit_message;

        // Patterns to match issue references
        let patterns = [
            (r"(?i)fix(?:es|ed)?\s+(?:#|gr-)?(\w+)", "fixes"),
            (r"(?i)clos(?:es|ed)?\s+(?:#|gr-)?(\w+)", "closes"),
            (r"(?i)refs?\s+(?:#|gr-)?(\w+)", "refs"),
            (r"(?i)(?:#|gr-)(\w+)", "refs"),
        ];

        let mut links: Vec<CommitIssueLink> = Vec::new();

        for (pattern, link_type) in &patterns {
            let re = Regex::new(pattern).unwrap();
            for cap in re.captures_iter(commit_msg) {
                if let Some(id_match) = cap.get(1) {
                    let ref_id = id_match.as_str();

                    // Try to find matching issue
                    for issue in &issues {
                        if issue.id.contains(ref_id) || issue.id.ends_with(ref_id) {
                            // Avoid duplicates
                            if !links.iter().any(|l| l.issue_id == issue.id) {
                                links.push(CommitIssueLink {
                                    issue_id: issue.id.clone(),
                                    issue_title: issue.title.clone(),
                                    link_type: link_type.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        let json = serde_json::to_string_pretty(&links)
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

        let todo_regex = Regex::new(r"(?i)(TODO|FIXME|HACK|XXX)[\s:]+(.+)$").unwrap();

        let mut todos: Vec<TodoItem> = Vec::new();
        let target_line = params.0.line_number;

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = (line_idx + 1) as i32;

            // If specific line requested, only look at that line
            if let Some(target) = target_line {
                if line_num != target {
                    continue;
                }
            }

            if let Some(cap) = todo_regex.captures(line) {
                let todo_type = cap.get(1).map(|m| m.as_str()).unwrap_or("TODO");
                let todo_text = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                if !todo_text.is_empty() {
                    let file_name = std::path::Path::new(file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(file_path);

                    todos.push(TodoItem {
                        line: line_num,
                        text: format!("{}: {}", todo_type, todo_text),
                        suggested_title: format!("[{}] {}", file_name, todo_text),
                        context: line.trim().to_string(),
                    });
                }
            }
        }

        let json = serde_json::to_string_pretty(&todos)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
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
