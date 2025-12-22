use grits_core::models::Issue;
use grits_core::search::SearchIndex;

#[test]
fn test_search_basic() {
    let mut index = SearchIndex::new();
    let issue1 = Issue {
        id: "1".to_string(),
        title: "Fix bug in search".to_string(),
        description: "The search is not working correctly".to_string(),
        ..Default::default()
    };
    let issue2 = Issue {
        id: "2".to_string(),
        title: "Add new feature".to_string(),
        description: "We need a new cool feature".to_string(),
        ..Default::default()
    };

    index.index_issues(&[issue1, issue2]);

    let results = index.search("bug");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "1");

    let results = index.search("feature");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "2");

    // Partial match test
    let results = index.search("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "1");
}
