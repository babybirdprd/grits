# Beads (Rust Port)

This is the Rust implementation of [Beads](https://github.com/steveyegge/beads), a distributed, git-backed issue tracker.

**Status**: Proof of Concept (Functionally Complete)

## Goals

*   **Portability**: Support compilation to WebAssembly (WASM) to run in browsers and editors.
*   **Simplicity**: Remove the background daemon requirement; use direct SQLite access.
*   **Performance**: Leverage Rust's performance and safety.

## Quick Start

### Prerequisites
*   Rust (latest stable)
*   Git

### Build
```bash
cargo build --release
```

### Run
```bash
# Run via cargo
cargo run -p beads-cli -- list

# Or run the binary directly after build
./target/release/bd list
```

## Documentation

*   [Architecture](docs/architecture.md): Overview of the workspace, library design, and WASM strategy.
*   [CLI Usage](docs/cli_usage.md): Guide to the `bd` commands (`create`, `list`, `sync`, etc.).
*   [Development](docs/development.md): How to build, test, and verify compatibility.
*   [Compatibility](docs/compatibility.md): Details on interoperability with the Go implementation.
*   [WASM Status](docs/wasm_compatibility.md): Current status and roadmap for WebAssembly support.

## Project Structure

*   `beads-core/`: The core library containing models, storage, and sync logic.
*   `beads-cli/`: The command-line interface application.
*   `docs/`: Detailed documentation.

## License
Same as the main Beads repository.
