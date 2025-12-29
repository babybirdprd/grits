# Grits

A Git-native, local-first issue tracker with a **Twin Engine** architecture:        
- 🤖 **Agent Engine**: MCP server + agent-native CLI (inspect, workon, pulse, refactor)
- 👀 **Visual Engine**: VS Code extension with 3D topology visualization

**Status**: v2.0.0 — "Solid Graph Command Center"

## Quick Start

### Prerequisites
- Rust (latest stable)
- Git
- Node.js 18+ (for VS Code extension)

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

## v2.0 Highlights

### 🚀 Agent-Native Commands

```bash
# Session hydration - get project context instantly
gr pulse

# One-shot context for any target
gr inspect gr-abc123   # Issue
gr inspect src/main.rs # File

# Start working (creates branch + sets status + outputs context)
gr workon gr-abc123

# Fuzzy shorthand updates  
gr set gr-abc pri:1 stat:ip +l:urgent

# Auto-fix dependency cycles
gr refactor --apply --cycle 0
```

### 🔬 Solid Graph Topology

```bash
# Build topology cache (Tree-sitter: Rust, TS, JS, Python, Go)
gr analysis rebuild

# Compute Solid Score (architectural health metric)
gr stats --topology

# Detect and auto-fix cycles
gr refactor                    # Show cycles + suggestions
gr refactor --apply --cycle 0  # Apply fix
gr refactor --undo --target f  # Restore from backup
```

| Feature | Description |
|---------|-------------|
| **Betti Numbers** | B₀=components, B₁=cycles, B₂=voids |
| **Solid Score** | Unified health metric (0-100%) |
| **Edge Persistence** | Identify weakest links in cycles |
| **Star Neighborhoods** | Context loading for AI editing |
| **Monorepo Support** | Cargo, pnpm, turbo, go.work detection |

### 📺 VS Code Dashboard

The extension opens as a **full Webview Panel** with:

- **3D Topology View**: React Three Fiber visualization with orbit controls
- **Vitals Dashboard**: Solid Score gauge, Spaghetti Meter, PageRank hotspots
- **List/Kanban/Graph**: Standard issue management
- **Gutter Decorations**: Issue indicators in editor margins

---

## Twin Engine Architecture

### 🔗 Tethered Sync
- **SQLite** (`.grits/grits.db`) — Fast local queries
- **JSONL** (`.grits/issues.jsonl`) — Git-versioned, human-readable
- All commands auto-sync between engines

### 🤖 Agent Engine (MCP Server)

```bash
gr serve-mcp
```

**Antigravity Config** (`.vscode/mcp.json`):
```json
{
    "servers": {
        "grits": {
            "command": "gr",
            "args": ["serve-mcp"],
            "env": { "GRITS_PROJECT_ROOT": "${workspaceFolder}" }
        }
    }
}
```

**16 MCP Tools:**
| Category | Tools |
|----------|-------|
| CRUD | `list_issues`, `create_issue`, `update_issue`, `close_issue`, `get_issue` |
| Search | `search_issues`, `find_related_issues`, `detect_duplicates` |
| Strategic | `get_next_task`, `summarize_sprint`, `cleanup_stale` |
| Context | `suggest_issue_for_error`, `infer_issue_from_diff` |

---

## CLI Reference (v2.2.3)

### Agent-Native (NEW)
| Command | Purpose |
|---------|---------|
| `gr pulse` | Session hydration |
| `gr inspect <target>` | One-shot context |
| `gr workon <id>` | Start work (branch + status) |
| `gr set <id> <changes>` | Fuzzy updates |
| `gr refactor` | Auto-fix cycles |

### Core
| Command | Purpose |
|---------|---------|
| `gr create` | Create issue |
| `gr update` | Update issue |
| `gr list` | List issues |
| `gr ready` | Actionable issues |
| `gr sync` | Git sync |

### Analysis
| Command | Purpose |
|---------|---------|
| `gr analysis rebuild` | Build topology cache |
| `gr analysis star <sym>` | Star neighborhood |
| `gr analysis volumes <file>` | Feature clusters |
| `gr analysis check-layers` | Architectural invariants |
| `gr analysis search <query>` | BM25 search |

---

## Project Structure

```
grits/
├── grits-core/           # Core library (WASM-compatible)
│   └── src/topology/     # Solid Graph analysis
│       ├── analysis.rs   # Betti numbers, PageRank, persistence
│       ├── refactor.rs   # Auto-fix cycle mutations
│       └── workspace.rs  # Monorepo detection
├── grits-cli/            # CLI + MCP server
├── extension/            # VS Code extension
│   └── webview/          # React + Three.js dashboard
└── .agent/workflows/     # Agent workflow rules
```

---

## Documentation

- [AGENTS.md](AGENTS.md): Comprehensive agent guide
- [docs/cli_usage.md](docs/cli_usage.md): Full command reference
- [docs/architecture.md](docs/architecture.md): Twin Engine design

## Credits

Inspired by [Beads](https://github.com/steveyegge/beads) by Steve Yegge — 
"A memory upgrade for your coding agent."

Based on the ["Solid Graph" philosophy](https://arxiv.org/html/2512.19736v1) from algebraic topology.

## License

MIT
