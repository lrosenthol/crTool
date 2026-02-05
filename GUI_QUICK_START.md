# crTool GUI - Quick Start Guide

## What You Have Now

Your crTool project now includes a fully functional GUI application for extracting and validating C2PA manifests in JPEG Trust format.

## Build & Run

### Option 1: Using the Build Script
```bash
# Build both CLI and GUI in release mode
./build.sh --all --release

# Or just the GUI
./build.sh --gui-only --release
```

### Option 2: Using Cargo Directly
```bash
# Build the GUI
cargo build --release -p crtool-gui

# Run the GUI
cargo run --release -p crtool-gui
```

## Using the GUI

1. **Launch the application**
   ```bash
   cargo run --release -p crtool-gui
   ```

2. **Click "📂 Select Image File"**
   - Choose a JPEG, PNG, or WebP file with a C2PA manifest
   - Try: `testset/test_ingredient_manifest.jpg`

3. **View Results**
   - ✓ Green checkmark if manifest is valid
   - ✗ Red X with error details if invalid
   - Asset hash and manifest label displayed
   - Tree view of manifest structure
   - Toggle "Show Raw JSON" for the full JSON

## Project Structure

```
crTool/
├── src/
│   ├── lib.rs           # NEW: Library API for GUI
│   └── main.rs          # Existing CLI (unchanged)
├── crtool-gui/          # NEW: GUI application
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       └── main.rs      # GUI implementation
└── Cargo.toml           # Workspace configuration
```

## Key Features

### Library API (`src/lib.rs`)
- `extract_jpt_manifest()` - Extract manifests in JPEG Trust format
- `validate_json_value()` - Validate against indicators schema
- Returns structured data (no console output)

### GUI Application
- Native file picker (macOS/Windows/Linux)
- Automatic manifest extraction
- Real-time validation
- Collapsible tree view
- Syntax-highlighted JSON
- Color-coded validation results
- Detailed error messages

## Documentation

- **GUI Usage**: [`crtool-gui/README.md`](crtool-gui/README.md)
- **Architecture**: [`Cursor-Docs/GUI_IMPLEMENTATION.md`](Cursor-Docs/GUI_IMPLEMENTATION.md)
- **Setup Complete**: [`Cursor-Docs/GUI_SETUP_COMPLETE.md`](Cursor-Docs/GUI_SETUP_COMPLETE.md)

## Testing

Test with your existing files:
```bash
# Run the GUI
cargo run --release -p crtool-gui

# Then select files from:
# - testset/test_ingredient_manifest.jpg (has manifest)
# - testset/*.json (for validation testing)
```

## Verification

Check everything is set up correctly:
```bash
./verify-gui-setup.sh
```

Or manually:
```bash
cargo check --lib              # Check library compiles
cargo check --bin crtool       # Check CLI compiles
cargo check -p crtool-gui      # Check GUI compiles
cargo test --lib               # Run library tests
```

## Cross-Platform

The GUI works on:
- ✅ **macOS** - Native Metal rendering
- ✅ **Windows** - Native DirectX/Vulkan rendering  
- ✅ **Linux** - OpenGL/Vulkan rendering

## Library Independence

The new library (`src/lib.rs`) can be used by:
- The GUI application
- The CLI (if refactored)
- External Rust projects
- Future web services
- Other language bindings (via FFI)

## Distribution

For release distribution:
```bash
# Build optimized binaries
cargo build --release --workspace

# Binaries are at:
# - target/release/crtool       (CLI)
# - target/release/crtool-gui   (GUI)
```

### macOS Distribution
Consider creating an `.app` bundle:
```bash
# Basic app structure
mkdir -p crTool.app/Contents/MacOS
cp target/release/crtool-gui crTool.app/Contents/MacOS/
# Add Info.plist, icon, etc.
```

### Windows Distribution
Add app icon and version info to `crtool-gui/Cargo.toml`:
```toml
[package.metadata.winresource]
OriginalFilename = "crtool-gui.exe"
FileDescription = "C2PA Content Credential Tool"
ProductName = "crTool GUI"
```

## Next Steps

1. **Test the GUI**: `cargo run --release -p crtool-gui`
2. **Try with your files**: Use `testset/` files
3. **Customize if needed**: Edit `crtool-gui/src/main.rs`
4. **Add features**: See enhancement ideas in `GUI_SETUP_COMPLETE.md`

## Support

If you encounter issues:
- **Build errors**: Update Rust with `rustup update`
- **Missing c2pa-rs**: Ensure `../c2pa-rs/sdk` exists (sibling directory)
- **GUI won't start**: Update graphics drivers
- **File dialog issues**: On Linux, ensure XDG portal is installed

## Summary

You now have:
- ✅ A clean library API for manifest extraction and validation
- ✅ A cross-platform GUI application
- ✅ Workspace structure for easy development
- ✅ Documentation and build scripts
- ✅ All original CLI functionality preserved

**Ready to use!** Run `cargo run --release -p crtool-gui` to start. 🚀
