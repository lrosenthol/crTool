# Setup and Testing Guide

This guide walks you through setting up the Content Credential Tool repository and running tests.

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **C/C++ compiler**:
  - macOS: Install Xcode Command Line Tools (`xcode-select --install`)
  - Linux: Install build-essential (`apt-get install build-essential`)
  - Windows: Install Visual Studio Build Tools
- **Git**: For cloning repositories

## Initial Setup

### 1. Clone the Repositories

This project requires the c2pa-rs library as a local dependency. Clone both repositories as siblings:

```bash
# Create a parent directory
mkdir c2pa-projects
cd c2pa-projects

# Clone c2pa-rs (the dependency)
git clone https://github.com/contentauth/c2pa-rs.git

# Clone crTool (this project)
git clone https://github.com/lrosenthol/crTool.git

# Your directory structure should look like:
# c2pa-projects/
#   ├── c2pa-rs/
#   └── crTool/
```

### 2. Verify the Setup

```bash
cd crTool
./verify_setup.sh
```

This script checks:
- Rust installation
- c2pa-rs dependency location
- Test files presence
- Test certificates
- Build success
- Test execution

### 3. Build the Project

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release
```

The binary will be at:
- Debug: `target/debug/crTool`
- Release: `target/release/crTool`

## Running Tests

The project includes a comprehensive test suite that covers:
- Multiple image formats (JPEG, PNG, WebP)
- Multiple manifest types (simple, full)
- File validation
- Manifest verification

### Run All Tests

```bash
cargo test
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Specific Test

```bash
# Test a specific image/manifest combination
cargo test test_dog_jpg_simple_manifest

# Run all tests for a specific format
cargo test test_dog_jpg

# Run the batch test
cargo test test_all_images_both_manifests
```

### Test Structure

```
tests/
├── integration_tests.rs      # Main test suite
├── common/
│   └── mod.rs                # Test helpers
└── fixtures/
    └── certs/                # Test certificates
        ├── ed25519.pub       # Public certificate
        └── ed25519.pem       # Private key
```

### Test Coverage

The test suite covers:

1. **Per-image tests**: Each test image with each manifest type
   - `test_dog_jpg_simple_manifest` - Dog.jpg + simple_manifest.json
   - `test_dog_jpg_full_manifest` - Dog.jpg + full_manifest.json
   - `test_dog_png_simple_manifest` - Dog.png + simple_manifest.json
   - `test_dog_png_full_manifest` - Dog.png + full_manifest.json
   - `test_dog_webp_simple_manifest` - Dog.webp + simple_manifest.json
   - `test_dog_webp_full_manifest` - Dog.webp + full_manifest.json

2. **Batch test**: `test_all_images_both_manifests`
   - Tests all combinations in one go
   - Provides summary statistics

3. **Validation tests**:
   - `test_fixtures_exist` - Verifies test infrastructure
   - `test_images_exist` - Checks test images
   - `test_manifests_exist` - Validates manifest files
   - `test_certs_exist` - Confirms certificates
   - `test_output_files_are_readable` - Validates output files

### Test Output

Successful tests create signed images in `target/test_output/`:

```
target/test_output/
├── Dog_simple.jpg
├── Dog_full.jpg
├── Dog_simple.png
├── Dog_full.png
├── Dog_simple.webp
└── Dog_full.webp
```

## Test Certificates

The tests use Ed25519 certificates that are included in the repository:
- `tests/fixtures/certs/ed25519.pub` - Public certificate chain
- `tests/fixtures/certs/ed25519.pem` - Private key

These certificates are from the c2pa-rs test infrastructure and pass all validation checks.

**Important**: These are for testing only. For production use, obtain certificates from a trusted Certificate Authority.

## Generating Example Certificates

The repository includes example certificates for the CLI tool:

```bash
./generate_test_certs.sh
```

This creates self-signed certificates in `examples/certs/` for all supported algorithms:
- ES256, ES384, ES512 (Elliptic Curve)
- PS256, PS384, PS512 (RSA with PSS padding)

## Testing the CLI

### Basic Usage Test

```bash
# Build the release binary
cargo build --release

# Test with simple manifest
./target/release/crTool \
  --manifest examples/simple_manifest.json \
  --input testfiles/Dog.jpg \
  --output output/Dog_signed.jpg \
  --cert examples/certs/es256_cert.pem \
  --key examples/certs/es256_private.pem \
  --algorithm es256
```

### Verify the Output

Use the c2pa-tool to verify the signed image:

```bash
# Install c2pa-tool
cargo install c2pa-tool

# Verify the manifest
c2pa output/Dog_signed.jpg
```

## Continuous Integration

The repository includes GitHub Actions workflows in `.github/workflows/ci.yml`:

- **Test workflow**: Runs on push/PR
  - Tests on Linux, macOS, and Windows
  - Tests with stable and beta Rust
  - Runs formatting checks (cargo fmt)
  - Runs linting (cargo clippy)
  - Runs all tests

- **Build workflow**: Creates release binaries
  - Builds for multiple platforms
  - Uploads artifacts

**Note**: CI builds may require adjustments for the local c2pa-rs dependency path.

## Troubleshooting

### "failed to load manifest" error

Ensure the c2pa-rs repository is cloned at `../c2pa-rs/` relative to this project.

### Test failures

1. Verify setup: `./verify_setup.sh`
2. Clean and rebuild: `cargo clean && cargo build`
3. Run tests individually to isolate issues
4. Check that test files exist in `testfiles/` and `tests/fixtures/`

### Build errors

1. Update Rust: `rustup update`
2. Check system dependencies (C++ compiler, OpenSSL)
3. Clear cargo cache: `rm -rf ~/.cargo/registry/cache`

### OpenSSL errors (Linux)

```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install pkg-config openssl-devel
```

## Development Workflow

1. Make changes to source code
2. Format code: `cargo fmt`
3. Check for issues: `cargo clippy`
4. Run tests: `cargo test`
5. Build: `cargo build --release`
6. Commit and push

## Additional Resources

- [C2PA Specification](https://c2pa.org/specifications/specifications/1.0/index.html)
- [c2pa-rs Documentation](https://docs.rs/c2pa/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

## Getting Help

- Check existing [Issues](https://github.com/lrosenthol/crTool/issues)
- Create a new issue with:
  - Rust version (`rustc --version`)
  - OS and version
  - Steps to reproduce
  - Error messages or logs
