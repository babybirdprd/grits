# Grits

A Git-native, local-first issue tracker with a **Twin Engine** architecture:        
- 🤖 **Agent Engine**: MCP server for AI integration (Antigravity, Claude, etc.)
- 👀 **Visual Engine**: VS Code extension with WASM-powered UI

**Status**: Production Ready (v0.1.0)

## Quick Start

### Prerequisites
- Rust (latest stable)
- Git
- Node.js 18+ (for VS Code extension)

### Option 1: Download Binary (Recommended)

Download the latest release from [GitHub Releases](https://github.com/your-org/grits/releases):
- **Windows**: `gr-x86_64-pc-windows-msvc.exe`
- **macOS (Intel)**: `gr-x86_64-apple-darwin`
- **macOS (Apple Silicon)**: `gr-aarch64-apple-darwin`
- **Linux**: `gr-x86_64-unknown-linux-gnu`

Rename to `gr` (or `gr.exe`) and add to your PATH.

### Option 2: Build from Source

```bash
# Clone and enter directory
git clone https://github.com/your-org/grits.git
cd grits

# Install all dependencies (Rust + Node)
pnpm run install:all

# Build entire ecosystem (CLI, WASM, Extension)
pnpm run build

# Install CLI globally
cargo install --path grits-cli
```

> **Updating**: `cargo install --path grits-cli --force`

### Initialize a Project
```bash
cd your-project
gr onboard
```

### CLI Usage
```bash
gr create "Fix login bug" -t bug -p 1         # Create issue
gr list --status open                          # List open issues
gr update abc123 --status in-progress          # Update status
gr close abc123                                # Close issue
gr sync                                        # Sync with Git
```

## Twin Engine Architecture

### 🤖 Agent Engine (MCP Server)

Run the MCP server for AI agent integration:
```bash
gr serve-mcp
```

**Available Tools:**
| Category | Tools |
|----------|-------|
| **CRUD** | `list_issues`, `create_issue`, `update_issue`, `close_issue`, `get_issue` |
| **Contextual** | `find_related_issues`, `suggest_issue_for_error`, `infer_issue_from_diff` |
| **Bulk** | `bulk_triage`, `detect_duplicates`, `cleanup_stale` |
| **Workflow** | `get_next_task`, `link_commit_to_issues`, `generate_issue_from_todo` |
| **Smart Queries** | `search_issues`, `get_issue_graph`, `summarize_sprint` |

**Antigravity Configuration** (`.vscode/mcp.json`):
```json
{
    "servers": {
        "grits": {
            "command": "gr",
            "args": ["serve-mcp"]
        }
    }
}
```

### 👀 Visual Engine (VS Code Extension)

The extension provides a rich UI for `.jsonl` issue files:

- **List View**: Virtualized spreadsheet with inline editing
- **Kanban View**: Drag-and-drop board by status
- **Graph View**: Dependency visualization
- **Agenda View**: Focus mode for high-priority items

**Install Extension:**

Download `grits-kanban-*.vsix` from [Releases](https://github.com/your-org/grits/releases) and:
```bash
code --install-extension grits-kanban-0.1.0.vsix
```

Or build from source:
```bash
# From the root directory
pnpm run build

# Then press F5 in VS Code in this workspace
```

## Project Structure

```
grits/
├── grits-core/         # Core library (WASM-compatible)
│   └── src/wasm.rs     # WASM bridge for UI
├── grits-cli/          # CLI + MCP server
│   └── src/mcp.rs      # MCP tool implementations
├── extension/          # VS Code extension
│   ├── src/            # Extension host code
│   └── webview/        # React UI
├── .vscode/mcp.json    # MCP server config
└── .agent/workflows/   # Agent workflow rules
```

## Documentation

- [Architecture](docs/architecture.md): Twin Engine design
- [CLI Usage](docs/cli_usage.md): Full command reference
- [WASM Status](docs/wasm_compatibility.md): WebAssembly support
- [Development](docs/development.md): Build & test guide

## License
MIT
