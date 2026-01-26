# Ingredient Metadata Support - RESOLVED

## Problem Statement (Original)

The `metadata` field in `ingredients_from_files` (e.g., in `testset/p-actions-placed-manifest-metadata.json`) was not being written to the output manifest:

```json
{
  "ingredients_from_files": [
    {
      "file_path": "test_ingredient_manifest.jpg",
      "label": "test_ingredient",
      "relationship": "componentOf",
      "metadata": {
        "com.adobe.repo.asset-id": "{hz-acp-asset-id }",
        "com.adobe.repo.version": "{hz-acp-version-id}"
      }
    }
  ]
}
```

These custom metadata fields did not appear in the generated C2PA manifest.

## Resolution (FIXED)

### Changes to c2pa-rs

The `AssertionMetadata` struct was enhanced to support arbitrary key/value pairs via:

1. Added `additional_fields: HashMap<String, Value>` with `#[serde(flatten)]`
2. Added `set_field()` method to add arbitrary key/value pairs
3. Added `set_additional_fields()` method to set multiple fields at once
4. Added `get_field()` and `additional_fields()` accessor methods

**Location:** `c2pa-rs/sdk/src/assertions/assertion_metadata.rs`

### Changes to c2pa-testfile-maker

Updated ingredient processing in both locations to parse and set metadata:

**1. Main CLI processor (`src/main.rs`):**

```rust
// Set metadata if provided
// This supports both standard C2PA AssertionMetadata fields and arbitrary custom fields
if let Some(metadata_obj) = ingredient_def.get("metadata") {
    if let Some(metadata_map) = metadata_obj.as_object() {
        use c2pa::assertions::AssertionMetadata;
        let mut assertion_metadata = AssertionMetadata::new();

        // Iterate through all key-value pairs in the metadata object
        for (key, value) in metadata_map {
            // Use set_field to add arbitrary key/value pairs
            assertion_metadata = assertion_metadata.set_field(key, value.clone());
        }

        ingredient.set_metadata(assertion_metadata);
        println!("  Set {} metadata field(s) on ingredient", metadata_map.len());
    }
}
```

**2. Test helper (`tests/common/mod.rs`):**

Same implementation added to `process_ingredients_with_thumbnails()`.

## Verification

The fix was verified by running the test suite:

```bash
cargo test --test integration_tests test_testset_manifests
```

Checking the output manifest confirms the metadata is present:

```bash
cat target/test_output/testset/p-actions-placed-manifest-metadata_manifest_jpt.json | \
  jq '.manifests[] | select(.assertions."c2pa.ingredient.v3".instance_id == "test_ingredient") | \
  .assertions."c2pa.ingredient.v3".metadata'
```

**Output:**
```json
{
  "dateTime": "2026-01-23T20:52:22.733Z",
  "com.adobe.repo.version": "{hz-acp-version-id}",
  "com.adobe.repo.asset-id": "{hz-acp-asset-id }"
}
```

✅ **Both custom fields are now successfully preserved in the output manifest!**

## Supported Metadata

The `metadata` field in `ingredients_from_files` now supports:

### Standard C2PA AssertionMetadata Fields
- `dateTime`: ISO 8601 timestamp (automatically added)
- `reviewRatings`: Array of review ratings
- `dataSource`: Structured data source information
- `regionOfInterest`: Spatial/temporal regions
- `localizations`: Localized string translations
- `reference`: Hashed URI reference to another assertion

### Custom/Arbitrary Fields
Any key/value pairs not matching standard fields are preserved as additional metadata, for example:
- `com.adobe.repo.asset-id`
- `com.adobe.repo.version`
- Any other custom namespaced fields

## Example Usage

```json
{
  "ingredients_from_files": [
    {
      "file_path": "ingredient.jpg",
      "label": "my_ingredient",
      "relationship": "componentOf",
      "metadata": {
        "com.example.custom-field": "custom value",
        "com.example.count": 42,
        "com.example.nested": {
          "foo": "bar",
          "items": [1, 2, 3]
        }
      }
    }
  ]
}
```

All fields in the `metadata` object will be preserved in the ingredient's AssertionMetadata.

## Implementation Notes

- The metadata is stored in the ingredient's `metadata` field as `AssertionMetadata`
- Custom fields are serialized at the same level as standard fields using serde's `flatten` attribute
- The `dateTime` field is automatically added if not explicitly provided
- All standard JSON types are supported: strings, numbers, booleans, objects, arrays
- Empty metadata objects are skipped during serialization
