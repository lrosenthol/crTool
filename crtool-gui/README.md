# crTool GUI

A graphical user interface for extracting and validating C2PA manifests from media files.

## Features

- 📂 **Multi-document**: Open multiple C2PA files at once; each document has its own tab.
- 📑 **Dockable tabs**: Tabs can be moved, split, and **undocked into separate windows** (egui_dock).
- 📥 **Open multiple files** via:
  - **File → Open...**: Multi-select in the file dialog
  - **Drag & drop** onto the main window (all dropped files are opened)
  - **macOS**: Drop on app icon or “Open With” (all files are opened)
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

1. Launch the application.
2. Open one or more files:
   - Click **“📂 Select File(s)...”** (or **File → Open...**) and choose one or more C2PA-supported files, or
   - Drag and drop files onto the window, or (on macOS) onto the app icon.
3. Each file opens in its own tab; you can drag tabs to reorder, split the view, or use the tab context menu to **“Move tab to new window”** to undock.
4. Use **File → Close** to close the active tab, **Close All** to close all documents, and **Save As...** to export the active tab’s manifest as JSON.

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
- **egui_dock**: Multi-document tabs with undockable windows
- **rfd**: Native file dialogs (multi-file open supported)
- **crtool**: Core library for manifest extraction and validation

## Cross-Platform Support

The GUI works on:
- ✅ macOS
- ✅ Windows
- ✅ Linux

## License

Apache License 2.0 - See [LICENSE](../LICENSE) file for details.
