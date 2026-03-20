# Quick Start Guide

Get started with Content Credential Tool in a few minutes. Choose the **CLI** (command line) or **GUI** (graphical app).

## Prerequisites

- **Rust 1.70+** — [rustup.rs](https://rustup.rs/)
- **c2pa-rs** — Clone as a sibling of crTool (see [SETUP.md](SETUP.md))

```text
parent/
├── crTool/
└── c2pa-rs/
```

## Quick Start: CLI

### 1. Build

```bash
cargo build --release -p crTool
```

### 2. Sign an image

Use the included test certs with `--allow-self-signed`:

```bash
./target/release/crTool \
  --manifest examples/simple_manifest.json \
  tests/fixtures/assets/raw/Dog.jpg \
  --output test_signed.jpg \
  --cert tests/fixtures/certs/ed25519.pub \
  --key tests/fixtures/certs/ed25519.pem \
  --algorithm ed25519 \
  --allow-self-signed
```

For other certs (e.g. self-signed ES256), see README.md “Generating Test Certificates” and use the same positional input style (input file(s) after `--manifest`).

### 3. Verify (optional)

```bash
cargo install c2pa-tool
c2pa test_signed.jpg
```

---

## Quick Start: GUI

### 1. Build and run

```bash
cargo build --release -p crTool-gui
cargo run --release -p crTool-gui
```

Or use the build script:

```bash
./build.sh --gui-only --release
cargo run --release -p crTool-gui
```

### 2. Use the GUI

1. Click **"📂 Select File(s)..."** (or **File → Open...**) or drag and drop files.
2. Choose one or more C2PA-supported files (e.g. JPEG, PNG, WebP with a manifest, such as `testset/test_ingredient_manifest.jpg`).
3. View results:
   - ✓ Green checkmark if valid, ✗ red X with errors if invalid
   - Trust status (Trusted/Untrusted) from C2PA and Content Credentials trust lists
   - Asset hash and manifest label
   - Tree view and "Show Raw JSON" toggle

### 3. Verify setup (optional)

```bash
./verify-gui-setup.sh
```

---

## Build both CLI and GUI

```bash
cargo build --release --workspace
```

Binaries:

- `target/release/crTool` — CLI  
- `target/release/crTool-gui` — GUI  

---

## What’s next?

- **CLI**: See [README.md](README.md) for all options (extract, validate, multiple files, globs, `--trust`).
- **GUI**: See [crtool-gui/README.md](crtool-gui/README.md).
- **Setup**: Full clone and verify steps in [SETUP.md](SETUP.md).
- **Development**: Hooks, fmt, clippy in [DEVELOPMENT.md](DEVELOPMENT.md).

## Troubleshooting

**"Input file does not exist"** — Check path; try absolute path.

**"Failed to create signer"** — Ensure cert and key exist and algorithm matches key type.

**"Failed to sign and embed manifest"** — Check manifest JSON and that input format is supported.

**Build errors** — Ensure `c2pa-rs` is at `../c2pa-rs/` and run `./verify_setup.sh`.

**GUI won’t start** — Update Rust and graphics drivers; on Linux ensure XDG portal is available for file dialogs.

**Help** — CLI: `./target/release/crTool --help`
