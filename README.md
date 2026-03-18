# Content Credential Tool

A Rust-based tool (CLI and GUI) that uses the [c2pa-rs](https://github.com/contentauth/c2pa-rs) library to create and embed C2PA (Coalition for Content Provenance and Authenticity) manifests into media assets.

## Tools

- **CLI (`crTool`)**: Command-line tool for creating, embedding, extracting, and validating C2PA manifests (see `crtool-cli/README.md`)
- **GUI (`crTool-gui`)**: Graphical interface for extracting and validating C2PA manifests (see `crtool-gui/README.md`)

## Features

- 🧪 **Test asset creation** — create signed C2PA assets from structured test case JSON files
- 🔐 Sign with various cryptographic algorithms (ES256, ES384, ES512, PS256, PS384, PS512, ED25519)
- 🖼️ Support for multiple media formats (JPEG, PNG, and more)
- 🧩 **File-based ingredient loading** — automatically load and embed parent/component ingredients
- 📤 **Extract manifests** — extract C2PA manifests from signed files to crJSON format
- ✅ **JSON Schema validation** — validate crJSON manifests against the crJSON schema
- 📊 **Profile evaluation** — evaluate crJSON against YAML asset profiles
- ⚡ Built with Rust for performance and safety

## Installation

### Prerequisites

- Rust 1.70 or later
- C/C++ compiler (required for some dependencies)

### Building from Source

**Important**: This project depends on a local copy of the c2pa-rs library. The `Cargo.toml` references it via a relative path.

```bash
# Clone both repositories as siblings
git clone https://github.com/lrosenthol/crTool.git
git clone https://github.com/contentauth/c2pa-rs.git

# Build the project
cd crTool

# Install git hooks (recommended for contributors)
./scripts/install-hooks.sh

# Build
cargo build --release
```

The compiled CLI binary will be available at `target/release/crTool`.

### Building the GUI

```bash
cargo build --release -p crTool      # CLI only
cargo build --release -p crTool-gui  # GUI only

# Run the GUI
cargo run --release -p crTool-gui
```

See [`crtool-gui/README.md`](crtool-gui/README.md) for more details.

**Directory Structure Required:**

```
parent-directory/
├── crTool/               (this repository)
└── c2pa-rs/              (c2pa-rs SDK)
```

**Note for CI/CD**: The GitHub Actions CI workflow automatically checks out the c2pa-rs repository as a sibling directory.

### For Contributors

The project uses automated code formatting and linting:

- **Pre-commit hooks** check formatting (`cargo fmt`) and linting (`cargo clippy`)
- Run `./scripts/install-hooks.sh` to install the hooks
- Run `./scripts/format.sh` to format all code

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed development workflow information.

## Usage

### Modes of Operation

The CLI has four distinct modes:

| Mode | Flag | Description |
|------|------|-------------|
| **Create test asset** | `-t, --create-test <FILE>` | Read a test case JSON file and produce a signed asset |
| **Extract** | `-e, --extract` | Extract C2PA manifest from a signed asset to crJSON |
| **Validate** | `-v, --validate` | Validate JSON files against the crJSON schema |
| **Profile evaluation** | `--profile <FILE>` | Evaluate crJSON against a YAML asset profile |

### Options

- `<INPUT_FILE>...`: Path(s) to input media asset(s). Supports glob patterns (e.g., `"*.jpg"`). Not required for `--create-test`.
- `-t, --create-test <FILE>`: Path to a test case JSON file. Reads all signing configuration from the file (see [Test Case JSON Format](#test-case-json-format)).
- `-o, --output <PATH>`: Output file or directory. Required for `--create-test` and `--extract`. When processing multiple files, must be a directory.
- `-e, --extract`: Extract C2PA manifest from input file(s) to crJSON.
- `--trust`: Fetch and apply the official C2PA trust list and Content Credentials interim trust list during extraction. When enabled, output includes `signingCredential.trusted` or `signingCredential.untrusted` in `validationResults`. Requires network access.
- `-v, --validate`: Validate one or more JSON files against the crJSON schema.
- `--profile <FILE>`: Path to a YAML asset profile. When combined with `--extract`, evaluates the extracted crJSON immediately. When used alone (without `--extract`), treats input files as crJSON.
- `--report-format <FORMAT>`: Output format for the profile evaluation report. Options: `json` (default) or `yaml`.

---

## Creating Test Assets

The primary way to create signed C2PA assets is via the `--create-test` flag with a test case JSON file. This bundles all signing configuration — manifest, certificate, key, algorithm, and TSA URL — into a single reusable file.

```bash
./target/release/crTool \
  --create-test test-cases/positive/tc-created.json \
  --output output/tc-created.jpg
```

### Test Case JSON Format

Test case files follow the schema defined in `INTERNAL/schemas/test-case.schema.json`. All file paths in the JSON are resolved relative to the test case file's directory.

```json
{
  "testId": "validator.claimSignature.valid.created",
  "title": "Valid Claim Signature — c2pa.created Action",
  "description": "Optional human-readable description of what this test verifies.",
  "specVersion": "2.2",
  "inputAsset": "../../testfiles/Dog.jpg",
  "manifest": {
    "alg": "Ed25519",
    "claim_generator_info": [{ "name": "crTool/0.3.0", "version": "0.3.0" }],
    "title": "tc-created",
    "assertions": [
      {
        "label": "c2pa.actions",
        "data": {
          "actions": [
            {
              "action": "c2pa.created",
              "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia",
              "when": "2026-01-17T14:44:19Z"
            }
          ]
        }
      }
    ],
    "ingredients": []
  },
  "signingCert": "../../tests/fixtures/certs/ed25519.pub",
  "signingKey": "../../tests/fixtures/certs/ed25519.pem",
  "expectedResults": {
    "validationStatus": [
      { "code": "claimSignature.validated" }
    ]
  }
}
```

**Fields:**

| Field | Required | Description |
|-------|----------|-------------|
| `testId` | Yes | Unique identifier for the test case |
| `title` | No | Human-readable title |
| `description` | No | Description of what the test verifies |
| `specVersion` | No | C2PA specification version |
| `inputAsset` | Yes | Path to the input media file (relative to this JSON file) |
| `manifest` | Yes | C2PA manifest object. The optional `alg` field sets the signing algorithm; if omitted, the algorithm is auto-detected from `signingCert`. |
| `signingCert` | Yes | Path to the signing certificate in PEM format (relative to this JSON file) |
| `signingKey` | No | Path to the private key in PEM format. Defaults to `signingCert` if omitted. |
| `tsaUrl` | No | Timestamp Authority URL |
| `expectedResults` | Yes | Expected validation results (used by validators, not the tool itself) |

**Algorithm auto-detection:** If `manifest.alg` is absent, the tool examines `signingCert` to determine the algorithm automatically (ES256/ES384/ES512 from ECDSA curve, Ed25519 from Ed25519 key, PS256 from RSA key).

### Test Cases Directory

The `test-cases/` directory contains pre-built test case files organized by conformance intent:

```
test-cases/
├── positive/             # Conformant assets — expect claimSignature.validated
│   ├── tc-created.json
│   ├── tc-changes-spatial.json
│   ├── tc-placed-with-ingredient.json
│   └── tc-opened-with-ingredient.json
└── negative/             # Non-conformant assets — validly signed but profile-violating
    ├── tc-n-created-nodst.json
    ├── tc-n-removed.json
    ├── tc-n-inception.json
    ├── tc-n-placed-empty-params.json
    └── tc-n-redacted-bad-reason.json
```

All test cases use the Ed25519 test certificates in `tests/fixtures/certs/` and `testfiles/Dog.jpg` as the input asset.

---

## Extracting Manifests

Extract a C2PA manifest from a signed file to crJSON format using `-e`/`--extract`. Output always requires `--output`.

```bash
# Single file to a specific output file
./target/release/crTool \
  -e signed_image.jpg \
  --output manifest.json

# Single file to a directory (auto-generates filename: <stem>_cr.json)
./target/release/crTool \
  -e signed_image.jpg \
  --output output_directory/
# Creates: output_directory/signed_image_cr.json

# Multiple files (output must be a directory)
./target/release/crTool \
  -e "output/*.jpg" \
  --output manifests/
# Creates: manifests/image1_cr.json, manifests/image2_cr.json, etc.

# Extract with trust list validation
./target/release/crTool \
  -e --trust signed_image.jpg \
  --output output_directory/
```

In extract mode, the output crJSON includes `@context`, `manifests`, and `validationResults`. Use `--trust` to report whether signing certificates are on the C2PA or Content Credentials trust lists.

### Extract + Profile Evaluation

Combine `--extract` and `--profile` to extract a manifest and immediately evaluate it against a YAML asset profile:

```bash
./target/release/crTool \
  -e signed_image.jpg \
  --output output/ \
  --profile profiles/my-profile.yaml
```

The profile report is written alongside the crJSON file as `<stem>-report.json` (or `.yaml` with `--report-format yaml`).

---

## Validating JSON Files

Validate crJSON files against the crJSON schema with `-v`/`--validate`. No `--output` is needed.

```bash
# Validate a single file
./target/release/crTool --validate manifest.json

# Validate multiple files
./target/release/crTool --validate manifest1.json manifest2.json

# Validate using glob patterns
./target/release/crTool --validate "manifests/*.json"
```

The tool validates against `INTERNAL/schemas/crJSON-schema.json` and exits with code 0 if all files are valid, non-zero otherwise.

Example output for a valid file:

```
=== Validating JSON files against crJSON schema ===

Loading schema from: "INTERNAL/schemas/crJSON-schema.json"

Schema compiled successfully

Validating: "manifest.json"
  ✓ Valid

=== Validation Summary ===
  Total files: 1
  Valid: 1
  Invalid: 0

✓ All files are valid!
```

Example output for an invalid file:

```
Validating: "invalid_manifest.json"
  ✗ Validation failed:
    - At /asset_info: "hash" is a required property
    - At /manifests/0/claim.v2/version: "string" is not of types "integer", "null"
```

---

## Profile Evaluation

Evaluate a crJSON file against a YAML asset profile using `--profile`. The profile report is written alongside the input file.

```bash
# Standalone: evaluate existing crJSON files
./target/release/crTool \
  my_manifest_cr.json \
  --profile profiles/photojournalism.yaml

# Combined with extract: extract then evaluate
./target/release/crTool \
  -e signed_image.jpg \
  --output output/ \
  --profile profiles/photojournalism.yaml \
  --report-format yaml
```

---

## Manifest JSON Format

The `manifest` object inside a test case file follows the c2pa-rs JSON manifest format:

```json
{
  "alg": "Ed25519",
  "claim_generator_info": [{ "name": "my-app/1.0.0", "version": "1.0.0" }],
  "title": "My Asset",
  "assertions": [
    {
      "label": "c2pa.actions",
      "data": {
        "actions": [
          {
            "action": "c2pa.edited",
            "when": "2024-01-07T12:00:00Z",
            "softwareAgent": "MyApp 1.0"
          }
        ]
      }
    }
  ],
  "ingredients": []
}
```

### Using File-Based Ingredients

Add `ingredients_from_files` to the manifest object to load ingredient assets from files. Paths are resolved relative to the test case JSON file's directory.

```json
{
  "alg": "Ed25519",
  "claim_generator_info": [{ "name": "my-app/1.0.0", "version": "1.0.0" }],
  "title": "Edited Photo",
  "assertions": [
    {
      "label": "c2pa.actions",
      "data": {
        "actions": [
          {
            "action": "c2pa.placed",
            "when": "2024-01-07T12:00:00Z",
            "parameters": { "ingredientIds": ["source_image"] }
          }
        ]
      }
    }
  ],
  "ingredients_from_files": [
    {
      "file_path": "../../testfiles/Dog.jpg",
      "label": "source_image",
      "title": "Original Image",
      "relationship": "componentOf"
    }
  ]
}
```

Each ingredient entry supports:

- **`file_path`** (required): Path to the ingredient file
- **`title`**: Human-readable title
- **`relationship`**: `"parentOf"` or `"componentOf"`
- **`label`**: Instance ID for referencing in actions via `ingredientIds`
- **`metadata`**: Custom key/value metadata fields

---

## Supported File Formats

`avi`, `avif`, `c2pa`, `dng`, `gif`, `heic`, `heif`, `jpg`/`jpeg`, `m4a`, `mov`, `mp3`, `mp4`, `pdf`, `png`, `svg`, `tiff`, `wav`, `webp`

---

## Architecture

The CLI (`crtool-cli`) is split into focused source modules:

| Module | Responsibility |
|--------|---------------|
| `main.rs` | CLI argument parsing (`clap`), mode dispatch |
| `processing.rs` | C2PA manifest signing, ingredient loading, algorithm detection |
| `test_case.rs` | Test case JSON deserialization, `--create-test` mode |
| `extraction.rs` | Manifest extraction, crJSON output, JSON schema validation, trust list fetching |
| `profile.rs` | Profile evaluation, report serialization |

The core library (`crtool`) provides manifest extraction/validation logic and schema paths shared between the CLI and GUI.

---

## Generating Test Certificates

For testing, generate self-signed certificates:

```bash
# ECDSA P-256
openssl ecparam -name prime256v1 -genkey -noout -out private_key.pem
openssl req -new -x509 -key private_key.pem -out certificate.pem -days 365
```

The test certificates in `tests/fixtures/certs/` include `ed25519.pub`/`ed25519.pem` (Ed25519) and `es256_cert.pem`/`es256_private.pem` (ES256).

---

## Examples

### Create a test asset

```bash
./target/release/crTool \
  --create-test test-cases/positive/tc-created.json \
  --output output/tc-created.jpg
```

### Extract a manifest

```bash
./target/release/crTool \
  -e output/tc-created.jpg \
  --output output/tc-created_cr.json
```

### Extract with trust validation

```bash
./target/release/crTool \
  -e --trust output/tc-created.jpg \
  --output output/
```

### Validate crJSON files

```bash
./target/release/crTool --validate "output/*_cr.json"
```

### Extract and evaluate against a profile

```bash
./target/release/crTool \
  -e output/tc-created.jpg \
  --output output/ \
  --profile profiles/photojournalism.yaml
```

---

## Error Handling

The tool provides detailed error messages for common issues:

- Missing or invalid input files
- Invalid or non-conformant test case JSON
- Certificate or key file errors
- Unsupported file formats
- File I/O errors

---

## Dependencies

Key dependencies:

- [c2pa](https://crates.io/crates/c2pa) — C2PA manifest creation and signing
- [clap](https://crates.io/crates/clap) — Command-line argument parsing
- [serde](https://crates.io/crates/serde) & [serde_json](https://crates.io/crates/serde_json) — JSON handling
- [jsonschema](https://crates.io/crates/jsonschema) — JSON Schema validation
- [anyhow](https://crates.io/crates/anyhow) — Error handling

---

## License

Apache License 2.0 — See [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

**Before contributing:**

1. Install git hooks: `./scripts/install-hooks.sh`
2. Ensure code is formatted: `cargo fmt`
3. Ensure clippy passes: `cargo clippy -- -D warnings`
4. Run tests: `cargo test`

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed development workflow and best practices.

## Resources

- [C2PA Specification](https://c2pa.org/specifications/specifications/2.2/index.html)
- [c2pa-rs Documentation](https://docs.rs/c2pa/)
- [Content Authenticity Initiative](https://contentauthenticity.org/)

## Troubleshooting

### Build Errors

Ensure you have:
1. The latest Rust toolchain (`rustup update`)
2. Required system libraries (OpenSSL, etc.)
3. A C/C++ compiler installed
4. The `c2pa-rs` repository cloned as a sibling directory

### "Failed to parse test case JSON" Error

Ensure the test case file matches the schema in `INTERNAL/schemas/test-case.schema.json`. All required fields (`testId`, `inputAsset`, `manifest`, `signingCert`, `expectedResults`) must be present.

### "Failed to read C2PA data from input file" Error

The file may not contain a C2PA manifest. Only files previously signed with `--create-test` or another C2PA tool will have extractable manifests.
