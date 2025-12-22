use grits_core::memory_store::MemoryStore;
use grits_core::models::Issue;
use grits_core::Store;

#[test]
fn test_memory_store_crud() {
    let store = MemoryStore::new();
    let issue = Issue {
        id: "1".to_string(),
        title: "Test Issue".to_string(),
        description: "Desc".to_string(),
        status: "open".to_string(),
        ..Default::default()
    };
    store.create_issue(&issue).unwrap();

    let fetched = store.get_issue("1").unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().title, "Test Issue");

    let mut updated = issue.clone();
    updated.status = "closed".to_string();
    store.update_issue(&updated).unwrap();

    let fetched_updated = store.get_issue("1").unwrap().unwrap();
    assert_eq!(fetched_updated.status, "closed");
}
