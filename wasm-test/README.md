# WASM Test Harness

This directory contains a simple test harness for the WASM build of `beads-core`.

## Prerequisites

- `wasm-pack` (install with `cargo install wasm-pack`)
- A simple HTTP server (e.g., `python3 -m http.server`)

## Running the Test

1. Build the WASM package:
   ```bash
   cd ../beads-core
   wasm-pack build --target web --out-dir ../wasm-test/pkg
   ```

2. Serve the directory:
   ```bash
   cd ../wasm-test
   python3 -m http.server
   ```

3. Open your browser at `http://localhost:8000` and check the console.

## Mock Implementation

The JS bindings are currently mock implementations defined in:
- `rust/beads-core/js/beads_fs.js`
- `rust/beads-core/js/beads_git.js`

These satisfy the linker requirements but do not persist data or run real git commands.
