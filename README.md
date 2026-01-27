# Grits

A Git-native, local-first issue tracker designed for **AI Agents** and humans.

**Status**: v3.0.0 — "Active State Store"

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

## 🤖 The State Store Protocol (Planner/Coder)

Grits transforms from a passive tracker into an **Active State Store (World Model)** for agents. It enforces a strict separation between research and execution to minimize LLM "halucinations" and context drift.

### The Lifecycle of an Issue
1. **User Intent** (`description`): The high-level goal set by a human or orchestrator.
2. **Planner Agent**: Analyzes the intent, researches the code via `gr star`, and populates the **Implementation Plan** (`design`).
3. **Coder Agent**: Wakes up to a **Rich Pulse** JSON. If `design` is present, it begins coding. It logs every iteration/rollout to the **Execution Log** (`notes`) via `gr update --append`.

### Key Commands for Agents
- `gr pulse`: Returns a full **Rich Context** JSON (Intent + Plan + Rules + History).
- `gr update --append`: Persistent append-only logging for the Lab Notebook (`notes`).
- `gr workon`: Automatically warns if a Coder starts work without a `design`.

## Core Workflow

### 1. Check Pulse
Hydrate your session (or your agent's context) with the current project status.
```bash
gr pulse
```

### 2. Create Issue
```bash
gr create "Fix login bug" --description "intent" --design "plan" --start-work
```

### 3. Hierarchy & Linking
Link existing tasks or migrate them into a hierarchy.
```bash
gr dep add gr-child123 gr-parentabc --migrate
# ✅ Result: Child is renamed to gr-parentabc.n
```

### 4. Context Awareness (Blind Assembly)
Bundle relevant code even if you don't have a topology map yet.
```bash
gr context assemble --symbols "README.md,src/main.rs"
```

## CLI Reference (v3.2.0)

| Field | Purpose | Agent Map |
|-------|---------|-----------|
| `description` | The User's Intent | **Intent** |
| `design` | Implementation Plan | **Strategy** |
| `acceptance_criteria` | Success Definitions | **Proof** |
| `notes` | Execution Log (Append-only) | **Memory** |

| Command | Purpose |
|---------|---------|
| `gr show <id>` | Displays Rich Context + Execution Log |
| `gr update --append` | Marks notes as append-only (Lab Notebook) |
| `gr pulse` | Rich Context JSON for Session Hydration |
| `gr context assemble` | Fallback "Blind" assembly for raw file paths |

---

- **Planner Skill**: [.agent/skills/grits-plan/SKILL.md](.agent/skills/grits-plan/SKILL.md) — Self-contained Architect instructions.
- **Coder Skill**: [.agent/skills/grits-code/SKILL.md](.agent/skills/grits-code/SKILL.md) — Self-contained Builder instructions.
- **Workflows**: [.agent/workflows/](.agent/workflows/) — Automated turbo-execution paths.
- **Standard**: Follows the [Agent Skills](https://agentskills.io) open specification.

---

## Architecture

**Twin Engine**:
- **SQLite** (`.grits/grits.db`) — Fast local queries.
- **JSONL** (`.grits/issues.jsonl`) — Git-versioned source of truth.

**Graph-Lite**:
- Uses a lightweight AST parser to build a dependency graph.
- Supports **Blind Assembly** fallback for non-topological file access.

## License

MIT