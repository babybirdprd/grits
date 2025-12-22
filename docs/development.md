# Development

## Prerequisites

*   **Rust Toolchain**: Install via [rustup](https://rustup.rs/).
*   **Git**: Required for syncing and version control integration.

## Building

Build the project from the `rust/` directory:

```bash
cd rust
cargo build
```

This will build both `beads-core` and `beads-cli`.

## Running

Run the CLI directly via `cargo`:

```bash
cd rust
cargo run -p beads-cli -- <args>
```

Example:
```bash
cargo run -p beads-cli -- list
```

## Testing

Run unit and integration tests:

```bash
cd rust
cargo test
```

## Compatibility Verification

To ensure the Rust port remains compatible with the Go implementation, use the verification script. This script requires both `bd-go` and `bd-rust` binaries to be present in the root directory.

```bash
# From the repo root
./scripts/verify_compat.sh
```

This script performs a sequence of operations (init, create, update, sync) using both implementations and asserts that they interoperate correctly.
