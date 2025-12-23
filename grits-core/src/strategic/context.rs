use crate::Store;
use serde::{Deserialize, Serialize};

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
pub struct TodoItem {
    pub line: i32,
    pub text: String,
    pub suggested_title: String,
    pub context: String,
}

pub fn suggest_issue_for_error(
    store: &dyn Store,
    error_message: &str,
    limit: i32,
) -> anyhow::Result<Vec<ErrorMatch>> {
    let issues = store.list_issues(None, None, None, None, None, None)?;
    let error_lower = error_message.to_lowercase();
    let error_words: std::collections::HashSet<&str> = error_lower.split_whitespace().collect();

    let mut matches = Vec::new();
    for issue in issues {
        if issue.status == "closed" {
            continue;
        }

        let text = format!("{} {}", issue.title, issue.description).to_lowercase();
        let score = error_words
            .iter()
            .filter(|word| text.contains(*word))
            .count() as i32;

        if score > 0 {
            matches.push(ErrorMatch {
                id: issue.id.clone(),
                title: issue.title.clone(),
                match_score: score,
                matched_text: issue.title.clone(), // Could be expanded to snippet
            });
        }
    }

    matches.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    Ok(matches.into_iter().take(limit as usize).collect())
}

pub fn infer_issue_from_diff(diff_content: &str) -> anyhow::Result<InferredIssue> {
    // Basic heuristics to infer issue from diff
    // In a real app, this might call an LLM or do deeper AST analysis
    let lines: Vec<&str> = diff_content.lines().collect();
    let mut files_changed = Vec::new();

    for line in &lines {
        if line.starts_with("--- a/") || line.starts_with("+++ b/") {
            let path = &line[6..];
            if !files_changed.contains(&path.to_string()) {
                files_changed.push(path.to_string());
            }
        }
    }

    let suggested_title = if files_changed.is_empty() {
        "Updates to project".to_string()
    } else {
        format!("Updates to {}", files_changed[0])
    };

    Ok(InferredIssue {
        suggested_title,
        suggested_description: format!(
            "Changes detected in {} files.\n\n{}",
            files_changed.len(),
            diff_content.chars().take(200).collect::<String>()
        ),
        suggested_type: "task".to_string(),
        files_changed,
    })
}

pub fn generate_issue_from_todo(
    file_path: &str,
    content: &str,
    target_line: Option<i32>,
) -> anyhow::Result<Vec<TodoItem>> {
    let mut todos = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let line_num = (i + 1) as i32;
        if let Some(t) = target_line {
            if t != line_num {
                continue;
            }
        }

        let line_lower = line.to_lowercase();
        if let Some(pos) = line_lower.find("todo:") {
            let text = line[pos + 5..].trim().to_string();
            todos.push(TodoItem {
                line: line_num,
                text: text.clone(),
                suggested_title: format!("TODO: {}", text),
                context: format!("{}:{}", file_path, line_num),
            });
        }
    }
    Ok(todos)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitIssueLink {
    pub issue_id: String,
    pub issue_title: String,
    pub link_type: String, // "fixes", "refs", "closes"
}

pub fn link_commit_to_issues(
    store: &dyn Store,
    commit_message: &str,
) -> anyhow::Result<Vec<CommitIssueLink>> {
    let issues = store.list_issues(None, None, None, None, None, None)?;

    // Patterns to match issue references
    let patterns = [
        (r"(?i)fix(?:es|ed)?\s+(?:#|gr-)?(\w+)", "fixes"),
        (r"(?i)clos(?:es|ed)?\s+(?:#|gr-)?(\w+)", "closes"),
        (r"(?i)refs?\s+(?:#|gr-)?(\w+)", "refs"),
        (r"(?i)(?:#|gr-)(\w+)", "refs"),
    ];

    let mut links = Vec::new();

    for (pattern, link_type) in &patterns {
        let re = regex::Regex::new(pattern)?;
        for cap in re.captures_iter(commit_message) {
            if let Some(id_match) = cap.get(1) {
                let ref_id = id_match.as_str();

                // Try to find matching issue
                for issue in &issues {
                    if issue.id.contains(ref_id) || issue.id.ends_with(ref_id) {
                        // Avoid duplicates
                        if !links
                            .iter()
                            .any(|l: &CommitIssueLink| l.issue_id == issue.id)
                        {
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

    Ok(links)
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
    fn test_suggest_issue_for_error() {
        let issues = vec![
            Issue {
                id: "gr-1".into(),
                title: "NullPointerException in user login".into(),
                status: "open".into(),
                ..Default::default()
            },
            Issue {
                id: "gr-2".into(),
                title: "Database connection timeout".into(),
                status: "open".into(),
                ..Default::default()
            },
        ];

        let store = MockStore { issues };
        let matches = suggest_issue_for_error(&store, "NullPointer in login", 5).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "gr-1");
    }

    #[test]
    fn test_link_commit_to_issues() {
        let issues = vec![Issue {
            id: "gr-123".into(),
            title: "Bug 123".into(),
            status: "open".into(),
            ..Default::default()
        }];

        let store = MockStore { issues };
        let links = link_commit_to_issues(&store, "This fixes #123").unwrap();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].issue_id, "gr-123");
        assert_eq!(links[0].link_type, "fixes");
    }
}
