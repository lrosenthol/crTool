# Content Credential Tool

A Rust-based tool (CLI and GUI) which leverages the [c2pa-rs](https://github.com/contentauth/c2pa-rs) library to perform a variety of operations on Content Credentials (from the [C2PA](https://c2pa.org)).

## Tools

- **CLI (`crTool`)**: Command-line tool for creating, embedding, extracting, and validating C2PA manifests
- **GUI (`crTool-gui`)**: Graphical interface for extracting and validating C2PA manifests (see [crtool-gui/README.md](crtool-gui/README.md))

## Features

- 🧪 **Test asset creation** — create signed C2PA assets from structured test case JSON files
- 🔐 **Sign with various cryptographic algorithms** (ES256, ES384, ES512, PS256, PS384, PS512, ED25519)
- 🖼️ **Support for multiple media formats** (JPEG, PNG, and more)
- 🧩 **File-based ingredient loading** — automatically load and embed parent/component ingredients
- 📤 **Extract manifests** — extract C2PA manifests from signed files to crJSON format
- ✅ **JSON Schema validation** — validate crJSON manifests against the crJSON schema
- 📊 **Profile evaluation** — evaluate crJSON against YAML asset profiles
- ⚡ **Built with Rust** for performance and safety

## Documentation

| Document                                     | When to consult it                                                             |
| -------------------------------------------- | ------------------------------------------------------------------------------ |
| **This file** (README.md)                    | Complete CLI reference — all options, manifest format, examples, schemas       |
| [QUICKSTART.md](QUICKSTART.md)               | First time here and want to run the tool in minutes                            |
| [SETUP.md](SETUP.md)                         | One-time environment setup with verification steps                             |
| [DEVELOPMENT.md](DEVELOPMENT.md)             | Contributing — workflow, code standards, testing, git process, PR submission   |
| [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) | Workspace layout, module responsibilities, build artifacts                     |
| [tests/README.md](tests/README.md)           | Test structure, fixtures, certificates, known issues                           |
| [crtool-gui/README.md](crtool-gui/README.md) | GUI features, usage, and platform notes                                        |
| [examples/README.md](examples/README.md)     | Example manifest files and how to use them                                     |
| [TEST-FILE-CREATION-README.md](TEST-FILE-CREATION-README.md) | Test case JSON schema, manifest format, ingredient fields, test case directory |

---

## Installation

See [SETUP.md](SETUP.md) for full setup instructions (prerequisites, cloning all sibling repositories, and verification). See [QUICKSTART.md](QUICKSTART.md) to get running in minutes.

## Usage

### Modes of Operation

The CLI has four distinct modes, plus a batch mode for running multiple commands in one go:

| Mode                   | Flag                          | Description                                              |
| ---------------------- | ----------------------------- | -------------------------------------------------------- |
| **Create test asset**  | `-t, --create-test <PATTERN>` | Read test case JSON file(s) and produce signed assets    |
| **Extract**            | `-e, --extract`               | Extract C2PA manifest from a signed asset to crJSON      |
| **Validate**           | `-v, --validate`              | Validate JSON files against the crJSON schema            |
| **Profile evaluation** | `--profile <FILE>`            | Evaluate crJSON against a YAML asset profile             |
| **Batch**              | `-b, --batch <FILE>`          | Run multiple commands in sequence from a batch JSON file |

### Options

- `<INPUT_FILE>...`: Path(s) to input media asset(s). Supports glob patterns (e.g., `"*.jpg"`). When used with `--create-test`, the CLI inputs override the `inputAsset` field in the test case JSON, allowing the same test config to be applied to any asset. If the test case JSON has no `inputAsset` and no CLI inputs are provided, an error is returned.
- `-t, --create-test <PATTERN>`: Path or glob pattern for test case JSON file(s). Supports glob patterns (e.g., `"test-cases/positive/tc-*.json"`, `"test-cases/**/*.json"`). Reads all signing configuration from each matched file (see [Test Case JSON Format](#test-case-json-format)). When multiple test cases match, `--output` must be a directory.
- `-o, --output <PATH>`: Output file or directory. Required for `--create-test` and `--extract`. When processing multiple files, must be a directory.
- `-e, --extract`: Extract C2PA manifest from input file(s) to crJSON.
- `--trust`: Fetch and apply the official C2PA trust list and Content Credentials interim trust list during extraction. When enabled, output includes `signingCredential.trusted` or `signingCredential.untrusted` in `validationResults`. Requires network access.
- `-v, --validate`: Validate one or more JSON files against the crJSON schema.
- `--profile <FILE>`: Path to a YAML asset profile. When combined with `--extract`, evaluates the extracted crJSON immediately. When used alone (without `--extract`), treats input files as crJSON.
- `--report-format <FORMAT>`: Output format for the profile evaluation report. Options: `json` (default) or `yaml`.
- `-b, --batch <FILE>`: Path to a batch JSON file. Runs each command entry in sequence (see [Batch Mode](#batch-mode)).
- `-q, --quiet`: Suppress all progress output. Errors are still written to stderr.
- `-l, --log <FILE>`: Write all progress output to the specified log file in addition to stdout.
- `-h, --help`: Print help and exit.
- `-V, --version`: Print the tool version and exit.

---

## Creating Test Assets

The primary way to create signed C2PA assets is via the `--create-test` flag with a test case JSON file. This bundles all signing configuration — manifest, certificate, key, algorithm, and TSA URL — into a single reusable file.

```bash
./target/release/crTool \
  --create-test test-cases/positive/tc-created.json \
  --output output/tc-created.jpg
```

You can also pass a **glob pattern** to process multiple test cases at once. The output must be a directory when multiple test cases match:

```bash
# Process all positive test cases (each uses its own inputAsset from JSON)
./target/release/crTool \
  --create-test "test-cases/positive/tc-*.json" \
  --output output/

# Process all test cases in any subdirectory
./target/release/crTool \
  --create-test "test-cases/**/*.json" \
  --output output/

# Apply a glob of test cases to a specific input file
./target/release/crTool \
  --create-test "test-cases/positive/tc-*.json" \
  my-image.jpg \
  --output output/
```

### Test Case JSON Format

See [TEST-FILE-CREATION-README.md](TEST-FILE-CREATION-README.md) for the full test case schema, field reference, manifest format, ingredient configuration, and the pre-built test cases directory layout.

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

Evaluate a crJSON file against a YAML asset profile using `--profile`. Profiles are YAML documents that define a set of statements — each statement uses a [json-formula](https://opensource.adobe.com/json-formula/) expression to query the crJSON and produce a boolean outcome and localized report text. The evaluation is performed by the [profile-evaluator-rs](https://github.com/lrosenthol/profile-evaluator-rs) library, which in turn uses [json-formula-rs](https://github.com/lrosenthol/json-formula-rs) as its expression engine.

The `profiles/` directory contains built-in profiles:

| Profile                           | Description                                    |
| --------------------------------- | ---------------------------------------------- |
| `human-illustration_profile.yml`  | Human-created illustration (no AI generation)  |
| `real-life-capture_profile.yml`   | Real-world captured content (camera or device) |
| `real-media_profile.yml`          | Real media with authentic provenance           |
| `fully-generative-ai_profile.yml` | Fully AI-generated content                     |

The profile report is written alongside the input file as `<stem>-report.json` (or `.yaml` with `--report-format yaml`).

```bash
# Standalone: evaluate existing crJSON files
./target/release/crTool \
  my_manifest_cr.json \
  --profile profiles/human-illustration_profile.yml

# Combined with extract: extract then evaluate in one step
./target/release/crTool \
  -e signed_image.jpg \
  --output output/ \
  --profile profiles/human-illustration_profile.yml \
  --report-format yaml
```

---

## Batch Mode

Use `-b`/`--batch` to execute multiple commands in sequence from a single JSON file. The batch file is an array of command objects; each entry is run in order and a summary is printed at the end.

```bash
./target/release/crTool --batch my-batch.json

# Quiet: suppress progress, errors still shown
./target/release/crTool --batch my-batch.json --quiet

# Log all output to a file as well as stdout
./target/release/crTool --batch my-batch.json --log run.log
```

### Batch file format

```json
[
  {
    "command": "extract",
    "arguments": ["-o", "output/"],
    "inputFiles": ["input.jpg"]
  },
  {
    "command": "profile",
    "arguments": ["-o", "output/", "--profile", "profile.yaml"],
    "inputFiles": ["input.jpg"]
  },
  {
    "command": "test-cases",
    "arguments": [
      "--create-test",
      "test-cases/positive/tc-created.json",
      "-o",
      "output/"
    ],
    "inputFiles": ["input.jpg"]
  }
]
```

| Field        | Required | Description                                                             |
| ------------ | -------- | ----------------------------------------------------------------------- |
| `command`    | Yes      | Operation to perform: `extract`, `profile`, `test-cases`, or `validate` |
| `arguments`  | No       | Additional CLI arguments for the operation                              |
| `inputFiles` | No       | Input file paths or glob patterns                                       |

The JSON schema for batch files is at `INTERNAL/schemas/batch.schema.json`.

The `command` field sets the mode. For `extract` and `validate`, the corresponding flag (`--extract` / `--validate`) is injected automatically if not already present in `arguments`. For `profile` and `test-cases`, the required flags (`--profile` and `--create-test`) must appear in `arguments`.

---

## Supported File Formats

`avi`, `avif`, `c2pa`, `dng`, `gif`, `heic`, `heif`, `jpg`/`jpeg`, `m4a`, `mov`, `mp3`, `mp4`, `pdf`, `png`, `svg`, `tiff`, `wav`, `webp`

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

### Create a test asset with a different input (CLI override)

```bash
# Override the inputAsset in the JSON with a specific file
./target/release/crTool \
  --create-test test-cases/positive/tc-created.json \
  my-custom-image.jpg \
  --output output/my-custom-image-signed.jpg
```

### Apply the same test config to multiple files

```bash
# Output must be a directory when processing multiple inputs
./target/release/crTool \
  --create-test test-cases/positive/tc-created.json \
  tests/fixtures/assets/*.jpg \
  --output output/
```

### Process all test cases with a glob pattern

```bash
# Each test case uses its own inputAsset from the JSON
./target/release/crTool \
  --create-test "test-cases/positive/tc-*.json" \
  --output output/

# Apply all test cases to a specific input file
./target/release/crTool \
  --create-test "test-cases/**/*.json" \
  my-image.jpg \
  --output output/
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

- [c2pa](https://crates.io/crates/c2pa) — C2PA manifest creation and signing (local path: `../c2pa-rs/sdk`)
- [profile-evaluator-rs](https://github.com/lrosenthol/profile-evaluator-rs) — YAML asset profile evaluation (local path: `../profile-evaluator-rs`)
- [json-formula-rs](https://github.com/lrosenthol/json-formula-rs) — [json-formula](https://opensource.adobe.com/json-formula/) expression engine used by the profile evaluator (local path: `../json-formula-rs`)
- [clap](https://crates.io/crates/clap) — Command-line argument parsing
- [serde](https://crates.io/crates/serde) & [serde_json](https://crates.io/crates/serde_json) — JSON handling
- [jsonschema](https://crates.io/crates/jsonschema) — JSON Schema validation
- [anyhow](https://crates.io/crates/anyhow) — Error handling

---

## License

Apache License 2.0 — See [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! See [DEVELOPMENT.md](DEVELOPMENT.md) for the full guide: setup, code standards, git workflow, and how to submit a pull request.

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
4. All sibling repositories cloned: `c2pa-rs`, `profile-evaluator-rs`, and `json-formula-rs`

### "Failed to parse test case JSON" Error

Ensure the test case file matches the schema in `INTERNAL/schemas/test-case.schema.json`. Required fields are `testId`, `manifest`, `signingCert`, and `expectedResults`. `inputAsset` is optional in the JSON but must be supplied on the command line if omitted.

### "Failed to read C2PA data from input file" Error

The file may not contain a C2PA manifest. Only files previously signed with `--create-test` or another C2PA tool will have extractable manifests.
