# crTool GUI

A graphical user interface for extracting and validating C2PA manifests from media files.

## Features

- 📂 **File Selection**: Native file dialogs for easy file selection
- 🔍 **Manifest Extraction**: Extracts C2PA manifests in crJSON format (Content Credentials)
- 🔒 **Trust list validation**: Loads the official C2PA trust list and Content Credentials interim trust list at startup so that signing certificate trust status (Trusted / Untrusted) is shown for each manifest
- ✅ **Validation**: Validates extracted manifests against the crJSON schema (`INTERNAL/schemas/crJSON-schema.json`)
- 📊 **Visual Display**: 
  - Structured tree view of manifest data
  - Syntax-highlighted raw JSON view
  - Clear validation error messages
- 🎨 **Modern UI**: Built with egui for a clean, responsive interface

## Building

From the repository root:

```bash
# Build the GUI application
cargo build --release -p crTool-gui

# Run the GUI application
cargo run --release -p crTool-gui
```

## Usage

1. Launch the application
2. Click "📂 Select Image File"
3. Choose an image file with a C2PA manifest (JPEG, PNG, WebP)
4. View the extracted manifest and validation results

The application will:
- Load the C2PA and Content Credentials trust lists (requires network on first launch) and use them for certificate validation
- Extract the manifest in crJSON format
- Validate the manifest against the crJSON schema
- Display trust status (Trusted/Untrusted) and any validation errors
- Show the manifest data in a structured tree view

## Requirements

- Rust 1.70 or later
- The parent `crtool` library must be built
- The `c2pa-rs` library must be available as a sibling directory (same as CLI requirement)

## Architecture

The GUI is built using:
- **egui**: Immediate mode GUI framework
- **eframe**: Native window framework for egui
- **rfd**: Native file dialogs
- **crtool**: Core library for manifest extraction and validation

## Cross-Platform Support

The GUI works on:
- ✅ macOS
- ✅ Windows
- ✅ Linux

## License

MIT License - See LICENSE file for details
