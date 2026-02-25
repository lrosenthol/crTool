# Development Setup and Workflow

This document covers the development workflow and tooling for crTool (workspace: library, CLI, GUI).

## Initial setup

After cloning (see [SETUP.md](SETUP.md)):

```bash
cd crTool
./scripts/install-hooks.sh
```

This installs pre-commit hooks that run `cargo fmt` and `cargo clippy` before each commit.

## Editor setup (VS Code / Cursor)

The repo includes `.vscode/settings.json` with:

- Format on save for Rust
- Optional Clippy on save
- Recommended extension: **rust-analyzer**

Install rust-analyzer via the extensions panel if not prompted.

## Development workflow

### Formatting

```bash
cargo fmt
# or
./scripts/format.sh
```

Check without writing:

```bash
cargo fmt -- --check
```

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

(Omit `--all-features` if the workspace has optional features that don’t build in your environment.)

### Building and running

```bash
# Debug
cargo build
cargo run -p crTool -- --help
cargo run -p crTool-gui

# Release
cargo build --release
cargo run --release -p crTool -- ...
cargo run --release -p crTool-gui
```

### Tests

```bash
cargo test
# More stable for integration tests:
cargo test -- --test-threads=1
```

See [TESTING.md](TESTING.md) and [tests/README.md](tests/README.md).

### Pre-commit hooks

Hooks live in `.git-hooks/`. To reinstall:

```bash
./scripts/install-hooks.sh
```

To skip (not recommended): `git commit --no-verify`.

## Code quality

Code is expected to:

- Pass `cargo fmt -- --check`
- Pass `cargo clippy -- -D warnings` (with the appropriate feature set)
- Pass `cargo test`
- Build with no compiler warnings

These are enforced by pre-commit hooks and CI (see `.github/workflows/`).

## CI

GitHub Actions (e.g. `.github/workflows/ci.yml`) run on push/PR:

- Build and test on Linux, macOS, Windows
- Format and Clippy checks
- Optional: release builds

CI checks out c2pa-rs as a sibling so the path dependency works.

## Helper scripts

- `scripts/install-hooks.sh` — Install git pre-commit hooks  
- `scripts/format.sh` — Format all Rust code  
- `build.sh` — Build CLI and/or GUI (e.g. `--all --release`, `--gui-only`, `--mac-app`)  
- `verify_setup.sh` — Full environment and test verification  
- `verify-gui-setup.sh` — Library, CLI, and GUI build check  

## Dependencies

- **Library (root)**: c2pa (path), serde, serde_json, anyhow, thiserror, jsonschema  
- **CLI (crtool-cli)**: crtool, c2pa, clap, reqwest, image, pem, jsonschema, etc.  
- **GUI (crtool-gui)**: crtool, eframe, egui, rfd, egui_code_editor, egui_json_tree, reqwest  

c2pa is a path dependency to `../c2pa-rs/sdk`.

## Troubleshooting

**"Failed to find c2pa"**  
Clone c2pa-rs as a sibling of crTool (see SETUP.md).

**Hooks not running**  
Reinstall: `./scripts/install-hooks.sh`.

**Formatting**  
Run `cargo fmt`, then `git diff` before committing.

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy](https://rust-lang.github.io/rust-clippy/)
- [Rustfmt](https://rust-lang.github.io/rustfmt/)
- [C2PA Specification](https://c2pa.org/specifications/)
