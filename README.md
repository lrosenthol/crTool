# C2PA Testfile Maker

A Rust-based command-line tool that uses the [c2pa-rs](https://github.com/contentauth/c2pa-rs) library to create and embed C2PA (Coalition for Content Provenance and Authenticity) manifests into media assets based on JSON configuration files.

## Features

- 📝 Create C2PA manifests from JSON configuration files
- 🔐 Sign manifests with various cryptographic algorithms (ES256, ES384, ES512, PS256, PS384, PS512, ED25519)
- 🖼️ Support for multiple media formats (JPEG, PNG, and more)
- 📁 Flexible output options (specific file or directory)
- ⚡ Built with Rust for performance and safety

## Installation

### Prerequisites

- Rust 1.70 or later
- C/C++ compiler (required for some dependencies)

### Building from Source

**Important**: This project depends on a local copy of the c2pa-rs library. The `Cargo.toml` references it via a relative path.

```bash
# Clone both repositories as siblings
git clone https://github.com/lrosenthol/c2pa-testfile-maker.git
git clone https://github.com/contentauth/c2pa-rs.git

# Build the project
cd c2pa-testfile-maker

# Install git hooks (recommended for contributors)
./scripts/install-hooks.sh

# Build
cargo build --release
```

The compiled binary will be available at `target/release/c2pa-testfile-maker`.

**Directory Structure Required:**
```
parent-directory/
├── c2pa-testfile-maker/  (this repository)
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
c2pa-testfile-maker \
  --manifest <MANIFEST_JSON> \
  --input <INPUT_FILE> \
  --output <OUTPUT_PATH> \
  --cert <CERTIFICATE_FILE> \
  --key <PRIVATE_KEY_FILE> \
  [--algorithm <ALGORITHM>]
```

### Options

- `-m, --manifest <FILE>`: Path to the JSON manifest configuration file (required for signing, not needed for extract mode)
- `-i, --input <FILE>`: Path to the input media asset (JPEG, PNG, etc.) (required)
- `-o, --output <PATH>`: Path to the output file or directory (required)
- `-c, --cert <FILE>`: Path to the certificate file in PEM format (required for signing, not needed for extract mode)
- `-k, --key <FILE>`: Path to the private key file in PEM format (required for signing, not needed for extract mode)
- `-a, --algorithm <ALGORITHM>`: Signing algorithm (optional, auto-detected from certificate if not specified)
  - Supported: `es256`, `es384`, `es512`, `ps256`, `ps384`, `ps512`, `ed25519`
  - Auto-detection examines the certificate to determine the appropriate algorithm
- `-e, --extract`: Extract manifest from input file to JSON (read-only mode, no signing)
- `--allow-self-signed`: Allow self-signed certificates for testing/development (default: false)
  - ⚠️ **Warning**: Use only for development and testing with properly formatted certificates
  - Bypasses certificate chain validation during signer creation
  - Note: The c2pa library may still reject truly self-signed certificates during signing

### Example

```bash
./target/release/c2pa-testfile-maker \
  --manifest examples/manifest.json \
  --input examples/sample.jpg \
  --output output/signed_sample.jpg \
  --cert certs/certificate.pem \
  --key certs/private_key.pem
```

Note: The `--algorithm` parameter is optional. If not specified, the tool will automatically detect the appropriate algorithm from your certificate.

### Development/Testing with Test Certificates

For development and testing, you can use the included test certificates with the `--allow-self-signed` flag:

```bash
./target/release/c2pa-testfile-maker \
  --manifest examples/simple_manifest.json \
  --input testfiles/Dog.jpg \
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
./target/release/c2pa-testfile-maker \
  --manifest examples/manifest.json \
  --input examples/sample.jpg \
  --output output/ \
  --cert certs/certificate.pem \
  --key certs/private_key.pem
# Creates: output/sample.jpg
```

### Extracting Manifests

You can extract existing C2PA manifests from signed files using the `-e/--extract` option. This is useful for inspecting, analyzing, or archiving manifest data:

```bash
# Extract to a specific file
./target/release/c2pa-testfile-maker \
  --extract \
  --input signed_image.jpg \
  --output manifest.json

# Extract to a directory (auto-generates filename based on input)
./target/release/c2pa-testfile-maker \
  --extract \
  --input signed_image.jpg \
  --output output_directory/
# Creates: output_directory/signed_image_manifest.json
```

In extract mode:
- No certificate, key, or manifest file is required
- The tool reads the C2PA manifest from the input file using the c2pa-rs Reader
- The manifest is exported as formatted JSON
- If the output is a directory, the filename is auto-generated as `{input_stem}_manifest.json`
- The extracted JSON contains the complete manifest store including all assertions, signatures, and metadata


### Algorithm Auto-Detection

The tool can automatically detect the signing algorithm from your certificate, eliminating the need to specify `--algorithm`:

- **ES256/ES384/ES512**: Detected from ECDSA certificates based on the curve (P-256, P-384, or P-521)
- **PS256**: Detected from RSA certificates (defaults to PS256 for RSA keys)
- **Ed25519**: Detected from Ed25519 certificates

Example with auto-detection:

```bash
./target/release/c2pa-testfile-maker \
  --manifest examples/manifest.json \
  --input examples/sample.jpg \
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
    },
    {
      "label": "stds.schema-org.CreativeWork",
      "data": {
        "@context": "https://schema.org",
        "@type": "CreativeWork",
        "author": [
          {
            "@type": "Person",
            "name": "John Doe"
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

### Common Assertion Types

1. **Actions** (`c2pa.actions`): Records actions performed on the asset
2. **Creative Work** (`stds.schema-org.CreativeWork`): Metadata about the creative work
3. **EXIF** (`stds.exif`): Camera and image metadata
4. **Location** (`c2pa.location.broad`, `c2pa.location.narrow`): Geographic information
5. **Thumbnail** (`c2pa.thumbnail.claim.jpeg`, `c2pa.thumbnail.claim.png`): Preview images

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
