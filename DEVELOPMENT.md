# Development Setup

This document covers the development workflow and tooling for the crTool project.

## Initial Setup

### 1. Clone Repositories

```bash
# Clone both repositories as siblings
git clone https://github.com/lrosenthol/crTool.git
git clone https://github.com/contentauth/c2pa-rs.git

cd crTool
```

### 2. Install Git Hooks

Install pre-commit hooks that automatically check formatting and linting:

```bash
./scripts/install-hooks.sh
```

This will install hooks that run before each commit to:
- Check code formatting with `cargo fmt`
- Run linting checks with `cargo clippy`

### 3. Editor Setup (Cursor/VS Code)

The project includes workspace settings in `.vscode/settings.json` that enable:
- **Auto-format on save** - Rust files are automatically formatted when you save
- **Clippy on save** - Code is checked with clippy as you work
- **Recommended extensions** - Cursor will prompt to install rust-analyzer

**Required Extension:**
- `rust-analyzer` - Provides Rust language support and formatting

If Cursor doesn't auto-prompt, install it manually:
1. Open Command Palette (Cmd+Shift+P / Ctrl+Shift+P)
2. Type "Extensions: Install Extensions"
3. Search for "rust-analyzer" and install

## Development Workflow

### Code Formatting

The project uses `rustfmt` for consistent code formatting. Configuration is in `rustfmt.toml`.

**Format all code:**
```bash
cargo fmt
```

**Or use the helper script:**
```bash
./scripts/format.sh
```

**Check formatting without modifying files:**
```bash
cargo fmt -- --check
```

### Linting

The project uses `clippy` for additional code quality checks.

**Run clippy:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Pre-commit Hooks

Git hooks are stored in `.git-hooks/` and automatically run before commits:

**To bypass hooks (not recommended):**
```bash
git commit --no-verify
```

**To reinstall hooks:**
```bash
./scripts/install-hooks.sh
```

## Building and Testing

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Run the Tool

```bash
cargo run -- \
  --manifest examples/simple_manifest.json \
  --input testfiles/Dog.jpg \
  --output target/output.jpg \
  --cert examples/certs/es256_cert.pem \
  --key examples/certs/es256_private.pem
```

## Continuous Integration

The project uses GitHub Actions for CI. The workflow:
- Runs on push to `main`/`master` branches and on pull requests
- Tests on Ubuntu, macOS, and Windows
- Tests with stable and beta Rust
- Checks formatting and clippy lints
- Builds release binaries for multiple targets

See `.github/workflows/ci.yml` for details.

## Code Quality Standards

All code must:
- Pass `cargo fmt -- --check`
- Pass `cargo clippy -- -D warnings`
- Pass all tests
- Have no compiler warnings

These checks are enforced by:
1. Pre-commit hooks (local)
2. CI pipeline (GitHub Actions)

## Helper Scripts

Located in `scripts/`:
- `install-hooks.sh` - Install git pre-commit hooks
- `format.sh` - Format all Rust code

## Dependencies

### Runtime Dependencies
- c2pa-rs (via path dependency to sibling directory)
- clap - CLI argument parsing
- serde/serde_json - JSON handling
- anyhow/thiserror - Error handling

### Development Dependencies
- ed25519-dalek - For test certificate handling
- pem - PEM file parsing for tests

## Troubleshooting

### "Failed to find c2pa" Error

Ensure c2pa-rs is cloned as a sibling directory:
```
parent-directory/
├── crTool/
└── c2pa-rs/
```

### Pre-commit Hooks Not Running

Reinstall hooks:
```bash
./scripts/install-hooks.sh
```

### Formatting Issues

Run the formatter:
```bash
cargo fmt
```

Then review changes with `git diff` before committing.

## Best Practices

1. **Always run tests before pushing:**
   ```bash
   cargo test
   ```

2. **Keep commits focused and atomic**

3. **Write descriptive commit messages**

4. **Update documentation when changing features**

5. **Don't skip pre-commit hooks** unless absolutely necessary

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Rustfmt Options](https://rust-lang.github.io/rustfmt/)
- [C2PA Specification](https://c2pa.org/specifications/)
