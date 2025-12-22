# WASM Compatibility

> **Status: ✅ Verified Working** — `grits-core` compiles successfully to `wasm32-unknown-unknown`.

The `grits-core` library is designed for WebAssembly (WASM) compatibility, enabling it to run in browser-based environments (VS Code extensions, web UIs) without a native binary.

## Current Status

| Abstraction | Native | WASM |
|-------------|--------|------|
| **FileSystem** | `StdFileSystem` (std::fs) | `WasmFileSystem` (wasm-bindgen → JS) |
| **GitOps** | `StdGit` (std::process::Command) | `WasmGit` (wasm-bindgen → JS) |
| **Store** | `SqliteStore` (rusqlite) | `MemoryStore` (in-memory) |

*   **Core Logic**: Pure Rust, no OS-specific dependencies.
*   **Compilation**: Platform-specific deps are gated with `#[cfg(not(target_arch = "wasm32"))]`.

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

A basic browser-based test harness is available in `wasm-test/`.

```bash
# Install wasm-pack
cargo install wasm-pack

# Build WASM package
cd grits-core && wasm-pack build --target web --out-dir ../wasm-test/pkg

# Serve and test
cd ../wasm-test && python -m http.server
# Open http://localhost:8000 in browser
```

