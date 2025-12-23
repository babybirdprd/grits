use crate::models::Issue;
use crate::Store;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSuggestion {
    pub rank: usize,
    pub id: String,
    pub title: String,
    pub priority: i32,
    pub reason: String,
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

pub fn get_next_task(
    store: &dyn Store,
    current_file: Option<&str>,
    assignee: Option<&str>,
) -> anyhow::Result<Vec<TaskSuggestion>> {
    // Get all open/in-progress issues
    let issues = store.list_issues(None, None, None, None, None, None)?;

    // Filter to actionable (not closed, not blocked)
    let mut actionable: Vec<&Issue> = issues
        .iter()
        .filter(|i| i.status != "closed" && i.status != "blocked")
        .filter(|i| {
            if let Some(a) = assignee {
                if let Some(issue_assignee) = &i.assignee {
                    if issue_assignee != a {
                        return false;
                    }
                } else {
                    // Include unassigned if filtering by assignee?
                    // Usually "my next task" means assigned to me OR unassigned.
                    // But if assignee is explicit, let's match it.
                    return false;
                }
            }
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
    if let Some(file) = current_file {
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

    Ok(suggestions)
}

pub fn summarize_sprint(store: &dyn Store, days: i32) -> anyhow::Result<SprintSummary> {
    let issues = store.list_issues(None, None, None, None, None, None)?;
    let cutoff = Utc::now() - Duration::days(days as i64);

    let mut closed_count = 0;
    let mut created_count = 0;
    let mut in_progress_count = 0;
    let mut closed_titles = Vec::new();
    let mut in_progress_titles = Vec::new();

    for issue in issues {
        if issue.created_at >= cutoff {
            created_count += 1;
        }

        if issue.status == "closed" {
            if let Some(closed_at) = issue.closed_at {
                if closed_at >= cutoff {
                    closed_count += 1;
                    closed_titles.push(issue.title.clone());
                }
            }
        } else if issue.status == "in-progress" || issue.status == "started" {
            in_progress_count += 1;
            in_progress_titles.push(issue.title.clone());
        }
    }

    Ok(SprintSummary {
        period_days: days,
        issues_closed: closed_count,
        issues_created: created_count,
        issues_in_progress: in_progress_count,
        closed_titles,
        in_progress_titles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Dependency, Issue};
    use crate::Store;

    struct MockStore {
        issues: Vec<Issue>,
    }

    impl Store for MockStore {
        fn list_issues(
            &self,
            status: Option<&str>,
            _assignee: Option<&str>,
            _priority: Option<i32>,
            _issue_type: Option<&str>,
            _label: Option<&str>,
            _sort_by: Option<&str>,
        ) -> anyhow::Result<Vec<Issue>> {
            let mut results = self.issues.clone();
            if let Some(s) = status {
                results.retain(|i| i.status == s);
            }
            Ok(results)
        }
        fn get_issue(&self, id: &str) -> anyhow::Result<Option<Issue>> {
            Ok(self.issues.iter().find(|i| i.id == id).cloned())
        }
        fn create_issue(&self, _issue: &Issue) -> anyhow::Result<()> {
            Ok(())
        }
        fn update_issue(&self, _issue: &Issue) -> anyhow::Result<()> {
            Ok(())
        }
        fn get_config(&self, _key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }
        fn set_config(&self, _key: &str, _value: &str) -> anyhow::Result<()> {
            Ok(())
        }
        fn list_config(&self) -> anyhow::Result<Vec<(String, String)>> {
            Ok(vec![])
        }
        fn execute_raw(&self, _sql: &str) -> anyhow::Result<()> {
            Ok(())
        }
        fn generate_unique_id(
            &self,
            _prefix: &str,
            _title: &str,
            _description: &str,
            _user: &str,
        ) -> anyhow::Result<String> {
            Ok("test-1".to_string())
        }
        fn import_from_jsonl(
            &mut self,
            _path: &std::path::Path,
            _fs: &dyn crate::FileSystem,
        ) -> anyhow::Result<()> {
            Ok(())
        }
        fn export_to_jsonl(
            &self,
            _path: &std::path::Path,
            _fs: &dyn crate::FileSystem,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_get_next_task() {
        let issues = vec![
            Issue {
                id: "gr-1".into(),
                title: "High priority".into(),
                status: "open".into(),
                priority: 1,
                ..Default::default()
            },
            Issue {
                id: "gr-2".into(),
                title: "Low priority".into(),
                status: "open".into(),
                priority: 3,
                ..Default::default()
            },
            Issue {
                id: "gr-3".into(),
                title: "Blocked".into(),
                status: "open".into(),
                priority: 1,
                dependencies: vec![Dependency {
                    issue_id: "gr-3".into(),
                    depends_on_id: "gr-4".into(),
                    type_: "blocking".into(),
                    created_at: Utc::now(),
                    created_by: "test".into(),
                }],
                ..Default::default()
            },
            Issue {
                id: "gr-4".into(),
                title: "Blocker".into(),
                status: "open".into(),
                priority: 2,
                ..Default::default()
            },
        ];

        let store = MockStore { issues };
        let next = get_next_task(&store, None, None).unwrap();

        assert_eq!(next.len(), 3);
        assert_eq!(next[0].id, "gr-1");
        assert_eq!(next[1].id, "gr-4"); // gr-4 is blocker, gr-3 is blocked
        assert_eq!(next[2].id, "gr-2");
    }

    #[test]
    fn test_get_next_task_context_boost() {
        let issues = vec![
            Issue {
                id: "gr-1".into(),
                title: "Work on main.rs".into(),
                description: "Fix bugs in main.rs".into(),
                status: "open".into(),
                priority: 2,
                ..Default::default()
            },
            Issue {
                id: "gr-2".into(),
                title: "High priority".into(),
                status: "open".into(),
                priority: 1,
                ..Default::default()
            },
        ];

        let store = MockStore { issues };
        let next = get_next_task(&store, Some("main.rs"), None).unwrap();

        assert_eq!(next[0].id, "gr-1"); // Boosted due to context
    }
}
