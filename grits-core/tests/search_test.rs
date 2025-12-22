use grits_core::{search::SearchIndex, models::Issue};
use chrono::Utc;

#[test]
fn test_search() {
    let now = Utc::now();
    let issue1 = Issue {
        id: "1".to_string(),
        title: "Fix bug in search".to_string(),
        description: "Search is broken".to_string(),
        status: "open".to_string(),
        priority: 1,
        issue_type: "bug".to_string(),
        created_at: now,
        updated_at: now,
        ..Default::default()
    };
    let issue2 = Issue {
        id: "2".to_string(),
        title: "Add feature".to_string(),
        description: "New feature request".to_string(),
        status: "open".to_string(),
        priority: 1,
        issue_type: "feature".to_string(),
        created_at: now,
        updated_at: now,
        ..Default::default()
    };

    let issues = vec![issue1.clone(), issue2.clone()];
    let index = SearchIndex::new(&issues);

    let results = index.search("bug", &issues);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.id, "1");

    let results = index.search("feature", &issues);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.id, "2");
}
