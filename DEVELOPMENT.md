# Development and Contributing Guide

This document covers everything needed to contribute to crTool: environment setup, daily workflow, code standards, testing, git process, and how to submit a pull request.

---

## Getting Started

### 1. Fork and Clone

External contributors: fork the repository on GitHub first, then clone your fork.

```bash
git clone https://github.com/YOUR_USERNAME/crTool.git
cd crTool

# Add the upstream repository
git remote add upstream https://github.com/lrosenthol/crTool.git
```

Internal contributors can clone directly.

### 2. Clone the Sibling Dependencies

crTool uses three local path dependencies. Clone them all as siblings of crTool:

```bash
cd ..
git clone https://github.com/contentauth/c2pa-rs.git
git clone https://github.com/lrosenthol/json-formula-rs.git
git clone https://github.com/lrosenthol/profile-evaluator-rs.git
cd crTool
```

Directory layout must be:

```text
parent/
├── crTool/
├── c2pa-rs/
├── json-formula-rs/
└── profile-evaluator-rs/
```

### 3. Install Development Tools

```bash
# Install git hooks for automatic formatting and linting on each commit
./scripts/install-hooks.sh

# Verify everything builds and tests pass
cargo build
cargo test
```

---

## Editor Setup (VS Code / Cursor)

The repo includes `.vscode/settings.json` with:

- Format on save for Rust
- Optional Clippy on save
- Recommended extension: **rust-analyzer**

Install rust-analyzer via the extensions panel if not prompted.

---

## Development Workflow

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

(Omit `--all-features` if the workspace has optional features that don't build in your environment.)

### Building and Running

```bash
# Debug builds
cargo build
cargo run -p crTool -- --help
cargo run -p crTool-gui

# Release builds
cargo build --release
cargo run --release -p crTool -- ...
cargo run --release -p crTool-gui
```

---

## Testing

### Running Tests

```bash
# All tests (library, CLI integration, extraction, validation)
cargo test

# Safer for integration tests — avoids file-access races
cargo test -- --test-threads=1

# Run a specific test by name
cargo test test_dog_jpg_simple_manifest

# Run tests with output visible
cargo test -- --nocapture

# Run only the library tests
cargo test -p crtool
```

Successful integration tests write signed images to `target/test_output/` (e.g. `Dog_simple.jpg`, `Dog_full.png`).

### Test Layout

- **`tests/integration_tests.rs`** — Sign test images (Dog.jpg, Dog.png, Dog.webp) with various manifests, verify output
- **`tests/test_crjson_extraction.rs`** — Manifest extraction and crJSON output
- **`tests/test_validation.rs`** — Schema validation
- **`tests/common/mod.rs`** — Shared helpers and fixture checks

For full test documentation see [tests/README.md](tests/README.md).

### Test Certificates

Integration tests use **Ed25519** certificates from the c2pa-rs test infrastructure:

- `tests/fixtures/certs/ed25519.pub` — certificate chain
- `tests/fixtures/certs/ed25519.pem` — private key

No CA-signed certs are required for the test suite. For manual CLI testing you can also use these same certs with `--allow-self-signed`, or generate self-signed certs:

```bash
./generate_test_certs.sh   # outputs to examples/certs/
```

### Verifying Signed Output

```bash
cargo install c2pa-tool
c2pa target/test_output/Dog_simple.jpg
```

### Pre-commit Hooks

Hooks live in `.git-hooks/`. They run `cargo fmt --check` and `cargo clippy` before each commit. To reinstall:

```bash
./scripts/install-hooks.sh
```

To skip (not recommended): `git commit --no-verify`.

---

## Code Quality Standards

All contributions must meet these requirements:

| Check | Command |
|-------|---------|
| Formatting | `cargo fmt --all` |
| Linting | `cargo clippy --all-targets --all-features -- -D warnings` |
| Tests | `cargo test` |
| Compiler warnings | Build must be warning-free |

These are enforced by pre-commit hooks and CI (see `.github/workflows/`).

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use idiomatic Rust patterns and meaningful names
- No `unwrap()` in library or CLI code — use `Result` and `.context()`

```rust
// ✅ Good — uses Result and context
fn process_file(path: &Path) -> Result<Data> {
    let content = fs::read_to_string(path)
        .context("Failed to read file")?;
    Ok(Data::parse(&content)?)
}

// ❌ Bad — panics on error
fn process_file(path: &Path) -> Data {
    let content = fs::read_to_string(path).unwrap();
    Data::parse(&content).unwrap()
}
```

### Documentation

- Public API items must have `///` doc comments
- Complex logic should have inline comments
- Update `README.md` when adding user-facing features

---

## Git Workflow

### Branching

```bash
# Sync your local main with upstream
git checkout main
git pull upstream main      # (or: git pull origin main for internal contributors)

# Create a feature branch
git checkout -b feature/your-feature-name
```

### Commit Messages

Write clear, imperative-mood commit messages:

```
Add support for PNG thumbnail extraction

- Implement PNG-specific thumbnail handling
- Add tests for PNG thumbnail cases
- Update documentation with PNG examples

Fixes #123
```

**Guidelines:**
- Use present tense ("Add feature" not "Added feature")
- First line: concise, 50–72 characters
- Body: details if needed
- Reference issues with `Fixes #123` or `Relates to #456`

### Submitting a Pull Request

1. Push your branch to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```
2. Open a Pull Request on GitHub from your fork to `lrosenthol/crTool:main`.
3. Describe the changes, related issues, and testing performed.
4. Wait for CI to pass and at least one maintainer approval.
5. Address any review comments; a maintainer will merge once approved.

---

## CI

GitHub Actions (`.github/workflows/ci.yml`) run on push/PR:

- Build and test on Linux, macOS, Windows
- `cargo fmt --check` and `cargo clippy`
- Release builds

CI checks out c2pa-rs as a sibling so the path dependency resolves correctly. Use `--test-threads=1` locally to match CI behavior and avoid flakiness.

---

## Codebase Architecture

The CLI (`crtool-cli`) is split into focused source modules:

| Module          | Responsibility                                                                  |
| --------------- | ------------------------------------------------------------------------------- |
| `main.rs`       | CLI argument parsing (`clap`), `Logger`, mode dispatch, `run_cli`               |
| `batch.rs`      | Batch file parsing and sequential command execution (`--batch`)                 |
| `processing.rs` | C2PA manifest signing, ingredient loading, algorithm detection                  |
| `test_case.rs`  | Test case JSON deserialization, `--create-test` mode                            |
| `extraction.rs` | Manifest extraction, crJSON output, JSON schema validation, trust list fetching |
| `profile.rs`    | Profile evaluation, report serialization                                        |

The core library (`crtool`) provides manifest extraction/validation logic and schema paths shared between the CLI and GUI.

See [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) for the full workspace layout and directory tree.

---

## Helper Scripts

| Script | Purpose |
|--------|---------|
| `scripts/install-hooks.sh` | Install git pre-commit hooks |
| `scripts/format.sh` | Format all Rust code |
| `build.sh` | Build CLI and/or GUI (`--all --release`, `--gui-only`, `--mac-app`) |
| `verify_setup.sh` | Full environment and test verification |
| `verify-gui-setup.sh` | Library, CLI, and GUI build check |

---

## Common Tasks

### Updating Dependencies

```bash
cargo update                           # update all to latest compatible versions
cargo update -p dependency-name        # update one specific crate
cargo outdated                         # check for newer versions (requires: cargo install cargo-outdated)
```

### Debugging

```bash
RUST_BACKTRACE=1 cargo run -- [args]
RUST_BACKTRACE=full cargo run -- [args]
rust-lldb target/debug/crTool          # step-through debugger
```

### Performance Profiling

```bash
cargo build --release
perf record ./target/release/crTool [args] && perf report          # Linux
instruments -t "Time Profiler" ./target/release/crTool [args]      # macOS
```

---

## Troubleshooting

**"Failed to find c2pa"** — Clone `c2pa-rs`, `json-formula-rs`, and `profile-evaluator-rs` as siblings of crTool (see step 2 above).

**Hooks not running** — Reinstall: `./scripts/install-hooks.sh`.

**Formatting drift** — Run `cargo fmt`, then `git diff` before committing.

---

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bug report**: Open a GitHub Issue with description, steps to reproduce, expected vs. actual behavior, and system info (OS, Rust version)
- **Feature request**: Open a GitHub Issue with the use case, proposed solution, and alternatives considered

---

## Code of Conduct

Be respectful and inclusive, provide constructive feedback, and show empathy toward other contributors.

## License

By contributing, you agree that your contributions will be licensed under the same Apache-2.0 license as the project.

---

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy](https://rust-lang.github.io/rust-clippy/)
- [Rustfmt](https://rust-lang.github.io/rustfmt/)
- [C2PA Specification](https://c2pa.org/specifications/)
