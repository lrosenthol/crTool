# crTool (CLI)

Command-line interface for Content Credential Tool. Create, embed, extract, and validate C2PA manifests in media assets.

## Build and run

From the workspace root:

```bash
cargo build --release -p crTool
cargo run -p crTool -- --help
```

Binary: `target/release/crTool` (or `target/debug/crTool` for debug builds).

## Options summary

Full option list and examples are in the root [README](../README.md). Key options:

| Option | Description |
|--------|-------------|
| `-m, --manifest <FILE>` | JSON manifest config (required for signing) |
| `-o, --output <PATH>` | Output file or directory (required for signing/extract) |
| `-c, --cert`, `-k, --key` | Certificate and private key (required for signing) |
| `-e, --extract` | Extract manifest to crJSON (read-only) |
| `--trust` | Enable C2PA and Content Credentials trust list validation for extract/read |
| `-v, --validate` | Validate JSON files against crJSON schema |
| `--allow-self-signed` | Allow self-signed certs (testing only) |
| `--ingredients-dir <DIR>` | Base dir for `ingredients_from_files` paths |
| `--thumbnail-asset`, `--thumbnail-ingredients` | Generate thumbnails |
| `-a, --algorithm` | Signing algorithm (optional; auto-detected from cert if omitted) |

## Dependencies

- **crtool**: Core library (path `..`); provides manifest extraction/validation and schema path.
- **c2pa**: C2PA SDK (path `../../c2pa-rs/sdk`); must be present as a sibling of the crTool repo when building.

See the root [README](../README.md) and [QUICKSTART](../QUICKSTART.md) for full usage and examples.
