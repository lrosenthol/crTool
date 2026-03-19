# Project Structure

crTool is a Cargo workspace with a shared library and two applications: CLI and GUI.

```
crTool/
├── Cargo.toml                     # Workspace root (members: ., crtool-cli, crtool-gui)
├── src/
│   └── lib.rs                     # crtool library (manifest extraction, validation, Builder)
├── crtool-cli/
│   ├── Cargo.toml                 # CLI package (binary name: crTool)
│   ├── README.md
│   └── src/
│       └── main.rs                # CLI implementation (clap, sign/extract/validate)
├── crtool-gui/
│   ├── Cargo.toml                 # GUI package (binary name: crTool-gui)
│   ├── README.md
│   ├── build.rs
│   ├── macos/                     # macOS app bundle assets
│   │   ├── Info.plist
│   │   ├── crTool.entitlements
│   │   └── open_document_handler.m
│   └── src/
│       ├── main.rs                # GUI implementation (eframe/egui)
│       ├── macos_open_document.rs
│       └── security_scoped.rs
├── examples/
│   ├── README.md
│   ├── simple_manifest.json
│   ├── full_manifest.json
│   ├── with_ingredients.json
│   ├── with_ingredients_from_files.json
│   ├── simple_with_ingredient.json
│   ├── actions_v2_*.json
│   ├── asset_ref_manifest.json
│   ├── cloud_data_manifest.json
│   ├── depthmap_gdepth_manifest.json
│   ├── external_reference_manifest.json
│   ├── asset_type_manifest.json
│   ├── specVersion_manifest.json
│   └── ...
├── testset/                       # Additional test assets and JSON (GUI/manifest tests)
├── tests/
│   ├── README.md                  # Test suite documentation
│   ├── integration_tests.rs       # Sign + verify integration tests
│   ├── test_crjson_extraction.rs
│   ├── test_validation.rs
│   ├── common/
│   │   └── mod.rs                 # Test helpers
│   └── fixtures/
│       ├── certs/                # ed25519.pem, ed25519.pub, es256_*
│       ├── assets/               # Dog.jpg, Dog.png, Dog.webp
│       └── *.json                # Validation test fixtures
├── INTERNAL/
│   ├── cddl/                      # CDDL definitions
│   └── schemas/
│       ├── crJSON-schema.json
│       └── indicators-schema.json
├── scripts/
│   ├── install-hooks.sh           # Git pre-commit hooks (fmt, clippy)
│   └── format.sh
├── .github/workflows/
│   ├── ci.yml
│   └── format-check.yml
├── .git-hooks/
│   └── pre-commit
├── build.sh                       # Build script (--all, --gui-only, --mac-app)
├── verify_setup.sh                # Verify clone, deps, build, tests
├── verify-gui-setup.sh            # Verify GUI build
├── README.md
├── QUICKSTART.md
├── PROJECT_STRUCTURE.md
├── SETUP.md                       # Initial setup and first run
├── DEVELOPMENT.md                 # Development workflow and tooling
├── TESTING.md                     # Redirect → DEVELOPMENT.md#testing and tests/README.md
├── CONTRIBUTING.md                # Redirect → DEVELOPMENT.md
├── LICENSE
└── rustfmt.toml
```

## Workspace Members

| Member      | Role    | Binary / library      |
|------------|---------|------------------------|
| `.` (root) | Library | `crtool` (lib only)    |
| `crtool-cli` | CLI app | `crTool`              |
| `crtool-gui` | GUI app | `crTool-gui`          |

## Key Directories

### Source
- **src/lib.rs**: Shared library API (e.g. `extract_crjson_manifest`, `validate_json_value`, Builder from JSON). Used by both CLI and GUI.
- **crtool-cli/src/main.rs**: CLI (sign, extract, validate) with `clap`, file I/O, and cert handling.
- **crtool-gui/src/main.rs**: Native GUI for opening files, extracting manifests (crJSON), validation, tree view, and trust status.

### Examples and test data
- **examples/**: Manifest JSON examples (simple, full, ingredients, actions_v2, etc.).
- **tests/fixtures/assets/**: Dog.{jpg,png,webp} for integration tests.
- **testset/**: Extra test images and JSON for GUI and validation tests.

### Build and verification
- **build.sh**: `--all`, `--gui-only`, `--release`, `--mac-app` (macOS .app bundle).
- **verify_setup.sh**: Checks Rust, c2pa-rs path, fixtures, build, and tests.
- **verify-gui-setup.sh**: Checks library, CLI, and GUI build and lib tests.

### Schemas and internal
- **INTERNAL/schemas/**: crJSON and indicators JSON schemas for validation.
- **INTERNAL/cddl/**: CDDL definitions (reference only).

## Build Artifacts

- `target/release/crTool` — CLI binary  
- `target/release/crTool-gui` — GUI binary  
- `target/debug/` — Debug builds for both  
- `target/test_output/` — Signed images produced by integration tests  

## Dependencies

- **Root (crtool lib)**: `c2pa` (path: `../c2pa-rs/sdk`), `serde`, `serde_json`, `anyhow`, `thiserror`, `jsonschema`.
- **crtool-cli**: `crtool`, `c2pa`, `clap`, `reqwest`, `image`, `pem`, `x509-parser`, `jsonschema`, etc.
- **crtool-gui**: `crtool`, `eframe`, `egui`, `rfd`, `egui_code_editor`, `egui_json_tree`, `reqwest`, etc.

**Note**: The CLI and library expect a sibling `c2pa-rs` repo; see [SETUP.md](SETUP.md).

## Documentation Map

| Document | Audience | Contents |
|----------|----------|----------|
| **README.md** | All | Overview, installation, CLI reference, manifest format, examples |
| **QUICKSTART.md** | New users | Run the CLI or GUI in minutes |
| **SETUP.md** | New users | One-time environment setup with verification |
| **DEVELOPMENT.md** | Contributors | Workflow, code standards, testing, git process, PR guide |
| **tests/README.md** | Contributors | Full test structure, fixtures, certificates, known issues |
| **crtool-gui/README.md** | GUI users | GUI features, usage, and platform notes |
| **examples/README.md** | All | Example manifest files and how to use them |
| **Cursor-Docs/README.md** | Developers | Index of internal implementation and historical design notes |
