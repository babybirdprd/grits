use anyhow::Context;
use chrono::Utc;
use clap::{Parser, Subcommand};
use grits_core::{Issue, SqliteStore, StdFileSystem, Store};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

mod mcp;

#[derive(Parser)]
#[command(name = "gr")]
#[command(about = "Grits Issue Tracker - Git-native, local-first issue management")]
#[command(
    long_about = "A lightweight issue tracker with first-class dependency support.\nDesigned for both humans (VS Code extension) and AI agents (MCP server)."
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List issues with optional filters
    List {
        /// Filter by status (open, in-progress, blocked, closed)
        #[arg(long)]
        status: Option<String>,
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by priority (1=critical to 5=trivial)
        #[arg(long)]
        priority: Option<i32>,
        /// Filter by issue type (bug, feature, task, epic)
        #[arg(long = "type")]
        type_: Option<String>,
        /// Filter by label
        #[arg(long)]
        label: Option<String>,
        /// Sort by field (priority, created_at, updated_at)
        #[arg(long)]
        sort: Option<String>,
    },
    /// Show detailed information about an issue
    Show {
        /// Issue ID or prefix
        id: String,
    },
    /// Update one or more fields on an issue
    Update {
        /// Issue ID or prefix
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<i32>,
        #[arg(long = "type")]
        type_: Option<String>,
        #[arg(long)]
        assignee: Option<String>,

        #[arg(long, action = clap::ArgAction::Append)]
        add_label: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        remove_label: Vec<String>,

        #[arg(long, action = clap::ArgAction::Append, alias = "depends-on")]
        add_dependency: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        remove_dependency: Vec<String>,
    },
    /// Edit an issue in your $EDITOR
    Edit {
        /// Issue ID or prefix
        id: String,
    },
    /// Close an issue
    Close {
        /// Issue ID or prefix
        id: String,
    },
    /// Create a new issue
    Create {
        /// Issue title
        title: String,
        /// Issue description
        #[arg(short, long, default_value = "")]
        description: String,
        /// Issue type: bug, feature, task, epic
        #[arg(short = 't', long = "type", default_value = "bug")]
        type_: String,
        /// Priority: 1 (critical) to 5 (trivial)
        #[arg(short, long, default_value_t = 2)]
        priority: i32,
    },
    /// Export issues to JSONL format
    Export {
        #[arg(short, long, default_value = ".grits/issues.jsonl")]
        output: String,
    },
    /// Import issues from JSONL format
    Import {
        #[arg(short, long, default_value = ".grits/issues.jsonl")]
        input: String,
    },
    /// Git merge driver for grits JSONL files
    Merge {
        output: String,
        base: String,
        left: String,
        right: String,
        #[arg(long)]
        debug: bool,
    },
    /// Initialize grits in the current repository
    Onboard,
    /// Show issues ready to work on (no blockers)
    Ready,
    /// Synchronize issues with git remote
    Sync {
        /// Squash all changes into a single commit
        #[arg(long)]
        squash: bool,
        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Show issue statistics
    Stats,
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Start the MCP server for AI agent integration
    ServeMcp,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., user.name, issue_id_prefix)
        key: String,
        /// Value to set
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// List all configuration values
    List,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrontmatterDependency {
    id: String,
    #[serde(default = "default_dep_type")]
    #[serde(rename = "type")]
    dep_type: String,
}

fn default_dep_type() -> String {
    "blocking".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct IssueFrontmatter {
    title: String,
    status: String,
    priority: i32,
    #[serde(rename = "type")]
    issue_type: String,
    #[serde(default)]
    assignee: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dependencies: Vec<FrontmatterDependency>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Only init tracing for non-MCP commands (MCP needs clean stdio for JSON-RPC)
    if !matches!(cli.command, Commands::ServeMcp) {
        tracing_subscriber::fmt::init();
    }

    // Find DB
    // Note: ServeMcp uses find_db_path() to ensure it finds the project's database
    // even when launched from a different working directory by the IDE
    let db_path = if matches!(cli.command, Commands::Onboard) {
        PathBuf::from(".grits/grits.db")
    } else {
        find_db_path()
    };

    // Ensure parent dir exists if we are creating/serving
    if matches!(cli.command, Commands::Create { .. } | Commands::ServeMcp) {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }
    // Ensure output dir exists if we are exporting
    if let Commands::Export { output } = &cli.command {
        if let Some(parent) = std::path::Path::new(output).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    // Ensure parent dir exists if we are onboarding
    if let Commands::Onboard = &cli.command {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let is_onboard = matches!(cli.command, Commands::Onboard);
    let is_mcp = matches!(cli.command, Commands::ServeMcp);
    let jsonl_path = db_path.parent().unwrap().join("issues.jsonl");

    let mut store = SqliteStore::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open DB at {:?}: {}", db_path, e))?;

    // AUTO-IMPORT: Before any command runs, look for changes in the Human Engine (.jsonl)
    // This maintains the "Twin Engine" tether - what happens in the UI is seen by the CLI immediately.
    if jsonl_path.exists() && !is_onboard {
        let _ = store.import_from_jsonl(&jsonl_path, &StdFileSystem);
    }

    match cli.command {
        Commands::List {
            status,
            assignee,
            priority,
            type_,
            label,
            sort,
        } => {
            let issues = store.list_issues(
                status.as_deref(),
                assignee.as_deref(),
                priority,
                type_.as_deref(),
                label.as_deref(),
                sort.as_deref(),
            )?;

            use comfy_table::modifiers::UTF8_ROUND_CORNERS;
            use comfy_table::presets::UTF8_FULL;
            use comfy_table::{Cell, Table};

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

            table.set_header(vec!["ID", "Status", "Priority", "Title"]);

            for issue in issues {
                let status_str = issue.status.clone();
                let status_cell = if status_str == "bug" {
                    Cell::new(&status_str).fg(comfy_table::Color::Red)
                } else if status_str == "closed" {
                    Cell::new(&status_str).fg(comfy_table::Color::Green)
                } else if status_str == "open" {
                    Cell::new(&status_str).fg(comfy_table::Color::Yellow)
                } else {
                    Cell::new(&status_str)
                };

                let title_truncated = if issue.title.len() > 60 {
                    format!("{}...", &issue.title[..57])
                } else {
                    issue.title.clone()
                };

                table.add_row(vec![
                    Cell::new(&issue.id),
                    status_cell,
                    Cell::new(issue.priority),
                    Cell::new(title_truncated),
                ]);
            }
            println!("{}", table);
        }
        Commands::Show { id } => {
            if let Some(issue) = store.get_issue(&id)? {
                println!("ID:          {}", issue.id);
                println!("Title:       {}", issue.title);
                println!("Status:      {}", issue.status);
                println!("Priority:    {}", issue.priority);
                println!("Type:        {}", issue.issue_type);
                if let Some(assignee) = &issue.assignee {
                    println!("Assignee:    {}", assignee);
                }
                println!("Created:     {}", issue.created_at);
                println!("Updated:     {}", issue.updated_at);
                println!("------------------------------------------------------------");
                println!("{}", issue.description);

                if !issue.labels.is_empty() {
                    println!("\nLabels: {}", issue.labels.join(", "));
                }

                if !issue.dependencies.is_empty() {
                    println!("\nDependencies:");
                    for dep in issue.dependencies {
                        println!("  {} ({})", dep.depends_on_id, dep.type_);
                    }
                }

                if !issue.comments.is_empty() {
                    println!("\nComments:");
                    for comment in issue.comments {
                        println!("  {} at {}:", comment.author, comment.created_at);
                        println!("    {}", comment.text);
                    }
                }
            } else {
                eprintln!("Issue not found: {}", id);
            }
        }
        Commands::Update {
            id,
            title,
            description,
            status,
            priority,
            type_,
            assignee,
            add_label,
            remove_label,
            add_dependency,
            remove_dependency,
        } => {
            if let Some(mut issue) = store.get_issue(&id)? {
                let mut updated = false;
                let user_name = store
                    .get_config("user.name")?
                    .unwrap_or_else(|| "unknown".to_string());

                if let Some(t) = title {
                    issue.title = t;
                    updated = true;
                }
                if let Some(d) = description {
                    issue.description = d;
                    updated = true;
                }
                if let Some(s) = status {
                    issue.status = s;
                    updated = true;
                }
                if let Some(p) = priority {
                    issue.priority = p;
                    updated = true;
                }
                if let Some(t) = type_ {
                    issue.issue_type = t;
                    updated = true;
                }
                if let Some(a) = assignee {
                    issue.assignee = if a.is_empty() { None } else { Some(a) };
                    updated = true;
                }

                // Handle Labels
                for label in add_label {
                    if !issue.labels.contains(&label) {
                        issue.labels.push(label);
                        updated = true;
                    }
                }
                for label in remove_label {
                    if let Some(pos) = issue.labels.iter().position(|l| l == &label) {
                        issue.labels.remove(pos);
                        updated = true;
                    }
                }

                // Handle Dependencies
                for dep_str in add_dependency {
                    // Format: "ID" or "ID:TYPE"
                    let parts: Vec<&str> = dep_str.splitn(2, ':').collect();
                    let (dep_id, dep_type) = if parts.len() == 2 {
                        (parts[0], parts[1])
                    } else {
                        (parts[0], "blocking")
                    };

                    // Check if exists
                    if !issue
                        .dependencies
                        .iter()
                        .any(|d| d.depends_on_id == dep_id && d.type_ == dep_type)
                    {
                        use grits_core::models::Dependency;
                        issue.dependencies.push(Dependency {
                            issue_id: issue.id.clone(),
                            depends_on_id: dep_id.to_string(),
                            type_: dep_type.to_string(),
                            created_at: Utc::now(),
                            created_by: user_name.clone(),
                        });
                        updated = true;
                    }
                }
                for dep_id in remove_dependency {
                    // Remove any dependency on this ID
                    let initial_len = issue.dependencies.len();
                    issue.dependencies.retain(|d| d.depends_on_id != dep_id);
                    if issue.dependencies.len() != initial_len {
                        updated = true;
                    }
                }

                if updated {
                    issue.updated_at = Utc::now();
                    store
                        .update_issue(&issue)
                        .context("Failed to update issue")?;
                    println!("Updated issue {}", issue.id);
                } else {
                    println!("No changes provided.");
                }
            } else {
                eprintln!("Issue not found: {}", id);
            }
        }
        Commands::Edit { id } => {
            if let Some(mut issue) = store.get_issue(&id)? {
                let user_name = store
                    .get_config("user.name")?
                    .unwrap_or_else(|| "unknown".to_string());

                let frontmatter = IssueFrontmatter {
                    title: issue.title.clone(),
                    status: issue.status.clone(),
                    priority: issue.priority,
                    issue_type: issue.issue_type.clone(),
                    assignee: issue.assignee.clone(),
                    labels: issue.labels.clone(),
                    dependencies: issue
                        .dependencies
                        .iter()
                        .map(|d| FrontmatterDependency {
                            id: d.depends_on_id.clone(),
                            dep_type: d.type_.clone(),
                        })
                        .collect(),
                };

                let yaml = serde_yaml::to_string(&frontmatter)?;
                let content = format!("---\n{}---\n\n{}", yaml, issue.description);

                let mut file = tempfile::Builder::new().suffix(".md").tempfile()?;
                write!(file, "{}", content)?;

                let path = file.path().to_owned();
                file.keep()?; // Keep the file so editor can open it, we'll delete later or let OS handle tmp

                edit::edit_file(&path)?;

                let new_content = std::fs::read_to_string(&path)?;
                std::fs::remove_file(path)?;

                // Parse
                if new_content.starts_with("---") {
                    let parts: Vec<&str> = new_content.splitn(3, "---").collect();
                    if parts.len() >= 3 {
                        let yaml_part = parts[1];
                        let body_part = parts[2].trim().to_string();

                        let new_fm: IssueFrontmatter = serde_yaml::from_str(yaml_part)
                            .map_err(|e| anyhow::anyhow!("Invalid frontmatter: {}", e))?;

                        issue.title = new_fm.title;
                        issue.status = new_fm.status;
                        issue.priority = new_fm.priority;
                        issue.issue_type = new_fm.issue_type;
                        issue.assignee = new_fm.assignee;
                        issue.description = body_part;
                        issue.updated_at = Utc::now();
                        issue.labels = new_fm.labels;

                        // Reconcile dependencies
                        // Convert new_fm.dependencies (Vec<FrontmatterDependency>) to Vec<Dependency>
                        // We try to preserve existing metadata if possible.
                        let mut new_deps = Vec::new();
                        for fd in new_fm.dependencies {
                            // Find existing
                            if let Some(existing) = issue
                                .dependencies
                                .iter()
                                .find(|d| d.depends_on_id == fd.id && d.type_ == fd.dep_type)
                            {
                                new_deps.push(existing.clone());
                            } else {
                                // Create new
                                use grits_core::models::Dependency;
                                new_deps.push(Dependency {
                                    issue_id: issue.id.clone(),
                                    depends_on_id: fd.id,
                                    type_: fd.dep_type,
                                    created_at: Utc::now(),
                                    created_by: user_name.clone(),
                                });
                            }
                        }
                        issue.dependencies = new_deps;

                        store
                            .update_issue(&issue)
                            .context("Failed to update issue")?;
                        println!("Updated issue {}", issue.id);
                    } else {
                        eprintln!("Invalid format: missing frontmatter delimiters");
                    }
                } else {
                    // Assume just description if no frontmatter?
                    // Or error out? Better to be safe.
                    eprintln!("Invalid format: file must start with ---");
                }
            } else {
                eprintln!("Issue not found: {}", id);
            }
        }
        Commands::Close { id } => {
            if let Some(mut issue) = store.get_issue(&id)? {
                if issue.status != "closed" {
                    issue.status = "closed".to_string();
                    issue.closed_at = Some(Utc::now());
                    issue.updated_at = Utc::now();
                    store
                        .update_issue(&issue)
                        .context("Failed to close issue")?;
                    println!("Closed issue {}", issue.id);
                } else {
                    println!("Issue {} is already closed.", issue.id);
                }
            } else {
                eprintln!("Issue not found: {}", id);
            }
        }
        Commands::Export { output } => {
            let fs = StdFileSystem;
            let output_path = std::path::Path::new(&output);
            store
                .export_to_jsonl(output_path, &fs)
                .context(format!("Failed to export issues to {}", output))?;
            println!("Exported issues to {}", output);
        }
        Commands::Import { input } => {
            let fs = StdFileSystem;
            let input_path = std::path::Path::new(&input);
            store
                .import_from_jsonl(input_path, &fs)
                .context(format!("Failed to import issues from {}", input))?;
            println!("Imported issues from {}", input);
        }
        Commands::Merge {
            output,
            base,
            left,
            right,
            debug,
        } => {
            let fs = StdFileSystem;
            grits_core::merge::merge3way(&output, &base, &left, &right, debug, &fs)?;
        }
        Commands::Onboard => {
            // Check git init
            if !std::path::Path::new(".git").exists() {
                println!("Not a git repository. Initializing...");
                std::process::Command::new("git").arg("init").status()?;
            }

            // Create .grits
            let grits_dir = std::path::Path::new(".grits");
            if !grits_dir.exists() {
                std::fs::create_dir(grits_dir)?;
                println!("Created .grits directory.");
            }

            // Create .gitignore
            let gitignore_path = std::path::Path::new(".gitignore");
            let mut gitignore_content = String::new();
            if gitignore_path.exists() {
                gitignore_content = std::fs::read_to_string(gitignore_path)?;
            }
            if !gitignore_content.contains("grits.db") {
                println!("Adding grits.db to .gitignore...");
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(gitignore_path)?;
                writeln!(file, "\n.grits/grits.db")?;
            }

            // User config
            // Try to read git config
            let output = std::process::Command::new("git")
                .args(["config", "user.name"])
                .output();

            let default_user = if let Ok(out) = output {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            } else {
                String::new()
            };

            print!("Enter your username [{}]: ", default_user);
            use std::io::Write;
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            let user = if input.is_empty() {
                default_user
            } else {
                input.to_string()
            };

            if !user.is_empty() {
                store.set_config("user.name", &user)?;
                println!("Configured user.name = {}", user);
            } else {
                println!("No user configured.");
            }

            println!("Onboarding complete!");
        }
        Commands::Ready => {
            // Get current user
            let user = store.get_config("user.name")?;
            let assignee = user.as_deref().unwrap_or("unassigned");

            // List issues not closed, assigned to user or unassigned (if user not set?)
            // Requirement: "alias for listing open issues assigned to user or unassigned"
            // If we have a user, we filter by that user.
            // If we don't have a user, maybe list unassigned?

            // Let's implement: Status != closed AND Assignee = <user>
            // But list_issues currently filters via exact match or unassigned.
            // Store::list_issues doesn't support "NOT closed". It supports "status = ?"
            // So we might need to filter in memory or fetch "open", "in_progress" separately?
            // "open" is default status. "closed" is closed.
            // We usually want everything NOT closed.
            // Since `list_issues` takes specific status, we can't easily say "not closed".
            // Let's fetch all and filter in memory for now, or fetch by common open statuses.
            // Given the limited "list_issues" SQL generation I wrote (AND logic), fetching all then filtering is safest without changing Store again.
            // Wait, I can pass None for status (all) and filter in loop.

            let all_issues = store.list_issues(None, None, None, None, None, None)?;

            println!("Ready issues for {}:", assignee);
            println!(
                "{:<10} {:<10} {:<10} {}",
                "ID", "STATUS", "PRIORITY", "TITLE"
            );
            println!("{:-<60}", "");

            for issue in all_issues {
                if issue.status == "closed" {
                    continue;
                }

                let matches_assignee = if let Some(a) = &issue.assignee {
                    if let Some(u) = &user {
                        a == u
                    } else {
                        // No user configured.
                        // Should we show unassigned? Or everything?
                        // "gr ready" implies "ready for ME".
                        // If I don't know who ME is, I can't filter by assignee efficiently.
                        // Maybe just show unassigned?
                        // Let's assume matches_assignee = true if user is None (show all open?) or false?
                        // Go with: if user is known, match it. If not, match nothing?
                        // Or maybe prompt user to configure user.name?
                        false
                    }
                } else {
                    // Issue is unassigned.
                    // Often "ready" queue includes unassigned issues one could pick up.
                    // Let's include unassigned.
                    true
                };

                if matches_assignee {
                    println!(
                        "{:<10} {:<10} {:<10} {}",
                        issue.id, issue.status, issue.priority, issue.title
                    );
                }
            }
        }
        Commands::Sync { squash, dry_run } => {
            let grits_dir = db_path.parent().unwrap();
            let git_root = grits_dir.parent().unwrap_or(std::path::Path::new("."));
            let git = grits_core::StdGit::new(git_root);
            let jsonl_path = grits_dir.join("issues.jsonl");
            let fs = StdFileSystem;
            grits_core::sync::run_sync(
                &mut store,
                &git,
                git_root,
                &jsonl_path,
                &fs,
                squash,
                dry_run,
            )
            .context("Sync failed")?;
            if dry_run {
                println!("Sync complete (dry-run).");
            } else {
                println!("Sync complete.");
            }
        }
        Commands::Stats => {
            let issues = store.list_issues(None, None, None, None, None, None)?;
            let total = issues.len();
            let mut by_status = std::collections::HashMap::new();
            let mut by_assignee = std::collections::HashMap::new();
            let mut by_priority = std::collections::HashMap::new();
            let mut by_type = std::collections::HashMap::new();

            for issue in issues {
                *by_status.entry(issue.status).or_insert(0) += 1;
                *by_assignee
                    .entry(issue.assignee.unwrap_or_else(|| "unassigned".to_string()))
                    .or_insert(0) += 1;
                *by_priority.entry(issue.priority).or_insert(0) += 1;
                *by_type.entry(issue.issue_type).or_insert(0) += 1;
            }

            println!("Total Issues: {}", total);

            println!("\nBy Status:");
            for (k, v) in &by_status {
                println!("  {:<12} {}", k, v);
            }

            println!("\nBy Priority:");
            let mut priorities: Vec<_> = by_priority.iter().collect();
            priorities.sort_by_key(|(k, _)| **k);
            for (k, v) in priorities {
                println!("  {:<12} {}", k, v);
            }

            println!("\nBy Type:");
            for (k, v) in &by_type {
                println!("  {:<12} {}", k, v);
            }

            println!("\nBy Assignee:");
            for (k, v) in &by_assignee {
                println!("  {:<12} {}", k, v);
            }
        }
        Commands::Config { command } => match command {
            ConfigCommands::Set { key, value } => {
                store
                    .set_config(&key, &value)
                    .context("Failed to set config")?;
                println!("{} = {}", key, value);
            }
            ConfigCommands::Get { key } => {
                if let Some(val) = store.get_config(&key)? {
                    println!("{}", val);
                } else {
                    eprintln!("Key not found: {}", key);
                }
            }
            ConfigCommands::List => {
                let items = store.list_config()?;
                for (k, v) in items {
                    println!("{} = {}", k, v);
                }
            }
        },
        Commands::Create {
            title,
            mut description,
            type_,
            priority,
        } => {
            // Interactive editing if description is empty
            if description.is_empty() {
                let mut content_init = String::new();

                // Check for templates
                let templates_dir = db_path.parent().unwrap().join("templates");
                if templates_dir.exists() {
                    let mut templates = Vec::new();
                    if let Ok(entries) = std::fs::read_dir(&templates_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.extension().map_or(false, |e| e == "md") {
                                templates.push(path);
                            }
                        }
                    }

                    if !templates.is_empty() {
                        println!("Available templates:");
                        for (i, t) in templates.iter().enumerate() {
                            println!("{}. {}", i + 1, t.file_name().unwrap().to_string_lossy());
                        }
                        println!("0. None (Default)");

                        print!("Select a template [0]: ");
                        std::io::stdout().flush()?;
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        let choice = input.trim().parse::<usize>().unwrap_or(0);

                        if choice > 0 && choice <= templates.len() {
                            if let Ok(t_content) = std::fs::read_to_string(&templates[choice - 1]) {
                                content_init = t_content;
                            }
                        }
                    }
                }

                let frontmatter = IssueFrontmatter {
                    title: title.clone(),
                    status: "open".to_string(),
                    priority,
                    issue_type: type_.clone(),
                    assignee: None,
                    labels: Vec::new(),
                    dependencies: Vec::new(),
                };

                let content = if content_init.is_empty() {
                    let yaml = serde_yaml::to_string(&frontmatter)?;
                    format!("---\n{}---\n\n{}", yaml, "")
                } else {
                    // Prepend frontmatter if template doesn't have it, or merge?
                    // Simple approach: If template has frontmatter, use it but override title/type/priority if passed?
                    // Or just use template as body.
                    // Let's assume template is body for now to be safe, but we wrap with our frontmatter.
                    // If template has frontmatter, we might double wrap, which is bad.
                    // Let's check for ---
                    if content_init.starts_with("---") {
                        content_init
                    } else {
                        let yaml = serde_yaml::to_string(&frontmatter)?;
                        format!("---\n{}---\n\n{}", yaml, content_init)
                    }
                };

                let mut file = tempfile::Builder::new().suffix(".md").tempfile()?;
                write!(file, "{}", content)?;

                let path = file.path().to_owned();
                file.keep()?;

                edit::edit_file(&path)?;

                let new_content = std::fs::read_to_string(&path)?;
                std::fs::remove_file(path)?;

                if new_content.starts_with("---") {
                    let parts: Vec<&str> = new_content.splitn(3, "---").collect();
                    if parts.len() >= 3 {
                        let body_part = parts[2].trim().to_string();
                        // We could also parse the YAML to allow user to change title/priority/type during creation!
                        // This is better UX.
                        let yaml_part = parts[1];
                        match serde_yaml::from_str::<IssueFrontmatter>(yaml_part) {
                            Ok(new_fm) => {
                                // Let's assume we use the parsed values if valid.
                                description = body_part;

                                let now = Utc::now();
                                // Use new_fm values
                                let prefix = store
                                    .get_config("issue_id_prefix")?
                                    .unwrap_or_else(|| "gr".to_string());
                                let user = store
                                    .get_config("user.name")?
                                    .unwrap_or_else(|| "unknown".to_string());
                                let short_id = store.generate_unique_id(
                                    &prefix,
                                    &new_fm.title,
                                    &description,
                                    &user,
                                )?;

                                let issue = Issue {
                                    id: short_id.clone(),
                                    content_hash: String::new(),
                                    title: new_fm.title,
                                    description,
                                    design: String::new(),
                                    acceptance_criteria: String::new(),
                                    notes: String::new(),
                                    status: new_fm.status,
                                    priority: new_fm.priority,
                                    issue_type: new_fm.issue_type,
                                    assignee: new_fm.assignee,
                                    estimated_minutes: None,
                                    created_at: now,
                                    updated_at: now,
                                    closed_at: None,
                                    external_ref: None,
                                    sender: String::new(),
                                    ephemeral: false,
                                    replies_to: String::new(),
                                    relates_to: Vec::new(),
                                    duplicate_of: String::new(),
                                    superseded_by: String::new(),

                                    deleted_at: None,
                                    deleted_by: String::new(),
                                    delete_reason: String::new(),
                                    original_type: String::new(),

                                    labels: new_fm.labels,
                                    dependencies: new_fm
                                        .dependencies
                                        .into_iter()
                                        .map(|fd| {
                                            use grits_core::models::Dependency;
                                            Dependency {
                                                issue_id: short_id.clone(),
                                                depends_on_id: fd.id,
                                                type_: fd.dep_type,
                                                created_at: now,
                                                created_by: user.clone(),
                                            }
                                        })
                                        .collect(),
                                    comments: Vec::new(),
                                };
                                store
                                    .create_issue(&issue)
                                    .context("Failed to create issue")?;
                                println!("Created issue {}", short_id);
                                return Ok(());
                            }
                            Err(e) => {
                                anyhow::bail!("Invalid frontmatter: {}", e);
                            }
                        }
                    } else {
                        anyhow::bail!("Invalid format: missing frontmatter delimiters");
                    }
                } else {
                    anyhow::bail!("Invalid format: file must start with ---");
                }
            }

            // Fallback only happens if description was NOT empty initially, which is handled above.
            // If description IS empty, we returned or bailed above.
            // So this path is only for non-interactive creation.
            let now = Utc::now();
            let prefix = store
                .get_config("issue_id_prefix")?
                .unwrap_or_else(|| "gr".to_string());
            let user = store
                .get_config("user.name")?
                .unwrap_or_else(|| "unknown".to_string());
            let short_id = store.generate_unique_id(&prefix, &title, &description, &user)?;

            let issue = Issue {
                id: short_id.clone(),
                content_hash: String::new(),
                title,
                description,
                design: String::new(),
                acceptance_criteria: String::new(),
                notes: String::new(),
                status: "open".to_string(),
                priority,
                issue_type: type_,
                assignee: None,
                estimated_minutes: None,
                created_at: now,
                updated_at: now,
                closed_at: None,
                external_ref: None,
                sender: String::new(),
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
            };

            store
                .create_issue(&issue)
                .context("Failed to create issue")?;
            println!("Created issue {}", short_id);
        }
        Commands::ServeMcp => {
            // Run async MCP server using tokio runtime
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async { mcp::run_server(db_path.clone()).await })?;
        }
    }

    // AUTO-EXPORT: Push any DB changes back to the Visual Engine (.jsonl)
    // This completes the "Twin Engine" tether - mutations from the CLI appear in the UI immediately.
    // Sync already handles its own export/import loop, so this is a safety measure for other commands.
    if !is_onboard && !is_mcp {
        let _ = store.export_to_jsonl(&jsonl_path, &StdFileSystem);
    }

    Ok(())
}

fn find_db_path() -> PathBuf {
    let mut current = match std::env::current_dir() {
        Ok(c) => c,
        Err(_) => return PathBuf::from(".grits/grits.db"),
    };

    loop {
        let p = current.join(".grits/grits.db");
        if p.exists() {
            return p;
        }
        if !current.pop() {
            break;
        }
    }
    // Default to .grits/grits.db in original CWD if not found (relative)
    // We can't easily get original CWD here since we popped `current`.
    // But `PathBuf::from` is relative to process CWD.
    PathBuf::from(".grits/grits.db")
}
