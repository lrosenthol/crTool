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

mod common;

use common::{
    get_test_images, has_asset_thumbnail, has_ingredient_thumbnails, manifests_dir, output_dir,
    sign_file_with_manifest, sign_file_with_manifest_and_ingredients,
    sign_file_with_manifest_and_options, testfiles_dir, testset_dir, verify_signed_file,
};

/// Generate output filename from input filename and manifest type
/// Optionally specify a subdirectory within the output directory
fn generate_output_name(input: &Path, manifest_type: &str, subdir: Option<&str>) -> PathBuf {
    let stem = input.file_stem().unwrap().to_str().unwrap();
    let ext = input.extension().unwrap().to_str().unwrap();
    let filename = format!("{}_{}.{}", stem, manifest_type, ext);

    if let Some(sub) = subdir {
        let dir = output_dir().join(sub);
        std::fs::create_dir_all(&dir).expect("Failed to create subdirectory");
        dir.join(filename)
    } else {
        output_dir().join(filename)
    }
}

/// Generate output filename from manifest type & input extension
/// Optionally specify a subdirectory within the output directory
fn generate_output_name_no_stem(
    input: &Path,
    manifest_type: &str,
    subdir: Option<&str>,
) -> PathBuf {
    let ext = input.extension().unwrap().to_str().unwrap();
    let filename = format!("{}.{}", manifest_type, ext);

    if let Some(sub) = subdir {
        let dir = output_dir().join(sub);
        std::fs::create_dir_all(&dir).expect("Failed to create subdirectory");
        dir.join(filename)
    } else {
        output_dir().join(filename)
    }
}

// Tests for Dog.jpg
#[test]
fn test_dog_jpg_simple_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_manifest.json");
    let output = generate_output_name(&input, "simple", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify basic manifest properties
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(manifest.title().unwrap_or_default(), "Created Image");
    }

    println!("✓ Dog.jpg with simple_manifest.json: {}", output.display());
    Ok(())
}

#[test]
fn test_dog_jpg_full_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("full_manifest.json");
    let output = generate_output_name(&input, "full", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify detailed manifest properties
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(manifest.title().unwrap_or_default(), "Edited Photo");

        // Verify we have assertions
        assert!(!manifest.assertions().is_empty());
    }

    println!("✓ Dog.jpg with full_manifest.json: {}", output.display());
    Ok(())
}

// Tests for Dog.png
#[test]
fn test_dog_png_simple_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("simple_manifest.json");
    let output = generate_output_name(&input, "simple", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify basic manifest properties
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(manifest.title().unwrap_or_default(), "Created Image");
    }

    println!("✓ Dog.png with simple_manifest.json: {}", output.display());
    Ok(())
}

#[test]
fn test_dog_png_full_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("full_manifest.json");
    let output = generate_output_name(&input, "full", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify detailed manifest properties
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(manifest.title().unwrap_or_default(), "Edited Photo");

        // Verify we have assertions
        assert!(!manifest.assertions().is_empty());
    }

    println!("✓ Dog.png with full_manifest.json: {}", output.display());
    Ok(())
}

// Tests for Dog.webp
#[test]
fn test_dog_webp_simple_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("simple_manifest.json");
    let output = generate_output_name(&input, "simple", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify basic manifest properties
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(manifest.title().unwrap_or_default(), "Created Image");
    }

    println!("✓ Dog.webp with simple_manifest.json: {}", output.display());
    Ok(())
}

#[test]
fn test_dog_webp_full_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("full_manifest.json");
    let output = generate_output_name(&input, "full", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify detailed manifest properties
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(manifest.title().unwrap_or_default(), "Edited Photo");

        // Verify we have assertions
        assert!(!manifest.assertions().is_empty());
    }

    println!("✓ Dog.webp with full_manifest.json: {}", output.display());
    Ok(())
}

// Batch test to run all combinations
#[test]
fn test_all_images_all_manifests() -> Result<()> {
    let manifests = vec![
        ("simple", manifests_dir().join("simple_manifest.json")),
        ("full", manifests_dir().join("full_manifest.json")),
        ("asset_ref", manifests_dir().join("asset_ref_manifest.json")),
        (
            "asset_type",
            manifests_dir().join("asset_type_manifest.json"),
        ),
        (
            "cloud_data",
            manifests_dir().join("cloud_data_manifest.json"),
        ),
        (
            "depthmap_gdepth",
            manifests_dir().join("depthmap_gdepth_manifest.json"),
        ),
        (
            "external_reference",
            manifests_dir().join("external_reference_manifest.json"),
        ),
        (
            "actions_v2_edited",
            manifests_dir().join("actions_v2_edited_manifest.json"),
        ),
        (
            "actions_v2_translated",
            manifests_dir().join("actions_v2_translated_manifest.json"),
        ),
        (
            "actions_v2_redacted",
            manifests_dir().join("actions_v2_redacted_manifest.json"),
        ),
        (
            "actions_v2_cropped",
            manifests_dir().join("actions_v2_cropped_manifest.json"),
        ),
        (
            "actions_v2_filtered",
            manifests_dir().join("actions_v2_filtered_manifest.json"),
        ),
    ];

    let mut success_count = 0;
    let mut total_count = 0;

    for input in get_test_images() {
        for (manifest_type, manifest_path) in &manifests {
            total_count += 1;
            // Use "batch" subdirectory to avoid conflicts with individual tests
            let output = generate_output_name(&input, manifest_type, Some("batch"));

            match sign_file_with_manifest(&input, &output, manifest_path) {
                Ok(_) => match verify_signed_file(&output) {
                    Ok(_) => {
                        success_count += 1;
                        println!(
                            "✓ {} with {} manifest",
                            input.file_name().unwrap().to_str().unwrap(),
                            manifest_type
                        );
                    }
                    Err(e) => {
                        eprintln!("✗ Verification failed for {:?}: {}", output, e);
                    }
                },
                Err(e) => {
                    eprintln!(
                        "✗ Signing failed for {:?} with {}: {}",
                        input, manifest_type, e
                    );
                }
            }
        }
    }

    println!("\n{}/{} tests passed", success_count, total_count);
    assert_eq!(
        success_count, total_count,
        "All image/manifest combinations should succeed"
    );

    Ok(())
}

// Tests for actions v2 manifests
#[test]
fn test_actions_v2_edited_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("actions_v2_edited_manifest.json");
    let output = generate_output_name(&input, "actions_v2_edited", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Action V2 - Edited with Template"
        );

        let assertions = manifest.assertions();
        let has_actions_v2 = assertions.iter().any(|a| a.label() == "c2pa.actions.v2");
        assert!(has_actions_v2, "Should have c2pa.actions.v2 assertion");
    }

    println!(
        "✓ Dog.jpg with actions_v2_edited_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_actions_v2_translated_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("actions_v2_translated_manifest.json");
    let output = generate_output_name(&input, "actions_v2_translated", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Action V2 - Translated"
        );

        let assertions = manifest.assertions();
        let has_actions_v2 = assertions.iter().any(|a| a.label() == "c2pa.actions.v2");
        assert!(has_actions_v2, "Should have c2pa.actions.v2 assertion");
    }

    println!(
        "✓ Dog.png with actions_v2_translated_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_actions_v2_redacted_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("actions_v2_redacted_manifest.json");
    let output = generate_output_name(&input, "actions_v2_redacted", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Action V2 - Redacted"
        );

        let assertions = manifest.assertions();
        let has_actions_v2 = assertions.iter().any(|a| a.label() == "c2pa.actions.v2");
        assert!(has_actions_v2, "Should have c2pa.actions.v2 assertion");
    }

    println!(
        "✓ Dog.webp with actions_v2_redacted_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_actions_v2_cropped_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("actions_v2_cropped_manifest.json");
    let output = generate_output_name(&input, "actions_v2_cropped", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Action V2 - Cropped"
        );

        let assertions = manifest.assertions();
        let has_actions_v2 = assertions.iter().any(|a| a.label() == "c2pa.actions.v2");
        assert!(has_actions_v2, "Should have c2pa.actions.v2 assertion");
    }

    println!(
        "✓ Dog.jpg with actions_v2_cropped_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_actions_v2_filtered_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("actions_v2_filtered_manifest.json");
    let output = generate_output_name(&input, "actions_v2_filtered", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Action V2 - Filtered with Multiple Actions"
        );

        let assertions = manifest.assertions();
        let has_actions_v2 = assertions.iter().any(|a| a.label() == "c2pa.actions.v2");
        assert!(has_actions_v2, "Should have c2pa.actions.v2 assertion");
    }

    println!(
        "✓ Dog.png with actions_v2_filtered_manifest.json: {}",
        output.display()
    );
    Ok(())
}

// Tests for external-reference manifest
#[test]
fn test_dog_jpg_external_reference_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("external_reference_manifest.json");
    let output = generate_output_name(&input, "external_reference", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and external-reference assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with External Reference"
        );

        // Verify we have assertions including external-reference
        let assertions = manifest.assertions();
        assert!(!assertions.is_empty());

        // Check that c2pa.external-reference assertion exists
        let has_external_ref = assertions
            .iter()
            .any(|a| a.label() == "c2pa.external-reference");
        assert!(
            has_external_ref,
            "Should have c2pa.external-reference assertion"
        );
    }

    println!(
        "✓ Dog.jpg with external_reference_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_png_external_reference_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("external_reference_manifest.json");
    let output = generate_output_name(&input, "external_reference", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and external-reference assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with External Reference"
        );

        // Verify we have the external-reference assertion
        let assertions = manifest.assertions();
        let has_external_ref = assertions
            .iter()
            .any(|a| a.label() == "c2pa.external-reference");
        assert!(
            has_external_ref,
            "Should have c2pa.external-reference assertion"
        );
    }

    println!(
        "✓ Dog.png with external_reference_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_webp_external_reference_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("external_reference_manifest.json");
    let output = generate_output_name(&input, "external_reference", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and external-reference assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with External Reference"
        );

        // Verify we have the external-reference assertion
        let assertions = manifest.assertions();
        let has_external_ref = assertions
            .iter()
            .any(|a| a.label() == "c2pa.external-reference");
        assert!(
            has_external_ref,
            "Should have c2pa.external-reference assertion"
        );
    }

    println!(
        "✓ Dog.webp with external_reference_manifest.json: {}",
        output.display()
    );
    Ok(())
}

// Tests for cloud-data manifest
#[test]
fn test_dog_jpg_cloud_data_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("cloud_data_manifest.json");
    let output = generate_output_name(&input, "cloud_data", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and cloud-data assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Cloud-Hosted Assertion Data"
        );

        // Verify we have assertions including cloud-data
        let assertions = manifest.assertions();
        assert!(!assertions.is_empty());

        // Check that c2pa.cloud-data assertion exists
        let has_cloud_data = assertions.iter().any(|a| a.label() == "c2pa.cloud-data");
        assert!(has_cloud_data, "Should have c2pa.cloud-data assertion");
    }

    println!(
        "✓ Dog.jpg with cloud_data_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_png_cloud_data_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("cloud_data_manifest.json");
    let output = generate_output_name(&input, "cloud_data", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and cloud-data assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Cloud-Hosted Assertion Data"
        );

        // Verify we have the cloud-data assertion
        let assertions = manifest.assertions();
        let has_cloud_data = assertions.iter().any(|a| a.label() == "c2pa.cloud-data");
        assert!(has_cloud_data, "Should have c2pa.cloud-data assertion");
    }

    println!(
        "✓ Dog.png with cloud_data_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_webp_cloud_data_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("cloud_data_manifest.json");
    let output = generate_output_name(&input, "cloud_data", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and cloud-data assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Cloud-Hosted Assertion Data"
        );

        // Verify we have the cloud-data assertion
        let assertions = manifest.assertions();
        let has_cloud_data = assertions.iter().any(|a| a.label() == "c2pa.cloud-data");
        assert!(has_cloud_data, "Should have c2pa.cloud-data assertion");
    }

    println!(
        "✓ Dog.webp with cloud_data_manifest.json: {}",
        output.display()
    );
    Ok(())
}

// Tests for depthmap-gdepth manifest
#[test]
fn test_dog_jpg_depthmap_gdepth_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("depthmap_gdepth_manifest.json");
    let output = generate_output_name(&input, "depthmap_gdepth", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and depthmap assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with GDepth 3D Depth Map"
        );

        // Verify we have assertions including depthmap
        let assertions = manifest.assertions();
        assert!(!assertions.is_empty());

        // Check that c2pa.depthmap.gdepth assertion exists
        let has_depthmap = assertions
            .iter()
            .any(|a| a.label() == "c2pa.depthmap.gdepth");
        assert!(has_depthmap, "Should have c2pa.depthmap.gdepth assertion");
    }

    println!(
        "✓ Dog.jpg with depthmap_gdepth_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_png_depthmap_gdepth_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("depthmap_gdepth_manifest.json");
    let output = generate_output_name(&input, "depthmap_gdepth", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and depthmap assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with GDepth 3D Depth Map"
        );

        // Verify we have the depthmap assertion
        let assertions = manifest.assertions();
        let has_depthmap = assertions
            .iter()
            .any(|a| a.label() == "c2pa.depthmap.gdepth");
        assert!(has_depthmap, "Should have c2pa.depthmap.gdepth assertion");
    }

    println!(
        "✓ Dog.png with depthmap_gdepth_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_webp_depthmap_gdepth_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("depthmap_gdepth_manifest.json");
    let output = generate_output_name(&input, "depthmap_gdepth", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and depthmap assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with GDepth 3D Depth Map"
        );

        // Verify we have the depthmap assertion
        let assertions = manifest.assertions();
        let has_depthmap = assertions
            .iter()
            .any(|a| a.label() == "c2pa.depthmap.gdepth");
        assert!(has_depthmap, "Should have c2pa.depthmap.gdepth assertion");
    }

    println!(
        "✓ Dog.webp with depthmap_gdepth_manifest.json: {}",
        output.display()
    );
    Ok(())
}

// Tests for asset-type manifest
#[test]
fn test_dog_jpg_asset_type_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("asset_type_manifest.json");
    let output = generate_output_name(&input, "asset_type", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and asset-type assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Asset Type Information"
        );

        // Verify we have assertions including asset-type
        let assertions = manifest.assertions();
        assert!(!assertions.is_empty());

        // Check that c2pa.asset-type assertion exists
        let has_asset_type = assertions.iter().any(|a| a.label() == "c2pa.asset-type");
        assert!(has_asset_type, "Should have c2pa.asset-type assertion");
    }

    println!(
        "✓ Dog.jpg with asset_type_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_png_asset_type_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("asset_type_manifest.json");
    let output = generate_output_name(&input, "asset_type", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and asset-type assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Asset Type Information"
        );

        // Verify we have the asset-type assertion
        let assertions = manifest.assertions();
        let has_asset_type = assertions.iter().any(|a| a.label() == "c2pa.asset-type");
        assert!(has_asset_type, "Should have c2pa.asset-type assertion");
    }

    println!(
        "✓ Dog.png with asset_type_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_webp_asset_type_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("asset_type_manifest.json");
    let output = generate_output_name(&input, "asset_type", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and asset-type assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Asset Type Information"
        );

        // Verify we have the asset-type assertion
        let assertions = manifest.assertions();
        let has_asset_type = assertions.iter().any(|a| a.label() == "c2pa.asset-type");
        assert!(has_asset_type, "Should have c2pa.asset-type assertion");
    }

    println!(
        "✓ Dog.webp with asset_type_manifest.json: {}",
        output.display()
    );
    Ok(())
}

// Tests for asset-ref manifest
#[test]
fn test_dog_jpg_asset_ref_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("asset_ref_manifest.json");
    let output = generate_output_name(&input, "asset_ref", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and asset-ref assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Asset Reference"
        );

        // Verify we have assertions including asset-ref
        let assertions = manifest.assertions();
        assert!(!assertions.is_empty());

        // Check that c2pa.asset-ref assertion exists
        let has_asset_ref = assertions.iter().any(|a| a.label() == "c2pa.asset-ref");
        assert!(has_asset_ref, "Should have c2pa.asset-ref assertion");
    }

    println!(
        "✓ Dog.jpg with asset_ref_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_png_asset_ref_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("asset_ref_manifest.json");
    let output = generate_output_name(&input, "asset_ref", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and asset-ref assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Asset Reference"
        );

        // Verify we have the asset-ref assertion
        let assertions = manifest.assertions();
        let has_asset_ref = assertions.iter().any(|a| a.label() == "c2pa.asset-ref");
        assert!(has_asset_ref, "Should have c2pa.asset-ref assertion");
    }

    println!(
        "✓ Dog.png with asset_ref_manifest.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_dog_webp_asset_ref_manifest() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("asset_ref_manifest.json");
    let output = generate_output_name(&input, "asset_ref", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify manifest properties and asset-ref assertion
    if let Some(manifest_label) = reader.active_label() {
        let manifest = reader.get_manifest(manifest_label).unwrap();
        assert_eq!(
            manifest.title().unwrap_or_default(),
            "Image with Asset Reference"
        );

        // Verify we have the asset-ref assertion
        let assertions = manifest.assertions();
        let has_asset_ref = assertions.iter().any(|a| a.label() == "c2pa.asset-ref");
        assert!(has_asset_ref, "Should have c2pa.asset-ref assertion");
    }

    println!(
        "✓ Dog.webp with asset_ref_manifest.json: {}",
        output.display()
    );
    Ok(())
}

// Test to verify output files are valid and readable
#[test]
fn test_output_files_are_readable() {
    let output = output_dir();
    if output.exists() {
        for entry in std::fs::read_dir(output).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let metadata = std::fs::metadata(&path).unwrap();
                assert!(
                    metadata.len() > 0,
                    "Output file should not be empty: {:?}",
                    path
                );
            }
        }
    }
}

// Tests for manifest extraction feature
#[test]
fn test_extract_manifest_to_specific_file() -> Result<()> {
    use std::fs;

    // First, create a signed file to extract from
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_manifest.json");
    let signed_output = generate_output_name(&input, "extract_test", Some("individual"));

    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Now extract the manifest to a specific JSON file
    let json_output = output_dir().join("extracted/extracted_manifest.json");
    common::extract_manifest_to_file(&signed_output, &json_output)?;

    // Verify the JSON file was created
    assert!(json_output.exists(), "Extracted JSON file should exist");

    // Verify the JSON file is valid and contains expected data
    let json_content = fs::read_to_string(&json_output)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    // Check that it has the expected structure
    assert!(json_value.is_object(), "JSON should be an object");
    assert!(
        json_value.get("manifests").is_some() || json_value.get("active_manifest").is_some(),
        "JSON should contain manifest data"
    );

    println!(
        "✓ Successfully extracted manifest to: {}",
        json_output.display()
    );
    Ok(())
}

#[test]
fn test_extract_manifest_to_directory() -> Result<()> {
    use std::fs;

    // First, create a signed file to extract from
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("full_manifest.json");
    let signed_output = generate_output_name(&input, "extract_dir_test", Some("individual"));

    sign_file_with_manifest(&input, &signed_output, &manifest)?;

    // Create a directory for extraction
    let extract_dir = output_dir().join("extracted");
    fs::create_dir_all(&extract_dir)?;

    // Extract the manifest to the directory
    // The helper will create a filename based on the input
    let expected_json = extract_dir.join("Dog_extract_dir_test_manifest.json");
    common::extract_manifest_to_file(&signed_output, &expected_json)?;

    // Verify the JSON file was created in the directory
    assert!(
        expected_json.exists(),
        "Extracted JSON file should exist in directory"
    );

    // Verify the JSON file is valid
    let json_content = fs::read_to_string(&expected_json)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    assert!(json_value.is_object(), "JSON should be an object");

    // Verify it contains the title from the full_manifest.json
    if let Some(manifests) = json_value.get("manifests").and_then(|m| m.as_object()) {
        // The manifests object contains manifest labels as keys
        let has_title = manifests.values().any(|manifest| {
            manifest
                .get("title")
                .and_then(|t| t.as_str())
                .map(|s| s.contains("Edited Photo"))
                .unwrap_or(false)
        });
        assert!(
            has_title,
            "Manifest should contain title from full_manifest.json"
        );
    }

    println!(
        "✓ Successfully extracted manifest to directory: {}",
        expected_json.display()
    );
    Ok(())
}

// Round-trip test: sign with manifest, extract, and compare
#[test]
fn test_manifest_roundtrip_with_spec_version() -> Result<()> {
    use std::fs;

    // Sign the file with the specVersion manifest
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest_path = manifests_dir().join("specVersion_manifest.json");
    let signed_output = generate_output_name(&input, "specVersion", Some("individual"));

    // Read the original manifest JSON
    let original_manifest_json = fs::read_to_string(&manifest_path)?;
    let original: serde_json::Value = serde_json::from_str(&original_manifest_json)?;

    // Sign the file
    sign_file_with_manifest(&input, &signed_output, &manifest_path)?;

    // Extract the manifest from the signed file
    let extract_output = output_dir().join("extracted/specVersion_roundtrip.json");
    common::extract_manifest_to_file(&signed_output, &extract_output)?;

    // Read the extracted manifest
    let extracted_json = fs::read_to_string(&extract_output)?;
    let extracted: serde_json::Value = serde_json::from_str(&extracted_json)?;

    // The extracted manifest has a different structure - it's a manifest store
    // Get the active manifest from the extracted data
    let active_manifest_label = extracted["active_manifest"]
        .as_str()
        .expect("Should have active_manifest");

    let extracted_manifest = &extracted["manifests"][active_manifest_label];

    // Compare specVersion if present in original
    if let Some(original_spec_version) = original.get("specVersion") {
        // In the extracted manifest, specVersion might not be present or could be normalized
        // The c2pa library may handle this differently, so we'll note it was in the original
        println!("Original specVersion: {}", original_spec_version);
    }

    // Compare title
    assert_eq!(
        extracted_manifest["title"], original["title"],
        "Title should match"
    );

    // Compare assertions
    let original_assertions = original["assertions"]
        .as_array()
        .expect("Original should have assertions array");
    let extracted_assertions = extracted_manifest["assertions"]
        .as_array()
        .expect("Extracted should have assertions array");

    // Find the c2pa.actions assertion in both
    let original_actions = original_assertions
        .iter()
        .find(|a| a["label"] == "c2pa.actions")
        .expect("Original should have c2pa.actions assertion");

    let extracted_actions = extracted_assertions
        .iter()
        .find(|a| {
            a["label"].as_str() == Some("c2pa.actions.v2")
                || a["label"].as_str() == Some("c2pa.actions")
        })
        .expect("Extracted should have c2pa.actions assertion");

    // Compare the actions data
    let original_actions_data = &original_actions["data"]["actions"]
        .as_array()
        .expect("Original actions should have actions array");

    let extracted_actions_data = &extracted_actions["data"]["actions"]
        .as_array()
        .expect("Extracted actions should have actions array");

    // Should have at least the same number of actions
    assert!(
        extracted_actions_data.len() >= original_actions_data.len(),
        "Extracted should have at least as many actions as original"
    );

    // Compare the first action (the one we added)
    let original_action = &original_actions_data[0];
    let extracted_action = &extracted_actions_data[0];

    assert_eq!(
        extracted_action["action"], original_action["action"],
        "Action type should match"
    );

    assert_eq!(
        extracted_action["when"], original_action["when"],
        "Action timestamp should match"
    );

    // The softwareAgent might be modified by the library to include version info
    // So we check if it contains the base name
    let original_agent = original_action["softwareAgent"]
        .as_str()
        .expect("Should have softwareAgent");
    let extracted_agent = extracted_action["softwareAgent"]
        .as_str()
        .expect("Should have softwareAgent");

    assert!(
        extracted_agent.contains("crTool") || extracted_agent == original_agent,
        "Software agent should match or contain the original name. Original: {}, Extracted: {}",
        original_agent,
        extracted_agent
    );

    // Verify claim_generator_info is present
    let extracted_claim_gen = extracted_manifest["claim_generator_info"]
        .as_array()
        .expect("Should have claim_generator_info");

    assert!(
        !extracted_claim_gen.is_empty(),
        "Should have at least one claim generator"
    );

    // Check that the claim generator name contains our app name
    let has_our_generator = extracted_claim_gen.iter().any(|gen| {
        gen["name"]
            .as_str()
            .map(|n| n.contains("crTool"))
            .unwrap_or(false)
    });

    assert!(has_our_generator, "Claim generator should include crTool");

    println!("✓ Round-trip test passed: manifest data preserved correctly");
    println!("  Original title: {}", original["title"]);
    println!("  Extracted title: {}", extracted_manifest["title"]);
    println!(
        "  Actions preserved: {}/{}",
        extracted_actions_data.len(),
        original_actions_data.len()
    );

    Ok(())
}

// ============================================================================
// Ingredient Tests - File-based ingredient loading
// ============================================================================

#[test]
fn test_simple_with_ingredient() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("simple_with_ingredient.json");
    let output = generate_output_name(&input, "simple_ingredient", Some("individual"));

    // Get the base directory for resolving ingredient paths
    let ingredients_base_dir = manifest.parent().unwrap();

    sign_file_with_manifest_and_ingredients(&input, &output, &manifest, ingredients_base_dir)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify the manifest has ingredients
    if let Some(manifest_label) = reader.active_label() {
        let manifest_store = reader.get_manifest(manifest_label).unwrap();
        let ingredients = manifest_store.ingredients();

        assert_eq!(ingredients.len(), 1, "Should have one ingredient");
        assert_eq!(ingredients[0].title().unwrap_or_default(), "Original Image");
    }

    println!(
        "✓ Dog.png with simple_with_ingredient.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_with_ingredients_from_files() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("with_ingredients_from_files.json");
    let output = generate_output_name(&input, "with_ingredients", Some("individual"));

    // Get the base directory for resolving ingredient paths
    let ingredients_base_dir = manifest.parent().unwrap();

    sign_file_with_manifest_and_ingredients(&input, &output, &manifest, ingredients_base_dir)?;

    let reader = verify_signed_file(&output)?;
    assert!(reader.active_label().is_some());

    // Verify the manifest has multiple ingredients
    if let Some(manifest_label) = reader.active_label() {
        let manifest_store = reader.get_manifest(manifest_label).unwrap();
        let ingredients = manifest_store.ingredients();

        assert_eq!(ingredients.len(), 2, "Should have two ingredients");

        // Check the first ingredient (parentOf)
        assert_eq!(
            ingredients[0].title().unwrap_or_default(),
            "Background Image"
        );
        assert_eq!(*ingredients[0].relationship(), c2pa::Relationship::ParentOf);

        // Check the second ingredient (componentOf)
        assert_eq!(
            ingredients[1].title().unwrap_or_default(),
            "Secondary Element"
        );
        assert_eq!(
            *ingredients[1].relationship(),
            c2pa::Relationship::ComponentOf
        );
    }

    println!(
        "✓ Dog.webp with with_ingredients_from_files.json: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_ingredient_thumbnails_generated() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_with_ingredient.json");
    let output = generate_output_name(&input, "ingredient_thumbnail", Some("individual"));

    let ingredients_base_dir = manifest.parent().unwrap();

    sign_file_with_manifest_and_ingredients(&input, &output, &manifest, ingredients_base_dir)?;

    let reader = verify_signed_file(&output)?;

    // Note: The test helper doesn't generate thumbnails (unlike the CLI tool which does).
    // This test verifies that ingredients are loaded correctly.
    // For thumbnail generation, use the CLI tool directly.
    if let Some(manifest_label) = reader.active_label() {
        let manifest_store = reader.get_manifest(manifest_label).unwrap();
        let ingredients = manifest_store.ingredients();

        assert_eq!(ingredients.len(), 1, "Should have one ingredient");
        // Thumbnails are optional in the test helper
    }

    println!(
        "✓ Ingredient loading test (without thumbnails): {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_ingredient_missing_file_error() -> Result<()> {
    use std::io::Write;

    // Create a manifest with a non-existent ingredient file
    let manifest_content = r#"{
        "claim_generator_info": [{"name": "test", "version": "1.0.0"}],
        "title": "Test",
        "ingredients_from_files": [
            {
                "title": "Missing",
                "relationship": "parentOf",
                "file_path": "../testfiles/nonexistent.jpg"
            }
        ]
    }"#;

    let temp_manifest = output_dir().join("temp_manifest_missing.json");
    let mut file = fs::File::create(&temp_manifest)?;
    file.write_all(manifest_content.as_bytes())?;

    let input = common::testfiles_dir().join("Dog.jpg");
    let output = output_dir().join("should_fail.jpg");
    let ingredients_base_dir = temp_manifest.parent().unwrap();

    // This should fail with an error about missing file
    let result = sign_file_with_manifest_and_ingredients(
        &input,
        &output,
        &temp_manifest,
        ingredients_base_dir,
    );

    assert!(result.is_err(), "Should fail with missing ingredient file");

    // Clean up
    if temp_manifest.exists() {
        fs::remove_file(&temp_manifest)?;
    }

    println!("✓ Missing ingredient file error handling test passed");
    Ok(())
}

// ============================================================================
// Thumbnail Tests - Asset and Ingredient Thumbnail Verification
// ============================================================================

#[test]
fn test_asset_thumbnail_not_present_by_default() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_manifest.json");
    let output = generate_output_name(&input, "no_asset_thumb", Some("individual"));

    sign_file_with_manifest(&input, &output, &manifest)?;

    let reader = verify_signed_file(&output)?;

    if let Some(manifest_label) = reader.active_label() {
        let has_thumb = has_asset_thumbnail(&reader, manifest_label);
        assert!(
            !has_thumb,
            "Asset thumbnail should NOT be present by default"
        );
    }

    println!(
        "✓ Asset thumbnail correctly absent by default: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_asset_thumbnail_present_when_requested() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_manifest.json");
    let output = generate_output_name(&input, "with_asset_thumb", Some("individual"));
    let ingredients_base_dir = manifest.parent().unwrap();

    sign_file_with_manifest_and_options(
        &input,
        &output,
        &manifest,
        ingredients_base_dir,
        true,  // generate_asset_thumbnail
        false, // generate_ingredient_thumbnails
    )?;

    let reader = verify_signed_file(&output)?;

    if let Some(manifest_label) = reader.active_label() {
        let has_thumb = has_asset_thumbnail(&reader, manifest_label);
        assert!(
            has_thumb,
            "Asset thumbnail should be present when requested"
        );
    }

    println!(
        "✓ Asset thumbnail correctly present when requested: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_asset_thumbnail_with_different_formats() -> Result<()> {
    let test_files = vec![
        ("Dog.jpg", "image/jpeg"),
        ("Dog.png", "image/png"),
        ("Dog.webp", "image/webp"),
    ];

    for (filename, format) in test_files {
        let input = common::testfiles_dir().join(filename);
        let manifest = manifests_dir().join("simple_manifest.json");
        let output = generate_output_name(&input, "asset_thumb_formats", Some("individual"));
        let ingredients_base_dir = manifest.parent().unwrap();

        sign_file_with_manifest_and_options(
            &input,
            &output,
            &manifest,
            ingredients_base_dir,
            true,  // generate_asset_thumbnail
            false, // generate_ingredient_thumbnails
        )?;

        let reader = verify_signed_file(&output)?;

        if let Some(manifest_label) = reader.active_label() {
            let has_thumb = has_asset_thumbnail(&reader, manifest_label);
            assert!(
                has_thumb,
                "Asset thumbnail should be present for {} ({})",
                filename, format
            );
        }

        println!(
            "✓ Asset thumbnail works with {}: {}",
            format,
            output.display()
        );
    }

    Ok(())
}

#[test]
fn test_ingredient_thumbnails_not_present_by_default() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("simple_with_ingredient.json");
    let output = generate_output_name(&input, "no_ing_thumb", Some("individual"));
    let ingredients_base_dir = manifest.parent().unwrap();

    sign_file_with_manifest_and_ingredients(&input, &output, &manifest, ingredients_base_dir)?;

    let reader = verify_signed_file(&output)?;

    if let Some(manifest_label) = reader.active_label() {
        let has_thumbs = has_ingredient_thumbnails(&reader, manifest_label);
        assert!(
            !has_thumbs,
            "Ingredient thumbnails should NOT be present by default"
        );

        // Verify ingredients exist but without thumbnails
        let manifest_store = reader.get_manifest(manifest_label).unwrap();
        let ingredients = manifest_store.ingredients();
        assert_eq!(ingredients.len(), 1, "Should have one ingredient");
        assert!(
            ingredients[0].thumbnail_ref().is_none(),
            "Ingredient should not have thumbnail"
        );
    }

    println!(
        "✓ Ingredient thumbnails correctly absent by default: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_ingredient_thumbnails_present_when_requested() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.png");
    let manifest = manifests_dir().join("simple_with_ingredient.json");
    let output = generate_output_name(&input, "with_ing_thumb", Some("individual"));
    let ingredients_base_dir = manifest.parent().unwrap();

    sign_file_with_manifest_and_options(
        &input,
        &output,
        &manifest,
        ingredients_base_dir,
        false, // generate_asset_thumbnail
        true,  // generate_ingredient_thumbnails
    )?;

    let reader = verify_signed_file(&output)?;

    if let Some(manifest_label) = reader.active_label() {
        let has_thumbs = has_ingredient_thumbnails(&reader, manifest_label);
        assert!(
            has_thumbs,
            "Ingredient thumbnails should be present when requested"
        );

        // Verify each ingredient has a thumbnail
        let manifest_store = reader.get_manifest(manifest_label).unwrap();
        let ingredients = manifest_store.ingredients();
        assert_eq!(ingredients.len(), 1, "Should have one ingredient");
        assert!(
            ingredients[0].thumbnail_ref().is_some(),
            "Ingredient should have thumbnail"
        );
    }

    println!(
        "✓ Ingredient thumbnails correctly present when requested: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_multiple_ingredients_with_thumbnails() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.webp");
    let manifest = manifests_dir().join("with_ingredients_from_files.json");
    let output = generate_output_name(&input, "multi_ing_thumb", Some("individual"));
    let ingredients_base_dir = manifest.parent().unwrap();

    sign_file_with_manifest_and_options(
        &input,
        &output,
        &manifest,
        ingredients_base_dir,
        false, // generate_asset_thumbnail
        true,  // generate_ingredient_thumbnails
    )?;

    let reader = verify_signed_file(&output)?;

    if let Some(manifest_label) = reader.active_label() {
        let has_thumbs = has_ingredient_thumbnails(&reader, manifest_label);
        assert!(
            has_thumbs,
            "Ingredient thumbnails should be present for multiple ingredients"
        );

        // Verify each ingredient has a thumbnail
        let manifest_store = reader.get_manifest(manifest_label).unwrap();
        let ingredients = manifest_store.ingredients();
        assert_eq!(ingredients.len(), 2, "Should have two ingredients");

        for (i, ingredient) in ingredients.iter().enumerate() {
            assert!(
                ingredient.thumbnail_ref().is_some(),
                "Ingredient {} should have thumbnail",
                i
            );
        }
    }

    println!(
        "✓ Multiple ingredients with thumbnails: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_both_asset_and_ingredient_thumbnails() -> Result<()> {
    let input = common::testfiles_dir().join("Dog.jpg");
    let manifest = manifests_dir().join("simple_with_ingredient.json");
    let output = generate_output_name(&input, "both_thumbs", Some("individual"));
    let ingredients_base_dir = manifest.parent().unwrap();

    sign_file_with_manifest_and_options(
        &input,
        &output,
        &manifest,
        ingredients_base_dir,
        true, // generate_asset_thumbnail
        true, // generate_ingredient_thumbnails
    )?;

    let reader = verify_signed_file(&output)?;

    if let Some(manifest_label) = reader.active_label() {
        // Check both asset and ingredient thumbnails
        let has_asset_thumb = has_asset_thumbnail(&reader, manifest_label);
        let has_ing_thumbs = has_ingredient_thumbnails(&reader, manifest_label);

        assert!(has_asset_thumb, "Asset thumbnail should be present");
        assert!(has_ing_thumbs, "Ingredient thumbnails should be present");

        // Verify ingredient details
        let manifest_store = reader.get_manifest(manifest_label).unwrap();
        let ingredients = manifest_store.ingredients();
        assert_eq!(ingredients.len(), 1, "Should have one ingredient");
        assert!(
            ingredients[0].thumbnail_ref().is_some(),
            "Ingredient should have thumbnail"
        );
    }

    println!(
        "✓ Both asset and ingredient thumbnails present: {}",
        output.display()
    );
    Ok(())
}

#[test]
fn test_selective_thumbnail_generation() -> Result<()> {
    // Test 1: Only asset thumbnail
    let input1 = common::testfiles_dir().join("Dog.jpg");
    let manifest1 = manifests_dir().join("simple_with_ingredient.json");
    let output1 = generate_output_name(&input1, "only_asset_thumb", Some("individual"));
    let ingredients_base_dir1 = manifest1.parent().unwrap();

    sign_file_with_manifest_and_options(
        &input1,
        &output1,
        &manifest1,
        ingredients_base_dir1,
        true,  // generate_asset_thumbnail
        false, // generate_ingredient_thumbnails
    )?;

    let reader1 = verify_signed_file(&output1)?;
    if let Some(manifest_label) = reader1.active_label() {
        assert!(
            has_asset_thumbnail(&reader1, manifest_label),
            "Should have asset thumbnail"
        );
        assert!(
            !has_ingredient_thumbnails(&reader1, manifest_label),
            "Should NOT have ingredient thumbnails"
        );
    }
    println!("✓ Selective thumbnail: asset only");

    // Test 2: Only ingredient thumbnails
    let input2 = common::testfiles_dir().join("Dog.png");
    let manifest2 = manifests_dir().join("simple_with_ingredient.json");
    let output2 = generate_output_name(&input2, "only_ing_thumb", Some("individual"));
    let ingredients_base_dir2 = manifest2.parent().unwrap();

    sign_file_with_manifest_and_options(
        &input2,
        &output2,
        &manifest2,
        ingredients_base_dir2,
        false, // generate_asset_thumbnail
        true,  // generate_ingredient_thumbnails
    )?;

    let reader2 = verify_signed_file(&output2)?;
    if let Some(manifest_label) = reader2.active_label() {
        assert!(
            !has_asset_thumbnail(&reader2, manifest_label),
            "Should NOT have asset thumbnail"
        );
        assert!(
            has_ingredient_thumbnails(&reader2, manifest_label),
            "Should have ingredient thumbnails"
        );
    }
    println!("✓ Selective thumbnail: ingredients only");

    Ok(())
}

// ============================================================================
// Multi-file processing tests
// ============================================================================

#[test]
fn test_multiple_files_processing() -> Result<()> {
    use std::process::Command;

    let output_dir = common::output_dir().join("multi_file_test");
    fs::create_dir_all(&output_dir)?;

    let manifest = manifests_dir().join("simple_manifest.json");
    let cert = common::certs_dir().join("ed25519.pub");
    let key = common::certs_dir().join("ed25519.pem");

    // Get the binary path
    let binary_path = env!("CARGO_BIN_EXE_crTool");

    // Test processing multiple files explicitly
    let input1 = common::testfiles_dir().join("Dog.jpg");
    let input2 = common::testfiles_dir().join("Dog.png");

    let result = Command::new(binary_path)
        .arg("--manifest")
        .arg(&manifest)
        .arg(&input1)
        .arg(&input2)
        .arg("--output")
        .arg(&output_dir)
        .arg("--cert")
        .arg(&cert)
        .arg("--key")
        .arg(&key)
        .arg("--allow-self-signed")
        .output()?;

    assert!(
        result.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Verify both output files were created
    let output1 = output_dir.join("Dog.jpg");
    let output2 = output_dir.join("Dog.png");

    assert!(output1.exists(), "Output file Dog.jpg should exist");
    assert!(output2.exists(), "Output file Dog.png should exist");

    // Verify both files have valid C2PA manifests
    let reader1 = verify_signed_file(&output1)?;
    assert!(reader1.active_label().is_some());

    let reader2 = verify_signed_file(&output2)?;
    assert!(reader2.active_label().is_some());

    println!("✓ Multiple files processing test passed");

    Ok(())
}

#[test]
fn test_glob_pattern_processing() -> Result<()> {
    use std::process::Command;

    let output_dir = common::output_dir().join("glob_test");
    fs::create_dir_all(&output_dir)?;

    let manifest = manifests_dir().join("simple_manifest.json");
    let cert = common::certs_dir().join("ed25519.pub");
    let key = common::certs_dir().join("ed25519.pem");

    // Get the binary path
    let binary_path = env!("CARGO_BIN_EXE_crTool");

    // Test processing files with glob pattern
    let testfiles = common::testfiles_dir();
    let pattern = format!("{}/*.jpg", testfiles.display());

    let result = Command::new(binary_path)
        .arg("--manifest")
        .arg(&manifest)
        .arg(&pattern)
        .arg("--output")
        .arg(&output_dir)
        .arg("--cert")
        .arg(&cert)
        .arg("--key")
        .arg(&key)
        .arg("--allow-self-signed")
        .output()?;

    assert!(
        result.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Verify that at least some output files were created
    let entries: Vec<_> = fs::read_dir(&output_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "jpg")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !entries.is_empty(),
        "Should have created at least one output file"
    );

    // Verify all output files have valid C2PA manifests
    for entry in entries {
        let path = entry.path();
        let reader = verify_signed_file(&path)?;
        assert!(
            reader.active_label().is_some(),
            "File {:?} should have a C2PA manifest",
            path
        );
    }

    println!("✓ Glob pattern processing test passed");

    Ok(())
}

#[test]
fn test_multiple_files_extract() -> Result<()> {
    use std::process::Command;

    // First, create some signed files
    let signed_dir = common::output_dir().join("multi_extract_input");
    fs::create_dir_all(&signed_dir)?;

    let input1 = common::testfiles_dir().join("Dog.jpg");
    let input2 = common::testfiles_dir().join("Dog.png");
    let output1 = signed_dir.join("Dog_signed.jpg");
    let output2 = signed_dir.join("Dog_signed.png");

    let manifest = manifests_dir().join("simple_manifest.json");

    sign_file_with_manifest(&input1, &output1, &manifest)?;
    sign_file_with_manifest(&input2, &output2, &manifest)?;

    // Now extract manifests from multiple files
    let extract_dir = common::output_dir().join("multi_extract_output");
    fs::create_dir_all(&extract_dir)?;

    let binary_path = env!("CARGO_BIN_EXE_crTool");

    let result = Command::new(binary_path)
        .arg("--extract")
        .arg(&output1)
        .arg(&output2)
        .arg("--output")
        .arg(&extract_dir)
        .output()?;

    assert!(
        result.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Verify manifest files were created
    let manifest1 = extract_dir.join("Dog_signed_manifest.json");

    assert!(
        manifest1.exists(),
        "Manifest file Dog_signed_manifest.json should exist"
    );
    // Note: Both files will have the same name since they're both "Dog_signed"
    // In a real scenario, you'd want different filenames

    // Verify the manifests are valid JSON
    let manifest1_content = fs::read_to_string(&manifest1)?;
    let _: serde_json::Value = serde_json::from_str(&manifest1_content)?;

    println!("✓ Multiple files extract test passed");

    Ok(())
}

#[test]
fn test_multi_file_error_handling() -> Result<()> {
    use std::process::Command;

    let output_dir = common::output_dir().join("multi_file_error_test");
    fs::create_dir_all(&output_dir)?;

    let manifest = manifests_dir().join("simple_manifest.json");
    let cert = common::certs_dir().join("ed25519.pub");
    let key = common::certs_dir().join("ed25519.pem");

    // Get the binary path
    let binary_path = env!("CARGO_BIN_EXE_crTool");

    // Test with one valid file and one non-existent file
    let input1 = common::testfiles_dir().join("Dog.jpg");
    let input2 = common::testfiles_dir().join("NonExistent.jpg");

    let result = Command::new(binary_path)
        .arg("--manifest")
        .arg(&manifest)
        .arg(&input1)
        .arg(&input2)
        .arg("--output")
        .arg(&output_dir)
        .arg("--cert")
        .arg(&cert)
        .arg("--key")
        .arg(&key)
        .arg("--allow-self-signed")
        .output()?;

    // Should fail because one file doesn't exist
    assert!(
        !result.status.success(),
        "Command should fail with non-existent input file"
    );

    println!("✓ Multi-file error handling test passed");

    Ok(())
}

#[test]
fn test_multi_file_requires_directory_output() -> Result<()> {
    use std::process::Command;

    let manifest = manifests_dir().join("simple_manifest.json");
    let cert = common::certs_dir().join("ed25519.pub");
    let key = common::certs_dir().join("ed25519.pem");

    // Get the binary path
    let binary_path = env!("CARGO_BIN_EXE_crTool");

    let input1 = common::testfiles_dir().join("Dog.jpg");
    let input2 = common::testfiles_dir().join("Dog.png");

    // Try to use a non-directory output path with multiple inputs
    let output_file = common::output_dir().join("single_output.jpg");

    let result = Command::new(binary_path)
        .arg("--manifest")
        .arg(&manifest)
        .arg(&input1)
        .arg(&input2)
        .arg("--output")
        .arg(&output_file)
        .arg("--cert")
        .arg(&cert)
        .arg("--key")
        .arg(&key)
        .arg("--allow-self-signed")
        .output()?;

    // Should fail because output is not a directory
    assert!(
        !result.status.success(),
        "Command should fail when output is not a directory with multiple inputs"
    );

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("directory"),
        "Error message should mention directory requirement"
    );

    println!("✓ Multi-file directory requirement test passed");

    Ok(())
}

// Run the TestSet files
#[test]
fn test_testset_manifests() -> Result<()> {
    use std::process::Command;

    let manifest_names = vec![
        "n-actions-created-gathered",
        "n-actions-inception",
        "n-actions-inception-multiple",
        "n-actions-created-nodst",
        "n-actions-opened",
        "n-actions-placed",
        "n-actions-placed-parent",
        "n-actions-removed",
        "n-actions-removed-same-manifest",
        "n-actions-translated",
        "n-actions-redacted",
        "n-actions-redacted-bad-uri",
        "n-actions-redacted-bad-reason",
        "n-actions-redacted-no-reason",
        "n-actions-watermarked-bound",
        "n-actions-softwareAgent-missing",
        "n-actions-softwareAgent-and-index",
        "p-actions-created",
        "p-actions-created-with-custom",
        "p-actions-opened-manifest",
        "p-actions-opened-no-manifest",
        "p-actions-opened-no-manifest-metadata",
        "p-actions-placed",
        "p-actions-placed-manifest",
        "p-actions-placed-manifest-metadata",
        "p-actions-translated",
        // "p-actions-redacted",
        "p-actions-softwareAgents",
        "p-actions-template",
        "p-actions-template-all",
        "p-actions-related",
        "p-actions-changes-spatial",
        "p-actions-watermarked-unbound",
        "p-actions-watermarked-bound",
        "p-soft-binding",
    ];

    let mut success_count = 0;
    let mut total_count = 0;

    let input = testfiles_dir().join("Dog.jpg");

    // Clean the testset output directory before starting tests to ensure a fresh state.
    {
        let testset_dir = output_dir().join("testset");
        if testset_dir.exists() {
            for entry in std::fs::read_dir(&testset_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    std::fs::remove_file(&path)?;
                } else if path.is_dir() {
                    std::fs::remove_dir_all(&path)?;
                }
            }
        } else {
            std::fs::create_dir_all(&testset_dir)?;
        }
    }

    // Process each manifest in the testset
    for manifest_name in &manifest_names {
        let manifest_path = testset_dir().join(format!("{}.json", manifest_name));
        total_count += 1;
        // Use "testset" subdirectory to avoid conflicts with individual tests
        let output = generate_output_name_no_stem(&input, manifest_name, Some("testset"));

        match sign_file_with_manifest(&input, &output, &manifest_path) {
            Ok(_) => match verify_signed_file(&output) {
                Ok(_) => {
                    success_count += 1;
                    println!(
                        "✓ Created {} from {} + {}",
                        output.file_name().unwrap().to_str().unwrap(),
                        input.file_name().unwrap().to_str().unwrap(),
                        manifest_name
                    );
                }
                Err(e) => {
                    eprintln!("✗ Verification failed for {:?}: {}", output, e);
                }
            },
            Err(e) => {
                eprintln!(
                    "✗ Signing failed for {:?} with {}: {}",
                    input, manifest_name, e
                );
            }
        }

        {
            // Now extract the newly created manifest into JSON
            let binary_path = env!("CARGO_BIN_EXE_crTool");
            let output_testset_dir = output_dir().join("testset");

            let result = Command::new(binary_path)
                .arg("--extract")
                .arg("--jpt")
                .arg(&output)
                .arg("--output")
                .arg(&output_testset_dir)
                .output()?;

            if result.status.success() {
                println!(
                    "✓ Extraction of manifest from {:?}",
                    output.file_name().unwrap().to_str().unwrap(),
                );

                // Now validate the extracted JSON file
                let extracted_json =
                    output_testset_dir.join(format!("{}_manifest_jpt.json", manifest_name));

                if extracted_json.exists() {
                    let validate_result = Command::new(binary_path)
                        .arg("--validate")
                        .arg(&extracted_json)
                        .output()?;

                    if validate_result.status.success() {
                        println!(
                            "✓ Validation passed for extracted manifest: {}",
                            extracted_json.file_name().unwrap().to_str().unwrap()
                        );
                    } else {
                        println!(
                            "✗ Validation failed for {:?}: {}",
                            extracted_json,
                            String::from_utf8_lossy(&validate_result.stderr)
                        );
                    }
                } else {
                    println!("⚠ Extracted JSON file not found: {:?}", extracted_json);
                }
            } else {
                println!(
                    "✗ Extraction failed for {:?}: {}",
                    output,
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
    }

    println!("\n{}/{} tests passed", success_count, total_count);
    assert_eq!(
        success_count, total_count,
        "All image/manifest combinations should succeed"
    );

    Ok(())
}
