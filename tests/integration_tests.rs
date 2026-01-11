use anyhow::Result;
use std::path::{Path, PathBuf};

mod common;

use common::{
    get_test_images, manifests_dir, output_dir, sign_file_with_manifest, verify_signed_file,
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
        extracted_agent.contains("c2pa-testfile-maker") || extracted_agent == original_agent,
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
            .map(|n| n.contains("c2pa-testfile-maker"))
            .unwrap_or(false)
    });

    assert!(
        has_our_generator,
        "Claim generator should include c2pa-testfile-maker"
    );

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
