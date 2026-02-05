# GUI Implementation Guide

This document describes the architecture of the GUI implementation for the Content Credential Tool.

## Architecture Overview

The project is now structured as a Cargo workspace with two main components:

```
crTool/
├── Cargo.toml           # Workspace root + library/CLI
├── src/
│   ├── lib.rs           # Library API (NEW)
│   └── main.rs          # CLI binary
└── crtool-gui/          # GUI crate (NEW)
    ├── Cargo.toml
    ├── src/
    │   └── main.rs
    └── README.md
```

## Library API (`src/lib.rs`)

The library exposes a clean, non-CLI API for GUI integration:

### Core Functions

1. **`extract_jpt_manifest(path)`**
   - Extracts C2PA manifest in JPEG Trust format
   - Returns `ManifestExtractionResult` with:
     - Parsed JSON manifest
     - Active manifest label
     - Computed asset hash
     - Raw JSON string

2. **`validate_json_value(json, schema_path)`**
   - Validates a JSON value against the indicators schema
   - Returns `ValidationResult` with:
     - Validation status (pass/fail)
     - Detailed error messages with JSON paths

3. **`validate_json_file(file_path, schema_path)`**
   - Convenience wrapper for validating JSON files

4. **`default_schema_path()`**
   - Returns path to the bundled indicators schema

### Design Principles

- **No `println!`**: All output is returned as structured data
- **Result types**: Uses `Result<T, anyhow::Error>` for error handling
- **Serializable**: All result types derive `Serialize`/`Deserialize`
- **Self-contained**: Minimal dependencies beyond what the CLI already uses

## GUI Implementation (`crtool-gui/`)

Built with **egui**, a pure Rust immediate-mode GUI framework.

### Features

1. **File Selection**
   - Native file dialogs via `rfd` crate
   - Filters for image formats (JPEG, PNG, WebP)

2. **Manifest Extraction**
   - Automatic extraction on file selection
   - Display of active manifest label and asset hash

3. **Validation Display**
   - Color-coded validation status
   - Detailed error messages with JSON paths
   - Expandable error list

4. **JSON Visualization**
   - Collapsible tree view of manifest structure
   - Syntax-highlighted values
   - Toggle for raw JSON view
   - Searchable/scrollable content

### UI Structure

```rust
CrtoolApp {
    selected_file: Option<PathBuf>,
    extraction_result: Option<Result<ManifestExtractionResult, String>>,
    validation_result: Option<ValidationResult>,
    show_raw_json: bool,
    schema_path: PathBuf,
}
```

### Key Components

- **File picker**: Native OS file dialogs
- **Status display**: Shows validation results with color coding
- **Tree view**: Recursive JSON display with collapsible sections
- **Raw JSON view**: Monospace, syntax-highlighted text editor

## Building & Running

### CLI Only
```bash
cargo build --release
cargo run --release -- --help
```

### GUI Only
```bash
cargo build --release -p crtool-gui
cargo run --release -p crtool-gui
```

### Both (Workspace)
```bash
cargo build --release --workspace
```

## Development Workflow

1. **Modify library**: Edit `src/lib.rs`
2. **Test library**: `cargo test --lib`
3. **Update CLI**: Edit `src/main.rs` if needed
4. **Update GUI**: Edit `crtool-gui/src/main.rs`
5. **Test GUI**: `cargo run -p crtool-gui`

## Cross-Platform Support

### macOS ✅
- Native file dialogs
- Metal rendering backend
- Full egui support

### Windows ✅
- Native file dialogs
- DirectX/Vulkan rendering
- Full egui support

### Linux ✅
- XDG file dialogs
- OpenGL/Vulkan rendering
- Full egui support

## Future Enhancements

Potential improvements for the GUI:

1. **Drag & Drop**: Drop image files directly onto the window
2. **Batch Processing**: Select and process multiple files
3. **Export Results**: Save validation reports
4. **Manifest Comparison**: Compare manifests from different files
5. **Search/Filter**: Search within manifest JSON
6. **Thumbnails**: Show image previews
7. **History**: Keep track of recently processed files
8. **Custom Schema**: Allow loading custom validation schemas

## Dependencies

### Core Library
- `c2pa`: C2PA manifest handling
- `serde_json`: JSON parsing
- `jsonschema`: Schema validation

### GUI-Specific
- `eframe`: Native window framework
- `egui`: Immediate-mode GUI
- `rfd`: Native file dialogs

## Testing

### Library Tests
```bash
cargo test --lib
```

### Integration Tests (CLI)
```bash
cargo test --test integration_tests
```

### GUI Manual Testing
```bash
cargo run -p crtool-gui
# Then test with files in testset/ directory
```

## Troubleshooting

### GUI Won't Start
- Ensure OpenGL/Metal/DirectX drivers are up to date
- Check terminal output for error messages

### File Dialog Issues
- On Linux, ensure XDG desktop portal is installed
- On macOS, app needs file access permissions

### Schema Not Found
- Verify `INTERNAL/schemas/indicators-schema.json` exists
- Check `CARGO_MANIFEST_DIR` environment variable

## License

MIT License - See LICENSE file for details
