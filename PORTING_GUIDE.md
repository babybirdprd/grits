# Rust Porting Guide

## Current Status (PoC)
We have established the foundational structure for the Rust port of `grits`.
- **Workspace**: `rust/` contains the cargo workspace.
- **grits-core**: Library crate containing domain models (`Issue`), storage logic (`Store` wrapper around `rusqlite`), git integration, and sync logic.
- **grits-cli**: Binary crate using `clap` for CLI parsing. Implements `list`, `create`, and `sync` commands.
- **Interoperability**: The Rust CLI reads and writes to the same SQLite database (`.grits/grits.db`) as the Go implementation. It writes to the `dirty_issues` table to ensure the Go `bd export` command picks up changes.

---

## Progress Assessment
**Overall Completion: ~80%**

| Component | Status | Notes |
| :--- | :--- | :--- |
| **Core Models** | 🟢 Complete | `Issue` struct updated with `Dependency`, `Comment` types. `relates_to` is `Vec<String>`. |
| **Storage** | 🟢 Complete | Read/write works. `export_to_jsonl` implemented. |
| **ID Generation** | 🟢 Complete | Ported Base36 logic and hash generation (prefix, length, nonce) from Go. |
| **CLI** | 🟢 Complete | `create`, `list`, `sync`, `config`, `stats` implemented. Added support for labels and dependencies in `update` and `edit`. |
| **Git Integration** | 🟢 Complete | `GitOps` trait implemented in `grits-core`. `StdGit` provides `std::process::Command` implementation. |
| **Merge Logic** | 🟢 Complete | 3-way merge algorithm ported including tombstone handling. |
| **Sync Logic** | 🟢 Complete | `bd sync` command implemented with conflict resolution. |
| **Compatibility** | 🟢 Verified | Cross-language test suite `scripts/verify_compat.sh` passes. |
| **UX/Error Handling** | 🟢 Improved | Added `anyhow::Context` and cleaned up CLI output. |
| **WASM Prep** | 🟢 Complete | `FileSystem` trait abstracts IO in `grits-core`. |
| **WASM Support** | 🟢 Complete | `grits-core` compiles for `wasm32-unknown-unknown`. `MemoryStore` implemented. |

---

## Next Steps for the Next Agent

Your goal is to expand feature parity.

### 1. Feature Parity
* **Task**: Enhance `bd sync` to support more flags present in Go version (e.g., `--squash`, `--dry-run`).

### 3. CI Integration
* **Task**: Add `scripts/verify_compat.sh` to the repository's CI pipeline (e.g., GitHub Actions) to ensure ongoing compatibility.

---

## Architecture Notes
* **No Daemon**: We are intentionally dropping the Daemon/RPC architecture. Use SQLite file locking for concurrency safety.
* **WASM Goal**: Keep `grits-core` pure Rust where possible. Abstract IO and Git operations to allow future WASM compilation.
* **IO Abstraction**: `grits-core` uses the `FileSystem` trait for all file operations. Concrete implementations (like `StdFileSystem`) are injected at runtime.

## Helpful Commands
* **Build**: `cd rust && cargo build`
* **Run**: `cd rust && cargo run -p grits-cli -- <args>`
* **Test**: `cd rust && cargo test`
