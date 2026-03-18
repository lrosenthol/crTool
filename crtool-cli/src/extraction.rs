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

use anyhow::{Context, Result};
use c2pa::Settings;
use crtool::{
    build_trust_settings, extract_crjson_manifest_with_settings, C2PA_TRUST_ANCHORS_URL,
    INTERIM_ALLOWED_LIST_URL, INTERIM_TRUST_ANCHORS_URL, INTERIM_TRUST_CONFIG_URL,
};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};

/// Fetch a URL and return the response body as a string.
fn fetch_url(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("crTool/1.0")
        .build()
        .context("Failed to create HTTP client")?;
    let response = client
        .get(url)
        .send()
        .context(format!("Failed to fetch {}", url))?;
    let status = response.status();
    let body = response
        .text()
        .context(format!("Failed to read response body from {}", url))?;
    if !status.is_success() {
        anyhow::bail!("{} returned {}: {}", url, status, body);
    }
    Ok(body)
}

/// Build `Settings` for extraction.
/// When `with_trust` is true, fetches and applies the C2PA and Content Credentials trust lists.
/// Otherwise, trust verification is disabled so certificates are not reported as untrusted.
pub fn extraction_settings(with_trust: bool) -> Result<Settings> {
    if with_trust {
        println!("Loading C2PA and Content Credentials trust lists...");
        let c2pa_anchors = fetch_url(C2PA_TRUST_ANCHORS_URL)
            .context("Failed to fetch official C2PA trust list")?;
        let interim_anchors = fetch_url(INTERIM_TRUST_ANCHORS_URL)
            .context("Failed to fetch interim trust anchors")?;
        let trust_anchors = format!(
            "{}\n{}",
            c2pa_anchors.trim_end(),
            interim_anchors.trim_end()
        );
        let allowed_list =
            fetch_url(INTERIM_ALLOWED_LIST_URL).context("Failed to fetch interim allowed list")?;
        let trust_config =
            fetch_url(INTERIM_TRUST_CONFIG_URL).context("Failed to fetch interim trust config")?;
        println!("  Trust list validation enabled");
        build_trust_settings(
            &trust_anchors,
            Some(allowed_list.trim()),
            Some(trust_config.trim()),
        )
    } else {
        Ok(crtool::default_extraction_settings())
    }
}

/// Extract a C2PA manifest from `input_path` and write it as crJSON to `output_path`.
/// Returns the path of the written crJSON file.
pub fn extract_manifest(
    input_path: &Path,
    output_path: &Path,
    settings: &Settings,
) -> Result<PathBuf> {
    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {:?}", input_path);
    }

    println!("Extracting C2PA manifest (crJSON)...");
    println!("  Input: {:?}", input_path);

    let extract_result = extract_crjson_manifest_with_settings(input_path, settings).context(
        "Failed to read C2PA data from input file. The file may not contain a C2PA manifest.",
    )?;

    let active_label = &extract_result.active_label;
    println!("  Active manifest label: {}", active_label);

    let mut json_value: JsonValue = extract_result.manifest_value;
    if !json_value.get("@context").is_some() {
        if let Some(obj) = json_value.as_object_mut() {
            obj.insert(
                "@context".to_string(),
                serde_json::json!(["https://contentcredentials.org/crjson/context/v1"]),
            );
        }
    }

    const SUFFIX: &str = "_cr.json";

    let final_output_path = if output_path.is_dir() {
        let input_stem = input_path
            .file_stem()
            .context("Input file has no filename")?
            .to_str()
            .context("Invalid UTF-8 in filename")?;
        output_path.join(format!("{}{}", input_stem, SUFFIX))
    } else {
        output_path.to_path_buf()
    };

    if let Some(parent) = final_output_path.parent() {
        fs::create_dir_all(parent).context("Failed to create output directory")?;
    }

    let pretty_json = serde_json::to_string_pretty(&json_value).context("Failed to format JSON")?;
    fs::write(&final_output_path, pretty_json)
        .context("Failed to write manifest JSON to output file")?;

    println!("✓ Successfully extracted C2PA manifest");
    println!("  Output file: {:?}", final_output_path);

    Ok(final_output_path)
}

/// Validate one or more JSON files against the crJSON schema.
pub fn validate_json_files(
    input_paths: &[PathBuf],
    schema_path: &Path,
    schema_label: &str,
) -> Result<()> {
    println!(
        "=== Validating JSON files against {} schema ===\n",
        schema_label
    );

    if !schema_path.exists() {
        anyhow::bail!("Schema file not found at: {:?}", schema_path);
    }

    println!("Loading schema from: {:?}\n", schema_path);
    let schema_content = fs::read_to_string(schema_path).context("Failed to read schema file")?;

    let schema_json: JsonValue =
        serde_json::from_str(&schema_content).context("Failed to parse schema JSON")?;

    let compiled_schema = jsonschema::validator_for(&schema_json)
        .map_err(|e| anyhow::anyhow!("Failed to compile JSON schema: {}", e))?;

    println!("Schema compiled successfully\n");

    let mut total_files = 0;
    let mut valid_files = 0;
    let mut invalid_files = 0;
    let mut error_details = Vec::new();

    for input_path in input_paths {
        total_files += 1;
        println!("Validating: {:?}", input_path);

        let json_content = match fs::read_to_string(input_path) {
            Ok(content) => content,
            Err(e) => {
                println!("  ✗ ERROR: Failed to read file: {}\n", e);
                invalid_files += 1;
                error_details.push((input_path.clone(), format!("Failed to read file: {}", e)));
                continue;
            }
        };

        let json_value: JsonValue = match serde_json::from_str(&json_content) {
            Ok(value) => value,
            Err(e) => {
                println!("  ✗ ERROR: Invalid JSON: {}\n", e);
                invalid_files += 1;
                error_details.push((input_path.clone(), format!("Invalid JSON: {}", e)));
                continue;
            }
        };

        let validation_result = compiled_schema.validate(&json_value);
        match validation_result {
            Ok(_) => {
                println!("  ✓ Valid\n");
                valid_files += 1;
            }
            Err(errors) => {
                println!("  ✗ Validation failed:");
                let mut error_messages = Vec::new();
                for error in errors {
                    let instance_path = if error.instance_path.to_string().is_empty() {
                        "root".to_string()
                    } else {
                        error.instance_path.to_string()
                    };
                    let message = format!("    - At {}: {}", instance_path, error);
                    println!("{}", message);
                    error_messages.push(message);
                }
                println!();
                invalid_files += 1;
                error_details.push((input_path.clone(), error_messages.join("\n")));
            }
        }
    }

    println!("=== Validation Summary ===");
    println!("  Total files: {}", total_files);
    println!("  Valid: {}", valid_files);
    println!("  Invalid: {}", invalid_files);

    if invalid_files > 0 {
        println!("\n=== Files with Validation Errors ===");
        for (path, error) in error_details {
            println!("\n{:?}:", path);
            println!("{}", error);
        }
        anyhow::bail!("{} file(s) failed validation", invalid_files);
    } else {
        println!("\n✓ All files are valid!");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_validate_json_files_with_valid_manifest() {
        // A C2PA manifest template is not crJSON — validation against crJSON schema should fail
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples")
            .join("simple_manifest.json");

        if manifest_path.exists() {
            let schema_path = crtool::crjson_schema_path();
            let result = validate_json_files(&[manifest_path.clone()], &schema_path, "crJSON");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_validate_json_files_with_invalid_json() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_invalid.json");

        let mut file = fs::File::create(&temp_file).expect("Failed to create temp file");
        writeln!(file, "{{ invalid json }}").expect("Failed to write temp file");
        drop(file);

        let schema_path = crtool::crjson_schema_path();
        let result = validate_json_files(std::slice::from_ref(&temp_file), &schema_path, "crJSON");
        assert!(result.is_err());

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_validate_json_files_with_nonexistent_file() {
        let nonexistent = PathBuf::from("/nonexistent/file.json");
        let schema_path = crtool::crjson_schema_path();
        let result = validate_json_files(&[nonexistent], &schema_path, "crJSON");
        assert!(result.is_err());
    }
}
