/*
Copyright 2025 Adobe. All rights reserved.
This file is licensed to you under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License. You may obtain a copy
of the License at http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under
the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR REPRESENTATIONS
OF ANY KIND, either express or implied. See the License for the specific language
governing permissions and limitations under the License.
*/

//! # crTool Library
//!
//! Core library for extracting and validating C2PA manifests in JPEG Trust format.

use anyhow::{Context, Result};
use c2pa::JpegTrustReader;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Result of extracting a manifest from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestExtractionResult {
    /// The input file path that was processed
    pub input_path: String,
    /// The active manifest label
    pub active_label: String,
    /// The computed asset hash (SHA-256)
    pub asset_hash: Option<String>,
    /// The extracted manifest as a JSON string
    pub manifest_json: String,
    /// Parsed manifest as serde_json::Value for easier processing
    pub manifest_value: serde_json::Value,
}

/// Result of validating a JSON file against the indicators schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// The file path that was validated
    pub file_path: String,
    /// Whether the file passed validation
    pub is_valid: bool,
    /// Validation error messages (empty if valid)
    pub errors: Vec<ValidationError>,
}

/// A single validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// The JSON path where the error occurred
    pub instance_path: String,
    /// The error message
    pub message: String,
}

/// Extract a C2PA manifest from a file in JPEG Trust format
///
/// # Arguments
///
/// * `input_path` - Path to the input file containing a C2PA manifest
///
/// # Returns
///
/// A `ManifestExtractionResult` containing the extracted manifest data
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist
/// - The file does not contain a valid C2PA manifest
/// - The manifest cannot be parsed
pub fn extract_jpt_manifest<P: AsRef<Path>>(input_path: P) -> Result<ManifestExtractionResult> {
    let input_path = input_path.as_ref();

    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {:?}", input_path);
    }

    // Use JPEG Trust Reader
    let mut jpt_reader = JpegTrustReader::from_file(input_path).context(
        "Failed to read C2PA data from input file. The file may not contain a C2PA manifest.",
    )?;

    // Compute asset hash
    let asset_hash = jpt_reader
        .compute_asset_hash_from_file(input_path)
        .ok();

    // Get the active manifest label
    let active_label = jpt_reader
        .inner()
        .active_label()
        .context("No active C2PA manifest found in the input file")?
        .to_string();

    // Get the manifest JSON
    let manifest_json = jpt_reader.json();

    // Parse to serde_json::Value
    let manifest_value: serde_json::Value = serde_json::from_str(&manifest_json)
        .context("Failed to parse extracted manifest JSON")?;

    Ok(ManifestExtractionResult {
        input_path: input_path.to_string_lossy().to_string(),
        active_label,
        asset_hash,
        manifest_json,
        manifest_value,
    })
}

/// Validate a JSON value against the JPEG Trust indicators schema
///
/// # Arguments
///
/// * `json_value` - The JSON value to validate
/// * `schema_path` - Path to the indicators schema JSON file
///
/// # Returns
///
/// A `ValidationResult` containing validation status and any errors
pub fn validate_json_value(
    json_value: &serde_json::Value,
    schema_path: &Path,
) -> Result<ValidationResult> {
    if !schema_path.exists() {
        anyhow::bail!("Schema file not found at: {:?}", schema_path);
    }

    let schema_content =
        fs::read_to_string(schema_path).context("Failed to read indicators schema file")?;

    let schema_json: serde_json::Value =
        serde_json::from_str(&schema_content).context("Failed to parse indicators schema JSON")?;

    // Compile the schema
    let compiled_schema = jsonschema::validator_for(&schema_json)
        .map_err(|e| anyhow::anyhow!("Failed to compile JSON schema: {}", e))?;

    // Validate
    let validation_result = compiled_schema.validate(json_value);

    let mut errors = Vec::new();
    let is_valid = match validation_result {
        Ok(_) => true,
        Err(validation_errors) => {
            for error in validation_errors {
                let instance_path = if error.instance_path.to_string().is_empty() {
                    "root".to_string()
                } else {
                    error.instance_path.to_string()
                };
                errors.push(ValidationError {
                    instance_path,
                    message: error.to_string(),
                });
            }
            false
        }
    };

    Ok(ValidationResult {
        file_path: String::new(), // Filled in by caller if needed
        is_valid,
        errors,
    })
}

/// Validate a JSON file against the JPEG Trust indicators schema
///
/// # Arguments
///
/// * `json_file_path` - Path to the JSON file to validate
/// * `schema_path` - Path to the indicators schema JSON file
///
/// # Returns
///
/// A `ValidationResult` containing validation status and any errors
pub fn validate_json_file<P: AsRef<Path>>(
    json_file_path: P,
    schema_path: &Path,
) -> Result<ValidationResult> {
    let json_file_path = json_file_path.as_ref();

    let json_content = fs::read_to_string(json_file_path)
        .context(format!("Failed to read file: {:?}", json_file_path))?;

    let json_value: serde_json::Value = serde_json::from_str(&json_content)
        .context(format!("Invalid JSON in file: {:?}", json_file_path))?;

    let mut result = validate_json_value(&json_value, schema_path)?;
    result.file_path = json_file_path.to_string_lossy().to_string();

    Ok(result)
}

/// Get the default schema path relative to the crate root
///
/// This returns the path to the bundled indicators schema.
pub fn default_schema_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("INTERNAL")
        .join("schemas")
        .join("indicators-schema.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schema_path_exists() {
        let schema_path = default_schema_path();
        assert!(
            schema_path.exists(),
            "Default schema path should exist: {:?}",
            schema_path
        );
    }

    #[test]
    fn test_validate_json_value_with_valid_data() {
        let schema_path = default_schema_path();
        if !schema_path.exists() {
            return; // Skip test if schema not found
        }

        // Load a valid test fixture
        let valid_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("valid_indicators.json");

        if valid_fixture.exists() {
            let json_content = fs::read_to_string(&valid_fixture).unwrap();
            let json_value: serde_json::Value = serde_json::from_str(&json_content).unwrap();

            let result = validate_json_value(&json_value, &schema_path).unwrap();
            assert!(result.is_valid, "Valid fixture should pass validation");
            assert!(result.errors.is_empty());
        }
    }

    #[test]
    fn test_validate_json_value_with_invalid_data() {
        let schema_path = default_schema_path();
        if !schema_path.exists() {
            return; // Skip test if schema not found
        }

        // Load an invalid test fixture
        let invalid_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("invalid_indicators.json");

        if invalid_fixture.exists() {
            let json_content = fs::read_to_string(&invalid_fixture).unwrap();
            let json_value: serde_json::Value = serde_json::from_str(&json_content).unwrap();

            let result = validate_json_value(&json_value, &schema_path).unwrap();
            assert!(!result.is_valid, "Invalid fixture should fail validation");
            assert!(!result.errors.is_empty());
        }
    }
}
