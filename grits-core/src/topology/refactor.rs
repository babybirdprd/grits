//! Refactoring engine for automated cycle-breaking
//!
//! This module provides the auto-apply functionality for `gr refactor --apply`,
//! allowing agents to automatically fix topology issues by commenting out
//! or restructuring the weakest dependency edges.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// The kind of refactoring action to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactorKind {
    /// Comment out an import/call statement
    CommentOut,
    /// Add a warning comment about the cycle
    AddWarningComment,
}

/// A single refactoring action to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorAction {
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub action: RefactorKind,
    pub original_code: String,
    pub modified_code: String,
    pub edge_from: String,
    pub edge_to: String,
    pub reasoning: String,
}

impl RefactorAction {
    /// Create a new comment-out refactoring action
    pub fn comment_out(
        file_path: &str,
        line_start: usize,
        line_end: usize,
        original_code: &str,
        edge_from: &str,
        edge_to: &str,
    ) -> Self {
        // Detect language from extension
        let comment_prefix = if file_path.ends_with(".py") {
            "# "
        } else if file_path.ends_with(".go") {
            "// "
        } else {
            "// "
        };

        // Generate commented-out version
        let modified_lines: Vec<String> = original_code
            .lines()
            .map(|line| format!("{}{}", comment_prefix, line))
            .collect();

        let warning = format!(
            "{}TODO(grits): Commented out to break cycle: {} -> {}",
            comment_prefix, edge_from, edge_to
        );

        let modified_code = format!("{}\n{}", warning, modified_lines.join("\n"));

        Self {
            file_path: file_path.to_string(),
            line_start,
            line_end,
            action: RefactorKind::CommentOut,
            original_code: original_code.to_string(),
            modified_code,
            edge_from: edge_from.to_string(),
            edge_to: edge_to.to_string(),
            reasoning: format!(
                "This edge ({} -> {}) was identified as the weakest link in the dependency cycle",
                edge_from, edge_to
            ),
        }
    }

    /// Generate a unified diff preview
    pub fn preview_diff(&self) -> String {
        let mut diff = String::new();
        diff.push_str(&format!("--- a/{}\n", self.file_path));
        diff.push_str(&format!("+++ b/{}\n", self.file_path));
        diff.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            self.line_start,
            self.original_code.lines().count(),
            self.line_start,
            self.modified_code.lines().count()
        ));

        for line in self.original_code.lines() {
            diff.push_str(&format!("-{}\n", line));
        }
        for line in self.modified_code.lines() {
            diff.push_str(&format!("+{}\n", line));
        }

        diff
    }

    /// Apply the refactoring to the file system
    pub fn apply(&self, backup_dir: Option<&Path>) -> Result<()> {
        let file_path = Path::new(&self.file_path);

        // Read original file
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        // Validate line numbers
        if self.line_start < 1 || self.line_end > lines.len() || self.line_start > self.line_end {
            bail!(
                "Invalid line range {}-{} for file with {} lines",
                self.line_start,
                self.line_end,
                lines.len()
            );
        }

        // Create backup if requested
        if let Some(backup_path) = backup_dir {
            fs::create_dir_all(backup_path)?;
            let backup_file = backup_path
                .join(file_path.file_name().unwrap_or_default())
                .with_extension("bak");
            fs::write(&backup_file, &content)?;
        }

        // Build new content
        let mut new_lines: Vec<String> = Vec::new();

        // Lines before the change (1-indexed to 0-indexed)
        for line in &lines[..self.line_start - 1] {
            new_lines.push(line.to_string());
        }

        // Insert modified code
        for line in self.modified_code.lines() {
            new_lines.push(line.to_string());
        }

        // Lines after the change
        for line in &lines[self.line_end..] {
            new_lines.push(line.to_string());
        }

        // Write back
        fs::write(file_path, new_lines.join("\n") + "\n")?;

        Ok(())
    }
}

/// Result of analyzing a file for refactoring opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorSuggestion {
    pub cycles_detected: usize,
    pub suggested_action: Option<RefactorAction>,
    pub all_cycles: Vec<Vec<String>>,
}

/// Undo a refactoring by restoring from backup
pub fn undo_refactor(file_path: &str, backup_dir: &Path) -> Result<()> {
    let file = Path::new(file_path);
    let backup_file = backup_dir
        .join(file.file_name().unwrap_or_default())
        .with_extension("bak");

    if !backup_file.exists() {
        bail!("No backup found for {}", file_path);
    }

    let backup_content = fs::read_to_string(&backup_file)?;
    fs::write(file, backup_content)?;
    fs::remove_file(&backup_file)?;

    Ok(())
}

/// Get the backup directory path for a project
pub fn get_backup_dir(project_root: &Path) -> PathBuf {
    project_root.join(".grits").join("backups")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_out_rust() {
        let action = RefactorAction::comment_out(
            "src/main.rs",
            10,
            10,
            "use crate::foo::bar;",
            "main.rs::main",
            "foo.rs::bar",
        );

        assert!(action.modified_code.contains("// TODO(grits)"));
        assert!(action.modified_code.contains("// use crate::foo::bar;"));
    }

    #[test]
    fn test_comment_out_python() {
        let action = RefactorAction::comment_out(
            "src/main.py",
            5,
            5,
            "from foo import bar",
            "main.py::main",
            "foo.py::bar",
        );

        assert!(action.modified_code.contains("# TODO(grits)"));
        assert!(action.modified_code.contains("# from foo import bar"));
    }

    #[test]
    fn test_preview_diff() {
        let action = RefactorAction::comment_out("src/test.rs", 1, 1, "use foo;", "a", "b");

        let diff = action.preview_diff();
        assert!(diff.contains("--- a/src/test.rs"));
        assert!(diff.contains("+++ b/src/test.rs"));
        assert!(diff.contains("-use foo;"));
        assert!(diff.contains("+// TODO(grits)"));
    }
}
