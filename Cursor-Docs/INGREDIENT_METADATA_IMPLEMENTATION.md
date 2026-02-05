# Ingredient Metadata Implementation

## Summary

Successfully implemented support for custom metadata on ingredients loaded from external files. This allows arbitrary key/value pairs to be attached to ingredients via the `metadata` field in `ingredients_from_files`.

## Changes Made

### 1. c2pa-rs Enhancements (upstream)

**File:** `c2pa-rs/sdk/src/assertions/assertion_metadata.rs`

Added support for arbitrary key/value pairs to the `AssertionMetadata` struct:

```rust
pub struct AssertionMetadata {
    // ... existing standard fields ...

    /// Arbitrary key/value pairs as permitted by the C2PA spec.
    /// Uses flatten to allow these fields to be serialized at the same level as known fields.
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    additional_fields: HashMap<String, Value>,
}
```

**New methods:**

- `set_field<S: Into<String>>(self, key: S, value: Value) -> Self` - Set a single arbitrary field
- `get_field(&self, key: &str) -> Option<&Value>` - Get a field value
- `additional_fields(&self) -> &HashMap<String, Value>` - Get all additional fields
- `set_additional_fields(self, fields: HashMap<String, Value>) -> Self` - Set multiple fields at once

### 2. crTool Implementation

#### Main CLI Processor

**File:** `src/main.rs`

Added metadata processing to the `process_ingredients()` function (after line 297):

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
            // This will work for custom fields like "com.adobe.repo.asset-id"
            assertion_metadata = assertion_metadata.set_field(key, value.clone());
        }

        ingredient.set_metadata(assertion_metadata);
        println!("  Set {} metadata field(s) on ingredient", metadata_map.len());
    }
}
```

#### Test Helper Function

**File:** `tests/common/mod.rs`

Added identical metadata processing logic to the `process_ingredients_with_thumbnails()` function (after line 239).

### 3. Documentation Updates

#### INGREDIENT_METADATA_ISSUE.md

- Updated to document the resolution
- Changed from "Issue Analysis" to "RESOLVED"
- Added verification steps and examples

#### README.md

- Updated "Ingredient Configuration" section
- Added "Ingredient Metadata Support" subsection
- Documented both standard and custom metadata fields
- Added comprehensive example showing metadata usage
- Explained the relationship between `label` and action parameters

## Usage

### JSON Manifest Example

```json
{
  "ingredients_from_files": [
    {
      "file_path": "source.jpg",
      "label": "my_ingredient",
      "title": "Source Image",
      "relationship": "componentOf",
      "metadata": {
        "com.example.asset-id": "asset-123",
        "com.example.version": "1.0",
        "com.example.data": {
          "nested": "object",
          "count": 42
        }
      }
    }
  ]
}
```

### Supported Metadata Types

**Standard C2PA fields** (optional):

- `dateTime`: ISO 8601 timestamp (auto-generated if not provided)
- `reviewRatings`: Array of review rating objects
- `dataSource`: Data source information
- `regionOfInterest`: Spatial/temporal regions
- `localizations`: Localized translations

**Custom fields:**

- Any key/value pairs using JSON types: string, number, boolean, object, array, null
- Best practice: use namespaced keys (e.g., `com.company.field-name`)

### Verification

The implementation was verified using the test suite:

```bash
cargo test --test integration_tests test_testset_manifests
```

Test case: `p-actions-placed-manifest-metadata.json`

**Output verification:**

```bash
cat target/test_output/testset/p-actions-placed-manifest-metadata_manifest_jpt.json | \
  jq '.manifests[] | select(.assertions."c2pa.ingredient.v3".instance_id == "test_ingredient") | \
  .assertions."c2pa.ingredient.v3".metadata'
```

**Result:**

```json
{
  "dateTime": "2026-01-23T20:52:22.733Z",
  "com.adobe.repo.version": "{hz-acp-version-id}",
  "com.adobe.repo.asset-id": "{hz-acp-asset-id }"
}
```

✅ All 53 integration tests pass.

## Technical Details

### Serialization Behavior

- Custom fields are serialized at the same level as standard fields using serde's `#[serde(flatten)]`
- Empty metadata objects are skipped during serialization (`skip_serializing_if = "HashMap::is_empty"`)
- The `dateTime` field is automatically added to AssertionMetadata unless explicitly provided
- All standard JSON types are fully supported through `serde_json::Value`

### C2PA Compliance

This implementation:

- ✅ Follows C2PA specification for assertion metadata extensibility
- ✅ Preserves all standard AssertionMetadata fields
- ✅ Allows spec-compliant arbitrary metadata
- ✅ Maintains proper CBOR serialization for C2PA manifests
- ✅ Supports round-trip serialization/deserialization

## Future Enhancements

Potential improvements for future consideration:

1. **Validation**: Add optional schema validation for custom metadata namespaces
2. **Metadata merging**: Support merging metadata from ingredient source files
3. **Templating**: Support variable substitution in metadata values (e.g., `${timestamp}`)
4. **Metadata inheritance**: Allow metadata to be inherited from parent ingredients

## Testing

All existing tests continue to pass with the new functionality. The specific test case for metadata (`p-actions-placed-manifest-metadata`) validates:

- Metadata parsing from JSON
- Metadata attachment to ingredients
- Metadata preservation through signing
- Metadata extraction and serialization to output manifest

## Migration Notes

**No breaking changes.** The `metadata` field is optional and backward compatible:

- Existing manifests without `metadata` work exactly as before
- Existing manifests with standard metadata fields work as before
- Only new: Custom/arbitrary metadata fields are now preserved instead of being ignored

## References

- C2PA Spec - Assertion Metadata: https://c2pa.org/specifications/specifications/2.2/specs/C2PA_Specification.html#_metadata_about_assertions
- Test Case: `testset/p-actions-placed-manifest-metadata.json`
- c2pa-rs PR: (to be added if submitted upstream)
