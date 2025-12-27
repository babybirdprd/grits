use crate::fs::FileSystem;
use crate::models::{Comment, Dependency, Issue};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufWriter, Write};
use std::path::Path;

const STATUS_TOMBSTONE: &str = "tombstone";
const STATUS_CLOSED: &str = "closed";

// Constants from Go implementation
const DEFAULT_TOMBSTONE_TTL_DAYS: i64 = 30;
const CLOCK_SKEW_GRACE_HOURS: i64 = 1;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct IssueKey {
    id: String,
    created_at: Option<DateTime<Utc>>,
    sender: String, // Go uses "CreatedBy" in local struct, mapped from "created_by" json.
                    // types.Issue doesn't have CreatedBy field at top level, only in Dependency?
                    // Go merge.go Issue struct has CreatedBy string `json:"created_by,omitempty"`.
                    // But types.Issue has Sender.
                    // Let's assume Sender corresponds to CreatedBy for identity?
                    // In Go types.Issue, there is "Sender" (messaging).
                    // But standard issues have created_at.
                    // Go merge.go: CreatedBy string `json:"created_by,omitempty"`.
                    // Does JSONL have created_by?
                    // DB schema: sender TEXT DEFAULT ''.
                    // Go export: json:"sender,omitempty".
                    // So models::Issue::sender is the one.
}

pub fn merge3way(
    output_path: &str,
    base_path: &str,
    left_path: &str,
    right_path: &str,
    debug: bool,
    fs: &impl FileSystem,
) -> Result<()> {
    if debug {
        eprintln!("=== DEBUG MODE ===");
        eprintln!("Output path: {}", output_path);
        eprintln!("Base path:   {}", base_path);
        eprintln!("Left path:   {}", left_path);
        eprintln!("Right path:  {}", right_path);
        eprintln!();
    }

    let base_issues = read_issues(base_path, fs)?;
    let left_issues = read_issues(left_path, fs)?;
    let right_issues = read_issues(right_path, fs)?;

    if debug {
        eprintln!("Base issues read: {}", base_issues.len());
        eprintln!("Left issues read: {}", left_issues.len());
        eprintln!("Right issues read: {}", right_issues.len());
        eprintln!();
    }

    let (result, conflicts) = merge_logic(base_issues, left_issues, right_issues);

    if debug {
        eprintln!("Merge complete:");
        eprintln!("  Merged issues: {}", result.len());
        eprintln!("  Conflicts: {}", conflicts.len());
        eprintln!();
    }

    // Write output
    let file = fs
        .open_write(Path::new(output_path))
        .context("Failed to create output file")?;
    let mut writer = BufWriter::new(file);

    for issue in result {
        let json = serde_json::to_string(&issue)?;
        writeln!(writer, "{}", json)?;
    }

    for conflict in conflicts {
        writeln!(writer, "{}", conflict)?;
    }

    if debug {
        eprintln!("Output written to: {}", output_path);
    }

    Ok(())
}

fn read_issues(path: &str, fs: &impl FileSystem) -> Result<Vec<Issue>> {
    let reader = fs
        .open_read(Path::new(path))
        .context(format!("Failed to open file: {}", path))?;
    let mut issues = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let issue: Issue = serde_json::from_str(&line).context("Failed to parse issue line")?;
        issues.push(issue);
    }
    Ok(issues)
}

fn make_key(issue: &Issue) -> IssueKey {
    IssueKey {
        id: issue.id.clone(),
        created_at: Some(issue.created_at), // Assuming exact match
        sender: issue.sender.clone(),
    }
}

fn is_tombstone(issue: &Issue) -> bool {
    issue.status == STATUS_TOMBSTONE
}

fn is_expired_tombstone(issue: &Issue, ttl_days: i64) -> bool {
    if !is_tombstone(issue) {
        return false;
    }
    // Tombstones without deleted_at are not expired (safety)
    let deleted_at = match issue.deleted_at {
        Some(d) => d,
        None => return false,
    };

    let ttl = if ttl_days == 0 {
        DEFAULT_TOMBSTONE_TTL_DAYS
    } else {
        ttl_days
    };

    // Add grace period
    let effective_ttl = Duration::days(ttl) + Duration::hours(CLOCK_SKEW_GRACE_HOURS);

    let expiration_time = deleted_at + effective_ttl;
    Utc::now() > expiration_time
}

fn merge_logic(base: Vec<Issue>, left: Vec<Issue>, right: Vec<Issue>) -> (Vec<Issue>, Vec<String>) {
    let mut base_map: HashMap<IssueKey, Issue> = HashMap::new();
    for i in base {
        base_map.insert(make_key(&i), i);
    }

    let mut left_map: HashMap<IssueKey, Issue> = HashMap::new();
    let mut left_by_id: HashMap<String, Issue> = HashMap::new();
    for i in left {
        left_map.insert(make_key(&i), i.clone());
        left_by_id.insert(i.id.clone(), i);
    }

    let mut right_map: HashMap<IssueKey, Issue> = HashMap::new();
    let mut right_by_id: HashMap<String, Issue> = HashMap::new();
    for i in right {
        right_map.insert(make_key(&i), i.clone());
        right_by_id.insert(i.id.clone(), i);
    }

    let mut processed_keys: HashSet<IssueKey> = HashSet::new();
    let mut processed_ids: HashSet<String> = HashSet::new();
    let mut result = Vec::new();
    let mut conflicts = Vec::new();

    let mut all_keys: HashSet<IssueKey> = HashSet::new();
    all_keys.extend(base_map.keys().cloned());
    all_keys.extend(left_map.keys().cloned());
    all_keys.extend(right_map.keys().cloned());

    for key in all_keys {
        if processed_keys.contains(&key) {
            continue;
        }
        processed_keys.insert(key.clone());

        let base_issue = base_map.get(&key).cloned();
        let mut left_issue = left_map.get(&key).cloned();
        let mut right_issue = right_map.get(&key).cloned();

        let in_base = base_issue.is_some();
        let mut in_left = left_issue.is_some();
        let mut in_right = right_issue.is_some();

        // ID-based fallback matching
        if !in_left && in_right {
            if let Some(right) = &right_issue {
                if let Some(fallback) = left_by_id.get(&right.id) {
                    left_issue = Some(fallback.clone());
                    in_left = true;
                    processed_keys.insert(make_key(fallback));
                }
            }
        }
        if !in_right && in_left {
            if let Some(left) = &left_issue {
                if let Some(fallback) = right_by_id.get(&left.id) {
                    right_issue = Some(fallback.clone());
                    in_right = true;
                    processed_keys.insert(make_key(fallback));
                }
            }
        }

        let current_id = key.id.clone();
        if processed_ids.contains(&current_id) {
            continue;
        }
        processed_ids.insert(current_id);

        let left_tombstone = in_left && is_tombstone(left_issue.as_ref().unwrap());
        let right_tombstone = in_right && is_tombstone(right_issue.as_ref().unwrap());

        if in_base && in_left && in_right {
            let left_i = left_issue.unwrap();
            let right_i = right_issue.unwrap();
            let base_i = base_issue.unwrap();

            if left_tombstone && right_tombstone {
                result.push(merge_tombstones(left_i, right_i));
            } else if left_tombstone && !right_tombstone {
                if is_expired_tombstone(&left_i, DEFAULT_TOMBSTONE_TTL_DAYS) {
                    result.push(right_i);
                } else {
                    result.push(left_i);
                }
            } else if right_tombstone && !left_tombstone {
                if is_expired_tombstone(&right_i, DEFAULT_TOMBSTONE_TTL_DAYS) {
                    result.push(left_i);
                } else {
                    result.push(right_i);
                }
            } else {
                let (merged, conflict) = merge_issue(base_i, left_i, right_i);
                if !conflict.is_empty() {
                    conflicts.push(conflict);
                } else {
                    result.push(merged);
                }
            }
        } else if !in_base && in_left && in_right {
            let left_i = left_issue.unwrap();
            let right_i = right_issue.unwrap();

            if left_tombstone && right_tombstone {
                result.push(merge_tombstones(left_i, right_i));
            } else if left_tombstone && !right_tombstone {
                if is_expired_tombstone(&left_i, DEFAULT_TOMBSTONE_TTL_DAYS) {
                    result.push(right_i);
                } else {
                    result.push(left_i);
                }
            } else if right_tombstone && !left_tombstone {
                if is_expired_tombstone(&right_i, DEFAULT_TOMBSTONE_TTL_DAYS) {
                    result.push(left_i);
                } else {
                    result.push(right_i);
                }
            } else {
                // Merge with empty base
                let empty_base = Issue {
                    id: left_i.id.clone(),
                    created_at: left_i.created_at,
                    sender: left_i.sender.clone(),
                    // ... set defaults for others
                    content_hash: String::new(),
                    title: String::new(),
                    description: String::new(),
                    design: String::new(),
                    acceptance_criteria: String::new(),
                    notes: String::new(),
                    status: String::new(),
                    priority: 0,
                    issue_type: String::new(),
                    assignee: None,
                    estimated_minutes: None,
                    updated_at: left_i.updated_at, // Use left's updated at as base? No, empty base should have defaults.
                    closed_at: None,
                    external_ref: None,
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
                    affected_symbols: Vec::new(),
                    solid_volume: None,
                    topology_hash: String::new(),
                    is_solid: false,
                };
                let (merged, _) = merge_issue(empty_base, left_i, right_i);
                result.push(merged);
            }
        } else if in_base && in_left && !in_right {
            // Deleted in right
            if left_tombstone {
                result.push(left_issue.unwrap());
            }
            // Else implicitly deleted in right -> delete wins
        } else if in_base && !in_left && in_right {
            // Deleted in left
            if right_tombstone {
                result.push(right_issue.unwrap());
            }
            // Else implicitly deleted in left -> delete wins
        } else if !in_base && in_left && !in_right {
            result.push(left_issue.unwrap());
        } else if !in_base && !in_left && in_right {
            result.push(right_issue.unwrap());
        }
    }

    (result, conflicts)
}

fn merge_tombstones(left: Issue, right: Issue) -> Issue {
    if left.deleted_at.is_none() && right.deleted_at.is_none() {
        return left;
    }
    if left.deleted_at.is_none() {
        return right;
    }
    if right.deleted_at.is_none() {
        return left;
    }
    if is_time_after(left.deleted_at, right.deleted_at) {
        return left;
    }
    right
}

fn merge_issue(base: Issue, left: Issue, right: Issue) -> (Issue, String) {
    let mut result = base.clone();

    // Merge fields
    result.title = merge_field_by_updated_at(
        &base.title,
        &left.title,
        &right.title,
        &left.updated_at,
        &right.updated_at,
    );
    result.description = merge_field_by_updated_at(
        &base.description,
        &left.description,
        &right.description,
        &left.updated_at,
        &right.updated_at,
    );
    result.notes = merge_notes(&base.notes, &left.notes, &right.notes);
    result.status = merge_status(&base.status, &left.status, &right.status);
    result.priority = merge_priority(base.priority, left.priority, right.priority);
    result.issue_type = merge_field(&base.issue_type, &left.issue_type, &right.issue_type);

    result.updated_at = max_time(left.updated_at, right.updated_at);

    if result.status == STATUS_CLOSED {
        result.closed_at = max_time_opt(left.closed_at, right.closed_at);
    } else {
        result.closed_at = None;
    }

    result.dependencies = merge_dependencies(&left.dependencies, &right.dependencies);
    result.labels = merge_labels(&left.labels, &right.labels);
    result.comments = merge_comments(&left.comments, &right.comments);

    // Copy tombstone fields if status became tombstone
    if result.status == STATUS_TOMBSTONE {
        if is_time_after(left.deleted_at, right.deleted_at) {
            result.deleted_at = left.deleted_at;
            result.deleted_by = left.deleted_by;
            result.delete_reason = left.delete_reason;
            result.original_type = left.original_type;
        } else if right.deleted_at.is_some() {
            result.deleted_at = right.deleted_at;
            result.deleted_by = right.deleted_by;
            result.delete_reason = right.delete_reason;
            result.original_type = right.original_type;
        } else if left.deleted_at.is_some() {
            result.deleted_at = left.deleted_at;
            result.deleted_by = left.deleted_by;
            result.delete_reason = left.delete_reason;
            result.original_type = left.original_type;
        }
    }

    (result, String::new())
}

fn merge_field(base: &str, left: &str, right: &str) -> String {
    if base == left && base != right {
        return right.to_string();
    }
    if base == right && base != left {
        return left.to_string();
    }
    left.to_string()
}

fn merge_field_by_updated_at(
    base: &str,
    left: &str,
    right: &str,
    left_updated: &DateTime<Utc>,
    right_updated: &DateTime<Utc>,
) -> String {
    if base == left && base != right {
        return right.to_string();
    }
    if base == right && base != left {
        return left.to_string();
    }
    if left == right {
        return left.to_string();
    }
    if *left_updated > *right_updated {
        return left.to_string();
    }
    right.to_string()
}

fn merge_notes(base: &str, left: &str, right: &str) -> String {
    if base == left && base != right {
        return right.to_string();
    }
    if base == right && base != left {
        return left.to_string();
    }
    if left == right {
        return left.to_string();
    }
    if left.is_empty() {
        return right.to_string();
    }
    if right.is_empty() {
        return left.to_string();
    }
    format!("{}\n\n---\n\n{}", left, right)
}

fn merge_status(base: &str, left: &str, right: &str) -> String {
    if left == STATUS_TOMBSTONE || right == STATUS_TOMBSTONE {
        return STATUS_TOMBSTONE.to_string();
    }
    if left == STATUS_CLOSED || right == STATUS_CLOSED {
        return STATUS_CLOSED.to_string();
    }
    merge_field(base, left, right)
}

fn merge_priority(base: i32, left: i32, right: i32) -> i32 {
    if base == left && base != right {
        return right;
    }
    if base == right && base != left {
        return left;
    }
    if left == right {
        return left;
    }

    if left == 0 && right != 0 {
        return right;
    }
    if right == 0 && left != 0 {
        return left;
    }

    if left < right {
        return left;
    } // Lower is more urgent
    right
}

fn is_time_after(t1: Option<DateTime<Utc>>, t2: Option<DateTime<Utc>>) -> bool {
    match (t1, t2) {
        (Some(a), Some(b)) => a > b,
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (None, None) => false, // Equal -> return false
    }
}

fn max_time(t1: DateTime<Utc>, t2: DateTime<Utc>) -> DateTime<Utc> {
    if t1 > t2 {
        t1
    } else {
        t2
    }
}

fn max_time_opt(t1: Option<DateTime<Utc>>, t2: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    match (t1, t2) {
        (Some(a), Some(b)) => Some(if a > b { a } else { b }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn merge_dependencies(left: &[Dependency], right: &[Dependency]) -> Vec<Dependency> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for dep in left.iter().chain(right.iter()) {
        let key = format!("{}:{}:{}", dep.issue_id, dep.depends_on_id, dep.type_);
        if !seen.contains(&key) {
            seen.insert(key);
            result.push(dep.clone());
        }
    }
    result
}

fn merge_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for label in left.iter().chain(right.iter()) {
        if !seen.contains(label) {
            seen.insert(label.clone());
            result.push(label.clone());
        }
    }
    result
}

fn merge_comments(left: &[Comment], right: &[Comment]) -> Vec<Comment> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for c in left.iter().chain(right.iter()) {
        // Key by author + text (approximate deduping)
        // We ignore timestamp for deduping to handle slight skews or edits?
        // Go uses author+text.
        let key = format!("{}:{}", c.author, c.text);
        if !seen.contains(&key) {
            seen.insert(key);
            result.push(c.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_issue(id: &str, title: &str, updated_at: i64) -> Issue {
        Issue {
            id: id.to_string(),
            content_hash: String::new(),
            title: title.to_string(),
            description: "desc".to_string(),
            design: String::new(),
            acceptance_criteria: String::new(),
            notes: String::new(),
            status: "open".to_string(),
            priority: 0,
            issue_type: "bug".to_string(),
            assignee: None,
            estimated_minutes: None,
            created_at: Utc.timestamp_opt(1000, 0).unwrap(),
            updated_at: Utc.timestamp_opt(updated_at, 0).unwrap(),
            closed_at: None,
            external_ref: None,
            sender: "user".to_string(),
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
            affected_symbols: Vec::new(),
            solid_volume: None,
            topology_hash: String::new(),
            is_solid: false,
        }
    }

    fn create_tombstone(id: &str, deleted_at: i64) -> Issue {
        let mut issue = create_issue(id, "deleted", deleted_at);
        issue.status = STATUS_TOMBSTONE.to_string();
        issue.deleted_at = Some(Utc.timestamp_opt(deleted_at, 0).unwrap());
        issue
    }

    #[test]
    fn test_merge_no_conflict() {
        let now = Utc::now().timestamp();
        let base = create_issue("1", "Base Title", now - 200);
        let left = create_issue("1", "Left Title", now - 100);
        let right = create_issue("1", "Base Title", now - 200);

        let (result, conflicts) = merge_logic(vec![base], vec![left], vec![right]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Left Title");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_merge_conflict_latest_wins() {
        let now = Utc::now().timestamp();
        let base = create_issue("1", "Base Title", now - 300);
        let left = create_issue("1", "Left Title", now - 200);
        let right = create_issue("1", "Right Title", now - 100);

        let (result, conflicts) = merge_logic(vec![base], vec![left], vec![right]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Right Title");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_merge_tombstone_wins_over_edit() {
        let now = Utc::now().timestamp();
        let base = create_issue("1", "Base Title", now - 300);
        let left = create_tombstone("1", now - 200);
        let right = create_issue("1", "Right Title", now - 100);

        let (result, conflicts) = merge_logic(vec![base], vec![left], vec![right]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, STATUS_TOMBSTONE);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_merge_3way_additions() {
        let now = Utc::now().timestamp();
        let base = vec![];
        let left = vec![create_issue("L1", "Left Only", now)];
        let right = vec![create_issue("R1", "Right Only", now)];

        let (result, conflicts) = merge_logic(base, left, right);
        assert_eq!(result.len(), 2);

        let mut sorted = result;
        sorted.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(sorted[0].id, "L1");
        assert_eq!(sorted[1].id, "R1");
        assert!(conflicts.is_empty());
    }
}
