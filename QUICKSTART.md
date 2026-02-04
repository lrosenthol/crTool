# Quick Start Guide

Get started with Content Credential Tool in 5 minutes!

## Step 1: Build the Tool

```bash
cargo build --release
```

## Step 2: Generate Test Certificates

```bash
./generate_test_certs.sh
```

This creates self-signed certificates in `examples/certs/` for testing.

## Step 3: Create a Test Image

If you don't have a test image, create one or download one:

```bash
# Download a sample image (example using curl)
curl -o test.jpg https://via.placeholder.com/800x600.jpg
```

Or use any JPEG or PNG image you have.

## Step 4: Run the Tool

```bash
./target/release/crTool \
  --manifest examples/simple_manifest.json \
  --input test.jpg \
  --output test_signed.jpg \
  --cert examples/certs/es256_cert.pem \
  --key examples/certs/es256_private.pem \
  --algorithm es256
```

## Step 5: Verify the Manifest

Install and use the c2pa-tool to verify:

```bash
# Install verification tool
cargo install c2pa-tool

# Verify and display the manifest
c2pa test_signed.jpg
```

## What's Next?

1. **Customize manifests**: Edit the JSON files in `examples/` to match your use case
2. **Try different examples**:
   - `simple_manifest.json` - Basic manifest
   - `full_manifest.json` - Complete metadata
   - `with_ingredients.json` - Composite images
3. **Integrate**: Use this tool in your build pipeline or workflow
4. **Read the docs**: Check `README.md` for detailed documentation

## Troubleshooting

**Error: "Input file does not exist"**
- Make sure the input file path is correct
- Use absolute paths if relative paths don't work

**Error: "Failed to create signer"**
- Ensure certificate and key files exist
- Check that the algorithm matches your key type

**Error: "Failed to sign and embed manifest"**
- Verify your manifest JSON is valid
- Ensure the input file format is supported (JPEG, PNG, etc.)

## Common Use Cases

### Original Creation
```bash
# Mark an original photo with your authorship
./target/release/crTool \
  --manifest examples/simple_manifest.json \
  --input original.jpg \
  --output signed_original.jpg \
  --cert examples/certs/es256_cert.pem \
  --key examples/certs/es256_private.pem
```

### Edited Content
```bash
# Document editing history with full metadata
./target/release/crTool \
  --manifest examples/full_manifest.json \
  --input edited_photo.jpg \
  --output signed_edited.jpg \
  --cert examples/certs/es256_cert.pem \
  --key examples/certs/es256_private.pem
```

### Batch Processing
```bash
# Process multiple files
for file in input/*.jpg; do
  basename=$(basename "$file")
  ./target/release/crTool \
    --manifest examples/simple_manifest.json \
    --input "$file" \
    --output "output/$basename" \
    --cert examples/certs/es256_cert.pem \
    --key examples/certs/es256_private.pem
done
```

## Help

For more options:
```bash
./target/release/crTool --help
```

For detailed documentation, see [README.md](README.md).
