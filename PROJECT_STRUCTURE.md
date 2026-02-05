# Project Structure

```
crTool/
├── src/
│   └── main.rs                    # Main CLI application
├── examples/
│   ├── simple_manifest.json       # Basic manifest example
│   ├── full_manifest.json         # Complete metadata example
│   ├── with_ingredients.json      # Composite image example
│   └── README.md                  # Examples documentation
├── Cargo.toml                     # Rust project configuration
├── README.md                      # Main documentation
├── QUICKSTART.md                  # Quick start guide
├── LICENSE                        # MIT License
├── .gitignore                     # Git ignore rules
└── generate_test_certs.sh         # Certificate generation script
```

## Key Files

### Source Code
- **src/main.rs**: Complete CLI implementation with:
  - Argument parsing using `clap`
  - JSON manifest loading
  - C2PA Builder integration
  - Certificate-based signing
  - Error handling with detailed context

### Configuration Examples
Three comprehensive JSON manifest examples:
1. **simple_manifest.json**: Minimal setup for quick testing
2. **full_manifest.json**: Complete metadata with EXIF and location
3. **with_ingredients.json**: Multi-asset composition tracking

### Documentation
- **README.md**: Full documentation with API reference
- **QUICKSTART.md**: 5-minute getting started guide
- **examples/README.md**: Detailed examples documentation

### Tools
- **generate_test_certs.sh**: Generates test certificates for ES256, ES384, ES512, and PS256 algorithms

## Features Implemented

✅ JSON-driven manifest creation
✅ Multiple signing algorithms (ES256, ES384, ES512, PS256, PS384, PS512, ED25519)
✅ Flexible output (file or directory)
✅ Comprehensive error handling
✅ Complete documentation
✅ Example manifests
✅ Certificate generation tool
✅ Type-safe Rust implementation

## Build Artifacts

The project builds to:
- `target/release/crTool` - CLI binary (optimized)
- `target/debug/crTool` - CLI binary (debug)

## Dependencies

Core dependencies:
- `c2pa` v0.41 (with `file_io` and `unstable_api` features)
- `clap` v4.5 (CLI framework)
- `serde` + `serde_json` (JSON handling)
- `anyhow` + `thiserror` (Error handling)

## Usage Pattern

```
crTool \
  --manifest <JSON_FILE> \
  --input <MEDIA_FILE> \
  --output <OUTPUT_PATH> \
  --cert <CERT_PEM> \
  --key <KEY_PEM> \
  [--algorithm <ALG>]
```

## Next Steps

For users:
1. Run `cargo build --release`
2. Run `./generate_test_certs.sh`
3. Follow QUICKSTART.md

For developers:
1. Read src/main.rs for implementation details
2. Extend manifest examples as needed
3. Add new features following Rust best practices
