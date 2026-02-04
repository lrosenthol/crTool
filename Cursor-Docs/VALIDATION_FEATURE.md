# JSON Schema Validation Feature

## Overview

Added a new command-line option (`-v, --validate`) that validates JSON files against the JPEG Trust indicators schema located at `INTERNAL/schemas/indicators-schema.json`.

## Changes Made

### 1. Dependencies (Cargo.toml)
- Added `jsonschema = "0.23"` dependency for JSON Schema validation

### 2. Command-Line Interface (src/main.rs)
- Added `-v, --validate` flag to the CLI argument parser
- The flag enables validation mode where input files are validated against the schema
- Made `--output` option optional (it's not required for validation mode)
- Updated error messages to clarify when `--output` is required

### 3. Validation Implementation (src/main.rs)
- New function: `validate_json_files(input_paths: &[PathBuf]) -> Result<()>`
  - Loads the indicators schema from `INTERNAL/schemas/indicators-schema.json`
  - Compiles the schema using the `jsonschema` crate
  - Validates each input file against the schema
  - Provides detailed error messages with:
    - Path in the JSON where errors occur
    - Description of what is invalid
  - Returns summary statistics (total files, valid, invalid)
  - Returns exit code 0 for success, non-zero for validation failures

### 4. Main Function Integration
- Added validation mode check early in the main function
- When `--validate` is specified, the tool validates the input files and exits
- Validation mode doesn't require manifest, certificate, or key files

### 5. Unit Tests (src/main.rs)
Added unit tests:
- `test_validate_json_files_with_valid_manifest` - Tests with a valid manifest
- `test_validate_json_files_with_invalid_json` - Tests with malformed JSON
- `test_validate_json_files_with_nonexistent_file` - Tests error handling for missing files

### 6. Integration Tests (tests/test_validation.rs)
New test file with comprehensive integration tests:
- `test_validation_with_valid_indicators` - Single valid file
- `test_validation_with_minimal_valid_indicators` - Minimal valid file
- `test_validation_with_invalid_indicators` - File with schema violations
- `test_validation_with_malformed_json` - Syntactically invalid JSON
- `test_validation_with_multiple_files` - Multiple valid files
- `test_validation_with_mixed_valid_invalid_files` - Mix of valid and invalid files
- `test_validation_with_nonexistent_file` - Missing file error handling
- `test_validation_extracts_from_signed_file` - Validation of extracted manifests

### 7. Test Fixtures (tests/fixtures/)
Created test JSON files:
- `valid_indicators.json` - Complete valid indicators document
- `minimal_valid_indicators.json` - Minimal valid document (just `@context`)
- `invalid_indicators.json` - Document with schema violations

### 8. Documentation (README.md)
Updated documentation:
- Added validation feature to the Features list
- Added `-v, --validate` flag documentation in Options section
- Added new "Validating JSON Files" section with:
  - Usage examples
  - Description of validation behavior
  - Example output for valid and invalid files
- Updated Quick Start Examples with validation examples
- Added `jsonschema` to Dependencies section

## Usage

```bash
# Validate a single file (no --output required)
./target/release/crTool --validate manifest.json

# Validate multiple files
./target/release/crTool --validate file1.json file2.json

# Validate with glob patterns
./target/release/crTool --validate "manifests/*.json"
```

## Example Output

### Valid File
```
=== Validating JSON files against indicators schema ===

Loading schema from: "INTERNAL/schemas/indicators-schema.json"

Schema compiled successfully

Validating: "manifest.json"
  ✓ Valid

=== Validation Summary ===
  Total files: 1
  Valid: 1
  Invalid: 0

✓ All files are valid!
```

### Invalid File
```
=== Validating JSON files against indicators schema ===

Loading schema from: "INTERNAL/schemas/indicators-schema.json"

Schema compiled successfully

Validating: "invalid.json"
  ✗ Validation failed:
    - At /asset_info: "hash" is a required property
    - At /manifests/0/claim.v2/version: "string" is not of types "integer", "null"

=== Validation Summary ===
  Total files: 1
  Valid: 0
  Invalid: 1

=== Files with Validation Errors ===

"invalid.json":
    - At /asset_info: "hash" is a required property
    - At /manifests/0/claim.v2/version: "string" is not of types "integer", "null"
Error: 1 file(s) failed validation
```

## Testing

All tests pass:
- 5 unit tests in `src/main.rs`
- 12 integration tests in `tests/test_validation.rs`
- All existing tests continue to pass (53 integration tests, 16 JPT extraction tests)

Total: 86 tests passing

## Exit Codes

- `0` - All files are valid
- Non-zero - One or more files failed validation or an error occurred
