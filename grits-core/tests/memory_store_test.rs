use grits_core::{Issue, MemoryStore, StdFileSystem, Store};
use tempfile::tempdir;

#[test]
fn test_memory_store_crud() {
    let store = MemoryStore::new();

    // Create Issue
    let issue = Issue {
        id: "issue1".to_string(),
        title: "Test Issue".to_string(),
        status: "open".to_string(),
        ..Default::default()
    };
    store.create_issue(&issue).unwrap();

    // Read Issue
    let fetched = store.get_issue("issue1").unwrap().unwrap();
    assert_eq!(fetched.title, "Test Issue");

    // Update Issue
    let mut updated = fetched.clone();
    updated.status = "closed".to_string();
    store.update_issue(&updated).unwrap();

    let fetched_updated = store.get_issue("issue1").unwrap().unwrap();
    assert_eq!(fetched_updated.status, "closed");

    // List Issues
    let issues = store
        .list_issues(None, None, None, None, None, None)
        .unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].id, "issue1");
}

#[test]
fn test_memory_store_config() {
    let store = MemoryStore::new();

    store.set_config("user.name", "Test User").unwrap();
    let val = store.get_config("user.name").unwrap();
    assert_eq!(val, Some("Test User".to_string()));

    let missing = store.get_config("user.email").unwrap();
    assert_eq!(missing, None);
}

#[test]
fn test_memory_store_export_import() {
    let store1 = MemoryStore::new();
    let issue1 = Issue {
        id: "issue1".to_string(),
        title: "Export Test".to_string(),
        ..Default::default()
    };
    store1.create_issue(&issue1).unwrap();

    let temp_dir = tempdir().unwrap();
    let jsonl_path = temp_dir.path().join("issues.jsonl");

    // Use StdFileSystem for test, assuming MemoryStore logic supports it
    // Wait, MemoryStore export/import uses fs trait.
    // If we use StdFileSystem, it writes to disk.
    let fs = StdFileSystem;

    store1.export_to_jsonl(&jsonl_path, &fs).unwrap();

    // Import into new store
    let mut store2 = MemoryStore::new();
    store2.import_from_jsonl(&jsonl_path, &fs).unwrap();

    let fetched = store2.get_issue("issue1").unwrap().unwrap();
    assert_eq!(fetched.title, "Export Test");
}
