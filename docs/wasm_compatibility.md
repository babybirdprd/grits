# WASM Compatibility

The Rust port is designed with future WebAssembly (WASM) support in mind. This will allow Grits to run in browser-based environments (like VS Code extensions or web UIs) without requiring a native binary.

## Current Status

*   **FileSystem Abstraction**: The `FileSystem` trait in `grits-core` abstracts all file I/O operations.
    *   **Native**: `StdFileSystem` uses `std::fs`.
    *   **WASM**: `WasmFileSystem` delegates to JavaScript functions via `wasm-bindgen`.
*   **GitOps Abstraction**: The `GitOps` trait abstracts git commands.
    *   **Native**: `StdGit` uses `std::process::Command` to call the `git` binary.
    *   **WASM**: `WasmGit` delegates to JavaScript functions via `wasm-bindgen`.
*   **Store Abstraction**: The `Store` trait in `grits-core` abstracts the persistence layer.
    *   **Native**: `SqliteStore` uses `rusqlite` (SQLite).
    *   **WASM/All**: `MemoryStore` provides an in-memory implementation suitable for ephemeral sessions or testing.
*   **Core Logic**: The core logic is pure Rust and does not depend on OS-specific features.
*   **Compilation**: `grits-core` compiles for `wasm32-unknown-unknown`. Dependencies like `rusqlite` are gated behind `#[cfg(not(target_arch = "wasm32"))]`.

## JavaScript Bindings

The `grits-core` library exposes the following JS bindings when compiled for WASM:

*   **Modules**:
    *   `/js/grits_fs.js`: Exports filesystem functions (`fs_read_to_string`, `fs_write`, etc.). Reference mock implementations are provided in the source tree.
    *   `/js/grits_git.js`: Exports git functions (`git_init`, `git_commit`, etc.). Reference mock implementations are provided in the source tree.
*   **Classes**:
    *   `WasmFileSystem`: Rust wrapper around the JS filesystem module.
    *   `WasmGit`: Rust wrapper around the JS git module.

## Usage in WASM

To use Grits in a WASM environment:
1.  Compile `grits-core` with `wasm-pack`.
2.  Provide implementations for the functions in `grits_fs.js` and `grits_git.js` in the host environment.
3.  Instantiate `MemoryStore` or implement a custom `Store` (e.g., on top of IndexedDB).

## Testing

A basic browser-based test harness is available in `rust/wasm-test/`.
To run it:
1.  Install `wasm-pack` (`cargo install wasm-pack`).
2.  Build the WASM package: `cd rust/grits-core && wasm-pack build --target web --out-dir ../wasm-test/pkg`.
3.  Serve `rust/wasm-test/` (e.g., `python3 -m http.server`).
4.  Open browser to verify console output.
