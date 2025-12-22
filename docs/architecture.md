# Architecture

The Rust port of Beads follows a library-first architecture designed for portability (including WASM) and simplicity.

## Workspace Structure

The `rust/` directory is a Cargo workspace containing:

*   **`beads-core`**: The library crate. It contains the domain models, storage logic, sync algorithms, and abstractions for file system and git operations.
*   **`beads-cli`**: The binary crate. It handles command-line argument parsing (via `clap`), output formatting, and invokes `beads-core` logic.

## Key Decisions

### No Daemon
Unlike the original Go implementation, the Rust port does not use a background daemon or RPC. It accesses the SQLite database directly. Concurrency safety is managed via SQLite's file locking mechanism. This reduces complexity and system resource usage.

### WASM Compatibility
A major goal of the Rust port is to support compilation to WebAssembly (WASM). To achieve this:

*   **FileSystem Abstraction**: All file I/O in `beads-core` is performed through the `FileSystem` trait. This allows injecting a virtual file system in WASM environments where direct disk access is not available.
*   **GitOps Abstraction**: Git operations are abstracted behind the `GitOps` trait. The CLI uses `StdGit` (wrapping `std::process::Command`), but a WASM target can implement this trait using a pure-Rust git library or JS interop.

## Database & Sync

*   **Schema**: The Rust port shares the same SQLite schema as the Go version. It uses `rusqlite` for interaction.
*   **Data Compatibility**: It reads and writes to `.beads/beads.db`.
*   **Sync**: Synchronization logic (exporting to JSONL, git commit, pull, push, importing from JSONL) is implemented in `beads-core`. The `dirty_issues` table is used to track changes that need to be exported.
