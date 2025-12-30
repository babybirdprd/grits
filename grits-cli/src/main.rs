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

    /// Explicit project root directory
    #[arg(long, global = true)]
    root: Option<PathBuf>,
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
        /// Issue ID or prefix (optional if focus set via 'gr workon')
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long, alias = "notes")]
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
        #[arg(long, action = clap::ArgAction::Append)]
        add_symbol: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        remove_symbol: Vec<String>,
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
    Onboard {
        /// Skip interactive prompts
        #[arg(long)]
        non_interactive: bool,
    },
    /// Show issues ready to work on (no blockers)
    Ready {
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
    },
    /// Synchronize issues with git remote
    /// Show issue statistics
    Stats,
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Start the MCP server for AI agent integration
    ServeMcp,

    /// Strategic planning and advisor tools (AI focused)
    Advisory {
        #[command(subcommand)]
        command: AdvisoryCommands,
    },
    /// Deep issue and dependency analysis
    Analysis {
        #[command(subcommand)]
        command: AnalysisCommands,
    },
    /// Automated workflow and cleanup tools
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },
    /// Context-aware tools (errors, diffs, TODOs)
    Context {
        #[command(subcommand)]
        command: ContextCommands,
    },

    // ===== PHASE 2: Agent-Native Commands =====
    /// Inspect an issue, file, or symbol (one-shot context for agents)
    Inspect {
        /// Target: issue ID, file path, or symbol
        target: String,
    },

    /// Start working on an issue (creates branch, sets status, outputs context)
    Workon {
        /// Issue ID
        id: String,
        /// Custom branch name (default: grits/<issue_id>)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Session hydration - get project state and suggested next task
    Pulse {
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
    },

    /// Quick update with fuzzy key matching (e.g., "stat:ip pri:1 +label:bug")
    Set {
        /// Issue ID (optional if focus is set via 'gr workon')
        #[arg(long)]
        id: Option<String>,
        /// Changes in shorthand format
        changes: Vec<String>,
    },

    /// Auto-apply refactoring to break dependency cycles
    Refactor {
        /// Target file or symbol to analyze
        #[arg(long)]
        target: Option<String>,

        /// Apply the suggested refactoring (comment out weakest edge)
        #[arg(long)]
        apply: bool,

        /// Preview changes without modifying files
        #[arg(long)]
        dry_run: bool,

        /// Specific cycle index to fix (from gr inspect output)
        #[arg(long)]
        cycle: Option<usize>,

        /// Undo the last refactoring (restore from backup)
        #[arg(long)]
        undo: bool,
    },
}

#[derive(Subcommand)]
enum AdvisoryCommands {
    /// Suggest the next task to work on
    Next {
        /// Optional current file path to boost context
        #[arg(long)]
        file: Option<String>,
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
    },
    /// Summarize recent sprint activity
    Sprint {
        /// Number of days to summarize
        #[arg(long, default_value_t = 7)]
        days: i32,
    },
}

#[derive(Subcommand)]
enum AnalysisCommands {
    /// Show dependency graph (JSON output)
    Graph,
    /// Find potential duplicate issues
    Duplicates,
    /// Find issues related to a specific file
    Related { file: String },
    /// Search issues using natural language
    Search {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: i32,
    },
    /// Scan a directory recursively and build a unified symbol graph
    Scan {
        /// Directory to scan
        dir: String,
        /// Maximum depth to recurse (default: unlimited)
        #[arg(long)]
        max_depth: Option<usize>,
        /// Patterns to exclude (glob format)
        #[arg(long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,
        /// File extensions to include (default: rs,ts,js)
        #[arg(long, value_delimiter = ',')]
        extensions: Option<Vec<String>>,
        /// Output format: summary, json, or stats
        #[arg(long, default_value = "summary")]
        format: String,
    },
    /// Get code topology for an issue (symbols, dependencies)
    Topology {
        /// Issue ID to get topology for
        issue_id: String,
    },
    /// Validate if a file change creates circular dependencies
    ValidateTopology {
        /// File path to validate
        file: String,
    },
    /// Get star neighborhood for a symbol (all connected context)
    Star {
        /// File path to analyze
        file: String,
        /// Symbol name to get star for (optional, uses file if not specified)
        #[arg(long)]
        symbol: Option<String>,
        /// Depth of neighborhood (default: 1)
        #[arg(long, default_value_t = 1)]
        depth: usize,
    },
    /// Find all feature volumes (tightly coupled code clusters)
    Volumes {
        /// File path to analyze
        file: String,
    },
    /// Check layer architecture invariants
    CheckLayers {
        /// File to check (optional if --all is used)
        file: Option<String>,
        /// Layer config (YAML/JSON path or raw)
        #[arg(short, long)]
        config: Option<String>,
        /// Check the entire project graph from cache
        #[arg(short, long)]
        all: bool,
    },
    /// Rebuild the topology cache for the current project
    Rebuild {
        /// Directory to scan (default: current directory)
        dir: Option<String>,
    },
    /// Compare current project topology with the cached version
    Diff {
        /// Directory to scan (default: current directory)
        dir: Option<String>,
    },
    /// Export the topology graph to a file (DOT or JSON format)
    Export {
        /// Output file path
        output: String,
        /// Format: dot or json
        #[arg(long, default_value = "dot")]
        format: String,
        /// Directory to scan (default: current directory or cache)
        #[arg(long)]
        dir: Option<String>,
    },
    /// Prune orphaned nodes from the topology
    Prune {
        /// Delete orphaned symbols from the graph
        #[arg(long)]
        orphans: bool,
        /// Directory to scan (default: current directory)
        dir: Option<String>,
    },
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// Triage multiple issues at once
    Triage {
        /// Issue IDs to triage
        ids: Vec<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<i32>,
        #[arg(long)]
        assignee: Option<String>,
    },
    /// Find stale issues for cleanup
    Stale {
        #[arg(long, default_value_t = 30)]
        days: i32,
    },
}

#[derive(Subcommand)]
enum ContextCommands {
    /// Suggest issues matching an error message
    Error {
        message: String,
        #[arg(long, default_value_t = 5)]
        limit: i32,
    },
    /// Infer issue details from a git diff
    Diff {
        #[arg(long)]
        path: Option<String>,
    },
    /// Scan file for TODO comments
    Todo {
        file: String,
        #[arg(long)]
        line: Option<i32>,
    },
    /// Assemble a mini codebase for an issue (semantic tree-shaking for agents)
    Assemble {
        /// Issue ID to assemble context for
        #[arg(long)]
        issue: Option<String>,
        /// Seed symbols to expand from (comma-separated)
        #[arg(long, value_delimiter = ',', alias = "symbol")]
        symbols: Option<Vec<String>>,
        /// Output format: json, markdown
        #[arg(long, default_value = "markdown")]
        format: String,
        /// Neighborhood depth (default: 2)
        #[arg(long, default_value_t = 2)]
        depth: usize,
        /// Minimum PageRank threshold for inclusion (0.0-1.0)
        #[arg(long, default_value_t = 0.0)]
        threshold: f32,
    },
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
    let db_path = if matches!(cli.command, Commands::Onboard { .. }) {
        PathBuf::from(".grits/grits.db")
    } else {
        find_db_path(cli.root.clone())
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
    if let Commands::Onboard { .. } = &cli.command {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let is_onboard = matches!(cli.command, Commands::Onboard { .. });
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
            add_symbol,
            remove_symbol,
        } => {
            // Resolve ID from argument or focus file (sticky focus)
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

                // Handle Symbols
                for symbol in add_symbol {
                    if !issue.affected_symbols.contains(&symbol) {
                        issue.affected_symbols.push(symbol);
                        updated = true;
                    }
                }
                for symbol in remove_symbol {
                    let initial_len = issue.affected_symbols.len();
                    issue.affected_symbols.retain(|s| s != &symbol);
                    if issue.affected_symbols.len() != initial_len {
                        updated = true;
                    }
                }

                if updated {
                    // Update solid_volume if we have symbols
                    if !issue.affected_symbols.is_empty() {
                        use grits_core::topology::{
                            analysis::TopologicalAnalysis, cache::TopologyCache,
                        };
                        let cache_path = db_path.parent().unwrap().join("topology.json");
                        if let Ok(cache) = TopologyCache::load(&cache_path) {
                            // Compute star neighborhood for all symbols combined
                            let mut combined_neighbors = std::collections::HashSet::new();
                            let mut combined_edges = Vec::new();

                            for symbol_id in &issue.affected_symbols {
                                let star =
                                    TopologicalAnalysis::get_star(&cache.graph, symbol_id, 1);
                                for n in star.neighbors {
                                    combined_neighbors.insert(n);
                                }
                                for e in star.edges {
                                    combined_edges.push(e);
                                }
                            }

                            let volume_data = serde_json::json!({
                                "nodes": combined_neighbors,
                                "edges": combined_edges,
                            });
                            issue.solid_volume = Some(volume_data.to_string());
                        }
                    }

                    issue.updated_at = Utc::now();
                    store
                        .update_issue(&issue)
                        .context("Failed to update issue")?;
                    println!("Updated issue {}", issue.id);
                } else {
                    println!("No changes provided.");
                }
            } else {
                eprintln!("Issue not found: {}", resolved_id);
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
        Commands::Onboard { non_interactive } => {
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
            let user = if !non_interactive {
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

                if input.is_empty() {
                    default_user
                } else {
                    input.to_string()
                }
            } else {
                // Non-interactive mode: try git config or default to 'agent'
                let output = std::process::Command::new("git")
                    .args(["config", "user.name"])
                    .output();
                let mut u = if let Ok(out) = output {
                    String::from_utf8_lossy(&out.stdout).trim().to_string()
                } else {
                    "agent".to_string()
                };
                if u.is_empty() {
                    u = "agent".to_string();
                }
                u
            };

            if !user.is_empty() {
                store.set_config("user.name", &user)?;
                println!("Configured user.name = {}", user);
            } else {
                println!("No user configured.");
            }

            // Auto-build topology
            print!("Discovering files for topology...");
            use std::io::Write;
            std::io::stdout().flush().ok();
            use grits_core::topology::{cache::TopologyCache, scanner::DirectoryScanner};
            let scanner = DirectoryScanner::new();
            let graph = scanner.scan_with_progress(std::path::Path::new("."), |p| {
                if let Some(total) = p.total_files {
                    let pct = if total > 0 {
                        (p.files_scanned as f64 / total as f64 * 100.0) as u32
                    } else {
                        0
                    };
                    // Truncate filename if too long
                    let filename = if p.current_file.len() > 40 {
                        format!("...{}", &p.current_file[p.current_file.len() - 37..])
                    } else {
                        p.current_file.clone()
                    };
                    print!(
                        "\r\x1b[K[{:3}%] ({}/{}) Parsing {}",
                        pct, p.files_scanned, total, filename
                    );
                } else {
                    print!(
                        "\r\x1b[KScanning topology [{}] {}",
                        p.files_scanned, p.current_file
                    );
                }
                std::io::stdout().flush().ok();
            })?;
            println!();

            let cache = TopologyCache::from_graph(graph);
            cache.save(&grits_dir.join("topology.json"))?;
            println!("Saved topology cache.");

            // Initial export to issues.jsonl
            let jsonl_path = grits_dir.join("issues.jsonl");
            let fs = StdFileSystem;
            store.export_to_jsonl(&jsonl_path, &fs)?;
            println!("Exported initial issues.jsonl");

            println!("Onboarding complete!");
        }
        Commands::Ready {
            assignee: arg_assignee,
        } => {
            // Get current user if not provided
            let user = if arg_assignee.is_some() {
                arg_assignee
            } else {
                store.get_config("user.name")?
            };
            let display_assignee = user.as_deref().unwrap_or("unassigned");

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

            println!("Ready issues for {}:", display_assignee);
            println!(
                "{:<10} {:<10} {:<10} {}",
                "ID", "STATUS", "PRIORITY", "TITLE"
            );
            println!("{:-<60}", "");

            for candidate in &all_issues {
                if candidate.status == "closed" {
                    continue;
                }

                // Filter by assignee
                let matches_assignee = if let Some(a) = &candidate.assignee {
                    if let Some(u) = &user {
                        a == u
                    } else {
                        false
                    }
                } else {
                    true
                };

                if !matches_assignee {
                    continue;
                }

                // Fetch full issue to get dependencies
                if let Ok(Some(issue)) = store.get_issue(&candidate.id) {
                    let mut is_blocked = false;
                    for dep in &issue.dependencies {
                        if let Some(target) = all_issues.iter().find(|i| i.id == dep.depends_on_id)
                        {
                            if target.status != "closed" {
                                is_blocked = true;
                                break;
                            }
                        }
                    }

                    if !is_blocked {
                        println!(
                            "{:<10} {:<10} {:<10} {}",
                            issue.id, issue.status, issue.priority, issue.title
                        );
                    }
                }
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
            description,
            type_,
            priority,
        } => {
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
                affected_symbols: vec![],
                solid_volume: None,
                topology_hash: String::new(),
                is_solid: false,
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
        Commands::Advisory { command } => match command {
            AdvisoryCommands::Next { file, assignee } => {
                let suggestions = grits_core::strategic::advisor::get_next_task(
                    &store,
                    file.as_deref(),
                    assignee.as_deref(),
                )?;
                println!("Next suggested tasks:");
                for s in suggestions {
                    println!("{}. {} ({}) - {}", s.rank, s.title, s.id, s.reason);
                }
            }
            AdvisoryCommands::Sprint { days } => {
                let summary = grits_core::strategic::advisor::summarize_sprint(&store, days)?;
                println!("Sprint Summary (Last {} days):", days);
                println!("  Issues Created:     {}", summary.issues_created);
                println!("  Issues Closed:      {}", summary.issues_closed);
                println!("  Issues In-Progress: {}", summary.issues_in_progress);
                if !summary.closed_titles.is_empty() {
                    println!("\n  Closed:");
                    for t in summary.closed_titles {
                        println!("    - {}", t);
                    }
                }
            }
        },
        Commands::Analysis { command } => match command {
            AnalysisCommands::Graph => {
                let graph = grits_core::strategic::analysis::get_issue_graph(&store)?;
                println!("{}", serde_json::to_string_pretty(&graph)?);
            }
            AnalysisCommands::Duplicates => {
                let dups = grits_core::strategic::analysis::detect_duplicates(&store)?;
                if dups.is_empty() {
                    println!("No duplicates detected.");
                } else {
                    for d in dups {
                        println!(
                            "{}% match: {} and {}",
                            d.similarity_percent, d.issue_a, d.issue_b
                        );
                    }
                }
            }
            AnalysisCommands::Related { file } => {
                let related = grits_core::strategic::analysis::find_related_issues(&store, &file)?;
                if related.is_empty() {
                    println!("No related issues found for {}", file);
                } else {
                    for r in related {
                        println!("{} ({}) - {}", r.title, r.id, r.relevance);
                    }
                }
            }
            AnalysisCommands::Search { query, limit } => {
                let results =
                    grits_core::strategic::analysis::search_issues(&store, &query, limit)?;
                if results.is_empty() {
                    println!("No issues found matching '{}'", query);
                } else {
                    for r in results {
                        println!(
                            "{} ({}) [Score: {}] - {}",
                            r.title, r.id, r.relevance_score, r.snippet
                        );
                    }
                }
            }
            AnalysisCommands::Topology { issue_id } => {
                if let Some(issue) = store.get_issue(&issue_id)? {
                    if let Some(ref volume_json) = issue.solid_volume {
                        println!("{}", volume_json);
                    } else {
                        println!("{{\"nodes\": {{}}, \"edges\": []}}");
                    }
                } else {
                    eprintln!("Issue not found: {}", issue_id);
                }
            }
            AnalysisCommands::ValidateTopology { file } => {
                use grits_core::topology::{
                    analysis::TopologicalAnalysis, parser::CodeParser, SymbolGraph,
                };

                let lang = if file.ends_with(".rs") {
                    "rust"
                } else if file.ends_with(".ts") {
                    "typescript"
                } else if file.ends_with(".js") {
                    "javascript"
                } else {
                    println!("Skipped: Unsupported language (only .rs, .ts, .js supported)");
                    return Ok(());
                };

                let content = std::fs::read_to_string(&file)
                    .context(format!("Failed to read file: {}", file))?;

                let mut graph = SymbolGraph::new();
                let mut parser = CodeParser::new(lang).context("Failed to create parser")?;

                parser
                    .parse_file(&file, &content, &mut graph)
                    .context("Failed to parse file")?;

                // Find imports and try to load those files too
                let imports: Vec<String> = graph
                    .edges
                    .iter()
                    .filter(|(_, _, e)| e.relation == "imports")
                    .map(|(_, to, _)| to.clone())
                    .collect();

                let file_dir = std::path::Path::new(&file).parent();

                for import in imports {
                    // Try to resolve relative imports
                    let possible_paths = if let Some(dir) = file_dir {
                        vec![
                            dir.join(format!("{}.rs", import)),
                            dir.join(format!("{}.ts", import)),
                            dir.join(format!("{}.js", import)),
                            dir.join(&import).with_extension("rs"),
                            dir.join(&import).with_extension("ts"),
                            dir.join(&import).with_extension("js"),
                        ]
                    } else {
                        vec![]
                    };

                    for p in possible_paths {
                        if p.exists() {
                            if let Ok(import_content) = std::fs::read_to_string(&p) {
                                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
                                let import_lang = match ext {
                                    "rs" => "rust",
                                    "ts" => "typescript",
                                    "js" => "javascript",
                                    _ => continue,
                                };
                                if let Ok(mut sub_parser) = CodeParser::new(import_lang) {
                                    let rel = p.to_string_lossy();
                                    let _ =
                                        sub_parser.parse_file(&rel, &import_content, &mut graph);
                                }
                            }
                            break;
                        }
                    }
                }

                // Analyze topology
                let analysis = TopologicalAnalysis::analyze(&graph);

                println!("Topology Analysis for: {}", file);
                println!("  Nodes: {}", analysis.node_count);
                println!("  Edges: {}", analysis.edge_count);
                println!("  Triangles (2-simplexes): {}", analysis.triangle_count);
                println!("  Connected Components (Betti_0): {}", analysis.betti_0);
                println!("  Cycles (Betti_1): {}", analysis.betti_1);
                println!("  Feature Volumes: {}", analysis.feature_volumes.len());

                if !analysis.triangles.is_empty() {
                    println!("\n📐 Detected Triangles (tightly coupled clusters):");
                    for (i, tri) in analysis.triangles.iter().take(5).enumerate() {
                        println!(
                            "  {}. {} ↔ {} ↔ {}",
                            i + 1,
                            tri.nodes[0],
                            tri.nodes[1],
                            tri.nodes[2]
                        );
                    }
                    if analysis.triangles.len() > 5 {
                        println!("  ... and {} more", analysis.triangles.len() - 5);
                    }
                }

                if !analysis.feature_volumes.is_empty() {
                    println!("\n📦 Feature Volumes:");
                    for vol in &analysis.feature_volumes {
                        println!(
                            "  {} ({} nodes, cohesion: {:.2})",
                            vol.id,
                            vol.nodes.len(),
                            vol.cohesion_score
                        );
                    }
                }

                if analysis.betti_1 > 0 {
                    println!(
                        "\n⚠️  WARNING: {} circular dependencies detected!",
                        analysis.betti_1
                    );
                } else {
                    println!("\n✅ No circular dependencies found.");
                }
            }
            AnalysisCommands::Scan {
                dir,
                max_depth,
                exclude,
                extensions,
                format,
            } => {
                use grits_core::topology::{
                    analysis::TopologicalAnalysis, scanner::DirectoryScanner,
                };
                use std::path::Path;

                let mut scanner = DirectoryScanner::new();
                if let Some(depth) = max_depth {
                    scanner = scanner.with_max_depth(depth);
                }
                if let Some(excl) = exclude {
                    scanner = scanner.with_excludes(excl);
                }
                if let Some(exts) = extensions {
                    scanner = scanner.with_extensions(exts);
                }

                let graph = scanner.scan(Path::new(&dir))?;
                let analysis = TopologicalAnalysis::analyze(&graph);

                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(&analysis)?);
                } else if format == "stats" {
                    println!("Nodes:      {}", analysis.node_count);
                    println!("Edges:      {}", analysis.edge_count);
                    println!("Triangles:  {}", analysis.triangle_count);
                    println!("Volumes:    {}", analysis.feature_volumes.len());
                    println!("Betti_0:    {}", analysis.betti_0);
                    println!("Betti_1:    {}", analysis.betti_1);
                } else {
                    println!("Successfully scanned: {}", dir);
                    println!(
                        "Found {} symbols and {} dependencies.",
                        analysis.node_count, analysis.edge_count
                    );
                    println!("Topological Invariants:");
                    println!("  Connected Components (Betti_0): {}", analysis.betti_0);
                    println!("  Cycles (Betti_1):               {}", analysis.betti_1);
                    println!(
                        "  Feature Volumes:                {}",
                        analysis.feature_volumes.len()
                    );

                    if analysis.betti_1 > 0 {
                        println!(
                            "\n⚠️  WARNING: {} circular dependencies detected!",
                            analysis.betti_1
                        );
                    }
                }
            }
            AnalysisCommands::Star {
                file,
                symbol,
                depth,
            } => {
                use grits_core::topology::{
                    analysis::TopologicalAnalysis, parser::CodeParser, SymbolGraph,
                };

                let lang = if file.ends_with(".rs") {
                    "rust"
                } else if file.ends_with(".ts") {
                    "typescript"
                } else if file.ends_with(".js") {
                    "javascript"
                } else {
                    println!("Skipped: Unsupported language");
                    return Ok(());
                };

                let content = std::fs::read_to_string(&file)
                    .context(format!("Failed to read file: {}", file))?;

                let mut graph = SymbolGraph::new();
                let mut parser = CodeParser::new(lang).context("Failed to create parser")?;
                parser
                    .parse_file(&file, &content, &mut graph)
                    .context("Failed to parse")?;

                let center = symbol.unwrap_or_else(|| file.clone());
                let star = TopologicalAnalysis::get_star(&graph, &center, depth);

                println!("Star Neighborhood for: {}", star.center);
                println!("  Depth: {}", star.depth);
                println!("  Neighbors: {}", star.neighbors.len());

                if !star.neighbors.is_empty() {
                    println!("\n🌟 Connected Nodes:");
                    for neighbor in &star.neighbors {
                        println!("  - {}", neighbor);
                    }
                }

                if !star.edges.is_empty() {
                    println!("\n🔗 Edges:");
                    for (from, to, rel) in &star.edges {
                        println!("  {} --[{}]--> {}", from, rel, to);
                    }
                }
            }
            AnalysisCommands::Volumes { file } => {
                use grits_core::topology::{
                    analysis::TopologicalAnalysis, parser::CodeParser, SymbolGraph,
                };

                let lang = if file.ends_with(".rs") {
                    "rust"
                } else if file.ends_with(".ts") {
                    "typescript"
                } else if file.ends_with(".js") {
                    "javascript"
                } else {
                    println!("Skipped: Unsupported language");
                    return Ok(());
                };

                let content = std::fs::read_to_string(&file)
                    .context(format!("Failed to read file: {}", file))?;

                let mut graph = SymbolGraph::new();
                let mut parser = CodeParser::new(lang).context("Failed to create parser")?;
                parser
                    .parse_file(&file, &content, &mut graph)
                    .context("Failed to parse")?;

                let analysis = TopologicalAnalysis::analyze(&graph);

                println!("Feature Volumes in: {}", file);
                println!("  Total Triangles: {}", analysis.triangle_count);
                println!("  Total Volumes: {}", analysis.feature_volumes.len());

                for vol in &analysis.feature_volumes {
                    println!("\n📦 {}", vol.id);
                    println!("   Cohesion: {:.2}", vol.cohesion_score);
                    println!("   Nodes ({}):", vol.nodes.len());
                    for node in &vol.nodes {
                        println!("     - {}", node);
                    }
                }

                if analysis.feature_volumes.is_empty() {
                    println!("\nNo feature volumes found (no triangular dependencies).");
                }
            }
            AnalysisCommands::CheckLayers { file, config, all } => {
                use grits_core::topology::{
                    analysis::{InvariantResult, Layer, LayerConfig},
                    cache::TopologyCache,
                    parser::CodeParser,
                    SymbolGraph,
                };

                let mut graph = SymbolGraph::new();

                if all {
                    let cache_path = db_path.parent().unwrap().join("topology.json");
                    if !cache_path.exists() {
                        eprintln!("No cache found. Run 'gr analysis rebuild' first.");
                        return Ok(());
                    }
                    let cache = TopologyCache::load(&cache_path)?;
                    graph = cache.graph;
                } else if let Some(file_path) = &file {
                    let lang = if file_path.ends_with(".rs") {
                        "rust"
                    } else if file_path.ends_with(".ts") {
                        "typescript"
                    } else if file_path.ends_with(".js") {
                        "javascript"
                    } else {
                        println!("Skipped: Unsupported language");
                        return Ok(());
                    };

                    let content = std::fs::read_to_string(file_path)
                        .context(format!("Failed to read file: {}", file_path))?;

                    let mut parser = CodeParser::new(lang).context("Failed to create parser")?;
                    parser
                        .parse_file(file_path, &content, &mut graph)
                        .context("Failed to parse")?;
                } else {
                    eprintln!("Either a file path or --all must be provided.");
                    return Ok(());
                }

                // Parse layer config or use default
                let layer_config = if let Some(cfg) = config {
                    if std::path::Path::new(&cfg).exists() {
                        let cfg_content = std::fs::read_to_string(&cfg)?;
                        if cfg.ends_with(".yaml") || cfg.ends_with(".yml") {
                            serde_yaml::from_str(&cfg_content)?
                        } else {
                            serde_json::from_str(&cfg_content)?
                        }
                    } else {
                        // Try parsing raw string as JSON or YAML
                        serde_yaml::from_str(&cfg).or_else(|_| serde_json::from_str(&cfg))?
                    }
                } else {
                    // Default layer config
                    LayerConfig {
                        layers: vec![
                            Layer {
                                name: "domain".to_string(),
                                patterns: vec!["domain".to_string(), "model".to_string()],
                                allowed_deps: vec![],
                            },
                            Layer {
                                name: "application".to_string(),
                                patterns: vec!["service".to_string(), "handler".to_string()],
                                allowed_deps: vec!["domain".to_string()],
                            },
                            Layer {
                                name: "infrastructure".to_string(),
                                patterns: vec![
                                    "db".to_string(),
                                    "api".to_string(),
                                    "store".to_string(),
                                ],
                                allowed_deps: vec!["domain".to_string(), "application".to_string()],
                            },
                        ],
                    }
                };

                let result = InvariantResult::check(&graph, &layer_config);

                println!(
                    "Layer Invariant Check for: {}",
                    file.as_deref().unwrap_or("Full Project Graph")
                );
                println!(
                    "  Valid: {}",
                    if result.is_valid { "✅ Yes" } else { "❌ No" }
                );
                println!("  Violations: {}", result.layer_violations.len());
                println!("  Orphaned Nodes: {}", result.orphaned_nodes.len());

                if !result.layer_violations.is_empty() {
                    println!("\n⚠️  Layer Violations:");
                    for v in &result.layer_violations {
                        println!(
                            "  {} ({}) --> {} ({}): {}",
                            v.from_node, v.from_layer, v.to_node, v.to_layer, v.violation_type
                        );
                    }
                }

                if !result.orphaned_nodes.is_empty() {
                    println!("\n🔌 Orphaned Nodes:");
                    for node in &result.orphaned_nodes {
                        println!("  - {}", node);
                    }
                }
            }
            AnalysisCommands::Rebuild { dir } => {
                use grits_core::topology::{cache::TopologyCache, scanner::DirectoryScanner};
                use std::path::Path;

                let scan_dir = dir.unwrap_or_else(|| ".".to_string());
                let scanner = DirectoryScanner::new();
                let mut cache = TopologyCache::new();

                // Immediate feedback so user knows something is happening
                print!("Discovering files in '{}'...", scan_dir);
                std::io::stdout().flush().ok();

                cache.update_from_dir_with_progress(
                    Path::new(&scan_dir),
                    &scanner,
                    |progress| {
                        if let Some(total) = progress.total_files {
                            let pct = if total > 0 {
                                (progress.files_scanned as f64 / total as f64 * 100.0) as u32
                            } else {
                                0
                            };
                            // Truncate filename if too long
                            let filename = if progress.current_file.len() > 50 {
                                format!(
                                    "...{}",
                                    &progress.current_file[progress.current_file.len() - 47..]
                                )
                            } else {
                                progress.current_file.clone()
                            };
                            print!(
                                "\r\x1b[K[{:3}%] ({}/{}) Parsing {}",
                                pct, progress.files_scanned, total, filename
                            );
                        } else {
                            print!(
                                "\r\x1b[KScanning [{}] {}",
                                progress.files_scanned, progress.current_file
                            );
                        }
                        std::io::stdout().flush().ok();
                    },
                )?;
                println!(); // New line after progress

                let cache_path = db_path.parent().unwrap().join("topology.json");
                cache.save(&cache_path)?;

                let analysis =
                    grits_core::topology::analysis::TopologicalAnalysis::analyze(&cache.graph);
                println!("✅ Cache saved to: {:?}", cache_path);
                println!(
                    "   {} nodes, {} edges, {} cycles detected",
                    analysis.node_count, analysis.edge_count, analysis.betti_1
                );
            }
            AnalysisCommands::Diff { dir } => {
                use grits_core::topology::{cache::TopologyCache, scanner::DirectoryScanner};
                use std::path::Path;

                let scan_dir = dir.unwrap_or_else(|| ".".to_string());
                let scanner = DirectoryScanner::new();
                let cache_path = db_path.parent().unwrap().join("topology.json");

                if !cache_path.exists() {
                    eprintln!("No cache found. Run 'gr analysis rebuild' first.");
                    return Ok(());
                }

                let old_cache = TopologyCache::load(&cache_path)?;
                let new_graph = scanner.scan(Path::new(&scan_dir))?;

                // Compare graphs
                let old_edges: std::collections::HashSet<_> = old_cache
                    .graph
                    .edges
                    .iter()
                    .map(|(f, t, e)| (f.clone(), t.clone(), e.relation.clone()))
                    .collect();
                let new_edges: std::collections::HashSet<_> = new_graph
                    .edges
                    .iter()
                    .map(|(f, t, e)| (f.clone(), t.clone(), e.relation.clone()))
                    .collect();

                let added: Vec<_> = new_edges.difference(&old_edges).collect();
                let removed: Vec<_> = old_edges.difference(&new_edges).collect();

                println!("Topology Diff for: {}", scan_dir);
                println!("  Added Edges:   {}", added.len());
                println!("  Removed Edges: {}", removed.len());

                if !added.is_empty() {
                    println!("\n➕ Added Dependencies:");
                    for (from, to, rel) in added.iter().take(10) {
                        println!("  {} --[{}]--> {}", from, rel, to);
                    }
                    if added.len() > 10 {
                        println!("  ... and {} more", added.len() - 10);
                    }
                }

                if !removed.is_empty() {
                    println!("\n➖ Removed Dependencies:");
                    for (from, to, rel) in removed.iter().take(10) {
                        println!("  {} --[{}]--> {}", from, rel, to);
                    }
                    if removed.len() > 10 {
                        println!("  ... and {} more", removed.len() - 10);
                    }
                }

                // Check invariants
                let old_analysis =
                    grits_core::topology::analysis::TopologicalAnalysis::analyze(&old_cache.graph);
                let new_analysis =
                    grits_core::topology::analysis::TopologicalAnalysis::analyze(&new_graph);

                if new_analysis.betti_1 > old_analysis.betti_1 {
                    println!(
                        "\n⚠️  WARNING: Circular dependencies increased! ({} -> {})",
                        old_analysis.betti_1, new_analysis.betti_1
                    );
                }
                if new_analysis.betti_2 < old_analysis.betti_2 {
                    println!("\n💡 INFO: Topological voids decreased. Architecture is becoming more 'solid'.");
                }
            }
            AnalysisCommands::Export {
                output,
                format,
                dir,
            } => {
                use grits_core::topology::{cache::TopologyCache, scanner::DirectoryScanner};
                use std::path::Path;

                let graph = if let Some(d) = dir {
                    let scanner = DirectoryScanner::new();
                    scanner.scan(Path::new(&d))?
                } else {
                    let cache_path = db_path.parent().unwrap().join("topology.json");
                    if cache_path.exists() {
                        TopologyCache::load(&cache_path)?.graph
                    } else {
                        let scanner = DirectoryScanner::new();
                        scanner.scan(Path::new("."))?
                    }
                };

                if format == "json" {
                    let json = serde_json::to_string_pretty(&graph)?;
                    std::fs::write(&output, json)?;
                } else {
                    // Export to DOT
                    let mut dot = String::from("digraph G {\n");
                    dot.push_str("  rankdir=LR;\n");
                    dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n\n");

                    for (id, symbol) in &graph.nodes {
                        dot.push_str(&format!(
                            "  \"{}\" [label=\"{}\\n({})\", color=\"{}\"];\n",
                            id,
                            symbol.name,
                            symbol.kind,
                            if symbol.kind == "function" {
                                "green"
                            } else {
                                "blue"
                            }
                        ));
                    }

                    for (from, to, edge) in &graph.edges {
                        dot.push_str(&format!(
                            "  \"{}\" -> \"{}\" [label=\"{}\", penwidth={}];\n",
                            from,
                            to,
                            edge.relation,
                            if edge.strength > 0.8 { "2.0" } else { "1.0" }
                        ));
                    }

                    dot.push_str("}\n");
                    std::fs::write(&output, dot)?;
                }
                println!("Graph exported to: {}", output);
            }
            AnalysisCommands::Prune { orphans, dir } => {
                use grits_core::topology::{cache::TopologyCache, scanner::DirectoryScanner};
                use std::path::Path;

                if !orphans {
                    println!("Nothing to prune. Use --orphans.");
                    return Ok(());
                }

                let scan_dir = dir.unwrap_or_else(|| ".".to_string());
                let scanner = DirectoryScanner::new();
                let cache_path = db_path.parent().unwrap().join("topology.json");

                if !cache_path.exists() {
                    eprintln!("No cache found. Run 'gr analysis rebuild' first.");
                    return Ok(());
                }

                let mut cache = TopologyCache::load(&cache_path)?;
                let current_graph = scanner.scan(Path::new(&scan_dir))?;

                // Identify orphaned nodes: present in cache but NOT in current scan
                let current_nodes: std::collections::HashSet<_> =
                    current_graph.nodes.keys().cloned().collect();
                let cache_nodes: Vec<_> = cache.graph.nodes.keys().cloned().collect();

                let orphans_list: Vec<_> = cache_nodes
                    .into_iter()
                    .filter(|id| !current_nodes.contains(id))
                    .collect();

                if orphans_list.is_empty() {
                    println!("No orphaned nodes found.");
                    return Ok(());
                }

                println!("Found {} orphaned nodes.", orphans_list.len());
                for id in &orphans_list {
                    cache.graph.nodes.remove(id);
                    // Also remove any edges connected to this node
                    cache
                        .graph
                        .edges
                        .retain(|(from, to, _)| from != id && to != id);
                    println!("  - Pruned: {}", id);
                }

                cache.save(&cache_path)?;
                println!(
                    "✅ Shaved {} orphans. Topology is now cleaner.",
                    orphans_list.len()
                );
            }
        },
        Commands::Workflow { command } => match command {
            WorkflowCommands::Triage {
                ids,
                status,
                priority,
                assignee,
            } => {
                let result = grits_core::strategic::workflow::bulk_triage(
                    &mut store, ids, status, priority, assignee,
                )?;
                println!(
                    "Updated: {}, Failed: {}",
                    result.updated_count,
                    result.failed_ids.len()
                );
                if !result.failed_ids.is_empty() {
                    println!("Failed IDs: {:?}", result.failed_ids);
                }
            }
            WorkflowCommands::Stale { days } => {
                let stale = grits_core::strategic::workflow::cleanup_stale(&store, days)?;
                if stale.is_empty() {
                    println!("No stale issues found.");
                } else {
                    for s in stale {
                        println!(
                            "{} ({}) - Last updated {} days ago",
                            s.title, s.id, s.days_inactive
                        );
                    }
                }
            }
        },
        Commands::Context { command } => match command {
            ContextCommands::Error { message, limit } => {
                let matches = grits_core::strategic::context::suggest_issue_for_error(
                    &store, &message, limit,
                )?;
                if matches.is_empty() {
                    println!("No matching issues found for error.");
                } else {
                    for m in matches {
                        println!("Match Score {}: {} ({})", m.match_score, m.title, m.id);
                    }
                }
            }
            ContextCommands::Diff { path } => {
                let diff = if let Some(p) = path {
                    std::fs::read_to_string(p)?
                } else {
                    // Try to get diff via git
                    let grits_dir = db_path.parent().unwrap();
                    let git_root = grits_dir.parent().unwrap_or(std::path::Path::new("."));
                    let output = std::process::Command::new("git")
                        .args(["diff", "HEAD~1", "HEAD"])
                        .current_dir(git_root)
                        .output()?;
                    String::from_utf8_lossy(&output.stdout).to_string()
                };
                let inferred = grits_core::strategic::context::infer_issue_from_diff(&diff)?;
                println!("Inferred Issue:");
                println!("  Title:       {}", inferred.suggested_title);
                println!("  Type:        {}", inferred.suggested_type);
                println!("  Description: {}", inferred.suggested_description);

                // Topological Check
                let cache_path = db_path.parent().unwrap().join("topology.json");
                if cache_path.exists() {
                    use grits_core::topology::{
                        analysis::TopologicalAnalysis, cache::TopologyCache,
                        scanner::DirectoryScanner,
                    };
                    if let Ok(old_cache) = TopologyCache::load(&cache_path) {
                        let scanner = DirectoryScanner::new();
                        if let Ok(new_graph) = scanner.scan(std::path::Path::new(".")) {
                            let old_analysis = TopologicalAnalysis::analyze(&old_cache.graph);
                            let new_analysis = TopologicalAnalysis::analyze(&new_graph);
                            if new_analysis.betti_1 > old_analysis.betti_1 {
                                println!("\n⚠️  TOPOLOGY WARNING: This change introduces new circular dependencies ({} -> {}).", old_analysis.betti_1, new_analysis.betti_1);
                            }
                        }
                    }
                }
            }
            ContextCommands::Todo { file, line } => {
                let content = std::fs::read_to_string(&file)?;
                let todos = grits_core::strategic::context::generate_issue_from_todo(
                    &file, &content, line,
                )?;
                if todos.is_empty() {
                    println!("No TODOs found in {}", file);
                } else {
                    for t in todos {
                        println!(
                            "Line {}: {} (Suggested: {})",
                            t.line, t.text, t.suggested_title
                        );
                    }
                }
            }
            ContextCommands::Assemble {
                issue,
                symbols,
                format,
                depth,
                threshold,
            } => {
                use grits_core::context::MiniCodebase;
                use grits_core::topology::cache::TopologyCache;

                let cache_path = db_path.parent().unwrap().join("topology.json");

                // Ensure we have a topology cache
                if !cache_path.exists() {
                    eprintln!("Topology cache not found. Run 'gr analysis rebuild' first.");
                    std::process::exit(1);
                }

                let cache = TopologyCache::load(&cache_path)?;

                // Collect seed symbols from issue or direct symbols argument
                let mut seed_symbols: Vec<String> = symbols.unwrap_or_default();
                let issue_id = if let Some(id) = issue {
                    // Get issue and add its affected symbols
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
                    None
                };

                if seed_symbols.is_empty() {
                    eprintln!(
                        "No seed symbols. Provide --symbols or --issue with affected_symbols."
                    );
                    std::process::exit(1);
                }

                // Assemble the mini codebase
                let mini =
                    MiniCodebase::assemble(&cache.graph, seed_symbols, depth, threshold, issue_id);

                // Output in requested format
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

        // ===== PHASE 2: Agent-Native Command Handlers =====
        Commands::Inspect { target } => {
            use grits_core::topology::{analysis::TopologicalAnalysis, cache::TopologyCache};

            let cache_path = db_path.parent().unwrap().join("topology.json");

            // Determine if target is an issue ID, file, or symbol
            let mut result = serde_json::json!({});

            // Try as issue ID first
            if let Some(issue) = store.get_issue(&target)? {
                result["issue"] = serde_json::json!({
                    "id": issue.id,
                    "title": issue.title,
                    "status": issue.status,
                    "priority": issue.priority,
                    "type": issue.issue_type,
                    "description": issue.description,
                    "labels": issue.labels,
                    "assignee": issue.assignee,
                    "affected_symbols": issue.affected_symbols,
                });

                // Get solid volume for affected symbols
                if cache_path.exists() && !issue.affected_symbols.is_empty() {
                    if let Ok(cache) = TopologyCache::load(&cache_path) {
                        let mut all_neighbors = std::collections::HashSet::new();
                        let mut all_edges = Vec::new();

                        for symbol in &issue.affected_symbols {
                            let star = TopologicalAnalysis::get_star(&cache.graph, symbol, 1);
                            for n in star.neighbors {
                                all_neighbors.insert(n);
                            }
                            all_edges.extend(star.edges);
                        }

                        let analysis = TopologicalAnalysis::analyze(&cache.graph);

                        result["solid_volume"] = serde_json::json!({
                            "symbols": issue.affected_symbols,
                            "neighbors": all_neighbors,
                            "edges": all_edges,
                            "triangles": analysis.triangles.iter()
                                .filter(|t| issue.affected_symbols.iter().any(|s| t.nodes.contains(s)))
                                .collect::<Vec<_>>(),
                        });
                    }
                }

                // Find related issues via BM25
                let related =
                    grits_core::strategic::analysis::search_issues(&store, &issue.title, 3)?;
                result["related_issues"] = serde_json::json!(
                    related.into_iter()
                        .filter(|r| r.id != issue.id)
                        .map(|r| serde_json::json!({"id": r.id, "title": r.title, "score": r.relevance_score}))
                        .collect::<Vec<_>>()
                );
            } else if cache_path.exists() {
                // Try as file or symbol
                if let Ok(cache) = TopologyCache::load(&cache_path) {
                    let target_normalized = target.replace('\\', "/");
                    let star = TopologicalAnalysis::get_star(&cache.graph, &target_normalized, 2);

                    if !star.neighbors.is_empty() {
                        result["star_neighborhood"] = serde_json::json!({
                            "center": star.center,
                            "neighbors": star.neighbors,
                            "edges": star.edges,
                            "depth": star.depth,
                        });

                        // Get analysis for context
                        let analysis = TopologicalAnalysis::analyze(&cache.graph);
                        let score = analysis.solid_score();
                        result["solid_score"] = serde_json::json!({
                            "normalized": score.normalized,
                            "betti_cycles": analysis.betti_1,
                        });
                    }
                }
            }

            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::Workon { id, branch } => {
            use grits_core::topology::{analysis::TopologicalAnalysis, cache::TopologyCache};

            if let Some(mut issue) = store.get_issue(&id)? {
                let grits_dir = db_path.parent().unwrap();
                let git_root = grits_dir.parent().unwrap_or(std::path::Path::new("."));

                // 1. Only create/switch branch if --branch is explicitly provided
                let branch_name = if let Some(b) = branch {
                    let name = if b.is_empty() {
                        format!("grits/{}", issue.id)
                    } else {
                        b
                    };

                    let checkout = std::process::Command::new("git")
                        .args(["checkout", "-b", &name])
                        .current_dir(git_root)
                        .output();

                    match checkout {
                        Ok(output) if output.status.success() => {
                            println!("✓ Created branch: {}", name);
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            if stderr.contains("already exists") {
                                let _ = std::process::Command::new("git")
                                    .args(["checkout", &name])
                                    .current_dir(git_root)
                                    .output();
                                println!("✓ Switched to existing branch: {}", name);
                            } else {
                                eprintln!("⚠️ Could not create branch: {}", stderr);
                            }
                        }
                        Err(e) => eprintln!("⚠️ Git not available: {}", e),
                    }
                    Some(name)
                } else {
                    None
                };

                // 2. Set status to in-progress
                if issue.status != "in-progress" {
                    issue.status = "in-progress".to_string();
                    issue.updated_at = Utc::now();
                    store.update_issue(&issue)?;
                    println!("✓ Status set to: in-progress");
                }

                // 3. Save focus ID to .grits/focus for "sticky focus"
                let focus_path = grits_dir.join("focus");
                std::fs::write(&focus_path, &issue.id)?;
                println!("✓ Focus set to: {} (use 'gr set' without ID)", issue.id);

                // 4. Output context (similar to inspect)
                let cache_path = db_path.parent().unwrap().join("topology.json");
                let mut context = serde_json::json!({
                    "issue": {
                        "id": issue.id,
                        "title": issue.title,
                        "description": issue.description,
                        "priority": issue.priority,
                        "labels": issue.labels,
                        "affected_symbols": issue.affected_symbols,
                    },
                    "focus": issue.id.clone(),
                });

                if let Some(b) = branch_name {
                    context["branch"] = serde_json::json!(b);
                }

                if cache_path.exists() && !issue.affected_symbols.is_empty() {
                    if let Ok(cache) = TopologyCache::load(&cache_path) {
                        for symbol in &issue.affected_symbols {
                            let star = TopologicalAnalysis::get_star(&cache.graph, symbol, 1);
                            context["star_neighborhood"] = serde_json::json!({
                                "center": star.center,
                                "neighbors": star.neighbors,
                                "edges": star.edges,
                            });
                        }
                    }
                }

                println!("\n{}", serde_json::to_string_pretty(&context)?);
            } else {
                eprintln!("Issue not found: {}", id);
            }
        }

        Commands::Pulse { assignee } => {
            use grits_core::topology::{analysis::TopologicalAnalysis, cache::TopologyCache};

            // Check if grits is initialized
            if !db_path.exists() {
                eprintln!("Grits not initialized. Run `gr onboard` first to set up the project.");
                std::process::exit(1);
            }

            let cache_path = db_path.parent().unwrap().join("topology.json");
            let grits_dir = db_path.parent().unwrap();
            let git_root = grits_dir.parent().unwrap_or(std::path::Path::new("."));

            let mut pulse = serde_json::json!({});

            // 1. Solid Score
            if cache_path.exists() {
                if let Ok(cache) = TopologyCache::load(&cache_path) {
                    let analysis = TopologicalAnalysis::analyze(&cache.graph);
                    let score = analysis.solid_score();
                    pulse["solid_score"] = serde_json::json!({
                        "value": format!("{:.2}", score.normalized),
                        "betti_0": analysis.betti_0,
                        "betti_1": analysis.betti_1,
                        "betti_2": analysis.betti_2,
                        "triangles": analysis.triangle_count,
                    });
                }
            }

            // 2. In-progress issues (filtered by assignee if provided)
            let in_progress = store.list_issues(
                Some("in-progress"),
                assignee.as_deref(),
                None,
                None,
                None,
                None,
            )?;
            pulse["in_progress"] = serde_json::json!(in_progress
                .iter()
                .map(
                    |i| serde_json::json!({"id": &i.id, "title": &i.title, "priority": i.priority, "assignee": &i.assignee})
                )
                .collect::<Vec<_>>());

            // 2b. Unassigned in-progress issues (always shown for visibility)
            let all_in_progress = store.list_issues(
                Some("in-progress"),
                None, // No assignee filter
                None,
                None,
                None,
                None,
            )?;
            let unassigned: Vec<_> = all_in_progress
                .iter()
                .filter(|i| i.assignee.is_none())
                .map(
                    |i| serde_json::json!({"id": &i.id, "title": &i.title, "priority": i.priority}),
                )
                .collect();
            if !unassigned.is_empty() {
                pulse["unassigned_in_progress"] = serde_json::json!(unassigned);
            }

            // 3. Recent git activity
            let git_log = std::process::Command::new("git")
                .args(["log", "--oneline", "-3"])
                .current_dir(git_root)
                .output();

            if let Ok(output) = git_log {
                if output.status.success() {
                    let log_str = String::from_utf8_lossy(&output.stdout);
                    pulse["recent_commits"] = serde_json::json!(log_str
                        .lines()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>());
                }
            }

            // 4. Suggested next task
            let next_tasks =
                grits_core::strategic::advisor::get_next_task(&store, None, assignee.as_deref())?;
            if let Some(task) = next_tasks.first() {
                pulse["suggested_next"] = serde_json::json!({
                    "id": task.id,
                    "title": task.title,
                    "priority": task.priority,
                    "reason": task.reason,
                });
            }

            println!("{}", serde_json::to_string_pretty(&pulse)?);
        }

        Commands::Set { id, changes } => {
            // Get issue ID from argument or focus file
            let grits_dir = db_path.parent().unwrap();
            let focus_path = grits_dir.join("focus");

            let resolved_id = match id {
                Some(i) => i,
                None => {
                    if focus_path.exists() {
                        std::fs::read_to_string(&focus_path)?.trim().to_string()
                    } else {
                        eprintln!(
                            "No issue ID provided. Use 'gr workon <ID>' first or provide ID."
                        );
                        std::process::exit(1);
                    }
                }
            };

            if let Some(mut issue) = store.get_issue(&resolved_id)? {
                let mut updated = false;

                for change in changes {
                    // Parse: "key:value" or "+label:name" or "-label:name"
                    if change.starts_with('+') {
                        // Add label
                        if let Some(label) = change
                            .strip_prefix("+label:")
                            .or_else(|| change.strip_prefix("+l:"))
                        {
                            if !issue.labels.contains(&label.to_string()) {
                                issue.labels.push(label.to_string());
                                updated = true;
                                println!("+ Added label: {}", label);
                            }
                        }
                    } else if change.starts_with('-') {
                        // Remove label
                        if let Some(label) = change
                            .strip_prefix("-label:")
                            .or_else(|| change.strip_prefix("-l:"))
                        {
                            if let Some(pos) = issue.labels.iter().position(|l| l == label) {
                                issue.labels.remove(pos);
                                updated = true;
                                println!("- Removed label: {}", label);
                            }
                        }
                    } else if let Some((key, value)) = change.split_once(':') {
                        // Fuzzy key matching
                        let resolved_key = match key.to_lowercase().as_str() {
                            "stat" | "status" | "s" => "status",
                            "pri" | "priority" | "p" => "priority",
                            "type" | "t" => "type",
                            "assignee" | "assign" | "a" => "assignee",
                            "title" => "title",
                            "desc" | "description" => "description",
                            _ => {
                                eprintln!("Unknown key: {}", key);
                                continue;
                            }
                        };

                        match resolved_key {
                            "status" => {
                                // Fuzzy value matching
                                let resolved_status = match value.to_lowercase().as_str() {
                                    "o" | "open" => "open",
                                    "ip" | "in-progress" | "inprogress" | "progress" => {
                                        "in-progress"
                                    }
                                    "b" | "blocked" => "blocked",
                                    "c" | "closed" | "done" => "closed",
                                    _ => value,
                                };
                                issue.status = resolved_status.to_string();
                                updated = true;
                                println!("✓ status = {}", resolved_status);
                            }
                            "priority" => {
                                if let Ok(p) = value.parse::<i32>() {
                                    issue.priority = p;
                                    updated = true;
                                    println!("✓ priority = {}", p);
                                }
                            }
                            "type" => {
                                issue.issue_type = value.to_string();
                                updated = true;
                                println!("✓ type = {}", value);
                            }
                            "assignee" => {
                                issue.assignee = if value.is_empty() || value == "-" {
                                    None
                                } else {
                                    Some(value.to_string())
                                };
                                updated = true;
                                println!("✓ assignee = {:?}", issue.assignee);
                            }
                            "title" => {
                                issue.title = value.to_string();
                                updated = true;
                                println!("✓ title updated");
                            }
                            "description" => {
                                issue.description = value.to_string();
                                updated = true;
                                println!("✓ description updated");
                            }
                            _ => {}
                        }
                    }
                }

                if updated {
                    issue.updated_at = Utc::now();
                    store.update_issue(&issue)?;
                    println!("\nUpdated issue {}", issue.id);
                } else {
                    println!("No valid changes applied.");
                }
            } else {
                eprintln!("Issue not found: {}", resolved_id);
            }
        }

        Commands::Refactor {
            target,
            apply,
            dry_run,
            cycle,
            undo,
        } => {
            use grits_core::topology::{
                analysis::TopologicalAnalysis,
                cache::TopologyCache,
                refactor::{get_backup_dir, undo_refactor, RefactorAction},
            };

            let grits_dir = jsonl_path.parent().unwrap_or(std::path::Path::new("."));
            let topology_path = grits_dir.join("topology.json");
            let backup_dir = get_backup_dir(grits_dir);

            // Handle undo
            if undo {
                if let Some(file) = target.as_ref() {
                    match undo_refactor(file, &backup_dir) {
                        Ok(()) => println!("✓ Restored {} from backup", file),
                        Err(e) => eprintln!("Failed to undo: {}", e),
                    }
                } else {
                    eprintln!("--undo requires --target <file>");
                }
                return Ok(());
            }

            // Load topology
            if !topology_path.exists() {
                eprintln!("No topology cache found. Run 'gr analysis rebuild' first.");
                return Ok(());
            }

            let cache = TopologyCache::load(&topology_path)?;
            let analysis = TopologicalAnalysis::analyze(&cache.graph);

            // Get edge persistence to find cycles
            let edge_persistence = TopologicalAnalysis::compute_edge_persistence(&cache.graph);

            // Group by cycle_id to find unique cycles
            let mut cycle_ids: Vec<usize> = edge_persistence.iter().map(|e| e.cycle_id).collect();
            cycle_ids.sort();
            cycle_ids.dedup();

            if analysis.betti_1 == 0 || cycle_ids.is_empty() {
                println!(
                    "{}",
                    serde_json::json!({
                        "cycles_detected": 0,
                        "message": "No dependency cycles found. Architecture is clean!"
                    })
                );
                return Ok(());
            }

            // Select which cycle to fix
            let cycle_idx = cycle.unwrap_or(0);
            if cycle_idx >= cycle_ids.len() {
                eprintln!(
                    "Cycle index {} out of range (found {} cycles)",
                    cycle_idx,
                    cycle_ids.len()
                );
                return Ok(());
            }

            let selected_cycle_id = cycle_ids[cycle_idx];

            // Get suggested refactor for this cycle
            let suggestion = TopologicalAnalysis::suggest_refactor(&cache.graph, selected_cycle_id);

            // Get edges in this cycle for display
            let cycle_edges: Vec<_> = edge_persistence
                .iter()
                .filter(|e| e.cycle_id == selected_cycle_id)
                .map(|e| format!("{} -> {}", e.source, e.target))
                .collect();

            let mut result = serde_json::json!({
                "cycles_detected": cycle_ids.len(),
                "betti_1": analysis.betti_1,
                "selected_cycle": cycle_idx,
                "cycle_edges": cycle_edges,
            });

            if let Some(edge) = suggestion {
                // Find the source file and line for this edge
                let source_symbol = cache.graph.nodes.get(&edge.source);
                let (file_path, line) = if let Some(sym) = source_symbol {
                    (sym.file_path.clone(), 1) // TODO: Get actual line from AST
                } else {
                    (
                        edge.source
                            .split("::")
                            .next()
                            .unwrap_or("unknown")
                            .to_string(),
                        1,
                    )
                };

                // Create the refactor action
                let action = RefactorAction::comment_out(
                    &file_path,
                    line,
                    line,
                    &format!("// Import/call: {} -> {}", edge.source, edge.target),
                    &edge.source,
                    &edge.target,
                );

                result["suggested_refactor"] = serde_json::json!({
                    "edge": { "from": edge.source, "to": edge.target },
                    "file": file_path,
                    "line": line,
                    "action": "comment_out",
                    "confidence": 1.0 - edge.lifetime.min(1.0),
                    "reasoning": format!("Edge has persistence {:.2} (lower = weaker link)", edge.lifetime),
                });

                if apply || dry_run {
                    let diff = action.preview_diff();
                    result["preview"] = serde_json::json!(diff);

                    if apply && !dry_run {
                        match action.apply(Some(&backup_dir)) {
                            Ok(()) => {
                                result["applied"] = serde_json::json!(true);
                                result["backup_location"] =
                                    serde_json::json!(backup_dir.display().to_string());

                                // Auto-stage with git
                                let _ = std::process::Command::new("git")
                                    .args(["add", &file_path])
                                    .output();
                            }
                            Err(e) => {
                                result["applied"] = serde_json::json!(false);
                                result["error"] = serde_json::json!(e.to_string());
                            }
                        }
                    }
                }
            } else {
                result["suggested_refactor"] = serde_json::json!(null);
                result["message"] = serde_json::json!("No weak edge found in cycle");
            }

            println!("{}", serde_json::to_string_pretty(&result)?);
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

fn find_db_path(explicit_root: Option<PathBuf>) -> PathBuf {
    // First priority: Explicit root flag from CLI
    if let Some(root) = explicit_root {
        return root.join(".grits/grits.db");
    }

    // Second priority: GRITS_PROJECT_ROOT environment variable
    if let Ok(project_root) = std::env::var("GRITS_PROJECT_ROOT") {
        let project_path = PathBuf::from(&project_root);
        let db_path = project_path.join(".grits/grits.db");
        if db_path.exists() {
            return db_path;
        }
        // If the .grits directory or .git exists, use this path
        // (DB will be created on first use)
        let grits_dir = project_path.join(".grits");
        if grits_dir.exists() || project_path.join(".git").exists() {
            return db_path;
        }
    }

    // Fallback: walk up from current directory (original behavior)
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
