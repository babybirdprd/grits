# Grits: The Definitive Agent Guide

This documentation is the **sole source of truth** for AI agents using Grits. It provides an opinionated, intent-based workflow for managing issues and code topology.

> [!IMPORTANT]
> **Twin Engine Synchronization**: Grits uses a SQLite database for speed (`.grits/grits.db`) and a JSONL file for version control (`.grits/issues.jsonl`). All CLI commands automatically import/export between these engines. **Syncing with Git** via `gr sync` is mandatory for session continuity.

---

## 🛠️ The Agent's Golden Rules

1.  **Always Forward Slashes**: Symbol IDs and file paths **must** use `/` normalization, even on Windows (e.g., `src/main.rs::run`).
2.  **Run `gr pulse` First**: Start every session with `gr pulse` for instant context.
3.  **Use `gr workon`**: Start work with `gr workon <id>` to auto-set status and create branch.
4.  **Link Your Work**: Use `gr update <ID> --add-symbol <SYM>` to populate the **Focus View**.
5.  **Fix Cycles**: Use `gr refactor` to detect and auto-fix architectural issues.
6.  **Sync Before Handoff**: `gr sync` is mandatory before every session end.

---

## 🚀 Phase 1: Session Hydration

### Start Every Session With Pulse
```bash
# Get instant project context: in-progress issues, recent commits, suggested next task
gr pulse

# Filter by your assignee
gr pulse --assignee your_agent_name
```

**Output includes:**
- Solid Score (architectural health 0-100%)
- In-progress issues
- Last 5 Git commits  
- AI-suggested next task

### Quick Identity Check
```bash
gr config set user.name "your_agent_name"
```

---

## 🔍 Phase 2: Context Loading

### One-Shot Context with Inspect
```bash
# Load everything about an issue in one call
gr inspect gr-abc123

# Inspect a file's topology context
gr inspect src/store.rs

# Inspect a specific symbol
gr inspect src/store.rs::SqliteStore
```

**Output includes:**
- Issue metadata (if ID)
- Solid Volume (affected symbols)
- Star Neighborhood (connected code)
- Related issues (BM25 search)

### Search and Triage
```bash
# Natural language search
gr analysis search "circular dependency"

# Check for duplicates before creating
gr analysis duplicates
```

---

## 🏗️ Phase 3: Executing Work

### Start Working on an Issue
```bash
# All-in-one: creates branch, sets status, outputs context
gr workon gr-abc123

# With custom branch name
gr workon gr-abc123 --branch feature/my-fix
```

### Quick Updates with Set
```bash
# Fuzzy shorthand updates - way faster than gr update
gr set gr-abc pri:1 stat:ip +l:urgent

# Key shortcuts:
# stat: → status    (ip = in-progress, o = open, c = closed, b = blocked)
# pri: → priority   (1-5)
# a: → assignee
# +l: → add label   -l: → remove label
```

### Full Update Command
```bash
gr update gr-abc123 --status in-progress --priority 1
gr update gr-abc123 --add-symbol "src/store.rs::create_issue"
gr update gr-abc123 --add-dependency gr-xyz789
```

### Create Issues
```bash
gr create "Fix login bug" -t bug -p 1
gr create "Refactor store" -t task -p 2 -d "Detailed description here"
```

---

## 🔬 Phase 4: Topology Analysis

### Solid Score and Health Check
```bash
# Full topology rebuild
gr analysis rebuild

# Get solid score dashboard
gr stats --topology
```

### Detect and Fix Cycles
```bash
# Show cycles with suggested fix
gr refactor

# Apply the fix (comments out weakest edge)
gr refactor --apply --cycle 0

# Preview without modifying
gr refactor --apply --dry-run

# Undo if needed
gr refactor --undo --target src/store.rs
```

### Star Neighborhoods (Context Loading)
```bash
# Get all connected code for a symbol
gr analysis star "src/utils.rs" --depth 2

# Find feature volumes (tightly coupled clusters)
gr analysis volumes "src/engine.rs"
```

### Architectural Invariants
```bash
# Check against layers.yaml
gr analysis check-layers --all

# Validate a specific file
gr analysis validate-topology "src/main.rs"
```

---

## 🛡️ Phase 5: Verification

### Before Completing Work
```bash
# Rebuild topology and check for new cycles
gr analysis rebuild
gr analysis validate-topology src/changed_file.rs

# Ensure no new violations
gr analysis check-layers
```

### Block Bad Syncs
```bash
# Only sync if topology is clean
gr sync --validate-topology
```

---

## 📂 Phase 6: Session Handoff

### Sync Everything
```bash
# Export to JSONL + commit + push
gr sync
```

### Bulk Operations
```bash
gr workflow triage gr-id1 gr-id2 --status in_progress --assignee "next_agent"
```

### Create Handoff Issue
```bash
gr create "CONTINUITY: [Task] Handoff" \
  -d "State: Completed step 2. Next: Fix step 3." \
  -t task
```

---

## 📺 VS Code Extension

The extension now opens as a **full dashboard panel** with:

| View | Description |
|------|-------------|
| **3D Topology** | React Three Fiber visualization with orbit controls |
| **Vitals Dashboard** | Solid Score gauge, Spaghetti Meter, hotspots |
| **List/Kanban/Graph** | Standard issue management |
| **Gutter Decorations** | Issue indicators in editor |

### Populating Extension Views

| View | How to Populate |
|------|-----------------|
| **3D Topology** | `gr analysis rebuild` → nodes sized by PageRank |
| **Vitals** | Solid Score computed from topology |
| **Focus View** | `gr update <ID> --add-symbol <SYM>` |
| **Gutter Icons** | Linked symbols appear in editor margin |

---

## 🧮 Command Reference (v2.0)

### Agent-Native Commands (NEW)
| Command | Example | Purpose |
|---------|---------|---------|
| `gr pulse` | `gr pulse --assignee me` | Session hydration |
| `gr inspect` | `gr inspect gr-abc` | One-shot context |
| `gr workon` | `gr workon gr-abc` | Start work (branch + status) |
| `gr set` | `gr set abc pri:1 stat:ip` | Fuzzy updates |
| `gr refactor` | `gr refactor --apply` | Auto-fix cycles |

### Core Commands
| Command | Arguments | Use Case |
|---------|-----------|----------|
| `gr create` | `<TITLE>` `-d` `-t` `-p` | Create issue |
| `gr update` | `<ID>` `--status` `--add-symbol` | Update issue |
| `gr ready` | `--assignee` | Find actionable work |
| `gr sync` | `--validate-topology` | Save and push |

### Analysis Commands
| Command | Purpose |
|---------|---------|
| `gr analysis rebuild` | Build topology cache |
| `gr analysis star` | Get connected context |
| `gr analysis volumes` | Find code clusters |
| `gr analysis check-layers` | Verify architecture |
| `gr analysis search` | BM25 natural language search |
