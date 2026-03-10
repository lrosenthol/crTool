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
//! Core library for extracting and validating C2PA manifests in crJSON format.

use anyhow::{Context, Result};
use c2pa::{Context as C2paContext, Reader};

/// Re-export so callers (e.g. GUI, CLI) can use explicit Settings without depending on c2pa.
pub use c2pa::Settings;

/// File extensions for asset types supported by c2pa-rs for reading/embedding C2PA manifests.
/// Matches the formats listed in c2pa-rs [supported-formats](https://github.com/contentauth/c2pa-rs/blob/main/docs/supported-formats.md).
pub const SUPPORTED_ASSET_EXTENSIONS: &[&str] = &[
    "avi", "avif", "c2pa", "dng", "gif", "heic", "heif", "jpg", "jpeg", "m4a", "mov", "mp3", "mp4",
    "pdf", "png", "svg", "tif", "tiff", "wav", "webp",
];

/// Returns whether a file path has an extension that c2pa-rs supports for C2PA operations.
pub fn is_supported_asset_path<P: AsRef<Path>>(path: P) -> bool {
    let ext = match path.as_ref().extension().and_then(|e| e.to_str()) {
        Some(e) => e.to_lowercase(),
        None => return false,
    };
    SUPPORTED_ASSET_EXTENSIONS.contains(&ext.as_str())
}
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Builds a `validationResults` value that conforms to the crJSON schema: `activeManifest`
/// (required) with `success`, `informational`, `failure` arrays; optional `ingredientDeltas`.
fn validation_results_to_schema_shape(input: &serde_json::Value) -> serde_json::Value {
    let empty_status = serde_json::json!({
        "success": [],
        "informational": [],
        "failure": []
    });

    let active_manifest = if let Some(obj) = input.as_object() {
        if let Some(am) = obj.get("activeManifest").and_then(|v| v.as_object()) {
            let success = am
                .get("success")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let informational = am
                .get("informational")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let failure = am
                .get("failure")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "success": success,
                "informational": informational,
                "failure": failure
            })
        } else if obj.get("isValid").is_some() || obj.get("error").is_some() {
            // Legacy validationStatus: { isValid, error?, code?, explanation?, uri? }
            let code = obj
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("validation.legacy");
            let explanation = obj.get("explanation").and_then(|v| v.as_str());
            let url = obj.get("uri").and_then(|v| v.as_str());
            let entry = serde_json::json!({
                "code": code,
                "url": url,
                "explanation": explanation
            });
            let (success, failure) = match obj.get("isValid").and_then(|v| v.as_bool()) {
                Some(true) => (vec![entry], vec![]),
                Some(false) => (vec![], vec![entry]),
                None => (vec![], vec![entry]),
            };
            serde_json::json!({
                "success": success,
                "informational": [],
                "failure": failure
            })
        } else {
            empty_status
        }
    } else {
        empty_status.clone()
    };

    let ingredient_deltas = input
        .get("ingredientDeltas")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    serde_json::json!({
        "activeManifest": active_manifest,
        "ingredientDeltas": ingredient_deltas
    })
}

/// Normalizes crJSON so validation data is under `validationResults` in the shape required by
/// the crJSON schema. Only legacy `extras:validation_status` is moved and converted;
/// if the document already has `validationResults` (e.g. from c2pa-rs), it is left unchanged.
/// Idempotent when already normalized or when c2pa-rs already emitted validationResults.
pub fn normalize_crjson_validation_results(value: &mut serde_json::Value) {
    let obj = match value.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    if let Some(legacy) = obj.remove("extras:validation_status") {
        let conformant = validation_results_to_schema_shape(&legacy);
        obj.insert("validationResults".to_string(), conformant);
    }
}

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

/// Extracts a C2PA manifest in crJSON format using the given Settings (e.g. trust configuration).
/// Use this when you have explicit settings so that trust validation uses the same configuration
/// regardless of thread (avoids thread-local timing/threading issues).
pub fn extract_crjson_manifest_with_settings<P: AsRef<Path>>(
    input_path: P,
    settings: &Settings,
) -> Result<ManifestExtractionResult> {
    let input_path = input_path.as_ref();

    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {:?}", input_path);
    }

    let context = C2paContext::new()
        .with_settings(settings)
        .map_err(|e| anyhow::anyhow!("Invalid settings: {}", e))?;
    let reader = Reader::from_context(context)
        .with_file(input_path)
        .context(
            "Failed to read C2PA data from input file. The file may not contain a C2PA manifest.",
        )?;

    let active_label = reader
        .active_label()
        .context("No active C2PA manifest found in the input file")?
        .to_string();

    let manifest_json = reader.crjson();

    let mut manifest_value: serde_json::Value =
        serde_json::from_str(&manifest_json).context("Failed to parse extracted crJSON")?;

    normalize_crjson_validation_results(&mut manifest_value);

    let manifest_json = serde_json::to_string_pretty(&manifest_value)
        .context("Failed to re-serialize crJSON after normalization")?;

    Ok(ManifestExtractionResult {
        input_path: input_path.to_string_lossy().to_string(),
        active_label,
        asset_hash: None,
        manifest_json,
        manifest_value,
    })
}

/// Extract a C2PA manifest from a file in crJSON format using the c2pa-rs Reader.
///
/// Uses **thread-local** Settings. If you have applied trust via [`apply_trust_settings`],
/// ensure extraction runs on the same thread. Prefer [`extract_crjson_manifest_with_settings`]
/// when you have explicit settings so trust is applied consistently.
///
/// # Arguments
///
/// * `input_path` - Path to the input file containing a C2PA manifest
///
/// # Returns
///
/// A `ManifestExtractionResult` containing the extracted manifest data in crJSON format.
/// `asset_hash` is not computed and will be `None`.
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist
/// - The file does not contain a valid C2PA manifest
/// - The manifest cannot be parsed to crJSON
pub fn extract_crjson_manifest<P: AsRef<Path>>(input_path: P) -> Result<ManifestExtractionResult> {
    let input_path = input_path.as_ref();

    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {:?}", input_path);
    }

    let reader = Reader::from_file(input_path).context(
        "Failed to read C2PA data from input file. The file may not contain a C2PA manifest.",
    )?;

    let active_label = reader
        .active_label()
        .context("No active C2PA manifest found in the input file")?
        .to_string();

    let manifest_json = reader.crjson();

    let mut manifest_value: serde_json::Value =
        serde_json::from_str(&manifest_json).context("Failed to parse extracted crJSON")?;

    normalize_crjson_validation_results(&mut manifest_value);

    let manifest_json = serde_json::to_string_pretty(&manifest_value)
        .context("Failed to re-serialize crJSON after normalization")?;

    Ok(ManifestExtractionResult {
        input_path: input_path.to_string_lossy().to_string(),
        active_label,
        asset_hash: None,
        manifest_json,
        manifest_value,
    })
}

/// Validate a JSON value against a JSON schema.
///
/// # Arguments
///
/// * `json_value` - The JSON value to validate
/// * `schema_path` - Path to the schema JSON file
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

/// Validate a JSON file against a JSON schema.
///
/// # Arguments
///
/// * `json_file_path` - Path to the JSON file to validate
/// * `schema_path` - Path to the schema JSON file
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

/// Get the crJSON schema path relative to the crate root
///
/// Use this when validating crJSON documents (e.g. output of `--extract`).
pub fn crjson_schema_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("INTERNAL")
        .join("schemas")
        .join("crJSON-schema.json")
}

/// Trust list URLs: official C2PA trust list and Content Credentials interim list.
/// See <https://opensource.contentauthenticity.org/docs/c2patool/docs/usage/#configuring-trust-support>.
pub const C2PA_TRUST_ANCHORS_URL: &str =
    "https://raw.githubusercontent.com/c2pa-org/conformance-public/refs/heads/main/trust-list/C2PA-TRUST-LIST.pem";
pub const INTERIM_TRUST_ANCHORS_URL: &str = "https://contentcredentials.org/trust/anchors.pem";
pub const INTERIM_ALLOWED_LIST_URL: &str =
    "https://contentcredentials.org/trust/allowed.sha256.txt";
pub const INTERIM_TRUST_CONFIG_URL: &str = "https://contentcredentials.org/trust/store.cfg";

fn trust_settings_toml(
    trust_anchors: &str,
    allowed_list: Option<&str>,
    trust_config: Option<&str>,
) -> String {
    fn escape_toml_literal(s: &str) -> String {
        s.replace('\'', "''")
    }
    let mut toml = format!(
        "[trust]\ntrust_anchors = '''{}'''\n",
        escape_toml_literal(trust_anchors)
    );
    if let Some(al) = allowed_list {
        toml.push_str(&format!(
            "allowed_list = '''{}'''\n",
            escape_toml_literal(al)
        ));
    }
    if let Some(tc) = trust_config {
        toml.push_str(&format!(
            "trust_config = '''{}'''\n",
            escape_toml_literal(tc)
        ));
    }
    toml.push_str("\n[verify]\nverify_trust = true\n");
    toml
}

/// Builds Settings with trust validation enabled (trust anchors, optional allowed list and trust config).
/// Does not set thread-local; use the returned Settings with [`extract_crjson_manifest_with_settings`].
pub fn build_trust_settings(
    trust_anchors: &str,
    allowed_list: Option<&str>,
    trust_config: Option<&str>,
) -> Result<Settings> {
    let toml = trust_settings_toml(trust_anchors, allowed_list, trust_config);
    Settings::default()
        .with_toml(&toml)
        .map_err(|e| anyhow::anyhow!("Failed to build trust settings: {}", e))
}

/// Returns default Settings for extraction when trust lists are not used.
/// Trust verification remains enabled so that claimSignature shows a trust status (trusted or untrusted);
/// without trust anchors, certificates will report as untrusted.
pub fn default_extraction_settings() -> Settings {
    Settings::default()
}

/// Applies C2PA trust settings to the thread-local Settings used by Reader.
/// Call this before extracting or reading manifests to validate signing certificates against the
/// given trust anchors, optional allowed list, and optional trust config (EKU OIDs).
///
/// * `trust_anchors` - PEM bundle of trust anchor root certificates (required).
/// * `allowed_list` - Optional PEM bundle or SHA256 hash list of explicitly allowed signing certificates.
/// * `trust_config` - Optional list of allowed EKU OIDs in dot notation.
///
/// Also enables `verify.verify_trust` so that the SDK actually performs trust validation.
///
/// Prefer building settings with [`build_trust_settings`] and using [`extract_crjson_manifest_with_settings`]
/// so the same settings are used regardless of thread.
pub fn apply_trust_settings(
    trust_anchors: &str,
    allowed_list: Option<&str>,
    trust_config: Option<&str>,
) -> Result<()> {
    let toml = trust_settings_toml(trust_anchors, allowed_list, trust_config);
    Settings::from_toml(&toml)
        .map_err(|e| anyhow::anyhow!("Failed to apply trust settings: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crjson_schema_path_exists() {
        let schema_path = crjson_schema_path();
        assert!(
            schema_path.exists(),
            "crJSON schema path should exist: {:?}",
            schema_path
        );
    }
}
