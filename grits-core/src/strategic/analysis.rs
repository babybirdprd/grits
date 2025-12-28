use crate::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicatePair {
    pub issue_a: String,
    pub title_a: String,
    pub issue_b: String,
    pub title_b: String,
    pub similarity_percent: i32,
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
pub struct RelatedIssue {
    pub id: String,
    pub title: String,
    pub relevance: String,
}

pub fn detect_duplicates(store: &dyn Store) -> anyhow::Result<Vec<DuplicatePair>> {
    let issues = store.list_issues(None, None, None, None, None, None)?;
    let mut duplicates = Vec::new();

    for i in 0..issues.len() {
        for j in (i + 1)..issues.len() {
            let a = &issues[i];
            let b = &issues[j];

            if a.status == "closed" || b.status == "closed" {
                continue;
            }

            let a_title_lowered = a.title.to_lowercase();
            let b_title_lowered = b.title.to_lowercase();
            let a_words: HashSet<&str> = a_title_lowered.split_whitespace().collect();
            let b_words: HashSet<&str> = b_title_lowered.split_whitespace().collect();

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
    Ok(duplicates)
}

pub fn get_issue_graph(store: &dyn Store) -> anyhow::Result<IssueGraph> {
    let issues = store.list_issues(None, None, None, None, None, None)?;

    let nodes = issues
        .iter()
        .map(|i| GraphNode {
            id: i.id.clone(),
            title: i.title.clone(),
            status: i.status.clone(),
            priority: i.priority,
        })
        .collect();

    let edges = issues
        .iter()
        .flat_map(|i| {
            i.dependencies.iter().map(move |d| GraphEdge {
                from: i.id.clone(),
                to: d.depends_on_id.clone(),
                edge_type: d.type_.clone(),
            })
        })
        .collect();

    Ok(IssueGraph { nodes, edges })
}

pub fn find_related_issues(
    store: &dyn Store,
    file_path: &str,
) -> anyhow::Result<Vec<RelatedIssue>> {
    let issues = store.list_issues(None, None, None, None, None, None)?;
    let file_path_lower = file_path.to_lowercase();

    let mut related = Vec::new();
    for issue in issues {
        if issue.status == "closed" {
            continue;
        }

        let mut relevance = None;
        if issue.description.to_lowercase().contains(&file_path_lower) {
            relevance = Some("File explicitly mentioned in description");
        } else if issue.title.to_lowercase().contains(&file_path_lower) {
            relevance = Some("File explicitly mentioned in title");
        } else if issue
            .affected_symbols
            .iter()
            .any(|s| s.to_lowercase().contains(&file_path_lower))
        {
            relevance = Some("File linked to issue via symbols");
        }

        if let Some(reason) = relevance {
            related.push(RelatedIssue {
                id: issue.id.clone(),
                title: issue.title.clone(),
                relevance: reason.to_string(),
            });
        }
    }

    Ok(related)
}

pub fn search_issues(
    store: &dyn Store,
    query: &str,
    limit: i32,
) -> anyhow::Result<Vec<SearchResult>> {
    let issues = store.list_issues(None, None, None, None, None, None)?;
    let index = crate::search::SearchIndex::new(&issues);
    let results = index.search(query, &issues);

    Ok(results
        .into_iter()
        .take(limit as usize)
        .map(|(issue, score)| SearchResult {
            id: issue.id.clone(),
            title: issue.title.clone(),
            status: issue.status.clone(),
            relevance_score: (score * 100.0) as i32,
            snippet: issue.description.chars().take(100).collect::<String>()
                + if issue.description.len() > 100 {
                    "..."
                } else {
                    ""
                },
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Dependency, Issue};
    use crate::Store;
    use chrono::Utc;

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
    fn test_detect_duplicates() {
        let issues = vec![
            Issue {
                id: "gr-1".into(),
                title: "Login button broken".into(),
                status: "open".into(),
                ..Default::default()
            },
            Issue {
                id: "gr-2".into(),
                title: "Login button is broken".into(),
                status: "open".into(),
                ..Default::default()
            },
            Issue {
                id: "gr-3".into(),
                title: "Fix database issue".into(),
                status: "open".into(),
                ..Default::default()
            },
        ];

        let store = MockStore { issues };
        let dups = detect_duplicates(&store).unwrap();

        assert_eq!(dups.len(), 1);
        assert!(dups[0].similarity_percent >= 50);
        assert!(
            (dups[0].issue_a == "gr-1" && dups[0].issue_b == "gr-2")
                || (dups[0].issue_a == "gr-2" && dups[0].issue_b == "gr-1")
        );
    }

    #[test]
    fn test_get_issue_graph() {
        let issues = vec![
            Issue {
                id: "gr-1".into(),
                title: "Issue 1".into(),
                dependencies: vec![Dependency {
                    issue_id: "gr-1".into(),
                    depends_on_id: "gr-2".into(),
                    type_: "blocking".into(),
                    created_at: Utc::now(),
                    created_by: "test".into(),
                }],
                ..Default::default()
            },
            Issue {
                id: "gr-2".into(),
                title: "Issue 2".into(),
                ..Default::default()
            },
        ];

        let store = MockStore { issues };
        let graph = get_issue_graph(&store).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, "gr-1");
        assert_eq!(graph.edges[0].to, "gr-2");
    }
}
