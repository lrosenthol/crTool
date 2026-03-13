/*
Copyright 2026 Adobe. All rights reserved.
This file is licensed to you under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License. You may obtain a copy
of the License at http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under
the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR REPRESENTATIONS
OF ANY KIND, either express or implied. See the License for the specific language
governing permissions and limitations under the License.
*/

//! Tests for profile evaluation against crJSON indicators using the sample profiles
//! in the `profiles/` directory.

use anyhow::Result;
use profile_evaluator_rs::{evaluate_files, load_profile, serialize_report, OutputFormat};
use std::path::PathBuf;
use std::process::Command;

mod common;

fn profiles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("profiles")
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Find the compliance statement in the report and return its boolean value.
fn compliance_value(report: &serde_json::Value) -> Option<bool> {
    report
        .get("statements")
        .and_then(|s| s.as_array())
        .and_then(|sections| {
            sections.iter().find_map(|section| {
                section.as_array()?.iter().find_map(|stmt| {
                    if stmt.get("id")?.as_str()? == "c2pa:profile_compliance" {
                        stmt.get("value")?.as_bool()
                    } else {
                        None
                    }
                })
            })
        })
}

// ============================================================================
// Profile loading tests
// ============================================================================

#[test]
fn test_load_real_life_capture_profile() {
    let profile_path = profiles_dir().join("real-life-capture_profile.yml");
    assert!(profile_path.exists(), "Profile file should exist");
    load_profile(&profile_path).expect("real-life-capture profile should load without error");
}

#[test]
fn test_load_real_media_profile() {
    let profile_path = profiles_dir().join("real-media_profile.yml");
    assert!(profile_path.exists(), "Profile file should exist");
    load_profile(&profile_path).expect("real-media profile should load without error");
}

#[test]
fn test_load_human_illustration_profile() {
    let profile_path = profiles_dir().join("human-illustration_profile.yml");
    assert!(profile_path.exists(), "Profile file should exist");
    load_profile(&profile_path).expect("human-illustration profile should load without error");
}

#[test]
fn test_load_fully_generative_ai_profile() {
    let profile_path = profiles_dir().join("fully-generative-ai_profile.yml");
    assert!(profile_path.exists(), "Profile file should exist");
    load_profile(&profile_path).expect("fully-generative-ai profile should load without error");
}

// ============================================================================
// Compliance evaluation tests – compliant fixtures
// ============================================================================

#[test]
fn test_real_life_capture_profile_compliant() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("real-life-capture_profile.yml"),
        fixtures_dir().join("real_life_capture_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(true),
        "real_life_capture_indicators should be compliant with real-life-capture profile; report: {}",
        serde_json::to_string_pretty(&report).unwrap_or_default()
    );
    println!("✓ real-life-capture profile: compliant fixture passes");
    Ok(())
}

#[test]
fn test_real_media_profile_compliant() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("real-media_profile.yml"),
        fixtures_dir().join("real_life_capture_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(true),
        "real_life_capture_indicators should be compliant with real-media profile"
    );
    println!("✓ real-media profile: compliant fixture passes");
    Ok(())
}

#[test]
fn test_human_illustration_profile_compliant() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("human-illustration_profile.yml"),
        fixtures_dir().join("human_illustration_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(true),
        "human_illustration_indicators should be compliant with human-illustration profile; report: {}",
        serde_json::to_string_pretty(&report).unwrap_or_default()
    );
    println!("✓ human-illustration profile: compliant fixture passes");
    Ok(())
}

#[test]
fn test_fully_generative_ai_profile_compliant() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("fully-generative-ai_profile.yml"),
        fixtures_dir().join("generative_ai_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(true),
        "generative_ai_indicators should be compliant with fully-generative-ai profile; report: {}",
        serde_json::to_string_pretty(&report).unwrap_or_default()
    );
    println!("✓ fully-generative-ai profile: compliant fixture passes");
    Ok(())
}

// ============================================================================
// Compliance evaluation tests – non-compliant fixture
// ============================================================================

#[test]
fn test_real_life_capture_profile_non_compliant() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("real-life-capture_profile.yml"),
        fixtures_dir().join("non_compliant_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(false),
        "non_compliant_indicators should NOT be compliant with real-life-capture profile"
    );
    println!("✓ real-life-capture profile: non-compliant fixture correctly fails");
    Ok(())
}

#[test]
fn test_real_media_profile_non_compliant() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("real-media_profile.yml"),
        fixtures_dir().join("non_compliant_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(false),
        "non_compliant_indicators should NOT be compliant with real-media profile"
    );
    println!("✓ real-media profile: non-compliant fixture correctly fails");
    Ok(())
}

#[test]
fn test_generative_ai_profile_non_compliant() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("fully-generative-ai_profile.yml"),
        fixtures_dir().join("non_compliant_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(false),
        "non_compliant_indicators should NOT be compliant with fully-generative-ai profile"
    );
    println!("✓ fully-generative-ai profile: non-compliant fixture correctly fails");
    Ok(())
}

/// Generative-AI indicators should not be compliant with the real-life-capture profile.
#[test]
fn test_cross_profile_gen_ai_fails_real_life_capture() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("real-life-capture_profile.yml"),
        fixtures_dir().join("generative_ai_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(false),
        "generative_ai_indicators should NOT be compliant with real-life-capture profile"
    );
    println!("✓ cross-profile: gen-AI indicators correctly fail real-life-capture profile");
    Ok(())
}

/// Real-life-capture indicators should not be compliant with the fully-generative-ai profile.
#[test]
fn test_cross_profile_real_capture_fails_generative_ai() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("fully-generative-ai_profile.yml"),
        fixtures_dir().join("real_life_capture_indicators.json"),
    )?;

    assert_eq!(
        compliance_value(&report),
        Some(false),
        "real_life_capture_indicators should NOT be compliant with fully-generative-ai profile"
    );
    println!("✓ cross-profile: real-capture indicators correctly fail fully-generative-ai profile");
    Ok(())
}

// ============================================================================
// Report serialization tests
// ============================================================================

#[test]
fn test_serialize_report_json() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("real-media_profile.yml"),
        fixtures_dir().join("real_life_capture_indicators.json"),
    )?;

    let json_str = serialize_report(&report, OutputFormat::Json)?;

    // Valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str)?;
    assert!(
        parsed.get("statements").is_some(),
        "JSON report should have statements"
    );
    assert!(
        json_str.contains('\n'),
        "JSON report should be pretty-printed"
    );

    println!("✓ Report serializes to valid pretty-printed JSON");
    Ok(())
}

#[test]
fn test_serialize_report_yaml() -> Result<()> {
    let report = evaluate_files(
        profiles_dir().join("real-media_profile.yml"),
        fixtures_dir().join("real_life_capture_indicators.json"),
    )?;

    let yaml_str = serialize_report(&report, OutputFormat::Yaml)?;

    // Must round-trip through serde_yaml
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml_str)?;
    assert!(
        parsed.get("statements").is_some(),
        "YAML report should have statements"
    );

    println!("✓ Report serializes to valid YAML");
    Ok(())
}

// ============================================================================
// CLI integration tests (--profile flag)
// ============================================================================

#[test]
fn test_cli_standalone_profile_eval_json_output() -> Result<()> {
    let binary = common::cli_binary_path();
    let indicators = fixtures_dir().join("real_life_capture_indicators.json");
    let profile = profiles_dir().join("real-life-capture_profile.yml");

    let out_dir = common::output_dir().join("profile_eval");
    std::fs::create_dir_all(&out_dir)?;

    // Copy the indicators file into the output dir so the report lands there too
    let indicators_copy = out_dir.join("rlc_indicators.json");
    std::fs::copy(&indicators, &indicators_copy)?;

    let result = Command::new(&binary)
        .arg("--profile")
        .arg(&profile)
        .arg("--report-format")
        .arg("json")
        .arg(&indicators_copy)
        .output()?;

    assert!(
        result.status.success(),
        "CLI profile eval should succeed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let report_path = out_dir.join("rlc_indicators-report.json");
    assert!(
        report_path.exists(),
        "Report file should be created at {:?}",
        report_path
    );

    let content = std::fs::read_to_string(&report_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    assert!(
        parsed.get("statements").is_some(),
        "Report should have statements"
    );

    println!("✓ CLI --profile standalone eval writes JSON report");
    Ok(())
}

#[test]
fn test_cli_standalone_profile_eval_yaml_output() -> Result<()> {
    let binary = common::cli_binary_path();
    let indicators = fixtures_dir().join("generative_ai_indicators.json");
    let profile = profiles_dir().join("fully-generative-ai_profile.yml");

    let out_dir = common::output_dir().join("profile_eval");
    std::fs::create_dir_all(&out_dir)?;

    let indicators_copy = out_dir.join("genai_indicators.json");
    std::fs::copy(&indicators, &indicators_copy)?;

    let result = Command::new(&binary)
        .arg("--profile")
        .arg(&profile)
        .arg("--report-format")
        .arg("yaml")
        .arg(&indicators_copy)
        .output()?;

    assert!(
        result.status.success(),
        "CLI profile eval should succeed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let report_path = out_dir.join("genai_indicators-report.yaml");
    assert!(
        report_path.exists(),
        "YAML report file should be created at {:?}",
        report_path
    );

    let content = std::fs::read_to_string(&report_path)?;
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content)?;
    assert!(
        parsed.get("statements").is_some(),
        "YAML report should have statements"
    );

    println!("✓ CLI --profile standalone eval writes YAML report");
    Ok(())
}
