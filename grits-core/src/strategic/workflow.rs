use crate::Store;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StaleIssue {
    pub id: String,
    pub title: String,
    pub status: String,
    pub last_updated: String,
    pub days_inactive: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkTriageResult {
    pub updated_count: i32,
    pub updated_ids: Vec<String>,
    pub failed_ids: Vec<String>,
}

pub fn cleanup_stale(store: &dyn Store, days_inactive: i32) -> anyhow::Result<Vec<StaleIssue>> {
    let issues = store.list_issues(None, None, None, None, None, None)?;
    let cutoff = Utc::now() - Duration::days(days_inactive as i64);

    let mut stale = Vec::new();
    for issue in issues {
        if issue.status == "closed" {
            continue;
        }

        if issue.updated_at < cutoff {
            let days = (Utc::now() - issue.updated_at).num_days() as i32;
            stale.push(StaleIssue {
                id: issue.id.clone(),
                title: issue.title.clone(),
                status: issue.status.clone(),
                last_updated: issue.updated_at.to_rfc3339(),
                days_inactive: days,
            });
        }
    }

    Ok(stale)
}

pub fn bulk_triage(
    store: &mut dyn Store,
    issue_ids: Vec<String>,
    status: Option<String>,
    priority: Option<i32>,
    assignee: Option<String>,
) -> anyhow::Result<BulkTriageResult> {
    let mut updated_ids = Vec::new();
    let mut failed_ids = Vec::new();

    for id in issue_ids {
        match store.get_issue(&id) {
            Ok(Some(mut issue)) => {
                let mut changed = false;
                if let Some(ref s) = status {
                    issue.status = s.clone();
                    changed = true;
                }
                if let Some(p) = priority {
                    issue.priority = p;
                    changed = true;
                }
                if let Some(ref a) = assignee {
                    issue.assignee = Some(a.clone());
                    changed = true;
                }

                if changed {
                    issue.updated_at = Utc::now();
                    if let Err(_) = store.update_issue(&issue) {
                        failed_ids.push(id);
                    } else {
                        updated_ids.push(id);
                    }
                } else {
                    updated_ids.push(id);
                }
            }
            _ => failed_ids.push(id),
        }
    }

    Ok(BulkTriageResult {
        updated_count: updated_ids.len() as i32,
        updated_ids,
        failed_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Issue;
    use crate::Store;

    struct MockStore {
        issues: Vec<Issue>,
    }

    impl Store for MockStore {
        fn list_issues(
            &self,
            _status: Option<&str>,
            _assignee: Option<&str>,
            _priority: Option<i32>,
            _issue_type: Option<&str>,
            _label: Option<&str>,
            _sort_by: Option<&str>,
        ) -> anyhow::Result<Vec<Issue>> {
            Ok(self.issues.clone())
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
    fn test_cleanup_stale() {
        let issues = vec![
            Issue {
                id: "gr-1".into(),
                title: "Old issue".into(),
                status: "open".into(),
                updated_at: Utc::now() - Duration::days(40),
                ..Default::default()
            },
            Issue {
                id: "gr-2".into(),
                title: "New issue".into(),
                status: "open".into(),
                updated_at: Utc::now() - Duration::days(5),
                ..Default::default()
            },
        ];

        let store = MockStore { issues };
        let stale = cleanup_stale(&store, 30).unwrap();

        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].id, "gr-1");
    }

    #[test]
    fn test_bulk_triage() {
        let mut store = MockStore {
            issues: vec![
                Issue {
                    id: "gr-1".into(),
                    title: "Issue 1".into(),
                    status: "open".into(),
                    ..Default::default()
                },
                Issue {
                    id: "gr-2".into(),
                    title: "Issue 2".into(),
                    status: "open".into(),
                    ..Default::default()
                },
            ],
        };

        let result = bulk_triage(
            &mut store,
            vec!["gr-1".into(), "gr-2".into()],
            Some("in-progress".into()),
            None,
            None,
        )
        .unwrap();

        assert_eq!(result.updated_count, 2);
    }
}
