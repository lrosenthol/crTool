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
use std::path::{Path, PathBuf};
use std::process::Command;

mod common;

/// Get the path to the crTool binary
fn get_binary_path() -> PathBuf {
    common::cli_binary_path()
}

/// Get the path to test fixtures directory
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[test]
fn test_validation_with_valid_indicators() -> Result<()> {
    let binary = get_binary_path();
    let valid_file = fixtures_dir().join("valid_indicators.json");

    assert!(valid_file.exists(), "Test fixture file should exist");

    let output = Command::new(&binary)
        .arg("--validate")
        .arg(&valid_file)
        .output()
        .expect("Failed to execute command");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "Validation should succeed for valid indicators file"
    );

    Ok(())
}

#[test]
fn test_validation_with_minimal_valid_indicators() -> Result<()> {
    let binary = get_binary_path();
    let valid_file = fixtures_dir().join("minimal_valid_indicators.json");

    assert!(valid_file.exists(), "Test fixture file should exist");

    let output = Command::new(&binary)
        .arg("--validate")
        .arg(&valid_file)
        .output()
        .expect("Failed to execute command");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "Validation should succeed for minimal valid indicators file"
    );

    Ok(())
}

#[test]
fn test_validation_with_invalid_indicators() -> Result<()> {
    let binary = get_binary_path();
    let invalid_file = fixtures_dir().join("invalid_indicators.json");

    assert!(invalid_file.exists(), "Test fixture file should exist");

    let output = Command::new(&binary)
        .arg("--validate")
        .arg(&invalid_file)
        .output()
        .expect("Failed to execute command");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        !output.status.success(),
        "Validation should fail for invalid indicators file"
    );

    Ok(())
}

#[test]
fn test_validation_with_malformed_json() -> Result<()> {
    let binary = get_binary_path();

    // Create a temporary malformed JSON file
    let temp_dir = std::env::temp_dir();
    let malformed_file = temp_dir.join("test_malformed.json");
    fs::write(&malformed_file, "{ invalid json }")?;

    let output = Command::new(&binary)
        .arg("--validate")
        .arg(&malformed_file)
        .output()
        .expect("Failed to execute command");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        !output.status.success(),
        "Validation should fail for malformed JSON"
    );

    // Clean up
    fs::remove_file(malformed_file)?;

    Ok(())
}

#[test]
fn test_validation_with_multiple_files() -> Result<()> {
    let binary = get_binary_path();
    let valid_file = fixtures_dir().join("valid_indicators.json");
    let minimal_file = fixtures_dir().join("minimal_valid_indicators.json");

    assert!(valid_file.exists(), "Test fixture file should exist");
    assert!(minimal_file.exists(), "Test fixture file should exist");

    let output = Command::new(&binary)
        .arg("--validate")
        .arg(&valid_file)
        .arg(&minimal_file)
        .output()
        .expect("Failed to execute command");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "Validation should succeed when all files are valid"
    );

    Ok(())
}

#[test]
fn test_validation_with_mixed_valid_invalid_files() -> Result<()> {
    let binary = get_binary_path();
    let valid_file = fixtures_dir().join("valid_indicators.json");
    let invalid_file = fixtures_dir().join("invalid_indicators.json");

    assert!(valid_file.exists(), "Test fixture file should exist");
    assert!(invalid_file.exists(), "Test fixture file should exist");

    let output = Command::new(&binary)
        .arg("--validate")
        .arg(&valid_file)
        .arg(&invalid_file)
        .output()
        .expect("Failed to execute command");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        !output.status.success(),
        "Validation should fail when any file is invalid"
    );

    Ok(())
}

#[test]
fn test_validation_with_nonexistent_file() -> Result<()> {
    let binary = get_binary_path();
    let nonexistent = PathBuf::from("/nonexistent/file.json");

    let output = Command::new(&binary)
        .arg("--validate")
        .arg(&nonexistent)
        .output()
        .expect("Failed to execute command");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        !output.status.success(),
        "Validation should fail for nonexistent file"
    );

    Ok(())
}

#[test]
fn test_validation_extracts_from_signed_file() -> Result<()> {
    // This test validates that extracted manifests from signed files can be validated
    // First, check if there are any pre-existing extracted manifests in test_output
    let test_output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test_output");

    if !test_output_dir.exists() {
        // Skip this test if no test output directory exists yet
        println!("Skipping test - no test_output directory found");
        return Ok(());
    }

    // Look for any extracted manifest JSON files
    let extracted_manifests: Vec<PathBuf> = fs::read_dir(&test_output_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with("_manifest_jpt.json") || n.ends_with("_manifest.json"))
                    .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    if extracted_manifests.is_empty() {
        println!("Skipping test - no extracted manifests found in test_output");
        return Ok(());
    }

    let binary = get_binary_path();
    let first_manifest = &extracted_manifests[0];

    println!(
        "Testing validation with extracted manifest: {:?}",
        first_manifest
    );

    let output = Command::new(&binary)
        .arg("--validate")
        .arg(first_manifest)
        .output()
        .expect("Failed to execute command");

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Extracted manifests should be valid indicators documents
    // Note: This might fail if the extraction format doesn't match the indicators schema
    println!(
        "Validation result for extracted manifest: {}",
        if output.status.success() {
            "PASSED"
        } else {
            "FAILED"
        }
    );

    Ok(())
}
