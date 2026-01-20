# Ingredient Label Support Implementation

## Summary

Extended `ingredients_from_files` to support an optional `label` field that allows ingredients loaded from external files to be referenced by their label in actions (via `ingredientIds`).

## Problem

The test file `p-actions-opened-manifest.json` was failing because it used `ingredient_paths` (which wasn't supported) to load an ingredient file and reference it in actions by filename. The c2pa library validates that `ingredientIds` referenced in actions correspond to actual ingredients with matching instance IDs.

## Solution

Added support for an optional `label` field in `ingredients_from_files` entries. When specified, this label is set as the ingredient's `instance_id`, allowing it to be referenced in actions.

## Implementation Details

### Changes to `src/main.rs`

Modified the `process_ingredients()` function to check for and apply the `label` field:

```rust
// Set the label (instance_id) if provided
// This allows the ingredient to be referenced in actions by this label
if let Some(label) = ingredient_def.get("label").and_then(|v| v.as_str()) {
    ingredient.set_instance_id(label);
}
```

### Changes to `tests/common/mod.rs`

1. Updated `sign_file_with_manifest()` to automatically process `ingredients_from_files` from the manifest
2. Updated `process_ingredients_with_thumbnails()` to support the `label` field

### Changes to Manifest Files

Updated `testset/p-actions-opened-manifest.json` to use `ingredients_from_files` with label:

```json
{
  "assertions": [
    {
      "label": "c2pa.actions",
      "data": {
        "actions": [
          {
            "action": "c2pa.opened",
            "parameters": {
              "ingredientIds": ["test_ingredient.png"]
            }
          }
        ]
      }
    }
  ],
  "ingredients_from_files": [
    {
      "file_path": "test_ingredient.png",
      "label": "test_ingredient.png"
    }
  ]
}
```

## Manifest Format

The `ingredients_from_files` array now supports an optional `label` field:

```json
{
  "ingredients_from_files": [
    {
      "file_path": "path/to/ingredient.jpg",
      "title": "My Ingredient",
      "relationship": "parentOf",
      "label": "unique-ingredient-id"
    }
  ]
}
```

### Field Descriptions

- `file_path` (required): Path to the ingredient file, relative to the manifest directory
- `title` (optional): Human-readable title for the ingredient. Defaults to filename if not specified
- `relationship` (optional): Either "parentOf" or "componentOf". Describes the relationship to the asset
- `label` (optional): **NEW** - Unique identifier that can be referenced in actions via `ingredientIds`

## Usage Example

To create an ingredient that can be referenced in actions:

1. Add the ingredient to `ingredients_from_files` with a label:
```json
{
  "ingredients_from_files": [
    {
      "file_path": "background.jpg",
      "label": "bg-layer"
    }
  ]
}
```

2. Reference it in actions using the label:
```json
{
  "assertions": [
    {
      "label": "c2pa.actions",
      "data": {
        "actions": [
          {
            "action": "c2pa.placed",
            "parameters": {
              "ingredientIds": ["bg-layer"]
            }
          }
        ]
      }
    }
  ]
}
```

## Benefits

1. **Consistency**: Uses the existing `ingredients_from_files` structure
2. **Flexibility**: Label field is optional - only needed when referencing ingredients in actions
3. **Clarity**: The `label` field name matches the inline `ingredients` format used in non-file-based approaches
4. **Compatibility**: Existing manifests without labels continue to work

## Test Results

All 69 tests pass, including:
- 53 integration tests
- 16 JPT extraction tests

The `p-actions-opened-manifest` test now successfully:
1. Loads the ingredient from `test_ingredient.png`
2. Sets its instance_id to "test_ingredient.png"
3. Validates that the ingredientId in the action matches the ingredient
4. Signs and embeds the manifest successfully

## Files Modified

1. `src/main.rs` - Added label support in `process_ingredients()`
2. `tests/common/mod.rs` - Added label support in test helpers
3. `testset/p-actions-opened-manifest.json` - Converted to use `ingredients_from_files` with label
4. `tests/integration_tests.rs` - Uncommented the test for `p-actions-opened-manifest`
