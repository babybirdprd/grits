use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Detected workspace package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePackage {
    pub name: String,
    pub path: String,
    pub ecosystem: String, // "rust", "node", "python", "go"
}

/// Workspace configuration detected from the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub root: String,
    pub packages: Vec<WorkspacePackage>,
    pub ecosystem: String,
}

/// Detects workspace/monorepo configurations
pub struct WorkspaceDetector;

impl WorkspaceDetector {
    /// Detect workspace configuration from the given directory
    pub fn detect(dir: &Path) -> Result<Option<WorkspaceConfig>> {
        // Try each ecosystem in order
        if let Some(config) = Self::detect_rust_workspace(dir)? {
            return Ok(Some(config));
        }
        if let Some(config) = Self::detect_node_workspace(dir)? {
            return Ok(Some(config));
        }
        if let Some(config) = Self::detect_go_workspace(dir)? {
            return Ok(Some(config));
        }

        Ok(None)
    }

    /// Detect Cargo workspace
    fn detect_rust_workspace(dir: &Path) -> Result<Option<WorkspaceConfig>> {
        let cargo_toml = dir.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&cargo_toml)?;

        // Check for [workspace] section
        if !content.contains("[workspace]") {
            return Ok(None);
        }

        // Parse workspace members
        let mut packages = Vec::new();

        // Simple regex-free parsing for members = ["..."]
        if let Some(members_start) = content.find("members") {
            let rest = &content[members_start..];
            if let Some(bracket_start) = rest.find('[') {
                if let Some(bracket_end) = rest[bracket_start..].find(']') {
                    let members_str = &rest[bracket_start + 1..bracket_start + bracket_end];
                    for member in members_str.split(',') {
                        let member = member.trim().trim_matches('"').trim_matches('\'').trim();
                        if !member.is_empty() && !member.contains('*') {
                            // Handle glob patterns by checking if directory exists
                            let member_path = dir.join(member);
                            if member_path.exists() {
                                let name = member_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(member)
                                    .to_string();
                                packages.push(WorkspacePackage {
                                    name,
                                    path: member.to_string(),
                                    ecosystem: "rust".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        if packages.is_empty() {
            return Ok(None);
        }

        Ok(Some(WorkspaceConfig {
            root: dir.to_string_lossy().to_string(),
            packages,
            ecosystem: "rust".to_string(),
        }))
    }

    /// Detect Node.js workspace (pnpm, npm, yarn, lerna, turbo)
    fn detect_node_workspace(dir: &Path) -> Result<Option<WorkspaceConfig>> {
        let mut packages = Vec::new();

        // Check pnpm-workspace.yaml
        let pnpm_workspace = dir.join("pnpm-workspace.yaml");
        if pnpm_workspace.exists() {
            let content = std::fs::read_to_string(&pnpm_workspace)?;
            // Simple parsing for packages:
            if let Some(pkg_start) = content.find("packages:") {
                let rest = &content[pkg_start + 9..];
                for line in rest.lines() {
                    let trimmed = line.trim().trim_start_matches('-').trim();
                    if trimmed.starts_with("'") || trimmed.starts_with('"') {
                        let pkg = trimmed.trim_matches(|c| c == '\'' || c == '"').trim();
                        if !pkg.is_empty() && !pkg.contains('*') {
                            let pkg_path = dir.join(pkg);
                            if pkg_path.exists() {
                                let name = pkg_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(pkg)
                                    .to_string();
                                packages.push(WorkspacePackage {
                                    name,
                                    path: pkg.to_string(),
                                    ecosystem: "node".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check package.json workspaces
        if packages.is_empty() {
            let package_json = dir.join("package.json");
            if package_json.exists() {
                let content = std::fs::read_to_string(&package_json)?;
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(workspaces) = json.get("workspaces") {
                        let workspace_list = match workspaces {
                            serde_json::Value::Array(arr) => arr.clone(),
                            serde_json::Value::Object(obj) => obj
                                .get("packages")
                                .and_then(|p| p.as_array())
                                .cloned()
                                .unwrap_or_default(),
                            _ => Vec::new(),
                        };

                        for ws in workspace_list {
                            if let Some(ws_str) = ws.as_str() {
                                if !ws_str.contains('*') {
                                    let pkg_path = dir.join(ws_str);
                                    if pkg_path.exists() {
                                        let name = pkg_path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or(ws_str)
                                            .to_string();
                                        packages.push(WorkspacePackage {
                                            name,
                                            path: ws_str.to_string(),
                                            ecosystem: "node".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for turbo.json or lerna.json as indicators
        if packages.is_empty() {
            let turbo = dir.join("turbo.json");
            let lerna = dir.join("lerna.json");
            if turbo.exists() || lerna.exists() {
                // Scan for package.json in subdirectories
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() && path.join("package.json").exists() {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        if name != "node_modules" {
                            packages.push(WorkspacePackage {
                                name: name.clone(),
                                path: name,
                                ecosystem: "node".to_string(),
                            });
                        }
                    }
                }
            }
        }

        if packages.is_empty() {
            return Ok(None);
        }

        Ok(Some(WorkspaceConfig {
            root: dir.to_string_lossy().to_string(),
            packages,
            ecosystem: "node".to_string(),
        }))
    }

    /// Detect Go workspace (go.work)
    fn detect_go_workspace(dir: &Path) -> Result<Option<WorkspaceConfig>> {
        let go_work = dir.join("go.work");
        if !go_work.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&go_work)?;
        let mut packages = Vec::new();

        // Parse "use" directives
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use") && !trimmed.starts_with("use (") {
                let path = trimmed.strip_prefix("use").unwrap().trim();
                if !path.is_empty() {
                    let pkg_path = dir.join(path);
                    if pkg_path.exists() {
                        let name = pkg_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(path)
                            .to_string();
                        packages.push(WorkspacePackage {
                            name,
                            path: path.to_string(),
                            ecosystem: "go".to_string(),
                        });
                    }
                }
            }
        }

        // Handle multi-line use blocks
        if let Some(use_start) = content.find("use (") {
            let rest = &content[use_start + 5..];
            if let Some(use_end) = rest.find(')') {
                let use_block = &rest[..use_end];
                for line in use_block.lines() {
                    let path = line.trim();
                    if !path.is_empty() {
                        let pkg_path = dir.join(path);
                        if pkg_path.exists() {
                            let name = pkg_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(path)
                                .to_string();
                            packages.push(WorkspacePackage {
                                name,
                                path: path.to_string(),
                                ecosystem: "go".to_string(),
                            });
                        }
                    }
                }
            }
        }

        if packages.is_empty() {
            return Ok(None);
        }

        Ok(Some(WorkspaceConfig {
            root: dir.to_string_lossy().to_string(),
            packages,
            ecosystem: "go".to_string(),
        }))
    }

    /// Get package for a file path given a workspace config
    pub fn get_package_for_file(file_path: &str, config: &WorkspaceConfig) -> Option<String> {
        let file_normalized = file_path.replace('\\', "/");

        // Sort packages by path length (longest first) to match most specific
        let mut sorted_packages = config.packages.clone();
        sorted_packages.sort_by(|a, b| b.path.len().cmp(&a.path.len()));

        for pkg in sorted_packages {
            let pkg_normalized = pkg.path.replace('\\', "/");
            if file_normalized.starts_with(&pkg_normalized)
                || file_normalized.contains(&format!("/{}/", pkg_normalized))
                || file_normalized.contains(&format!("/{}", pkg_normalized))
            {
                return Some(pkg.name);
            }
        }

        None
    }
}

/// Build a map of file path prefix -> package name for efficient lookup
pub fn build_package_map(config: &WorkspaceConfig) -> HashMap<String, String> {
    config
        .packages
        .iter()
        .map(|p| (p.path.replace('\\', "/"), p.name.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_map() {
        let config = WorkspaceConfig {
            root: "/test".to_string(),
            packages: vec![WorkspacePackage {
                name: "core".to_string(),
                path: "packages/core".to_string(),
                ecosystem: "node".to_string(),
            }],
            ecosystem: "node".to_string(),
        };

        let map = build_package_map(&config);
        assert_eq!(map.get("packages/core"), Some(&"core".to_string()));
    }
}
