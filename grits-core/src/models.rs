use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Status is just a string in Go
pub type Status = String;
pub type IssueType = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub issue_id: String,
    pub depends_on_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub issue_id: String,
    pub author: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Issue {
    pub id: String,

    #[serde(skip)]
    pub content_hash: String,

    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub design: String,

    #[serde(default)]
    pub acceptance_criteria: String,

    #[serde(default)]
    pub notes: String,

    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub issue_type: IssueType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_minutes: Option<i32>,

    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,

    // Messaging fields
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub replies_to: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relates_to: Vec<String>,

    #[serde(default)]
    pub duplicate_of: String,
    #[serde(default)]
    pub superseded_by: String,

    // Tombstone fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deleted_by: String,
    #[serde(default)]
    pub delete_reason: String,
    #[serde(default)]
    pub original_type: String,

    // Extra fields for export
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Dependency>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<Comment>,

    // Topology fields
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_symbols: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solid_volume: Option<String>,

    #[serde(default)]
    pub topology_hash: String,

    #[serde(default)]
    pub is_solid: bool,
}
