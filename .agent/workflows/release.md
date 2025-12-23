---
description: How to release a new version of Grits
---

# Grits Release Workflow

Standardized process for releasing new versions of Grits to crates.io and GitHub.

## Pre-Release Checklist

// turbo-all

1. Ensure all tests pass:
```bash
cargo test --workspace
```

2. Verify WASM builds:
```bash
cargo build -p grits-core --target wasm32-unknown-unknown
```

3. Check for uncommitted changes:
```bash
git status
```

## Version Bump

Determine version based on change type:
- **Patch** (1.0.x): Bug fixes, docs
- **Minor** (1.x.0): New features, backward compatible
- **Major** (x.0.0): Breaking changes

Update these files with new version:
1. `grits-core/Cargo.toml` - `version = "X.Y.Z"`
2. `grits-cli/Cargo.toml` - `version = "X.Y.Z"` AND `grits-core = { version = "X.Y.Z"`
3. `README.md` - Status line version

## Release Commands

1. Commit version bump:
```bash
git add -A
git commit -m "chore: release vX.Y.Z - [description]"
```

2. Publish to crates.io (core first, then CLI):
```bash
cargo publish -p grits-core
# Wait ~60 seconds for indexing
cargo publish -p grits-cli
```

3. Tag and push:
```bash
git tag vX.Y.Z
git push origin main --tags
```

This triggers GitHub Actions which:
- Builds CLI for Linux, macOS (x86+arm), Windows
- Packages VS Code extension (.vsix)
- Creates GitHub Release with artifacts

## Post-Release Verification

1. Verify crates.io install:
```bash
cargo install grits-cli --force
gr --version
```

2. Check GitHub Release page:
   - https://github.com/babybirdprd/grits/releases
   - Should have 5 artifacts: 4 binaries + 1 VSIX

3. Test installed CLI:
```bash
gr --help
```
