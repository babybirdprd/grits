use grits_core::git::{GitOps, StdGit};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_git_operations() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let git = StdGit::new(root);

    // Init
    git.init().expect("git init failed");
    git.config("user.email", "test@example.com")
        .expect("config failed");
    git.config("user.name", "Test User").expect("config failed");

    // Create file
    let file_path = root.join("test.txt");
    fs::write(&file_path, "hello world").unwrap();

    // Add
    git.add(&file_path).expect("git add failed");

    // Commit
    git.commit("initial commit").expect("git commit failed");

    // Status
    let status = git.status().expect("git status failed");
    assert!(status.is_empty(), "status should be clean");

    // Verify commit exists by using show
    // We don't know the hash, but we can look at HEAD
    // But `git show` in git.rs takes a revision.
    // Let's modify the file and see status change
    fs::write(&file_path, "hello world 2").unwrap();
    let status_dirty = git.status().expect("git status failed");
    assert!(status_dirty.contains("test.txt"));

    // Add and commit again
    git.add(&file_path).expect("git add failed");
    git.commit("second commit").expect("git commit failed");

    // Show content of HEAD
    let content = git.show("HEAD:test.txt").expect("git show failed");
    assert_eq!(content, "hello world 2");
}
