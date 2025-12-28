# Grits: The Definitive Agent Guide

This documentation is the **sole source of truth** for AI agents using Grits. It provides an opinionated, intent-based workflow for managing issues and code topology.

> [!IMPORTANT]
> **Twin Engine Synchronization**: Grits uses a SQLite database for speed (`.grits/grits.db`) and a JSONL file for version control (`.grits/issues.jsonl`). All CLI commands automatically import/export between these engines. **Syncing with Git** via `gr sync` is mandatory for session continuity.

---

## 🛠️ The Agent's Golden Rules

1.  **Always Forward Slashes**: Symbol IDs and file paths **must** use `/` normalization, even on Windows (e.g., `src/main.rs::run`).
2.  **Rebuild Often**: Run `gr analysis rebuild` after any significant code change to update the topological cache (`.grits/topology.json`).
3.  **Link Your Work**: Use `gr update <ID> --add-symbol <SYM>` to populate the **Focus View** in the VS Code extension.
4.  **Ready Over List**: Use `gr ready` to find actionable work. It automatically filters out blocked tasks.
5.  **Sync Before Handoff**: `gr sync` is mandatory before every session end to preserve your progress for the next agent.

---

## 📍 Phase 1: Onboarding & State
*Goal: Understand who you are and what the project looks like.*

### Initializing
```bash
# 1. Identity yourself (Used for assignee filtering and created_by field)
gr config set user.name "your_agent_name"

# 2. Initialize Grits if not already present
gr onboard
```

### High-Level Pulse
```bash
# Get overall project health and issue distribution
gr stats

# View the full dependency graph in JSON format (ideal for prompt ingestion)
gr analysis graph
```

---

## 🔍 Phase 2: Finding Work
*Goal: Identify the highest-impact, actionable task.*

### Picking a Task
```bash
# 1. See what is actually reachable (excludes blocked tasks)
gr ready

# 2. Get an AI-ranked suggestion for the next task
gr advisory next --assignee "your_agent_name"

# 3. If working on a specific file, find its context
gr analysis related "src/core/logic.rs"
```

### Searching & Triage
```bash
# Search issues using natural language (BM25 ranked)
gr analysis search "circular dependency"

# Check for potential duplicates before creating new issues
gr analysis duplicates
```

---

## 🏗️ Phase 3: Execution & Context
*Goal: Load mathematical context before making edits and track your progress.*

### Topological Context (Solid Graph)
Grits uses **Weighted PageRank** and **Simplicial Complex Analysis** to identify high-impact code.

```bash
# 1. Get the "Star Neighborhood" of a file (all connected code context)
gr analysis star "src/utils.rs" --depth 2

# 2. Find "Feature Volumes" (tightly coupled clusters of code)
gr analysis volumes "src/engine.rs"
```

### Tracking Progress
```bash
# Create an issue with a specific type and priority
gr create "Refactor store trait" -t task -p 2

# Link specific code symbols to the issue (This powers the Extension Focus View)
gr update gr-abc123 --add-symbol "src/store.rs::SqliteStore"

# Update status as you work
gr update gr-abc123 --status in_progress
```

---

## 🛡️ Phase 4: Verification & Validation
*Goal: Ensure your changes are architecturally sound.*

### Architectural Invariants
```bash
# Check the entire project against defined layers in layers.yaml
gr analysis check-layers --all

# Check a specific file for circular dependencies
gr analysis validate-topology "src/main.rs"
```

### Sync Validation
```bash
# Block synchronization if new circular dependencies are detected
gr sync --validate-topology
```

---

## 📂 Phase 5: Session Continuity & Handoff
*Goal: Preserve context for the next session or agent.*

### Bulk Operations
```bash
# Triage multiple issues at once
gr workflow triage gr-id1 gr-id2 --status in_progress --assignee "next_agent"
```

### The Mandatory Handoff
```bash
# 1. Export all changes to issues.jsonl and sync with Git
gr sync

# 2. Create a "Handoff Epic" if leaving a complex multi-step task
gr create "CONTINUITY: [Task Name] Handoff" -d "State: Finished step 2. Next: Fix step 3." -t task
```

---

## 📺 VS Code Extension Integration

The Grits CLI directly populates the VS Code Extension views. Understanding this bridge is key to "showing" your work.

| Extension View | CLI Population Mechanism |
| :--- | :--- |
| **List View** | Standard `gr list` / `gr list --all` data. |
| **Kanban View** | Grouped by standard `status` field. |
| **Graph View** | Populated by **Issue Dependencies** (`--add-dependency`) and the **Symbol Graph** (`gr analysis scan`). Central nodes are determined by PageRank. |
| **Focus View** | Populated by the **Solid Volume** of an issue. Use `gr update <ID> --add-symbol <SYM>` to add symbols to this view. |

> [!TIP]
> To "own" the UI for a human reviewer, ensure you've linked the 3-5 most relevant symbols to your active issue. This makes them appear instantly in the **Focus View**, proving you've identified the correct impact area.

---

## 🧮 Command Reference (AI Cheat Sheet)

| Command | Arguments/Flags | Use Case |
| :--- | :--- | :--- |
| `gr create` | `<TITLE>` `-d` `-t` `-p` | POSITIONAL Title first. Non-interactive. |
| `gr update` | `<ID>` `--status` `--add-symbol` `--add-dependency` | Link topology and manage blockers. |
| `gr ready` | `--assignee` | Filter for actionable tasks you own. |
| `gr advisory` | `next` `sprint` | Strategic guidance and summaries. |
| `gr analysis` | `scan` `rebuild` `star` `volumes` `check-layers` | Deep topological and architectural analysis. |
| `gr context` | `error` `diff` `todo` | Map code/runtime states back to issues. |
| `gr workflow` | `triage` `stale` | Bulk updates and repository cleaning. |
