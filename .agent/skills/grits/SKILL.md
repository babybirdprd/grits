---
name: grits-issue-tracker
description: Manages tasks and retrieves codebase context using Grits (git-native issue tracker). Use this when the user asks to list/create/update issues, check project pulse, or find related files via the dependency graph.
version: 3.0.0
---

# Grits Issue Tracker Skill

## Goal
To efficiently manage project tasks and utilize topological code context for informed development, leveraging Grits' Git-native and local-first architecture with AI agent automation features.

## Instructions

### 1. **Project State & Session Hydration**
- **Quick Status**: `gr pulse` - Shows current work, blockers, and recent commits in clean JSON
- **Context-Aware**: Most commands automatically use the focused issue when no ID is specified

### 2. **Issue Management (Automated)**
- **List Issues**: 
  - `gr list` - Human-readable table
  - `gr list --format ids` - Just IDs (one per line, perfect for scripting)
  - `gr list --format json` - Structured JSON for parsing
- **Create with Auto-Discovery & Hierarchy**: 
  - `gr create "Fix auth bug" --scan-file "src/auth.rs" --scan-file "src/user.rs" --start-work`
  - `gr create "Sub-task title" --parent <PARENT_ID>` - Automatically generates `<PARENT_ID>.n`
  - Automatically discovers symbols, adds them to issue, and starts work
- **Update with File Scanning**: 
  - `gr update --scan-file "path/to/file.rs"` - Auto-discovers and adds symbols from files
  - Uses focused issue automatically (no `--id` needed after `gr workon`)
- **Dependency & Hierarchy Management**:
  - `gr dep add <child> <parent> [--type parent|blocks|related]`
  - `gr dep add <child> <parent> --migrate` - Renames existing child to follow parent's hierarchical ID

### 3. **Context-Aware Code Analysis**
- **Smart Star Neighborhood**: 
  - `gr star` - Uses focused issue's symbols automatically (no file argument needed)
  - `gr star --format symbols` - One symbol per line (cleanest for agents)
  - `gr star --format files` - One file per line
  - `gr star --format structured` - Categorized JSON (default)
- **Enhanced Context Assembly**:
  - `gr context assemble` - Uses focused issue automatically
  - `gr context assemble --auto-expand` - Includes star neighborhoods of all symbols
  - `gr context assemble --auto-expand --format json` - Machine-readable format

### 4. **Workflow Automation**
- **One-Shot Issue Creation**: `gr create "title" --scan-file "file1.rs" --scan-file "file2.rs" --start-work`
- **Sticky Focus**: After `gr workon <id>`, all commands auto-target that issue
- **Clean Outputs**: All commands provide agent-friendly structured output with noise filtering

### 5. **Setup & Maintenance**
- **Initialization**: `gr onboard` prepares the local environment
- **Analytics**: `gr stats` provides high-level overview
- **Configuration**: `gr config [set|get|list]` manages settings
- **Portability**: `gr export` and `gr import` manage JSONL source of truth

## Agent Workflow Examples

### Efficient Issue Creation and Context Assembly:
```bash
gr create "Fix auth bug" --scan-file "src/auth.rs" --scan-file "src/user.rs" --start-work
gr star --format symbols  # Uses focused issue automatically
gr context assemble --auto-expand  # Uses focused issue + expands context
```

This workflow creates an issue, discovers symbols, starts work, and provides full context in just 3 commands.

## Output Format Examples

### Clean Star Output:
```json
{
  "connected_files": ["src/auth.rs", "src/user.rs"],
  "connected_symbols": [
    "src/auth.rs::validate_user",
    "src/user.rs::User",
    "src/auth.rs::AuthError"
  ],
  "depth": 1
}
```

### Simple Symbol List:
```
src/auth.rs::validate_user
src/user.rs::User
src/auth.rs::AuthError
```

### Clean Pulse Output:
```json
{
  "status": "ready",
  "current_work": {
    "id": "gr-abc123",
    "title": "Fix auth bug", 
    "status": "in-progress",
    "connected_files": ["src/auth.rs", "src/user.rs"]
  },
  "recent_commits": ["a1b2c3d Fix login validation"]
}
```

## Learning by Example
Review the files in the `examples/` directory for authentic input/output signatures:
- **JSON Pulse**: `examples/pulse_example.txt`
- **Issue & Symbol Linkage**: `examples/workon_example.txt`
- **Graph Results**: `examples/star_example.txt`
- **Utilities & Setup**: `examples/utility_example.txt`

## Constraints
- **Path Consistency**: Always use forward slashes (`/`) for all file paths in `gr` commands
- **Tool Usage**: Use only the `gr` CLI via `run_command`
- **Data Integrity**: Do not modify `.grits/` directly
- **Context-Aware**: Leverage sticky focus - most commands work without explicit IDs after `gr workon`
