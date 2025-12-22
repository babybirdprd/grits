# Architecture

Grits implements a **Twin Engine** architecture: a headless core that powers both AI agents and human developers through specialized interfaces.

## Twin Engine Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        grits-core                            │
│              (Rust library, WASM-compatible)                 │
│    ┌─────────┐  ┌─────────┐  ┌──────────┐  ┌──────────┐    │
│    │ Models  │  │  Store  │  │   Sync   │  │  WASM    │    │
│    │         │  │ (SQLite)│  │ (Git)    │  │  Bridge  │    │
│    └─────────┘  └─────────┘  └──────────┘  └──────────┘    │
└────────────────────────┬────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          │                             │
          ▼                             ▼
┌─────────────────────┐      ┌─────────────────────┐
│   Engine A (Native) │      │   Engine B (WASM)   │
│   ┌───────────────┐ │      │   ┌───────────────┐ │
│   │  MCP Server   │ │      │   │ VS Code Ext   │ │
│   │  (gr serve)   │ │      │   │  (React UI)   │ │
│   └───────────────┘ │      │   └───────────────┘ │
│         ↓           │      │         ↓           │
│   AI Agents         │      │   Human Developers  │
│   (Antigravity)     │      │   (Visual UI)       │
└─────────────────────┘      └─────────────────────┘
```

## Engine A: MCP Server (Agent Interface)

The native CLI includes an MCP (Model Context Protocol) server that exposes issue management as structured tools.

**Location:** `grits-cli/src/mcp.rs`

**Tools:**
- `list_issues` - Query with filters
- `create_issue` - Create new issues
- `update_issue` - Modify existing issues
- `close_issue` - Mark issues closed
- `get_issue` - Get full details

**Protocol:** JSON-RPC 2.0 over stdio (MCP 2024-11-05 spec)

## Engine B: WASM + VS Code Extension (Human Interface)

The core library compiles to WebAssembly, enabling zero-latency UI in VS Code.

**WASM Bridge:** `grits-core/src/wasm.rs`
- `GritsWasm.parse_issues()` - Parse JSONL → JSON array
- `GritsWasm.update_field()` - Generic field updates with validation
- `GritsWasm.serialize_issues()` - JSON → JSONL
- `GritsWasm.create_issue()` - Create new issue

**VS Code Extension:** `extension/`
- Custom editor for `.jsonl` files
- File ↔ Webview synchronization
- Four specialized views (List, Kanban, Graph, Agenda)

## Workspace Structure

```
grits/
├── grits-core/           # Library crate (WASM-compatible)
│   ├── src/
│   │   ├── models.rs     # Issue, Comment, Dependency types
│   │   ├── store.rs      # SQLite storage (native only)
│   │   ├── sync.rs       # Git synchronization
│   │   ├── wasm.rs       # WASM bridge (GritsWasm)
│   │   └── fs.rs         # FileSystem abstraction
│   └── Cargo.toml
├── grits-cli/            # Binary crate
│   ├── src/
│   │   ├── main.rs       # CLI commands
│   │   └── mcp.rs        # MCP server implementation
│   └── Cargo.toml
└── extension/            # VS Code extension
    ├── src/
    │   └── extension.ts  # Custom editor provider
    └── webview/          # React frontend
        └── src/
            ├── App.tsx
            └── components/
```

## Key Design Decisions

### 1. Headless Core
All business logic lives in `grits-core`, with no I/O assumptions. This enables:
- Native CLI with SQLite + Git
- WASM builds for browser/editor environments
- Testability without mocking

### 2. Single Source of Truth
Both engines read/write the same `.grits/issues.jsonl` file:
- Changes sync automatically via file system watching
- No race conditions (file-level locking)
- Git-native versioning and conflict resolution

### 3. Type-Safe Tool Interface
MCP tools use strongly-typed parameters (`schemars` JSON Schema):
- AI agents get accurate parameter descriptions
- Validation happens at the boundary
- Rust's type system enforced throughout

### 4. Graceful Degradation
- No WASM? CLI works standalone
- No extension? Files are plain JSONL (editable in any editor)
- No MCP client? Use CLI commands directly

## Database & Sync

- **Schema**: SQLite database at `.grits/grits.db`
- **Export**: JSONL at `.grits/issues.jsonl` (synced to Git)
- **Sync**: `gr sync` exports → commits → pushes → imports changes

## Compatibility

The Rust implementation maintains compatibility with the original Go Beads implementation:
- Same SQLite schema
- Same JSONL export format
- Same Git conventions
