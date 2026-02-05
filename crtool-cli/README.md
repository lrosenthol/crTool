# crTool (CLI)

Command-line interface for Content Credential Tool. Create, embed, extract, and validate C2PA manifests in media assets.

## Build and run

From the workspace root:

```bash
cargo build --release -p crTool
cargo run -p crTool -- --help
```

Binary: `target/release/crTool` (or `target/debug/crTool` for debug builds).

## Dependencies

- **crtool**: Core library (path `..`); provides manifest extraction/validation and schema path.
- **c2pa**: C2PA SDK (path `../../c2pa-rs/sdk`); must be present as a sibling of the crTool repo when building.

See the root [README](../README.md) and [QUICKSTART](../QUICKSTART.md) for full usage.
