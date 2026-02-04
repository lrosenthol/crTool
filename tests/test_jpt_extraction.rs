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

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod common;

use common::{certs_dir, manifests_dir, output_dir, sign_file_with_manifest, testfiles_dir};

/// Helper to generate output filename for extraction tests
fn generate_extraction_output(input: &str, use_jpt: bool, subdir: &str) -> PathBuf {
    let dir = output_dir().join(subdir);
    fs::create_dir_all(&dir).expect("Failed to create subdirectory");

    let suffix = if use_jpt {
        "_manifest_jpt.json"
    } else {
        "_manifest.json"
    };

    dir.join(format!(
        "{}{}",
        input
            .trim_end_matches(".jpg")
            .trim_end_matches(".png")
            .trim_end_matches(".webp"),
        suffix
    ))
}

/// Get the binary path for the CLI tool
fn get_binary_path() -> String {
    env!("CARGO_BIN_EXE_crTool").to_string()
}

// ============================================================================
// Basic JPEG Trust Extraction Tests
// ============================================================================

#[test]
fn test_extract_normal_format() -> Result<()> {
    // First, create a signed file
    let input = testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_manifest.json");
    let signed_output = output_dir().join("jpt_tests/test_normal_signed.jpg");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Extract in normal format
    let extract_output = generate_extraction_output("test_normal_signed", false, "jpt_tests");

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

    // Verify the output file exists
    assert!(extract_output.exists(), "Output file should exist");

    // Verify it's valid JSON
    let json_content = fs::read_to_string(&extract_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    // Verify it has standard format structure
    assert!(json_value.is_object(), "JSON should be an object");
    assert!(
        json_value.get("manifests").is_some() || json_value.get("active_manifest").is_some(),
        "JSON should contain manifest data"
    );

    // Should NOT have JPEG Trust specific fields
    assert!(
        json_value.get("@context").is_none(),
        "Normal format should not have @context field"
    );

    println!("✓ Normal format extraction test passed");
    Ok(())
}

#[test]
fn test_extract_jpt_format() -> Result<()> {
    // First, create a signed file
    let input = testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_manifest.json");
    let signed_output = output_dir().join("jpt_tests/test_jpt_signed.jpg");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Extract in JPEG Trust format
    let extract_output = generate_extraction_output("test_jpt_signed", true, "jpt_tests");

    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--extract")
        .arg("--jpt")
        .arg(&signed_output)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        result.status.success(),
        "Extraction failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Verify the output file exists
    assert!(extract_output.exists(), "Output file should exist");

    // Verify it's valid JSON
    let json_content = fs::read_to_string(&extract_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    // Verify it has JPEG Trust format structure
    assert!(json_value.is_object(), "JSON should be an object");

    // Check for JPEG Trust specific fields
    assert!(
        json_value.get("@context").is_some(),
        "JPEG Trust format should have @context field"
    );

    // Verify @context contains JPEG Trust vocabulary
    if let Some(context) = json_value.get("@context") {
        let context_str = context.to_string();
        assert!(
            context_str.contains("jpeg.org/jpegtrust"),
            "@context should reference JPEG Trust vocabulary"
        );
    }

    // Check for manifests array (JPEG Trust uses array, not object)
    assert!(
        json_value
            .get("manifests")
            .and_then(|m| m.as_array())
            .is_some(),
        "JPEG Trust format should have manifests as array"
    );

    // Check for asset_info (if asset hash was computed)
    if json_value.get("asset_info").is_some() {
        println!("  Asset info included in output");
    }

    println!("✓ JPEG Trust format extraction test passed");
    Ok(())
}

#[test]
fn test_jpt_without_extract_fails() -> Result<()> {
    let input = testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_manifest.json");
    let cert = certs_dir().join("ed25519.pub");
    let key = certs_dir().join("ed25519.pem");
    let output = output_dir().join("jpt_tests/should_fail.jpg");

    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--manifest")
        .arg(&manifest)
        .arg(&input)  // Positional argument, not --input
        .arg("--output")
        .arg(&output)
        .arg("--cert")
        .arg(&cert)
        .arg("--key")
        .arg(&key)
        .arg("--jpt")
        .arg("--allow-self-signed")
        .output()?;

    // Should fail because --jpt is only valid with --extract
    assert!(
        !result.status.success(),
        "--jpt without --extract should fail. stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("--jpt") && stderr.contains("extract"),
        "Error message should mention --jpt and extract requirement. Got: {}",
        stderr
    );

    println!("✓ --jpt without --extract correctly fails");
    Ok(())
}

// ============================================================================
// Multiple File Extraction Tests
// ============================================================================

#[test]
fn test_extract_multiple_files_normal_format() -> Result<()> {
    // Create multiple signed files
    let manifest = manifests_dir().join("simple_manifest.json");
    let extract_dir = output_dir().join("jpt_tests/multi_extract_normal");
    fs::create_dir_all(&extract_dir)?;

    let inputs = vec![
        ("Dog.jpg", testfiles_dir().join("Dog.jpg")),
        ("Dog.png", testfiles_dir().join("Dog.png")),
    ];

    let mut signed_files = Vec::new();
    for (name, input) in &inputs {
        let signed = output_dir().join(format!("jpt_tests/multi_normal_{}", name));
        sign_file_with_manifest(input, &signed, &manifest)?;
        signed_files.push(signed);
    }

    // Extract all files in normal format
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
        "Multi-file extraction failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Verify output files were created
    for signed in &signed_files {
        let filename = signed.file_stem().unwrap().to_str().unwrap();
        let expected_output = extract_dir.join(format!("{}_manifest.json", filename));
        assert!(
            expected_output.exists(),
            "Output file should exist: {:?}",
            expected_output
        );

        // Verify it's valid JSON
        let json_content = fs::read_to_string(&expected_output)?;
        let _: serde_json::Value = serde_json::from_str(&json_content)?;
    }

    println!("✓ Multiple file normal format extraction test passed");
    Ok(())
}

#[test]
fn test_extract_multiple_files_jpt_format() -> Result<()> {
    // Create multiple signed files
    let manifest = manifests_dir().join("full_manifest.json");
    let extract_dir = output_dir().join("jpt_tests/multi_extract_jpt");
    fs::create_dir_all(&extract_dir)?;

    let inputs = vec![
        ("Dog.jpg", testfiles_dir().join("Dog.jpg")),
        ("Dog.webp", testfiles_dir().join("Dog.webp")),
    ];

    let mut signed_files = Vec::new();
    for (name, input) in &inputs {
        let signed = output_dir().join(format!("jpt_tests/multi_jpt_{}", name));
        sign_file_with_manifest(input, &signed, &manifest)?;
        signed_files.push(signed);
    }

    // Extract all files in JPEG Trust format
    let binary = get_binary_path();
    let mut cmd = Command::new(binary);
    cmd.arg("--extract").arg("--jpt");

    for signed in &signed_files {
        cmd.arg(signed);
    }

    cmd.arg("--output").arg(&extract_dir);

    let result = cmd.output()?;

    assert!(
        result.status.success(),
        "Multi-file JPT extraction failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Verify output files were created with correct naming
    for signed in &signed_files {
        let filename = signed.file_stem().unwrap().to_str().unwrap();
        let expected_output = extract_dir.join(format!("{}_manifest_jpt.json", filename));
        assert!(
            expected_output.exists(),
            "JPT output file should exist: {:?}",
            expected_output
        );

        // Verify it's valid JPEG Trust format JSON
        let json_content = fs::read_to_string(&expected_output)?;
        let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

        // Verify JPEG Trust format
        assert!(
            json_value.get("@context").is_some(),
            "Should have @context in JPEG Trust format"
        );
    }

    println!("✓ Multiple file JPEG Trust format extraction test passed");
    Ok(())
}

// ============================================================================
// Format Comparison Tests
// ============================================================================

#[test]
fn test_format_differences() -> Result<()> {
    // Create a signed file
    let input = testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("full_manifest.json");
    let signed_output = output_dir().join("jpt_tests/compare_formats_signed.jpg");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Extract in both formats
    let normal_output = generate_extraction_output("compare_normal", false, "jpt_tests");
    let jpt_output = generate_extraction_output("compare_jpt", true, "jpt_tests");

    let binary = get_binary_path();

    // Extract normal format
    let result1 = Command::new(&binary)
        .arg("--extract")
        .arg(&signed_output)
        .arg("--output")
        .arg(&normal_output)
        .output()?;

    assert!(result1.status.success(), "Normal extraction should succeed");

    // Extract JPEG Trust format
    let result2 = Command::new(&binary)
        .arg("--extract")
        .arg("--jpt")
        .arg(&signed_output)
        .arg("--output")
        .arg(&jpt_output)
        .output()?;

    assert!(result2.status.success(), "JPT extraction should succeed");

    // Load both JSON files
    let normal_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&normal_output)?)?;
    let jpt_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&jpt_output)?)?;

    // Verify key differences

    // 1. JPEG Trust has @context, normal doesn't
    assert!(
        jpt_json.get("@context").is_some(),
        "JPT should have @context"
    );
    assert!(
        normal_json.get("@context").is_none(),
        "Normal format should not have @context"
    );

    // 2. JPEG Trust uses manifests array, normal uses manifests object
    let jpt_manifests = jpt_json.get("manifests").unwrap();
    let normal_manifests = normal_json.get("manifests").unwrap();

    assert!(jpt_manifests.is_array(), "JPT manifests should be an array");
    assert!(
        normal_manifests.is_object(),
        "Normal manifests should be an object"
    );

    // 3. Both should contain manifest data
    assert!(
        !jpt_manifests.as_array().unwrap().is_empty(),
        "JPT should have at least one manifest"
    );
    assert!(
        !normal_manifests.as_object().unwrap().is_empty(),
        "Normal should have at least one manifest"
    );

    println!("✓ Format differences test passed");
    println!("  - JPEG Trust format has @context field");
    println!("  - JPEG Trust uses manifests array vs object");
    println!("  - Both contain complete manifest data");

    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_extract_file_without_manifest() -> Result<()> {
    let input = testfiles_dir().join("Dog.jpg"); // Unsigned file
    let extract_output = output_dir().join("jpt_tests/no_manifest_extraction.json");

    let binary = get_binary_path();

    // Try to extract from unsigned file (normal format)
    let result1 = Command::new(&binary)
        .arg("--extract")
        .arg(&input)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        !result1.status.success(),
        "Extraction from unsigned file should fail"
    );

    // Try to extract from unsigned file (JPEG Trust format)
    let result2 = Command::new(&binary)
        .arg("--extract")
        .arg("--jpt")
        .arg(&input)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        !result2.status.success(),
        "JPT extraction from unsigned file should fail"
    );

    println!("✓ Error handling for unsigned files works correctly");
    Ok(())
}

#[test]
fn test_extract_nonexistent_file() -> Result<()> {
    let input = testfiles_dir().join("NonExistent.jpg");
    let extract_output = output_dir().join("jpt_tests/nonexistent_extraction.json");

    let binary = get_binary_path();

    // Try to extract from nonexistent file
    let result = Command::new(&binary)
        .arg("--extract")
        .arg("--jpt")
        .arg(&input)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        !result.status.success(),
        "Extraction from nonexistent file should fail"
    );

    println!("✓ Error handling for nonexistent files works correctly");
    Ok(())
}

// ============================================================================
// Programmatic API Tests (using helper functions)
// ============================================================================

#[test]
fn test_helper_extract_normal_format() -> Result<()> {
    // Create a signed file
    let input = testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("simple_manifest.json");
    let signed_output = output_dir().join("jpt_tests/helper_normal_signed.png");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Use helper function to extract
    let extract_output = output_dir().join("jpt_tests/helper_normal_extracted.json");
    common::extract_manifest_to_file(&signed_output, &extract_output)?;

    // Verify output
    assert!(extract_output.exists(), "Output should exist");
    let json_content = fs::read_to_string(&extract_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    assert!(json_value.is_object(), "Should be valid JSON");
    assert!(
        json_value.get("@context").is_none(),
        "Normal format should not have @context"
    );

    println!("✓ Helper function for normal format works");
    Ok(())
}

#[test]
fn test_helper_extract_jpt_format() -> Result<()> {
    // Create a signed file
    let input = testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("full_manifest.json");
    let signed_output = output_dir().join("jpt_tests/helper_jpt_signed.webp");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Use helper function to extract in JPEG Trust format
    let extract_output = output_dir().join("jpt_tests/helper_jpt_extracted.json");
    common::extract_manifest_to_file_jpt(&signed_output, &extract_output)?;

    // Verify output
    assert!(extract_output.exists(), "Output should exist");
    let json_content = fs::read_to_string(&extract_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    assert!(json_value.is_object(), "Should be valid JSON");
    assert!(
        json_value.get("@context").is_some(),
        "JPEG Trust format should have @context"
    );

    println!("✓ Helper function for JPEG Trust format works");
    Ok(())
}

// ============================================================================
// Edge Cases and Integration Tests
// ============================================================================

#[test]
fn test_jpt_with_complex_manifest() -> Result<()> {
    // Use a complex manifest with actions v2
    let input = testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("actions_v2_filtered_manifest.json");
    let signed_output = output_dir().join("jpt_tests/complex_signed.jpg");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Extract in JPEG Trust format
    let extract_output = output_dir().join("jpt_tests/complex_jpt.json");

    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--extract")
        .arg("--jpt")
        .arg(&signed_output)
        .arg("--output")
        .arg(&extract_output)
        .output()?;

    assert!(
        result.status.success(),
        "Complex manifest extraction should succeed"
    );

    // Verify the output
    let json_content = fs::read_to_string(&extract_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    // Should have JPEG Trust format
    assert!(json_value.get("@context").is_some());

    // Should have manifests with assertions
    if let Some(manifests) = json_value.get("manifests").and_then(|m| m.as_array()) {
        assert!(!manifests.is_empty(), "Should have manifests");

        // Check for assertions in JPEG Trust format (object, not array)
        if let Some(manifest) = manifests.first() {
            if let Some(assertions) = manifest.get("assertions") {
                // In JPEG Trust format, assertions are an object
                assert!(
                    assertions.is_object() || assertions.is_array(),
                    "Assertions should be present"
                );
            }
        }
    }

    println!("✓ JPEG Trust extraction with complex manifest works");
    Ok(())
}

#[test]
fn test_jpt_output_to_directory() -> Result<()> {
    // Test that when output is a directory, proper filenames are generated
    let input = testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("simple_manifest.json");
    let signed_output = output_dir().join("jpt_tests/dir_test_signed.png");

    fs::create_dir_all(signed_output.parent().unwrap())?;
    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Create output directory
    let extract_dir = output_dir().join("jpt_tests/dir_extract");
    fs::create_dir_all(&extract_dir)?;

    // Extract with directory as output
    let binary = get_binary_path();
    let result = Command::new(binary)
        .arg("--extract")
        .arg("--jpt")
        .arg(&signed_output)
        .arg("--output")
        .arg(&extract_dir)
        .output()?;

    assert!(result.status.success(), "Directory output should work");

    // Verify the expected filename was created
    let expected_file = extract_dir.join("dir_test_signed_manifest_jpt.json");
    assert!(
        expected_file.exists(),
        "Should create file with _manifest_jpt.json suffix"
    );

    println!("✓ JPEG Trust extraction to directory works");
    Ok(())
}
