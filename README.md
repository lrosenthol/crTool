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

- `-m, --manifest <FILE>`: Path to the JSON manifest configuration file (required)
- `-i, --input <FILE>`: Path to the input media asset (JPEG, PNG, etc.) (required)
- `-o, --output <PATH>`: Path to the output file or directory (required)
- `-c, --cert <FILE>`: Path to the certificate file in PEM format (required)
- `-k, --key <FILE>`: Path to the private key file in PEM format (required)
- `-a, --algorithm <ALGORITHM>`: Signing algorithm (default: es256)
  - Supported: `es256`, `es384`, `es512`, `ps256`, `ps384`, `ps512`, `ed25519`

### Example

```bash
./target/release/c2pa-testfile-maker \
  --manifest examples/manifest.json \
  --input examples/sample.jpg \
  --output output/signed_sample.jpg \
  --cert certs/certificate.pem \
  --key certs/private_key.pem \
  --algorithm es256
```

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

