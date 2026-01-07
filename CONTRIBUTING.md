# Contributing to c2pa-testfile-maker

Thank you for your interest in contributing to c2pa-testfile-maker! This document provides guidelines and instructions for contributing.

## Getting Started

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/c2pa-testfile-maker.git
cd c2pa-testfile-maker

# Add the upstream repository
git remote add upstream https://github.com/lrosenthol/c2pa-testfile-maker.git
```

### 2. Set Up Dependencies

```bash
# Clone the required c2pa-rs dependency as a sibling directory
cd ..
git clone https://github.com/contentauth/c2pa-rs.git
cd c2pa-testfile-maker
```

### 3. Install Development Tools

```bash
# Install git hooks for automatic formatting and linting checks
./scripts/install-hooks.sh

# Verify everything works
cargo build
cargo test
```

## Development Workflow

### Creating a Branch

```bash
# Update your local main branch
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/your-feature-name
```

### Making Changes

1. **Write your code** following Rust best practices
2. **Format your code** (automatic via pre-commit hook, or run `cargo fmt`)
3. **Run clippy** to catch common issues: `cargo clippy -- -D warnings`
4. **Add/update tests** for your changes
5. **Run all tests**: `cargo test`
6. **Update documentation** if needed

### Code Quality Standards

All contributions must meet these requirements:

#### ✅ Formatting
```bash
cargo fmt --all
```
- Code must be formatted with `rustfmt`
- Configuration is in `rustfmt.toml`
- Pre-commit hooks will check this automatically

#### ✅ Linting
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
- No clippy warnings allowed
- Fix all clippy suggestions or add `#[allow]` attributes with justification

#### ✅ Tests
```bash
cargo test
```
- All tests must pass
- New features should include tests
- Bug fixes should include regression tests

#### ✅ Documentation
- Public APIs must be documented with `///` doc comments
- Complex logic should have inline comments
- Update README.md if adding user-facing features

### Commit Messages

Write clear, descriptive commit messages:

```
Add support for PNG thumbnail extraction

- Implement PNG-specific thumbnail handling
- Add tests for PNG thumbnail cases
- Update documentation with PNG examples

Fixes #123
```

**Guidelines:**
- Use present tense ("Add feature" not "Added feature")
- First line should be concise (50-72 chars)
- Provide details in the body if needed
- Reference issues/PRs with `Fixes #123` or `Relates to #456`

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in release mode (faster)
cargo test --release
```

### Submitting a Pull Request

1. **Push your changes** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** on GitHub from your fork to `lrosenthol/c2pa-testfile-maker:main`

3. **Fill out the PR template** (if available) with:
   - Description of changes
   - Related issues
   - Testing performed
   - Screenshots (if UI changes)

4. **Wait for review**:
   - CI checks must pass
   - At least one maintainer approval required
   - Address any review comments

5. **Merge**: Once approved, a maintainer will merge your PR

## Code Style Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use idiomatic Rust patterns
- Prefer explicit types over `var` when it aids clarity
- Use meaningful variable and function names

### Error Handling

```rust
// ✅ Good - uses Result and context
fn process_file(path: &Path) -> Result<Data> {
    let content = fs::read_to_string(path)
        .context("Failed to read file")?;
    Ok(Data::parse(&content)?)
}

// ❌ Bad - uses unwrap
fn process_file(path: &Path) -> Data {
    let content = fs::read_to_string(path).unwrap();
    Data::parse(&content).unwrap()
}
```

### Documentation

```rust
/// Processes a C2PA manifest and embeds it into an asset.
///
/// # Arguments
///
/// * `manifest` - The JSON manifest configuration
/// * `input` - Path to the input asset file
/// * `output` - Path where the signed asset will be written
///
/// # Errors
///
/// Returns an error if:
/// - The manifest JSON is invalid
/// - The input file cannot be read
/// - The signing process fails
///
/// # Example
///
/// ```no_run
/// # use c2pa_testfile_maker::*;
/// process_manifest(manifest, input, output)?;
/// ```
pub fn process_manifest(manifest: &str, input: &Path, output: &Path) -> Result<()> {
    // Implementation
}
```

## Project Structure

```
c2pa-testfile-maker/
├── .git-hooks/          # Git hooks (run via scripts/install-hooks.sh)
├── .github/             # GitHub Actions workflows
├── examples/            # Example manifests and test certificates
├── scripts/             # Helper scripts
├── src/                 # Source code
│   └── main.rs         # Main application logic
├── testfiles/          # Test input files
├── tests/              # Integration tests
│   ├── common/         # Test utilities
│   ├── fixtures/       # Test data
│   └── *.rs            # Test files
├── Cargo.toml          # Project dependencies
├── rustfmt.toml        # Formatting configuration
└── README.md           # User documentation
```

## Common Tasks

### Updating Dependencies

```bash
# Update all dependencies to latest compatible versions
cargo update

# Update a specific dependency
cargo update -p dependency-name

# Check for outdated dependencies
cargo outdated  # requires: cargo install cargo-outdated
```

### Debugging

```bash
# Run with debug output
RUST_BACKTRACE=1 cargo run -- [args]

# Run with full backtrace
RUST_BACKTRACE=full cargo run -- [args]

# Use a debugger (requires rust-lldb or rust-gdb)
rust-lldb target/debug/c2pa-testfile-maker
```

### Performance Profiling

```bash
# Build with release optimizations
cargo build --release

# Profile with perf (Linux)
perf record ./target/release/c2pa-testfile-maker [args]
perf report

# Profile with Instruments (macOS)
instruments -t "Time Profiler" ./target/release/c2pa-testfile-maker [args]
```

## Getting Help

- **Questions?** Open a GitHub Discussion
- **Bug report?** Open a GitHub Issue with:
  - Description of the problem
  - Steps to reproduce
  - Expected vs actual behavior
  - System info (OS, Rust version)
- **Feature request?** Open a GitHub Issue describing:
  - The use case
  - Proposed solution
  - Alternative approaches considered

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other contributors

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT).

## Recognition

Contributors will be recognized in:
- GitHub contributors list
- Release notes for significant contributions
- README.md (for major features)

Thank you for contributing to c2pa-testfile-maker! 🎉
