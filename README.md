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

### Build & Install
```bash
cargo build --release
# Add to PATH or use alias
alias gr="./target/release/gr"
```

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
| Tool | Description |
|------|-------------|
| `list_issues` | List issues with filters (status, priority, assignee) |
| `create_issue` | Create new issue |
| `update_issue` | Update issue fields |
| `close_issue` | Close issue by ID |
| `get_issue` | Get full issue details |

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
```bash
cd extension
npm install
npm run compile
# Then F5 to launch Extension Development Host
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
