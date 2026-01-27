use chrono::Utc;
use clap::{Parser, Subcommand};
use grits_core::{Issue, SqliteStore, StdFileSystem, Store};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gr")]
#[command(about = "Grits Issue Tracker - Graph-Lite, local-first issue management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Explicit project root directory
    #[arg(long, global = true)]
    root: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    // --- CRUD ---
    /// List issues with optional filters
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        priority: Option<i32>,
        #[arg(long = "type")]
        type_: Option<String>,
        #[arg(long)]
        label: Option<String>,
        #[arg(long)]
        sort: Option<String>,
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Show detailed information about an issue
    Show {
        id: String,
    },
    /// Update one or more fields on an issue
    Update {
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        design: Option<String>,
        #[arg(long)]
        acceptance_criteria: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long, alias = "append")]
        append_notes: bool,
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
        #[arg(long, action = clap::ArgAction::Append)]
        add_symbol: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        remove_symbol: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        scan_file: Vec<String>,
    },
    /// Create a new issue
    Create {
        title: String,
        #[arg(short, long, default_value = "")]
        description: String,
        #[arg(long, default_value = "")]
        design: String,
        #[arg(long, default_value = "")]
        acceptance_criteria: String,
        #[arg(long, default_value = "")]
        notes: String,
        #[arg(short = 't', long = "type", default_value = "bug")]
        type_: String,
        #[arg(short, long, default_value_t = 2)]
        priority: i32,
        #[arg(long, action = clap::ArgAction::Append)]
        scan_file: Vec<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        start_work: bool,
    },
    /// Close an issue
    Close {
        id: String,
    },
    /// Edit an issue in your $EDITOR
    Edit {
        id: String,
    },

    // --- WORKFLOW ---
    /// Session hydration - "What am I working on? What is blocked?"
    Pulse {
        #[arg(long)]
        assignee: Option<String>,
    },
    /// Start working on an issue (Sets sticky focus)
    Workon {
        id: Option<String>,
        #[arg(long)]
        clear: bool,
    },
    /// Get star neighborhood for a file (Connected Files)
    Star {
        file: Option<String>,
        #[arg(long, default_value_t = 1)]
        depth: usize,
        #[arg(long, default_value = "structured")]
        format: String,
    },
    /// Context-aware tools
    Context {
        #[command(subcommand)]
        command: ContextCommands,
    },

    // --- SETUP & UTILS ---
    Onboard {
        #[arg(long)]
        non_interactive: bool,
    },
    Export {
        #[arg(short, long, default_value = ".grits/issues.jsonl")]
        output: String,
    },
    Import {
        #[arg(short, long, default_value = ".grits/issues.jsonl")]
        input: String,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Stats,
    /// Manage dependencies between issues
    Dep {
        #[command(subcommand)]
        command: DepCommands,
    },
}

#[derive(Subcommand)]
enum DepCommands {
    /// Add a dependency/link between issues
    Add {
        child: String,
        parent: String,
        #[arg(short, long, default_value = "parent")]
        type_: String,
        /// Migrate the child issue to follow the parent's ID hierarchy
        #[arg(long)]
        migrate: bool,
    },
    /// Remove a dependency
    Remove { child: String, parent: String },
}

#[derive(Subcommand)]
enum ContextCommands {
    /// Assemble a mini codebase for an issue
    Assemble {
        #[arg(long)]
        issue: Option<String>,
        #[arg(long, value_delimiter = ',')]
        symbols: Option<Vec<String>>,
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long, default_value_t = 1)]
        depth: usize,
        #[arg(long)]
        auto_expand: bool,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    Set { key: String, value: String },
    Get { key: String },
    List,
}

// Helper structs for edit command frontmatter
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
    tracing_subscriber::fmt::init();

    let db_path = if matches!(cli.command, Commands::Onboard { .. }) {
        PathBuf::from(".grits/grits.db")
    } else {
        find_db_path(cli.root.clone())
    };

    if matches!(cli.command, Commands::Create { .. }) {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Ensure parent dir exists if we are onboarding
    if let Commands::Onboard { .. } = &cli.command {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let is_onboard = matches!(cli.command, Commands::Onboard { .. });
    let jsonl_path = db_path.parent().unwrap().join("issues.jsonl");

    let mut store = SqliteStore::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open DB at {:?}: {}", db_path, e))?;

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
            format,
        } => {
            let issues = store.list_issues(
                status.as_deref(),
                assignee.as_deref(),
                priority,
                type_.as_deref(),
                label.as_deref(),
                sort.as_deref(),
            )?;

            match format.as_str() {
                "json" => {
                    let json_issues: Vec<_> = issues
                        .iter()
                        .map(|i| {
                            serde_json::json!({
                                "id": i.id,
                                "title": i.title,
                                "status": i.status,
                                "priority": i.priority,
                                "type": i.issue_type
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&json_issues)?);
                }
                "ids" => {
                    for issue in issues {
                        println!("{}", issue.id);
                    }
                }
                _ => {
                    // Default table format
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
            }
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
                println!("------------------------------------------------------------");
                println!("Description:\n{}\n", issue.description);
                if !issue.design.is_empty() {
                    println!("Design (Implementation Plan):\n{}\n", issue.design);
                }
                if !issue.acceptance_criteria.is_empty() {
                    println!("Acceptance Criteria:\n{}\n", issue.acceptance_criteria);
                }
                if !issue.notes.is_empty() {
                    println!("Execution Log (Notes):\n{}\n", issue.notes);
                }
                if !issue.affected_symbols.is_empty() {
                    println!("\nConnected Files:");
                    for sym in issue.affected_symbols {
                        println!("  - {}", sym);
                    }
                }
                if !issue.dependencies.is_empty() {
                    println!("\nDependencies:");
                    for dep in issue.dependencies {
                        println!("  {} ({})", dep.depends_on_id, dep.type_);
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
            design,
            acceptance_criteria,
            notes,
            append_notes,
            status,
            priority,
            type_,
            assignee,
            add_label,
            remove_label,
            add_dependency,
            remove_dependency,
            add_symbol,
            remove_symbol,
            scan_file,
        } => {
            let grits_dir = db_path.parent().unwrap();
            let focus_path = grits_dir.join("focus");
            let resolved_id = match id {
                Some(i) => i,
                None => {
                    if focus_path.exists() {
                        std::fs::read_to_string(&focus_path)?.trim().to_string()
                    } else {
                        eprintln!(
                            "No issue ID provided. Use 'gr workon <ID>' first or provide --id."
                        );
                        std::process::exit(1);
                    }
                }
            };
            if let Some(mut issue) = store.get_issue(&resolved_id)? {
                if let Some(t) = title {
                    issue.title = t;
                }
                if let Some(d) = description {
                    issue.description = d;
                }
                if let Some(d) = design {
                    issue.design = d;
                }
                if let Some(ac) = acceptance_criteria {
                    issue.acceptance_criteria = ac;
                }
                if let Some(n) = notes {
                    if append_notes {
                        let timestamp = Utc::now().to_rfc3339();
                        if issue.notes.is_empty() {
                            issue.notes = format!("[{}] {}\n", timestamp, n);
                        } else {
                            issue.notes.push_str(&format!("\n[{}] {}\n", timestamp, n));
                        }
                    } else {
                        issue.notes = n;
                    }
                }
                if let Some(s) = status {
                    issue.status = s;
                }
                if let Some(p) = priority {
                    issue.priority = p;
                }
                if let Some(t) = type_ {
                    issue.issue_type = t;
                }
                if let Some(a) = assignee {
                    issue.assignee = if a.is_empty() { None } else { Some(a) };
                }

                // Handle Symbols
                for symbol in add_symbol {
                    if !issue.affected_symbols.contains(&symbol) {
                        issue.affected_symbols.push(symbol);
                    }
                }

                // Handle file scanning for auto-symbol discovery
                for file_path in scan_file {
                    let symbols = discover_symbols_in_file(&file_path)?;
                    for symbol in symbols {
                        if !issue.affected_symbols.contains(&symbol) {
                            issue.affected_symbols.push(symbol);
                        }
                    }
                }

                for symbol in remove_symbol {
                    issue.affected_symbols.retain(|s| s != &symbol);
                }

                // Handle Dependencies
                let user_name = store
                    .get_config("user.name")?
                    .unwrap_or("unknown".to_string());
                for dep_str in add_dependency {
                    let parts: Vec<&str> = dep_str.splitn(2, ':').collect();
                    let (dep_id, dep_type) = if parts.len() == 2 {
                        (parts[0], parts[1])
                    } else {
                        (parts[0], "blocking")
                    };
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
                    }
                }
                for dep_id in remove_dependency {
                    issue.dependencies.retain(|d| d.depends_on_id != dep_id);
                }

                // Handle Labels
                for label in add_label {
                    if !issue.labels.contains(&label) {
                        issue.labels.push(label);
                    }
                }
                for label in remove_label {
                    if let Some(pos) = issue.labels.iter().position(|l| l == &label) {
                        issue.labels.remove(pos);
                    }
                }

                issue.updated_at = Utc::now();
                store.update_issue(&issue)?;
                println!("Updated issue {}", issue.id);
            }
        }
        Commands::Create {
            title,
            description,
            design,
            acceptance_criteria,
            notes,
            type_,
            priority,
            scan_file,
            parent,
            start_work,
        } => {
            let now = Utc::now();
            let prefix = store
                .get_config("issue_id_prefix")?
                .unwrap_or("gr".to_string());
            let user = store
                .get_config("user.name")?
                .unwrap_or("unknown".to_string());

            let parent_id = match &parent {
                Some(p) => {
                    if let Some(parent_issue) = store.get_issue(p)? {
                        Some(parent_issue.id)
                    } else {
                        eprintln!("Parent issue not found: {}", p);
                        std::process::exit(1);
                    }
                }
                None => None,
            };

            let short_id = store.generate_unique_id(
                &prefix,
                &title,
                &description,
                &user,
                parent_id.as_deref(),
            )?;

            let mut issue = Issue {
                id: short_id.clone(),
                content_hash: String::new(),
                title,
                description,
                design,
                acceptance_criteria,
                notes,
                status: "open".to_string(),
                priority,
                issue_type: type_,
                created_at: now,
                updated_at: now,
                ..Default::default()
            };

            // Auto-discover symbols from scanned files
            for file_path in &scan_file {
                let symbols = discover_symbols_in_file(file_path)?;
                for symbol in symbols {
                    if !issue.affected_symbols.contains(&symbol) {
                        issue.affected_symbols.push(symbol);
                    }
                }
            }

            if let Some(p_id) = parent_id {
                use grits_core::models::Dependency;
                issue.dependencies.push(Dependency {
                    issue_id: issue.id.clone(),
                    depends_on_id: p_id,
                    type_: "parent".to_string(),
                    created_at: now,
                    created_by: user.clone(),
                });
            }

            store.create_issue(&issue)?;
            println!("Created issue {}", short_id);

            if !scan_file.is_empty() {
                println!("Auto-discovered symbols from {} files", scan_file.len());
            }

            // Auto-start work if requested
            if start_work {
                let grits_dir = db_path.parent().unwrap();
                let focus_path = grits_dir.join("focus");
                std::fs::write(&focus_path, &issue.id)?;

                issue.status = "in-progress".to_string();
                issue.updated_at = Utc::now();
                store.update_issue(&issue)?;

                println!("Started working on: {} {}", issue.id, issue.title);
            }
        }
        Commands::Close { id } => {
            if let Some(mut issue) = store.get_issue(&id)? {
                issue.status = "closed".to_string();
                issue.closed_at = Some(Utc::now());
                issue.updated_at = Utc::now();
                store.update_issue(&issue)?;
                println!("Closed issue {}", issue.id);
            }
        }
        Commands::Edit { id } => {
            if let Some(mut issue) = store.get_issue(&id)? {
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
                file.keep()?;
                edit::edit_file(&path)?;
                let new_content = std::fs::read_to_string(&path)?;
                std::fs::remove_file(path)?;

                if new_content.starts_with("---") {
                    let parts: Vec<&str> = new_content.splitn(3, "---").collect();
                    if parts.len() >= 3 {
                        let yaml_part = parts[1];
                        let body_part = parts[2].trim().to_string();
                        let new_fm: IssueFrontmatter = serde_yaml::from_str(yaml_part)?;
                        issue.title = new_fm.title;
                        issue.status = new_fm.status;
                        issue.priority = new_fm.priority;
                        issue.issue_type = new_fm.issue_type;
                        issue.assignee = new_fm.assignee;
                        issue.description = body_part;
                        issue.updated_at = Utc::now();
                        issue.labels = new_fm.labels;
                        store.update_issue(&issue)?;
                        println!("Updated issue {}", issue.id);
                    }
                }
            }
        }
        Commands::Pulse { assignee: _ } => {
            let grits_dir = db_path.parent().unwrap();
            let focus_path = grits_dir.join("focus");

            let mut pulse = serde_json::json!({
                "status": "ready"
            });

            // 1. Current Focus
            if focus_path.exists() {
                let focus_id = std::fs::read_to_string(&focus_path)?.trim().to_string();
                if let Some(issue) = store.get_issue(&focus_id)? {
                    pulse["current_work"] = serde_json::json!({
                        "id": issue.id,
                        "title": issue.title,
                        "status": issue.status,
                        "description": issue.description,
                        "design": issue.design,
                        "acceptance_criteria": issue.acceptance_criteria,
                        "notes": issue.notes,
                        "connected_files": issue.affected_symbols
                    });

                    // Check for blockers
                    let blockers: Vec<_> = issue
                        .dependencies
                        .iter()
                        .filter(|d| d.type_ == "blocking")
                        .map(|d| &d.depends_on_id)
                        .collect();
                    if !blockers.is_empty() {
                        pulse["blockers"] = serde_json::json!(blockers);
                        pulse["status"] = serde_json::json!("blocked");
                    }
                }
            } else {
                pulse["message"] =
                    serde_json::json!("No current work. Use 'gr workon <issue-id>' to start.");
            }

            // 2. Recent Activity (simplified)
            let git_root = grits_dir.parent().unwrap_or(std::path::Path::new("."));
            let git_log = std::process::Command::new("git")
                .args(["log", "--oneline", "-3", "--pretty=format:%h %s"])
                .current_dir(git_root)
                .output();
            if let Ok(output) = git_log {
                if output.status.success() {
                    let log_str = String::from_utf8_lossy(&output.stdout);
                    let commits: Vec<&str> = log_str.lines().collect();
                    if !commits.is_empty() {
                        pulse["recent_commits"] = serde_json::json!(commits);
                    }
                }
            }

            println!("{}", serde_json::to_string_pretty(&pulse)?);
        }
        Commands::Workon { id, clear } => {
            let grits_dir = db_path.parent().unwrap();
            let focus_path = grits_dir.join("focus");

            if clear {
                if focus_path.exists() {
                    std::fs::remove_file(&focus_path)?;
                }
                println!("Focus cleared.");
                return Ok(());
            }

            if let Some(issue_id) = id {
                if let Some(mut issue) = store.get_issue(&issue_id)? {
                    // Set focus
                    std::fs::write(&focus_path, &issue.id)?;

                    // Protocol check: Planner/Coder handoff
                    if issue.design.is_empty() {
                        println!("> [WARNING] Protocol Violation: 'design' (Implementation Plan) is empty.");
                        println!("> A Coder Agent should not start work until a Planner Agent has populated the design.");
                    }

                    // Update status
                    if issue.status != "in-progress" {
                        issue.status = "in-progress".to_string();
                        issue.updated_at = Utc::now();
                        store.update_issue(&issue)?;
                    }

                    // Output context
                    println!("Now working on: {} {}", issue.id, issue.title);
                    println!("{}", issue.description);
                    if !issue.affected_symbols.is_empty() {
                        println!("\nConnected Files:");
                        for sym in &issue.affected_symbols {
                            println!("  - {}", sym);
                        }
                    }
                } else {
                    eprintln!("Issue not found: {}", issue_id);
                }
            } else {
                eprintln!("Issue ID required.");
            }
        }
        Commands::Star {
            file,
            depth,
            format,
        } => {
            use grits_core::topology::{cache::TopologyCache, get_star, scanner::DirectoryScanner};

            let cache_path = db_path.parent().unwrap().join("topology.json");
            let graph = if cache_path.exists() {
                TopologyCache::load(&cache_path)?.graph
            } else {
                let scanner = DirectoryScanner::new();
                scanner.scan(std::path::Path::new("."))?
            };

            // Context-aware: use focused issue's symbols if no file specified
            let files_to_analyze = if let Some(f) = file {
                vec![f]
            } else {
                let grits_dir = db_path.parent().unwrap();
                let focus_path = grits_dir.join("focus");

                if focus_path.exists() {
                    let focus_id = std::fs::read_to_string(&focus_path)?.trim().to_string();
                    if let Some(issue) = store.get_issue(&focus_id)? {
                        if issue.affected_symbols.is_empty() {
                            eprintln!("No focused issue or files specified. Use: gr star <file>");
                            std::process::exit(1);
                        }
                        issue.affected_symbols
                    } else {
                        eprintln!("Focused issue not found. Use: gr star <file>");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("No focused issue or files specified. Use: gr star <file>");
                    std::process::exit(1);
                }
            };

            let mut all_neighbors = std::collections::HashSet::new();
            for file_path in &files_to_analyze {
                let star = get_star(&graph, file_path, depth);
                for neighbor in star.neighbors {
                    all_neighbors.insert(neighbor);
                }
            }

            // Filter and categorize symbols for cleaner output
            let mut files = Vec::new();
            let mut symbols = Vec::new();

            for neighbor in all_neighbors {
                if is_meaningful_symbol(&neighbor) {
                    if neighbor.contains("::")
                        && !neighbor.starts_with("std::")
                        && !neighbor.starts_with("serde::")
                    {
                        // This looks like a project symbol (file::symbol)
                        symbols.push(neighbor);
                    } else if neighbor.ends_with(".rs") || neighbor.contains("/") {
                        // This looks like a file path
                        files.push(neighbor);
                    }
                }
            }

            // Clean, structured output
            match format.as_str() {
                "files" => {
                    // Just the files, one per line (simplest for agents)
                    for file in files {
                        println!("{}", file);
                    }
                }
                "symbols" => {
                    // Just the symbols, one per line
                    for symbol in symbols {
                        println!("{}", symbol);
                    }
                }
                "json" => {
                    // Raw JSON array (legacy format)
                    let all: Vec<String> = files.into_iter().chain(symbols.into_iter()).collect();
                    println!("{}", serde_json::to_string_pretty(&all)?);
                }
                _ => {
                    // Default: structured format
                    let output = serde_json::json!({
                        "connected_files": files,
                        "connected_symbols": symbols,
                        "depth": depth
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
            }
        }
        Commands::Context { command } => match command {
            ContextCommands::Assemble {
                issue,
                symbols,
                format,
                depth,
                auto_expand,
            } => {
                use grits_core::context::MiniCodebase;
                use grits_core::topology::cache::TopologyCache;

                let cache_path = db_path.parent().unwrap().join("topology.json");
                let cache = if cache_path.exists() {
                    TopologyCache::load(&cache_path)?
                } else {
                    // Fallback: Create an empty cache if none exists to support "Blind" assembly
                    use grits_core::topology::SymbolGraph;
                    TopologyCache::from_graph(SymbolGraph::new())
                };

                let mut seed_symbols: Vec<String> = symbols.unwrap_or_default();
                let issue_id = if let Some(id) = issue {
                    if let Some(iss) = store.get_issue(&id)? {
                        for sym in &iss.affected_symbols {
                            if !seed_symbols.contains(sym) {
                                seed_symbols.push(sym.clone());
                            }
                        }
                        Some(iss.id)
                    } else {
                        eprintln!("Issue not found: {}", id);
                        std::process::exit(1);
                    }
                } else {
                    // Context-aware: use focused issue if no issue specified
                    let grits_dir = db_path.parent().unwrap();
                    let focus_path = grits_dir.join("focus");

                    if focus_path.exists() && seed_symbols.is_empty() {
                        let focus_id = std::fs::read_to_string(&focus_path)?.trim().to_string();
                        if let Some(iss) = store.get_issue(&focus_id)? {
                            for sym in &iss.affected_symbols {
                                if !seed_symbols.contains(sym) {
                                    seed_symbols.push(sym.clone());
                                }
                            }
                            Some(iss.id)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                // Auto-expand: include star neighborhoods of all seed symbols
                if auto_expand {
                    let mut expanded_symbols = std::collections::HashSet::new();
                    for seed in &seed_symbols {
                        let star = grits_core::topology::get_star(&cache.graph, seed, depth);
                        expanded_symbols.insert(star.center);
                        for neighbor in star.neighbors {
                            expanded_symbols.insert(neighbor);
                        }
                    }
                    seed_symbols = expanded_symbols.into_iter().collect();
                }

                // For simple assembly without PageRank threshold
                let mut mini =
                    MiniCodebase::assemble(&cache.graph, seed_symbols, depth, 0.0, issue_id);

                mini.hydrate_code(
                    db_path
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap_or(std::path::Path::new(".")),
                );

                match format.as_str() {
                    "json" => {
                        println!("{}", serde_json::to_string_pretty(&mini)?);
                    }
                    _ => {
                        println!("{}", mini.to_markdown());
                    }
                }
            }
        },
        Commands::Onboard { non_interactive: _ } => {
            if !std::path::Path::new(".git").exists() {
                println!("Not a git repository. Initializing...");
                std::process::Command::new("git").arg("init").status()?;
            }
            let grits_dir = std::path::Path::new(".grits");
            if !grits_dir.exists() {
                std::fs::create_dir(grits_dir)?;
            }

            let gitignore_path = std::path::Path::new(".gitignore");
            let mut gitignore_content = String::new();
            if gitignore_path.exists() {
                gitignore_content = std::fs::read_to_string(gitignore_path)?;
            }
            if !gitignore_content.contains("grits.db") {
                println!("Adding grits.db to .gitignore...");
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(gitignore_path)?;
                writeln!(file, "\n.grits/grits.db")?;
            }

            use grits_core::topology::{cache::TopologyCache, scanner::DirectoryScanner};
            let scanner = DirectoryScanner::new();
            let graph = scanner.scan(std::path::Path::new("."))?;
            let cache = TopologyCache::from_graph(graph);
            cache.save(&grits_dir.join("topology.json"))?;
            println!("Saved topology cache.");

            let jsonl_path = grits_dir.join("issues.jsonl");
            store.export_to_jsonl(&jsonl_path, &StdFileSystem)?;
            println!("Onboarding complete!");
        }
        Commands::Export { output } => {
            store.export_to_jsonl(std::path::Path::new(&output), &StdFileSystem)?;
        }
        Commands::Import { input } => {
            store.import_from_jsonl(std::path::Path::new(&input), &StdFileSystem)?;
        }
        Commands::Config { command } => match command {
            ConfigCommands::Set { key, value } => {
                store.set_config(&key, &value)?;
            }
            ConfigCommands::Get { key } => {
                if let Some(v) = store.get_config(&key)? {
                    println!("{}", v);
                }
            }
            ConfigCommands::List => {
                for (k, v) in store.list_config()? {
                    println!("{}={}", k, v);
                }
            }
        },
        Commands::Stats => {
            let issues = store.list_issues(None, None, None, None, None, None)?;
            println!("Total Issues: {}", issues.len());
        }
        Commands::Dep { command } => {
            match command {
                DepCommands::Add {
                    child,
                    parent,
                    type_,
                    migrate,
                } => {
                    let mut child_issue = store
                        .get_issue(&child)?
                        .ok_or_else(|| anyhow::anyhow!("Child issue not found: {}", child))?;
                    let parent_issue = store
                        .get_issue(&parent)?
                        .ok_or_else(|| anyhow::anyhow!("Parent issue not found: {}", parent))?;

                    let old_id = child_issue.id.clone();
                    let parent_id = parent_issue.id.clone();

                    if migrate {
                        let user = store
                            .get_config("user.name")?
                            .unwrap_or("unknown".to_string());
                        let new_id = store.generate_unique_id(
                            "",
                            &child_issue.title,
                            &child_issue.description,
                            &user,
                            Some(&parent_id),
                        )?;

                        println!("Migrating {} -> {}", old_id, new_id);

                        // 1. Delete old issue record
                        store
                            .execute_raw(&format!("DELETE FROM issues WHERE id = '{}'", old_id))?;
                        store.execute_raw(&format!(
                            "DELETE FROM labels WHERE issue_id = '{}'",
                            old_id
                        ))?;
                        store.execute_raw(&format!(
                            "DELETE FROM comments WHERE issue_id = '{}'",
                            old_id
                        ))?;
                        store.execute_raw(&format!(
                            "DELETE FROM dependencies WHERE issue_id = '{}'",
                            old_id
                        ))?;

                        // 2. Update all dependencies that pointed TO this issue
                        store.execute_raw(&format!("UPDATE dependencies SET depends_on_id = '{}' WHERE depends_on_id = '{}'", new_id, old_id))?;

                        // 3. Update ID and save as new
                        child_issue.id = new_id.clone();
                        child_issue.updated_at = Utc::now();
                        store.create_issue(&child_issue)?;

                        // 4. Update affected symbols/other links in issue data
                        // (They are already in the child_issue struct, create_issue will handle them)
                    }

                    // Add the dependency link
                    use grits_core::models::Dependency;
                    let user = store
                        .get_config("user.name")?
                        .unwrap_or("unknown".to_string());

                    // Check if it already exists
                    let already_exists = child_issue
                        .dependencies
                        .iter()
                        .any(|d| d.depends_on_id == parent_id && d.type_ == type_);

                    if !already_exists {
                        child_issue.dependencies.push(Dependency {
                            issue_id: child_issue.id.clone(),
                            depends_on_id: parent_id.clone(),
                            type_: type_.clone(),
                            created_at: Utc::now(),
                            created_by: user,
                        });
                        store.update_issue(&child_issue)?;
                        println!(
                            "Linked {} to {} (type: {})",
                            child_issue.id, parent_id, type_
                        );
                    } else {
                        println!("Link already exists.");
                    }
                }
                DepCommands::Remove { child, parent } => {
                    if let Some(mut issue) = store.get_issue(&child)? {
                        let original_len = issue.dependencies.len();
                        issue.dependencies.retain(|d| d.depends_on_id != parent);
                        if issue.dependencies.len() < original_len {
                            store.update_issue(&issue)?;
                            println!("Removed link from {} to {}", child, parent);
                        } else {
                            println!("No link found from {} to {}", child, parent);
                        }
                    } else {
                        eprintln!("Issue not found: {}", child);
                    }
                }
            }
        }
    }

    if !is_onboard {
        let _ = store.export_to_jsonl(&jsonl_path, &StdFileSystem);
    }

    Ok(())
}

fn find_db_path(explicit_root: Option<PathBuf>) -> PathBuf {
    if let Some(root) = explicit_root {
        return root.join(".grits/grits.db");
    }
    if let Ok(project_root) = std::env::var("GRITS_PROJECT_ROOT") {
        let project_path = PathBuf::from(&project_root);
        let db_path = project_path.join(".grits/grits.db");
        if db_path.exists() {
            return db_path;
        }
    }
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
    PathBuf::from(".grits/grits.db")
}

/// Auto-discover symbols in a file by scanning with the topology parser
fn discover_symbols_in_file(file_path: &str) -> anyhow::Result<Vec<String>> {
    use std::path::Path;

    let path = Path::new(file_path);
    if !path.exists() {
        return Ok(vec![file_path.to_string()]); // Return file path if file doesn't exist
    }

    // For now, return the file path itself as a symbol
    // This can be enhanced later to parse actual symbols using the topology parser
    Ok(vec![file_path.to_string()])
}

/// Filter out noisy/meaningless symbols for cleaner agent output
fn is_meaningful_symbol(symbol: &str) -> bool {
    // Filter out noise patterns
    if symbol.starts_with("Ok(") || symbol.starts_with("Some(") || symbol.starts_with("Err(") {
        return false;
    }

    if symbol.starts_with("drop(") || symbol.contains("row.get(") {
        return false;
    }

    // Filter out single expressions and literals
    if symbol.contains("?") && symbol.len() < 50 {
        return false;
    }

    // Filter out very short symbols (likely noise)
    if symbol.len() < 3 {
        return false;
    }

    // Filter out common stdlib imports (keep project-specific ones)
    let stdlib_prefixes = [
        "std::",
        "core::",
        "alloc::",
        "rusqlite::",
        "serde::",
        "chrono::",
        "anyhow::",
        "sha2::",
        "tokio::",
        "futures::",
        "tracing::",
    ];

    for prefix in &stdlib_prefixes {
        if symbol.starts_with(prefix) {
            return false;
        }
    }

    true
}
