# JPEG Trust Format Extraction Feature

## Summary

Added support for extracting C2PA manifests in JPEG Trust format using the `--jpt` flag with the `--extract` option.

## Changes Made

### Main Application (`src/main.rs`)

1. **Added `--jpt` flag**:
   - New command-line option that enables JPEG Trust format extraction
   - Only valid when used with `--extract` mode
   - Returns error if used in signing mode

2. **Updated `extract_manifest` function**:
   - Now accepts a `use_jpt_format` boolean parameter
   - Uses `JpegTrustReader` when JPT format is requested
   - Uses standard `Reader` for normal extraction
   - Automatically computes asset hash for JPT format
   - Different filename suffix for JPT format: `_manifest_jpt.json` vs `_manifest.json`

3. **Import changes**:
   - Added `JpegTrustReader` to imports from `c2pa` crate

### Test Support (`tests/common/mod.rs`)

1. **Added `extract_manifest_to_file_jpt` helper function**:
   - Parallel to existing `extract_manifest_to_file` function
   - Extracts in JPEG Trust format for use in tests

2. **Refactored extraction logic**:
   - Created internal `extract_manifest_impl` function
   - Shared logic between normal and JPT extraction
   - Properly handles asset hash computation for JPT format

### Comprehensive Tests (`tests/test_jpt_extraction.rs`)

Created a new test file with 16 comprehensive tests covering:

#### Basic Extraction Tests
- `test_extract_normal_format`: Verifies standard extraction still works
- `test_extract_jpt_format`: Verifies JPEG Trust format extraction
- `test_jpt_without_extract_fails`: Ensures `--jpt` is rejected without `--extract`

#### Multiple File Tests
- `test_extract_multiple_files_normal_format`: Multiple files in standard format
- `test_extract_multiple_files_jpt_format`: Multiple files in JPT format

#### Format Comparison Tests
- `test_format_differences`: Verifies key structural differences between formats
  - JPT has `@context` field, standard doesn't
  - JPT uses `manifests` array, standard uses object
  - Both contain complete manifest data

#### Error Handling Tests
- `test_extract_file_without_manifest`: Unsigned file error handling
- `test_extract_nonexistent_file`: Non-existent file error handling

#### Programmatic API Tests
- `test_helper_extract_normal_format`: Helper function for standard format
- `test_helper_extract_jpt_format`: Helper function for JPT format

#### Integration Tests
- `test_jpt_with_complex_manifest`: Complex manifests with actions v2
- `test_jpt_output_to_directory`: Directory output with auto-generated filenames

### Documentation Updates (`README.md`)

1. **Added `--jpt` flag documentation**:
   - Describes JPEG Trust format option
   - Lists key differences from standard format
   - Includes usage examples

2. **Updated extraction examples**:
   - Added examples for both standard and JPT format extraction
   - Single file extraction
   - Multiple file extraction
   - Directory output with both formats

3. **Added JPEG Trust format section**:
   - Explains format features
   - Lists key structural differences
   - Notes compatibility with JPEG Trust consumers

## Key Features of JPEG Trust Format

### Format Differences

**JPEG Trust Format**:
- `@context` field with JPEG Trust vocabulary (`https://jpeg.org/jpegtrust`)
- `asset_info` object with computed SHA-256 hash
- `manifests` as an array (not object)
- Different validation status structure (`extras:validation_status`)
- Compatible with JPEG Trust specification

**Standard Format**:
- No `@context` field
- `manifests` as an object with labels as keys
- Standard c2pa-rs validation structure
- Compatible with c2patool and c2pa-rs consumers

### Usage Examples

```bash
# Extract in standard format
c2pa-testfile-maker -e signed.jpg -o manifest.json

# Extract in JPEG Trust format
c2pa-testfile-maker -e --jpt signed.jpg -o manifest_jpt.json

# Extract multiple files in JPEG Trust format to directory
c2pa-testfile-maker -e --jpt signed1.jpg signed2.jpg -o output_dir/
```

## Test Results

All 70 tests pass:
- 2 unit tests (main.rs)
- 52 integration tests (integration_tests.rs)
- 16 JPT extraction tests (test_jpt_extraction.rs)

### JPT Test Coverage

✅ Basic extraction in both formats
✅ Multiple file extraction
✅ Format differences verification
✅ Error handling (unsigned files, non-existent files)
✅ Helper function testing
✅ Complex manifest support
✅ Directory output with auto-generated filenames
✅ `--jpt` flag validation (only with `--extract`)

## Technical Implementation

### JpegTrustReader Usage

The implementation leverages the `JpegTrustReader` from c2pa-rs SDK:

```rust
let mut jpt_reader = JpegTrustReader::from_file(input_path)?;

// Compute asset hash
jpt_reader.compute_asset_hash_from_file(input_path)?;

// Get JSON in JPEG Trust format
let manifest_json = jpt_reader.json();
```

### Asset Hash Computation

- Automatically computes SHA-256 hash of the asset
- Included in `asset_info` field
- Displayed in console output during extraction
- Base64-encoded in output JSON

## Backward Compatibility

- All existing functionality preserved
- Default behavior unchanged (standard format)
- `--jpt` is opt-in only
- All existing tests continue to pass

## Future Considerations

1. Could add option to skip asset hash computation for faster extraction
2. Could support other JPEG Trust format features as they become available
3. Could add validation against JPEG Trust schema
