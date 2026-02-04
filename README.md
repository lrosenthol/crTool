# Content Credential Tool

A Rust-based command-line tool that uses the [c2pa-rs](https://github.com/contentauth/c2pa-rs) library to create and embed C2PA (Coalition for Content Provenance and Authenticity) manifests into media assets based on JSON configuration files.

## Features

- 📝 Create C2PA manifests from JSON configuration files
- 🔐 Sign manifests with various cryptographic algorithms (ES256, ES384, ES512, PS256, PS384, PS512, ED25519)
- 🖼️ Support for multiple media formats (JPEG, PNG, and more)
- 📁 Flexible output options (specific file or directory)
- 🧩 **File-based ingredient loading** - automatically load and embed parent/component ingredients from files
- 📤 **Extract manifests** - extract C2PA manifests from signed files to JSON (standard or JPEG Trust format)
- ✅ **JSON Schema validation** - validate extracted manifests or indicators documents against the JPEG Trust schema
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

The compiled binary will be available at `target/release/crTool`.

**Directory Structure Required:**
```
parent-directory/
├── crTool/               (this repository)
└── c2pa-rs/              (c2pa-rs SDK)
```

**Note for CI/CD**: The GitHub Actions CI workflow automatically checks out the c2pa-rs repository as a sibling directory, so the path dependency will work correctly in CI environments.

### For Contributors

The project uses automated code formatting and linting:
- **Pre-commit hooks** check formatting (`cargo fmt`) and linting (`cargo clippy`)
- Run `./scripts/install-hooks.sh` to install the hooks
- Run `./scripts/format.sh` to format all code

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed development workflow information.

## Usage

### Basic Command Structure

```bash
crTool \
  --manifest <MANIFEST_JSON> \
  <INPUT_FILE(S)> \
  --output <OUTPUT_PATH> \
  --cert <CERTIFICATE_FILE> \
  --key <PRIVATE_KEY_FILE> \
  [--algorithm <ALGORITHM>]
```

### Options

- `<INPUT_FILE>...`: Path(s) to input media asset(s) (JPEG, PNG, etc.) (required). Supports multiple files and glob patterns (e.g., `*.jpg`, `images/*.png`)
- `-m, --manifest <FILE>`: Path to the JSON manifest configuration file (required for signing, not needed for extract or validate mode)
- `-o, --output <PATH>`: Path to the output file or directory (required for signing and extract modes, not needed for validate mode). When processing multiple files, output must be a directory
- `-c, --cert <FILE>`: Path to the certificate file in PEM format (required for signing, not needed for extract or validate mode)
- `-k, --key <FILE>`: Path to the private key file in PEM format (required for signing, not needed for extract or validate mode)
- `-a, --algorithm <ALGORITHM>`: Signing algorithm (optional, auto-detected from certificate if not specified)
  - Supported: `es256`, `es384`, `es512`, `ps256`, `ps384`, `ps512`, `ed25519`
  - Auto-detection examines the certificate to determine the appropriate algorithm
- `-e, --extract`: Extract manifest from input file to JSON (read-only mode, no signing)
- `--jpt`: Use JPEG Trust format for extraction (only valid with `--extract`)
  - Outputs manifest data in the JPEG Trust JSON format as defined in the JPEG Trust specification
  - Includes `@context` field with JPEG Trust vocabulary
  - Includes computed asset hash in `asset_info`
  - Formats manifests as an array instead of an object
  - Different validation status structure compared to standard format
- `-v, --validate`: Validate JSON files against the JPEG Trust indicators schema
  - Validates one or more JSON files against the schema at `INTERNAL/schemas/indicators-schema.json`
  - Useful for validating extracted manifests or custom indicators documents
  - Provides detailed error messages for validation failures
  - Returns exit code 0 if all files are valid, non-zero otherwise
- `--allow-self-signed`: Allow self-signed certificates for testing/development (default: false)
  - ⚠️ **Warning**: Use only for development and testing with properly formatted certificates
  - Bypasses certificate chain validation during signer creation
  - Note: The c2pa library may still reject truly self-signed certificates during signing
- `--ingredients-dir <DIR>`: Base directory for resolving relative ingredient file paths
  - If not specified, defaults to the manifest file's parent directory
  - Used when the manifest includes `ingredients_from_files` entries
- `--thumbnail-asset`: Generate a thumbnail for the main asset (default: false)
  - Creates a 256x256 JPEG thumbnail of the input file
  - Embedded in the manifest for preview purposes
- `--thumbnail-ingredients`: Generate thumbnails for all ingredients (default: false)
  - Creates thumbnails for each ingredient loaded from files
  - Only applies to ingredients specified in `ingredients_from_files`

### Example (Single File)

```bash
./target/release/crTool \
  --manifest examples/manifest.json \
  examples/sample.jpg \
  --output output/signed_sample.jpg \
  --cert certs/certificate.pem \
  --key certs/private_key.pem
```

### Example (Multiple Files)

Process multiple files with the same manifest:

```bash
# Using explicit file list
./target/release/crTool \
  --manifest examples/manifest.json \
  testfiles/Dog.jpg testfiles/C.jpg \
  --output output/ \
  --cert tests/fixtures/certs/ed25519.pub \
  --key tests/fixtures/certs/ed25519.pem \
  --allow-self-signed

# Using glob patterns
./target/release/crTool \
  --manifest examples/manifest.json \
  "testfiles/*.jpg" \
  --output output/ \
  --cert tests/fixtures/certs/ed25519.pub \
  --key tests/fixtures/certs/ed25519.pem \
  --allow-self-signed

# Multiple glob patterns
./target/release/crTool \
  --manifest examples/manifest.json \
  "testfiles/*.jpg" "images/*.png" \
  --output output/ \
  --cert tests/fixtures/certs/ed25519.pub \
  --key tests/fixtures/certs/ed25519.pem \
  --allow-self-signed
```

**Note**: When using glob patterns in shells like bash or zsh, you may need to quote the patterns to prevent shell expansion.

Note: The `--algorithm` parameter is optional. If not specified, the tool will automatically detect the appropriate algorithm from your certificate.

### Development/Testing with Test Certificates

For development and testing, you can use the included test certificates with the `--allow-self-signed` flag:

```bash
./target/release/crTool \
  --manifest examples/simple_manifest.json \
  testfiles/Dog.jpg \
  --output output/Dog_signed.jpg \
  --cert tests/fixtures/certs/ed25519.pub \
  --key tests/fixtures/certs/ed25519.pem \
  --algorithm ed25519 \
  --allow-self-signed
```

**Note**: The test certificates in `tests/fixtures/certs/` have a proper certificate chain and work with the `--allow-self-signed` flag. Simple self-signed certificates may still be rejected by the c2pa library during the signing process.

### Output to Directory

If the output path is a directory, the tool will create a file with the same name as the input file:

```bash
./target/release/crTool \
  --manifest examples/manifest.json \
  examples/sample.jpg \
  --output output/ \
  --cert certs/certificate.pem \
  --key certs/private_key.pem
# Creates: output/sample.jpg
```

When processing multiple files, output **must** be a directory:

```bash
./target/release/crTool \
  --manifest examples/manifest.json \
  "testfiles/*.jpg" \
  --output output/ \
  --cert tests/fixtures/certs/ed25519.pub \
  --key tests/fixtures/certs/ed25519.pem \
  --allow-self-signed
# Creates: output/Dog.jpg, output/C.jpg, etc.
```

### Extracting Manifests

You can extract existing C2PA manifests from signed files using the `-e/--extract` option. This is useful for inspecting, analyzing, or archiving manifest data:

```bash
# Extract from a single file to a specific file (standard format)
./target/release/crTool \
  -e signed_image.jpg \
  --output manifest.json

# Extract in JPEG Trust format
./target/release/crTool \
  -e --jpt signed_image.jpg \
  --output manifest_jpt.json

# Extract from a single file to a directory (auto-generates filename based on input)
./target/release/crTool \
  -e signed_image.jpg \
  --output output_directory/
# Creates: output_directory/signed_image_manifest.json

# Extract in JPEG Trust format to a directory
./target/release/crTool \
  -e --jpt signed_image.jpg \
  --output output_directory/
# Creates: output_directory/signed_image_manifest_jpt.json

# Extract from multiple files (output must be a directory)
./target/release/crTool \
  -e "output/*.jpg" \
  --output manifests/
# Creates: manifests/image1_manifest.json, manifests/image2_manifest.json, etc.

# Extract from multiple files in JPEG Trust format
./target/release/crTool \
  -e --jpt "output/*.jpg" \
  --output manifests/
# Creates: manifests/image1_manifest_jpt.json, manifests/image2_manifest_jpt.json, etc.
```

In extract mode:
- No certificate, key, or manifest file is required
- The tool reads the C2PA manifest from the input file using the c2pa-rs Reader
- The manifest is exported as formatted JSON
- If the output is a directory, the filename is auto-generated as:
  - `{input_stem}_manifest.json` for standard format
  - `{input_stem}_manifest_jpt.json` for JPEG Trust format
- The extracted JSON contains the complete manifest store including all assertions, signatures, and metadata

**JPEG Trust Format**:
- Use the `--jpt` flag to extract in JPEG Trust format
- This format follows the JPEG Trust specification with:
  - `@context` field with JPEG Trust vocabulary reference
  - `asset_info` with computed SHA-256 hash of the asset
  - `manifests` as an array (not an object)
  - Different validation status structure
  - Compatible with JPEG Trust consumers and validators


### Validating JSON Files

The tool can validate JSON files against the JPEG Trust indicators schema. This is useful for:
- Verifying extracted manifests conform to the indicators specification
- Validating custom indicators documents before processing
- Checking JSON files for compliance with the JPEG Trust standard

```bash
# Validate a single JSON file
./target/release/crTool \
  --validate extracted_manifest.json

# Validate multiple JSON files
./target/release/crTool \
  --validate manifest1.json manifest2.json manifest3.json

# Validate using glob patterns
./target/release/crTool \
  --validate "manifests/*.json"
```

In validation mode:
- No `--output` flag is required (validation doesn't produce any files)
- The tool loads the indicators schema from `INTERNAL/schemas/indicators-schema.json`
- Each input file is validated against the schema
- Detailed error messages are provided for validation failures, including:
  - The path in the JSON where the error occurred
  - A description of what is invalid
- The tool exits with code 0 if all files are valid, non-zero otherwise
- A summary report is displayed showing the number of valid and invalid files

Example output for a valid file:
```
=== Validating JSON files against indicators schema ===

Loading schema from: "INTERNAL/schemas/indicators-schema.json"

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
=== Validating JSON files against indicators schema ===

Loading schema from: "INTERNAL/schemas/indicators-schema.json"

Schema compiled successfully

Validating: "invalid_manifest.json"
  ✗ Validation failed:
    - At /asset_info: "hash" is a required property
    - At /manifests/0/claim.v2/version: "string" is not of types "integer", "null"

=== Validation Summary ===
  Total files: 1
  Valid: 0
  Invalid: 1

=== Files with Validation Errors ===

"invalid_manifest.json":
    - At /asset_info: "hash" is a required property
    - At /manifests/0/claim.v2/version: "string" is not of types "integer", "null"
Error: 1 file(s) failed validation
```


### Algorithm Auto-Detection

The tool can automatically detect the signing algorithm from your certificate, eliminating the need to specify `--algorithm`:

- **ES256/ES384/ES512**: Detected from ECDSA certificates based on the curve (P-256, P-384, or P-521)
- **PS256**: Detected from RSA certificates (defaults to PS256 for RSA keys)
- **Ed25519**: Detected from Ed25519 certificates

Example with auto-detection:

```bash
./target/release/crTool \
  --manifest examples/manifest.json \
  examples/sample.jpg \
  --output output/signed_sample.jpg \
  --cert certs/certificate.pem \
  --key certs/private_key.pem
# The tool will display: "Auto-detecting signing algorithm from certificate..."
# And show the detected algorithm
```

You can still override the auto-detection by explicitly specifying `--algorithm` if needed.

## Manifest JSON Format

The manifest JSON file defines the C2PA manifest structure. Here's a comprehensive example:

```json
{
  "claim_generator_info": {
    "name": "my-app/1.0.0",
    "version": "1.0.0"
  },
  "title": "My Edited Photo",
  "format": "image/jpeg",
  "instance_id": "xmp:iid:12345678-1234-1234-1234-123456789abc",
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

### Manifest Structure

- **claim_generator_info**: Information about the application creating the manifest
  - `name`: Name and version of the generator
  - `version`: Version string
- **title**: Human-readable title for the content
- **format**: MIME type of the asset (e.g., "image/jpeg", "image/png")
- **instance_id**: Unique identifier for this instance
- **assertions**: Array of assertions to include in the manifest
  - `label`: The assertion type identifier
  - `data`: The assertion data (format depends on the label)
- **ingredients**: Array of parent assets (for edited content)

### Using File-Based Ingredients

The tool supports automatically loading ingredients (parent or component assets) from files. This is useful when creating manifests for edited or composite images that combine multiple source files.

#### Ingredient Configuration

To use file-based ingredients, add an `ingredients_from_files` array to your manifest JSON. Each ingredient can have:

- **file_path** (required): Path to the ingredient file (relative to the manifest or absolute)
- **title**: Human-readable title for the ingredient
- **relationship**: Either `"parentOf"` (for source/parent assets) or `"componentOf"` (for elements/components)
- **label**: Instance ID for referencing the ingredient in actions (e.g., in `ingredientIds`)
- **metadata**: Custom metadata fields (see below)

#### Ingredient Metadata Support

The `metadata` field allows you to attach both standard C2PA metadata and custom key/value pairs to ingredients:

**Standard C2PA metadata fields:**
- `dateTime`: ISO 8601 timestamp
- `reviewRatings`: Array of review ratings
- `dataSource`: Structured data source information
- `regionOfInterest`: Spatial/temporal regions
- `localizations`: Localized string translations

**Custom metadata fields:**
Any additional key/value pairs are preserved as custom metadata. This is useful for application-specific or vendor-specific metadata.

Example manifest with ingredient metadata:

```json
{
  "claim_generator_info": [
    {
      "name": "my-app/1.0.0",
      "version": "1.0.0"
    }
  ],
  "title": "Edited Photo",
  "assertions": [
    {
      "label": "c2pa.actions",
      "data": {
        "actions": [
          {
            "action": "c2pa.placed",
            "when": "2024-01-07T12:00:00Z",
            "parameters": {
              "ingredientIds": ["source_image"]
            }
          }
        ]
      }
    }
  ],
  "ingredients_from_files": [
    {
      "file_path": "../originals/photo.jpg",
      "label": "source_image",
      "title": "Original Image",
      "relationship": "componentOf",
      "metadata": {
        "com.example.asset-id": "abc123",
        "com.example.version": "2.0",
        "com.example.custom-data": {
          "author": "John Doe",
          "project": "Project X"
        }
      }
    }
  ]
}
```

In this example:
- Standard fields like `title`, `relationship`, and `label` configure the ingredient
- The `metadata` object contains custom namespaced fields
- Custom metadata is preserved in the ingredient's AssertionMetadata
- The ingredient can be referenced in actions using its `label` value

Example manifest with ingredients (basic):

```json
{
  "claim_generator_info": [
    {
      "name": "my-app/1.0.0",
      "version": "1.0.0"
    }
  ],
  "title": "Edited Photo",
  "format": "image/jpeg",
  "assertions": [
    {
      "label": "c2pa.actions",
      "data": {
        "actions": [
          {
            "action": "c2pa.edited",
            "when": "2024-01-07T12:00:00Z"
          }
        ]
      }
    }
  ],
  "ingredients_from_files": [
    {
      "title": "Original Image",
      "relationship": "parentOf",
      "file_path": "../originals/photo.jpg"
    },
    {
      "title": "Logo Overlay",
      "relationship": "componentOf",
      "file_path": "../assets/logo.png"
    }
  ]
}
```

#### Using Ingredients in CLI

When your manifest includes `ingredients_from_files`, the tool will:

1. Resolve file paths relative to the manifest's directory (or `--ingredients-dir` if specified)
2. Load each ingredient file
3. Add them to the manifest with the specified relationship
4. Optionally generate thumbnails (use `--thumbnail-ingredients` flag)

Example command:

```bash
./target/release/crTool \
  --manifest examples/with_ingredients_from_files.json \
  output/edited_photo.jpg \
  --output output/signed_with_ingredients.jpg \
  --cert certs/certificate.pem \
  --key certs/private_key.pem \
  --thumbnail-asset \
  --thumbnail-ingredients
```

To specify a custom base directory for ingredient paths:

```bash
./target/release/crTool \
  --manifest manifest.json \
  edited.jpg \
  --output signed.jpg \
  --cert cert.pem \
  --key key.pem \
  --ingredients-dir /path/to/source/files
```

#### Ingredient Features

- **Optional thumbnail generation**: Use `--thumbnail-ingredients` to generate thumbnails for ingredients
- **Format support**: Ingredients can be in any supported image format (JPEG, PNG, WebP, etc.)
- **Relationship tracking**: Properly marks ingredients as parent sources or added components
- **Path resolution**: Supports both relative and absolute file paths

### Mixing Ingredient Types

You can use both inline ingredient definitions (via the standard `ingredients` array) and file-based ingredients (via `ingredients_from_files`) in the same manifest. The `ingredients_from_files` are processed separately and won't create duplicate entries.



### Common Assertion Types

1. **Actions** (`c2pa.actions`): Records actions performed on the asset
2. **Metadata** (`c2pa.metadata`): Camera and image metadata
3. **Thumbnail** (`c2pa.thumbnail.claim.jpeg`, `c2pa.thumbnail.claim.png`): Preview images

## Generating Test Certificates

For testing purposes, you can generate self-signed certificates:

```bash
# Generate a private key
openssl ecparam -name prime256v1 -genkey -noout -out private_key.pem

# Generate a self-signed certificate
openssl req -new -x509 -key private_key.pem -out certificate.pem -days 365
```

**Note**: Self-signed certificates are suitable for testing only. For production use, obtain certificates from a trusted Certificate Authority.

## Examples

See the `examples/` directory for:
- Sample manifest JSON files
- Test images
- Example certificates (for testing only)

### Example Manifests

- **simple_manifest.json**: Basic manifest with minimal configuration
- **full_manifest.json**: Complete manifest with multiple assertions
- **simple_with_ingredient.json**: Manifest with a single file-based ingredient
- **with_ingredients_from_files.json**: Manifest demonstrating multiple file-based ingredients
- **actions_v2_*.json**: Examples of C2PA Actions v2 assertions (cropped, edited, filtered, etc.)
- **asset_ref_manifest.json**: Asset reference assertion example
- **cloud_data_manifest.json**: Cloud data assertion example
- **depthmap_gdepth_manifest.json**: Depth map assertion example

### Quick Start Examples

Create a signed image with a simple manifest:

```bash
./target/release/crTool \
  --manifest examples/simple_manifest.json \
  testfiles/Dog.jpg \
  --output output/Dog_signed.jpg \
  --cert tests/fixtures/certs/ed25519.pub \
  --key tests/fixtures/certs/ed25519.pem \
  --algorithm ed25519 \
  --allow-self-signed
```

Create a signed image with ingredients and thumbnails:

```bash
./target/release/crTool \
  --manifest examples/with_ingredients_from_files.json \
  testfiles/Dog.webp \
  --output output/Dog_with_ingredients.webp \
  --cert tests/fixtures/certs/ed25519.pub \
  --key tests/fixtures/certs/ed25519.pem \
  --algorithm ed25519 \
  --thumbnail-asset \
  --thumbnail-ingredients \
  --allow-self-signed
```

Extract a manifest from a signed file:

```bash
./target/release/crTool \
  -e output/Dog_signed.jpg \
  --output output/extracted_manifest.json
```

Validate JSON files against the indicators schema:

```bash
# Validate a single file
./target/release/crTool --validate manifest.json

# Validate multiple files
./target/release/crTool --validate file1.json file2.json

# Validate with glob patterns
./target/release/crTool --validate "manifests/*.json"
```

## Architecture

The tool is structured as follows:

1. **CLI Parsing**: Uses `clap` for argument parsing
2. **Manifest Loading**: Reads and validates the JSON manifest
3. **Builder Creation**: Creates a C2PA Builder from the JSON configuration
4. **Signer Creation**: Initializes the cryptographic signer with the provided certificates
5. **Signing & Embedding**: Signs the manifest and embeds it into the output asset

## Error Handling

The tool provides detailed error messages for common issues:
- Missing or invalid input files
- Invalid JSON manifest format
- Certificate or key file errors
- Unsupported signing algorithms
- File I/O errors

## Dependencies

Key dependencies:
- [c2pa](https://crates.io/crates/c2pa) - C2PA manifest creation and signing
- [clap](https://crates.io/crates/clap) - Command-line argument parsing
- [serde](https://crates.io/crates/serde) & [serde_json](https://crates.io/crates/serde_json) - JSON handling
- [jsonschema](https://crates.io/crates/jsonschema) - JSON Schema validation
- [anyhow](https://crates.io/crates/anyhow) - Error handling

## License

[Specify your license here]

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

### "Failed to create signer" Error

Ensure your certificate and private key files are in PEM format and match the selected signing algorithm.

### "Failed to sign and embed manifest" Error

Check that:
1. The input file format is supported
2. The manifest JSON is valid
3. The signing algorithm matches your key type

### Build Errors

If you encounter build errors, ensure you have:
1. The latest Rust toolchain (`rustup update`)
2. Required system libraries (OpenSSL, etc.)
3. A C/C++ compiler installed
