//! MCP Server for Grits - Agent Engine
//!
//! Provides structured tool access for AI agents via the Model Context Protocol.

use chrono::Utc;
use grits_core::{Issue, SqliteStore, Store};
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
}

// Implement ServerHandler to enable the Service trait
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

    tracing::info!("Starting Grits MCP server...");

    // Use stdio transport and serve
    let transport = stdio();
    let service = server.serve(transport).await?;

    // Wait for the service to complete
    service.waiting().await?;

    Ok(())
}
