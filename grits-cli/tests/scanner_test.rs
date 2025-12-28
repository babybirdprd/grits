use grits_core::topology::{cache::TopologyCache, scanner::DirectoryScanner};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_scanner_with_excludes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a mock project structure
    fs::create_dir_all(root.join("src/ignored")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/lib.rs"), "fn lib() {}").unwrap();
    fs::write(root.join("src/ignored/mod.rs"), "fn ignored() {}").unwrap();

    // Scan without excludes
    let scanner_all = DirectoryScanner::new();
    let graph_all = scanner_all.scan(root).unwrap();
    assert!(graph_all.nodes.values().any(|s| s.name == "main"));
    assert!(graph_all.nodes.values().any(|s| s.name == "lib"));
    assert!(graph_all.nodes.values().any(|s| s.name == "ignored"));

    // Scan with excludes
    let scanner_excluded = DirectoryScanner::new().with_excludes(vec!["**/ignored/**".to_string()]);
    let graph_excluded = scanner_excluded.scan(root).unwrap();
    assert!(graph_excluded.nodes.values().any(|s| s.name == "main"));
    assert!(graph_excluded.nodes.values().any(|s| s.name == "lib"));
    assert!(!graph_excluded.nodes.values().any(|s| s.name == "ignored"));
}

#[test]
fn test_cache_save_load() {
    let dir = tempdir().unwrap();
    let cache_path = dir.path().join("topology.json");

    let mut cache = TopologyCache::new();
    cache.graph.add_dependency("A", "B", "calls");

    // Save
    cache.save(&cache_path).unwrap();
    assert!(cache_path.exists());

    // Load
    let loaded = TopologyCache::load(&cache_path).unwrap();
    assert!(loaded
        .graph
        .edges
        .iter()
        .any(|(f, t, _e)| f == "A" && t == "B"));
}

#[test]
fn test_scanner_max_depth() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join("a/b/c")).unwrap();
    fs::write(root.join("a/file1.rs"), "fn f1() {}").unwrap();
    fs::write(root.join("a/b/file2.rs"), "fn f2() {}").unwrap();
    fs::write(root.join("a/b/c/file3.rs"), "fn f3() {}").unwrap();

    // Depth 1 (relative to root, which is root/a/...)
    // Wait, DirectoryScanner depth is absolute depth from scan root.
    // root is 0, root/a is 1, root/a/b is 2.

    let scanner = DirectoryScanner::new().with_max_depth(1);
    let _graph = scanner.scan(root).unwrap();

    // Should NOT find anything because 'a' is at depth 1, and files are inside 'a'.
    // If we scan root, depth 1 includes root/a/*.rs (but there are none)
    // Wait, WalkDir depth 1 is immediate children.

    fs::write(root.join("top.rs"), "fn top() {}").unwrap();

    let scanner_d1 = DirectoryScanner::new().with_max_depth(1);
    let graph_d1 = scanner_d1.scan(root).unwrap();
    assert!(graph_d1.nodes.values().any(|s| s.name == "top"));
    assert!(!graph_d1.nodes.values().any(|s| s.name == "f1"));
}
