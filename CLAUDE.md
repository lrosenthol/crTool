# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

**crTool** is a Rust workspace for creating, extracting, and validating **C2PA (Coalition for Content Provenance and Authenticity)** manifests in media files. It provides signed cryptographic proofs of origin and editing history for images, video, and other media assets.

## Build & Test Commands

```bash
# Build (requires c2pa-rs cloned as sibling directory: ../c2pa-rs)
cargo build
cargo build --release

# Build specific workspace member
cargo build -p crTool          # core library
cargo build -p crtool-cli      # CLI
cargo build -p crtool-gui      # GUI

# Run tests (use --test-threads=1 for stable integration tests)
cargo test
cargo test -- --test-threads=1
cargo test -p crtool            # library tests only

# Lint & format
cargo fmt
cargo fmt -- --check           # check only (used in CI)
cargo clippy --all-targets --all-features -- -D warnings

# Build scripts
./build.sh --all --release     # CLI + GUI
./build.sh --cli-only
./build.sh --mac-app           # macOS .app bundle
```

## Architecture

### Workspace Structure
- **`src/`** — Core library (`crtool` crate): manifest extraction, JSON validation, trust settings
- **`crtool-cli/src/`** — CLI binary: argument parsing and all processing modes
- **`crtool-gui/src/`** — GUI binary using `egui`/`eframe`

### Critical Path Dependency
The project depends on `c2pa-rs` via a local path (`../c2pa-rs/sdk`). The `c2pa-rs` repository **must be cloned as a sibling directory** to this repo. CI handles this automatically via the `ci.yml` workflow.

### CLI Modules (`crtool-cli/src/`)

| Module | Purpose |
|--------|---------|
| `main.rs` | `clap`-based CLI parsing, `Logger`, `run_cli()` dispatcher, glob expansion |
| `processing.rs` | C2PA manifest signing (`process_single_file()`), ingredient loading, thumbnail generation, algorithm detection |
| `test_case.rs` | Test asset creation: reads `TestCase` JSON, resolves paths, calls processing |
| `extraction.rs` | Manifest extraction to crJSON, trust list fetching, JSON schema validation |
| `batch.rs` | Batch command execution from a batch JSON file |
| `profile.rs` | Evaluates crJSON against YAML asset profiles, generates reports |

### Core Library (`src/lib.rs`)
Exposes: `extract_crjson_manifest()`, `validate_json_file()`, `validate_json_value()`, `apply_trust_settings()`, `build_trust_settings()`, schema path helpers, and result types (`ManifestExtractionResult`, `ValidationResult`, `ValidationError`).

### CLI Operating Modes

| Flag | Mode | Input → Output |
|------|------|----------------|
| `-t / --create-test <PATTERN>` | Create test asset | Test case JSON (glob) → signed media file(s) |
| `-e / --extract` | Extract manifest | Signed media → crJSON + validation report |
| `-v / --validate` | Validate JSON | JSON file → schema pass/fail |
| `--profile <FILE>` | Profile evaluation | crJSON + YAML profile → JSON/YAML report |
| `-b / --batch <FILE>` | Batch execution | Batch JSON → sequential command results |

### Schemas & Test Assets
- **`INTERNAL/schemas/`** — `crJSON-schema.json`, `test-case.schema.json`, `batch.schema.json`
- **`tests/fixtures/assets/`** — Sample media files (Dog.jpg, Dog.png, Dog.webp)
- **`test-cases/`** — Positive and negative test cases (JSON)
- **`tests/fixtures/certs/`** — Test certificates (ed25519.pem, es256_*.pem)

## Code Quality

Pre-commit hooks run `cargo fmt --check` and `cargo clippy -- -D warnings`. Install with `./scripts/install-hooks.sh`. CI runs tests on Linux, macOS, and Windows with Rust stable and beta.

Formatting is governed by `rustfmt.toml`: `max_width = 100`, 4-space tabs, edition 2021.
