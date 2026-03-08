# Setup and First Run

This guide covers one-time setup: prerequisites, cloning, verification, and first build and test.

## Prerequisites

- **Rust 1.70+**: [rustup.rs](https://rustup.rs/)
- **C/C++ compiler**:
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: e.g. `apt-get install build-essential`
  - Windows: Visual Studio Build Tools
- **Git**

## Initial Setup

### 1. Clone repositories

The project depends on a local copy of c2pa-rs. Clone both as siblings:

```bash
mkdir c2pa-projects
cd c2pa-projects

git clone https://github.com/contentauth/c2pa-rs.git
git clone https://github.com/lrosenthol/crTool.git

# Result:
# c2pa-projects/
#   ├── c2pa-rs/
#   └── crTool/
```

### 2. Verify setup

```bash
cd crTool
./verify_setup.sh
```

This checks Rust, c2pa-rs location, test files, certificates, build, and test run.

### 3. Build

```bash
# Debug (faster compile)
cargo build

# Release (optimized)
cargo build --release
```

To build only the CLI or only the GUI:

```bash
cargo build --release -p crTool      # CLI
cargo build --release -p crTool-gui # GUI
```

Binaries:

- CLI: `target/release/crTool` (or `target/debug/crTool`)
- GUI: `target/release/crTool-gui` (or `target/debug/crTool-gui`)

## Running tests

From the repo root:

```bash
cargo test
```

For more stable integration tests (avoids occasional file-access races):

```bash
cargo test -- --test-threads=1
```

See [TESTING.md](TESTING.md) for test layout and certificates, and [tests/README.md](tests/README.md) for detailed test documentation.

## Generating example certificates

For CLI experimentation, you can create self-signed certificates (see README.md “Generating Test Certificates”). Integration tests use `tests/fixtures/certs/` (see [TESTING.md](TESTING.md)).

## Next steps

- **Quick run**: [QUICKSTART.md](QUICKSTART.md) — CLI and GUI in a few steps.
- **Full usage**: [README.md](README.md) — options, extract, validate, examples.
- **Contributing**: [DEVELOPMENT.md](DEVELOPMENT.md) — hooks, fmt, clippy, workflow.

## Troubleshooting

**"failed to load manifest" / "Failed to find c2pa"**  
Ensure c2pa-rs is at `../c2pa-rs/` relative to crTool.

**Test failures**  
Run `./verify_setup.sh`, then `cargo clean && cargo build` and `cargo test -- --test-threads=1`. Confirm `testfiles/` and `tests/fixtures/` exist.

**Build errors**  
Update Rust (`rustup update`), check C++ and OpenSSL (e.g. Linux: `pkg-config libssl-dev`), clear cache if needed: `rm -rf ~/.cargo/registry/cache`.

**OpenSSL (Linux)**  
- Ubuntu/Debian: `sudo apt-get install pkg-config libssl-dev`  
- Fedora/RHEL: `sudo dnf install pkg-config openssl-devel`
