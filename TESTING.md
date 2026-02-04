# Running Integration Tests

The integration tests embed C2PA manifests into test images using the provided JSON manifests.

## Prerequisites

**Note**: The c2pa library requires properly signed certificates with a valid certificate chain. Self-signed certificates will cause tests to fail with "the certificate was self-signed" errors.

To run the tests successfully, you need:

1. Valid certificates from a trusted Certificate Authority (CA)
2. Or use the `--ignored` flag to skip certificate validation tests

## Test Structure

The tests are located in `tests/integration_tests.rs` and test the following:

- **Per-image tests**: Each test image (Dog.jpg, Dog.png, Dog.webp) is signed with:
  - `simple_manifest.json` 
  - `full_manifest.json`

- **Batch test**: `test_all_images_both_manifests` runs all combinations

## Running Tests

### With Valid Certificates

If you have valid CA-signed certificates:

1. Place them in `tests/fixtures/certs/`:
   - `es256_cert.pem` - Your certificate
   - `es256_private.pem` - Your private key

2. Run the tests:
```bash
cargo test
```

### Without Valid Certificates

The tests will fail with self-signed certificates due to c2pa library validation. This is expected behavior.

You can still verify the tool works by:

1. Testing the CLI directly with your images:
```bash
./target/release/crTool \
  --manifest examples/simple_manifest.json \
  --input testfiles/Dog.jpg \
  --output output/Dog_simple.jpg \
  --cert your_valid_cert.pem \
  --key your_valid_key.pem
```

2. Check the test helper functions compile:
```bash
cargo test --lib
```

## Test Output

Successful tests will create signed images in `target/test_output/`:
- `Dog_simple.jpg` - Dog.jpg with simple manifest
- `Dog_full.jpg` - Dog.jpg with full manifest  
- `Dog_simple.png` - Dog.png with simple manifest
- `Dog_full.png` - Dog.png with full manifest
- `Dog_simple.webp` - Dog.webp with simple manifest
- `Dog_full.webp` - Dog.webp with full manifest

## Verification

To verify a signed image has a valid C2PA manifest:

```bash
# Install c2pa-tool
cargo install c2pa-tool

# Verify the manifest
c2pa target/test_output/Dog_simple.jpg
```

## Known Limitations

- Self-signed certificates are rejected by the c2pa library during signing
- The `verify_after_sign` and `verify_trust` settings don't affect the initial certificate validation during signer creation
- For development/testing with self-signed certs, use the CLI tool directly and handle validation externally

## Alternative: Manual Testing

If you don't have valid certificates, you can manually test the functionality:

1. Generate test certificates:
```bash
./generate_test_certs.sh
```

2. Test the CLI (will create signed files, though validation may warn about self-signed):
```bash
for img in testfiles/*.{jpg,png,webp}; do
  base=$(basename "$img" | sed 's/\.[^.]*$//')
  ext="${img##*.}"
  ./target/release/crTool \
    --manifest examples/simple_manifest.json \
    --input "$img" \
    --output "output/${base}_simple.${ext}" \
    --cert examples/certs/es256_cert.pem \
    --key examples/certs/es256_private.pem
done
```

## Future Improvements

To make tests work with self-signed certificates, we would need:
- A test-only callback signer that bypasses certificate validation
- Or upstream changes to c2pa-rs to add a "test mode" flag
- Or use of properly configured test CA certificates

