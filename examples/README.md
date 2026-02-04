# Example Manifests

This directory contains example C2PA manifest JSON files demonstrating various use cases.

## Available Examples

### 1. simple_manifest.json

A minimal manifest with basic metadata:
- Simple action (created)
- Basic author information
- License information

**Use case**: Marking an original creation with authorship and license.

### 2. full_manifest.json

A comprehensive manifest with extensive metadata:
- Multiple actions (opened, edited, filtered)
- Detailed author information with identifier
- EXIF camera metadata
- Keywords and licensing

**Use case**: Professional photo editing with complete provenance tracking.

### 3. with_ingredients.json

A manifest showing composite content from multiple sources:
- Multiple actions including compositing
- Ingredient list (parent images)
- Relationship tracking between assets

**Use case**: Creating composite images from multiple source files.

## Using These Examples

To use any of these examples with the tool:

```bash
# Simple example
./target/release/crTool \
  --manifest examples/simple_manifest.json \
  --input your_image.jpg \
  --output output.jpg \
  --cert your_cert.pem \
  --key your_key.pem

# Full metadata example
./target/release/crTool \
  --manifest examples/full_manifest.json \
  --input your_image.jpg \
  --output output.jpg \
  --cert your_cert.pem \
  --key your_key.pem

# With ingredients
./target/release/crTool \
  --manifest examples/with_ingredients.json \
  --input your_composite.jpg \
  --output output.jpg \
  --cert your_cert.pem \
  --key your_key.pem
```

## Customizing Manifests

You can customize these manifests by modifying:

1. **claim_generator_info**: Update with your application name and version
2. **title**: Change to describe your content
3. **format**: Match your output format (image/jpeg, image/png, etc.)
4. **actions**: Add or modify actions to reflect your workflow
5. **author**: Update with actual creator information
6. **assertions**: Add additional assertions as needed
7. **ingredients**: List parent assets if creating derivative works

## Common C2PA Actions

Here are standard C2PA action values you can use:

- `c2pa.created` - Original creation
- `c2pa.opened` - File was opened
- `c2pa.edited` - General editing
- `c2pa.filtered` - Applied filter
- `c2pa.color_adjustments` - Color/tone adjustments
- `c2pa.cropped` - Image was cropped
- `c2pa.resized` - Image was resized
- `c2pa.oriented` - Image was rotated
- `c2pa.converted` - Format conversion
- `c2pa.composited` - Combined multiple sources
- `c2pa.transcoded` - Media transcoding

## Assertion Types Reference

### Required/Common Assertions

- **c2pa.actions**: Documents the editing history

### Optional Assertions

- **c2pa.metadata**: Camera and capture metadata
- **c2pa.thumbnail.claim.jpeg**: Embedded thumbnail
- **c2pa.hash.data**: Hash of external data

## Validation

You can validate the created manifests using the c2pa tool:

```bash
# Install c2pa tool
cargo install c2pa-tool

# Validate and display manifest
c2pa output.jpg
```

## More Information

For more details on the C2PA specification:
- [C2PA Specification](https://c2pa.org/specifications/specifications/1.0/index.html)
- [Action Taxonomy](https://c2pa.org/specifications/specifications/1.0/specs/C2PA_Specification.html#_actions)
- [Assertion Reference](https://c2pa.org/specifications/specifications/1.0/specs/C2PA_Specification.html#_claim_assertions)
