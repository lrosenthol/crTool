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

//! crJSON extraction tests (CLI --extract outputs crJSON; Reader::crjson helper).

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod common;

use common::{
    manifests_dir, output_dir, sign_file_with_manifest, signed_assets_dir, testfiles_dir,
};

fn generate_extraction_output(input: &str, subdir: &str) -> PathBuf {
    let dir = output_dir().join(subdir);
    fs::create_dir_all(&dir).expect("Failed to create subdirectory");
    dir.join(
        input
            .trim_end_matches(".jpg")
            .trim_end_matches(".png")
            .trim_end_matches(".webp")
            .to_string()
            + "_cr.json",
    )
}

fn get_binary_path() -> std::path::PathBuf {
    common::cli_binary_path()
}

// ============================================================================
// Basic crJSON Extraction Tests
// ============================================================================

#[test]
fn test_extract_crjson_format() -> Result<()> {
    let input = testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_manifest.json");
    let signed_output = output_dir().join("crjson_tests/test_crjson_signed.jpg");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    let extract_output = generate_extraction_output("test_crjson_signed", "crjson_tests");

    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--extract")
        .arg(&signed_output)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        result.status.success(),
        "Extraction failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    assert!(extract_output.exists(), "Output file should exist");

    let json_content = fs::read_to_string(&extract_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    assert!(json_value.is_object(), "JSON should be an object");

    assert!(
        json_value.get("@context").is_some(),
        "crJSON format should have @context field"
    );
    if let Some(context) = json_value.get("@context") {
        let context_str = context.to_string();
        assert!(
            context_str.contains("contentcredentials.org/crjson"),
            "@context should reference crJSON vocabulary"
        );
    }

    assert!(
        json_value
            .get("manifests")
            .and_then(|m| m.as_array())
            .is_some(),
        "crJSON format should have manifests as array"
    );

    println!("✓ crJSON format extraction test passed");
    Ok(())
}

// ============================================================================
// Multiple File Extraction Tests
// ============================================================================

#[test]
fn test_extract_multiple_files_crjson_format() -> Result<()> {
    let manifest = manifests_dir().join("full_manifest.json");
    let extract_dir = output_dir().join("crjson_tests/multi_extract_crjson");
    fs::create_dir_all(&extract_dir)?;

    let inputs = vec![
        ("Dog.jpg", testfiles_dir().join("Dog.jpg")),
        ("Dog.webp", testfiles_dir().join("Dog.webp")),
    ];

    let mut signed_files = Vec::new();
    for (name, input) in &inputs {
        let signed = output_dir().join(format!("crjson_tests/multi_crjson_{}", name));
        sign_file_with_manifest(input, &signed, &manifest)?;
        signed_files.push(signed);
    }

    let binary = get_binary_path();
    let mut cmd = Command::new(binary);
    cmd.arg("--extract");

    for signed in &signed_files {
        cmd.arg(signed);
    }

    cmd.arg("--output").arg(&extract_dir);

    let result = cmd.output()?;

    assert!(
        result.status.success(),
        "Multi-file crJSON extraction failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    for signed in &signed_files {
        let filename = signed.file_stem().unwrap().to_str().unwrap();
        let expected_output = extract_dir.join(format!("{}_cr.json", filename));
        assert!(
            expected_output.exists(),
            "crJSON output file should exist: {:?}",
            expected_output
        );

        let json_content = fs::read_to_string(&expected_output)?;
        let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

        assert!(
            json_value.get("@context").is_some(),
            "Should have @context in crJSON format"
        );
    }

    println!("✓ Multiple file crJSON format extraction test passed");
    Ok(())
}

// ============================================================================
// Extracted output structure (crJSON)
// ============================================================================

#[test]
fn test_extract_produces_crjson_structure() -> Result<()> {
    let input = testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("full_manifest.json");
    let signed_output = output_dir().join("crjson_tests/compare_crjson_signed.jpg");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    let extract_output = generate_extraction_output("compare_crjson", "crjson_tests");
    fs::create_dir_all(extract_output.parent().unwrap())?;

    let binary = get_binary_path();
    let result = Command::new(&binary)
        .arg("--extract")
        .arg(&signed_output)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(result.status.success(), "Extraction should succeed");

    let json_value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&extract_output)?)?;

    assert!(
        json_value.get("@context").is_some(),
        "Extracted output should have @context (crJSON)"
    );
    let manifests = json_value.get("manifests").unwrap();
    assert!(manifests.is_array(), "manifests should be an array");
    assert!(
        !manifests.as_array().unwrap().is_empty(),
        "Should have at least one manifest"
    );

    println!("✓ Extracted output has crJSON structure");
    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_extract_file_without_manifest_crjson() -> Result<()> {
    let input = testfiles_dir().join("Dog.jpg");
    let extract_output = output_dir().join("crjson_tests/no_manifest_crjson.json");

    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--extract")
        .arg(&input)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        !result.status.success(),
        "crJSON extraction from unsigned file should fail"
    );

    println!("✓ crJSON extraction from unsigned file correctly fails");
    Ok(())
}

#[test]
fn test_extract_nonexistent_file_crjson() -> Result<()> {
    let input = testfiles_dir().join("NonExistent.jpg");
    let extract_output = output_dir().join("crjson_tests/nonexistent_crjson.json");

    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--extract")
        .arg(&input)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        !result.status.success(),
        "Extraction from nonexistent file should fail"
    );

    println!("✓ crJSON extraction from nonexistent file correctly fails");
    Ok(())
}

// ============================================================================
// Programmatic API Tests (helper function)
// ============================================================================

#[test]
fn test_helper_extract_crjson_format() -> Result<()> {
    let input = testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("full_manifest.json");
    let signed_output = output_dir().join("crjson_tests/helper_crjson_signed.webp");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    let extract_output = output_dir().join("crjson_tests/helper_crjson_extracted.json");
    common::extract_manifest_to_file_crjson(&signed_output, &extract_output)?;

    assert!(extract_output.exists(), "Output should exist");
    let json_content = fs::read_to_string(&extract_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    assert!(json_value.is_object(), "Should be valid JSON");
    assert!(
        json_value.get("@context").is_some(),
        "crJSON format should have @context"
    );

    println!("✓ Helper function for crJSON format works");
    Ok(())
}

// ============================================================================
// Edge Cases and Integration Tests
// ============================================================================

#[test]
fn test_crjson_with_complex_manifest() -> Result<()> {
    let input = testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("actions_v2_filtered_manifest.json");
    let signed_output = output_dir().join("crjson_tests/complex_crjson_signed.jpg");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    let extract_output = output_dir().join("crjson_tests/complex_crjson.json");

    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--extract")
        .arg(&signed_output)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        result.status.success(),
        "Complex manifest crJSON extraction should succeed"
    );

    let json_content = fs::read_to_string(&extract_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    assert!(json_value.get("@context").is_some());

    if let Some(manifests) = json_value.get("manifests").and_then(|m| m.as_array()) {
        assert!(!manifests.is_empty(), "Should have manifests");

        if let Some(manifest_obj) = manifests.first() {
            if let Some(assertions) = manifest_obj.get("assertions") {
                assert!(
                    assertions.is_object() || assertions.is_array(),
                    "Assertions should be present"
                );
            }
        }
    }

    println!("✓ crJSON extraction with complex manifest works");
    Ok(())
}

#[test]
fn test_crjson_output_to_directory() -> Result<()> {
    let input = testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("simple_manifest.json");
    let signed_output = output_dir().join("crjson_tests/dir_crjson_signed.png");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    let extract_dir = output_dir().join("crjson_tests/dir_crjson_extract");
    fs::create_dir_all(&extract_dir)?;

    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--extract")
        .arg(&signed_output)
        .arg("--output")
        .arg(&extract_dir)
        .output()?;

    assert!(result.status.success(), "Directory output should work");

    let expected_file = extract_dir.join("dir_crjson_signed_cr.json");
    assert!(
        expected_file.exists(),
        "Should create file with _cr.json suffix"
    );

    println!("✓ crJSON extraction to directory works");
    Ok(())
}

// ============================================================================
// Trust list validation (--trust with crJSON)
// ============================================================================

/// Asset signed with a certificate on the C2PA/Content Credentials trust lists;
/// extraction with --trust should report signingCredential.trusted.
const TRUSTED_ASSET: &str = "PXL_20260208_202351558.jpg";

/// Extracts crJSON with --trust and asserts the active manifest is reported as trusted.
/// Requires network to fetch trust lists on first run.
#[test]
fn test_extract_crjson_with_trust_reports_trusted() -> Result<()> {
    let input = signed_assets_dir().join(TRUSTED_ASSET);
    if !input.exists() {
        eprintln!("Skipping: fixture {} not found", input.display());
        return Ok(());
    }

    let out_dir = output_dir().join("trust_crjson");
    fs::create_dir_all(&out_dir)?;

    let binary = get_binary_path();
    let result = Command::new(&binary)
        .arg("--trust")
        .arg("--extract")
        .arg(&input)
        .arg("--output")
        .arg(&out_dir)
        .output()?;

    assert!(
        result.status.success(),
        "Extraction with --trust failed: stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("asset");
    let json_path = out_dir.join(format!("{}_cr.json", stem));
    assert!(
        json_path.exists(),
        "Expected output file {}",
        json_path.display()
    );

    let json_content = fs::read_to_string(&json_path)?;
    let value: serde_json::Value = serde_json::from_str(&json_content)?;

    let success_codes = value
        .get("validationResults")
        .and_then(|v| v.get("activeManifest"))
        .and_then(|v| v.get("success"))
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);

    let has_trusted = success_codes.iter().any(|entry| {
        entry
            .get("code")
            .and_then(|c| c.as_str())
            .map(|c| c == "signingCredential.trusted")
            .unwrap_or(false)
    });
    assert!(
        has_trusted,
        "Expected signingCredential.trusted in success list; success codes: {:?}",
        success_codes
            .iter()
            .filter_map(|e| e.get("code").and_then(|c| c.as_str()))
            .collect::<Vec<_>>()
    );

    let failures = value
        .get("validationResults")
        .and_then(|v| v.get("activeManifest"))
        .and_then(|v| v.get("failure"))
        .and_then(|v| v.as_array())
        .map_or([].as_slice(), |a| a.as_slice());
    assert!(
        failures.is_empty(),
        "Expected no validation failures; got: {:?}",
        failures
    );

    println!(
        "✓ --trust reports signingCredential.trusted for {}",
        TRUSTED_ASSET
    );
    Ok(())
}
