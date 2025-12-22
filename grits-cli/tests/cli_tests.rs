use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_onboard() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let path = temp.path();

    // Run gr onboard in temp dir
    let mut cmd = Command::cargo_bin("gr")?;
    cmd.current_dir(path)
        .arg("onboard")
        .write_stdin("\n") // Accept default username
        .assert()
        .success()
        .stdout(predicate::str::contains("Onboarding complete"));

    // Verify .grits dir created
    assert!(path.join(".grits").exists());
    assert!(path.join(".grits/grits.db").exists());
    assert!(path.join(".gitignore").exists());

    Ok(())
}

#[test]
fn test_create_list_show_close() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let path = temp.path();

    // Onboard first
    Command::cargo_bin("gr")?
        .current_dir(path)
        .arg("onboard")
        .write_stdin("\n")
        .assert()
        .success();

    // Create issue
    let assert = Command::cargo_bin("gr")?
        .current_dir(path)
        .arg("create")
        .arg("Test Issue")
        .arg("--description")
        .arg("This is a test")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone())?;
    // "Created issue gr-..."
    let id = stdout.trim().split_whitespace().last().unwrap();
    assert!(id.starts_with("gr-"));

    // List
    Command::cargo_bin("gr")?
        .current_dir(path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains(id))
        .stdout(predicate::str::contains("Test Issue"));

    // Show
    Command::cargo_bin("gr")?
        .current_dir(path)
        .arg("show")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Title:       Test Issue"))
        .stdout(predicate::str::contains("This is a test"));

    // Close
    Command::cargo_bin("gr")?
        .current_dir(path)
        .arg("close")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Closed issue {}", id)));

    // Verify status is closed
    Command::cargo_bin("gr")?
        .current_dir(path)
        .arg("show")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Status:      closed"));

    Ok(())
}

#[test]
fn test_path_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Onboard at root
    Command::cargo_bin("gr")?
        .current_dir(root)
        .arg("onboard")
        .write_stdin("\n")
        .assert()
        .success();

    // Create issue at root
    Command::cargo_bin("gr")?
        .current_dir(root)
        .arg("create")
        .arg("Root Issue")
        .arg("--description")
        .arg("Description for root issue")
        .assert()
        .success();

    // Create subdir
    let subdir = root.join("src/nested");
    fs::create_dir_all(&subdir)?;

    // Run list from subdir
    Command::cargo_bin("gr")?
        .current_dir(&subdir)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Issue"));

    Ok(())
}
