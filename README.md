# Grits

A Git-native, local-first issue tracker designed for **AI Agents** and humans.

**Status**: v2.9.0 — "Agent Automation"

## Quick Start

### Prerequisites
- Rust (latest stable)
- Git

### Install CLI

```bash
# Via Cargo (recommended)
cargo install grits-cli

# Or download binary from Releases
# https://github.com/babybirdprd/grits/releases
```

### Initialize a Project
```bash
cd your-project
gr onboard
```

## Core Workflow

### 1. Check Pulse
Hydrate your session (or your agent's context) with the current project status.
```bash
gr pulse
```

### 2. Create Issue
```bash
gr create "Fix login bug" --type bug --priority 1

# Create a sub-task (Hierarchical ID)
gr create "Validate email" --parent gr-abc123
# ✅ Result: gr-abc123.1
```

### 3. Hierarchy & Linking
Link existing tasks or migrate them into a hierarchy.
```bash
gr dep add gr-child123 gr-parentabc --migrate
# ✅ Result: Child is renamed to gr-parentabc.n
```

### 3. Work on Issue
Sets the "sticky focus" and updates status to `in-progress`.
```bash
gr workon <issue-id>
```

### 4. Context Awareness (Star Neighborhood)
Find files topologically connected to your target.
```bash
gr star src/auth.rs
```

### 5. Assemble Context
Bundle relevant code into a Markdown format for LLM analysis.
```bash
gr context assemble --issue <issue-id>
```

## CLI Reference

| Command | Purpose |
|---------|---------|
| `gr list` | List issues (filter by status, assignee, etc.) |
| `gr show <id>` | Show issue details |
| `gr update` | Update issue fields |
| `gr close <id>` | Close an issue |
| `gr dep add <c> <p>` | Link tasks (parent, blocks, related) |
| `gr pulse` | Session hydration (Focus + Blockers + Recent Commits) |
| `gr workon <id>` | Set focus and status |
| `gr star <file>` | Get connected files (imports/calls) |
| `gr context assemble` | Generate mini-codebase from focus or symbols |
| `gr export` | Export to JSONL (for git sync) |
| `gr import` | Import from JSONL |
| `gr stats` | Simple issue statistics |

---

## Agent Skills

Grits exposes its functionality as a standardized "Agent Skill" for AI agents. This allows agents to natively discover and utilize Grits for task management and context retrieval.

- **Skill Definition**: [.agent/skills/grits/SKILL.md](.agent/skills/grits/SKILL.md)
- **Standard**: Follows the [Agent Skills](https://agentskills.io) open specification.

To use this skill with a compatible agent (e.g., in a browser environment or agent runner), point the agent to the `.agent/skills/grits` directory.

---

## Documentation

- [AGENTS.md](AGENTS.md): Comprehensive agent guide
- [TASK.md](TASK.md): Current strategic plan

## Architecture

**Twin Engine**:
- **SQLite** (`.grits/grits.db`) — Fast local queries.
- **JSONL** (`.grits/issues.jsonl`) — Git-versioned source of truth.

**Graph-Lite**:
- Uses a lightweight AST parser to build a dependency graph.
- Provides "Star Neighborhoods" (connected files) to agents without heavy analysis.

## License

MIT