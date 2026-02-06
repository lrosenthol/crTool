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
use c2pa::{
    Builder, CallbackSigner, Ingredient, JpegTrustReader, Reader, Relationship, SigningAlg,
};
use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Path to the crTool CLI binary (when built as part of the workspace).
pub fn cli_binary_path() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_CRTOOL") {
        return PathBuf::from(path);
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let exe = if cfg!(windows) {
        "crTool.exe"
    } else {
        "crTool"
    };
    manifest_dir.join("target").join(profile).join(exe)
}

/// Test helper to get the path to test fixtures
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Test helper to get the path to test certificates
pub fn certs_dir() -> PathBuf {
    fixtures_dir().join("certs")
}

/// Test helper to get the path to test images
pub fn testfiles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testfiles")
}

/// Test helper to get the path to manifest examples
pub fn manifests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples")
}

/// Test helper to get the path to testset
#[allow(dead_code)]
pub fn testset_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testset")
}

/// Test helper to create output directory for test artifacts
#[allow(dead_code)]
pub fn output_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test_output");
    fs::create_dir_all(&dir).expect("Failed to create test output directory");
    dir
}

/// Collect resource identifiers (icon, thumbnail, data) from manifest JSON so we can load
/// them from the manifest's directory and add to the builder before signing.
fn collect_manifest_resource_identifiers(manifest: &serde_json::Value) -> HashSet<String> {
    let mut ids = HashSet::new();
    // claim_generator_info: array or single object with optional icon.identifier
    if let Some(cgi) = manifest.get("claim_generator_info") {
        let entries: Vec<&serde_json::Value> = cgi
            .as_array()
            .map(|a| a.iter().collect())
            .unwrap_or_else(|| vec![cgi]);
        for entry in entries {
            if let Some(icon) = entry.get("icon").and_then(|i| i.get("identifier")) {
                if let Some(s) = icon.as_str() {
                    ids.insert(s.to_string());
                }
            }
        }
    }
    // thumbnail: { identifier }
    if let Some(t) = manifest.get("thumbnail").and_then(|t| t.get("identifier")) {
        if let Some(s) = t.as_str() {
            ids.insert(s.to_string());
        }
    }
    // ingredients[].thumbnail.identifier and ingredients[].data.identifier
    if let Some(ingredients) = manifest.get("ingredients").and_then(|v| v.as_array()) {
        for ing in ingredients {
            if let Some(t) = ing.get("thumbnail").and_then(|t| t.get("identifier")) {
                if let Some(s) = t.as_str() {
                    ids.insert(s.to_string());
                }
            }
            if let Some(d) = ing.get("data").and_then(|d| d.get("identifier")) {
                if let Some(s) = d.as_str() {
                    ids.insert(s.to_string());
                }
            }
        }
    }
    // assertions[].data.templates[].icon.identifier (c2pa.actions.v2 action template icons)
    if let Some(assertions) = manifest.get("assertions").and_then(|v| v.as_array()) {
        for assertion in assertions {
            let data = assertion.get("data").and_then(|d| d.as_object());
            if let Some(templates) = data
                .and_then(|d| d.get("templates"))
                .and_then(|t| t.as_array())
            {
                for template in templates {
                    if let Some(icon) = template.get("icon").and_then(|i| i.get("identifier")) {
                        if let Some(s) = icon.as_str() {
                            ids.insert(s.to_string());
                        }
                    }
                }
            }
        }
    }
    ids
}

/// Load manifest-referenced resources (icons, thumbnails, etc.) from a base directory
/// and add them to the builder. Paths are resolved relative to base_dir.
fn add_manifest_resources_from_dir(
    builder: &mut Builder,
    manifest_json: &str,
    base_dir: &Path,
) -> Result<()> {
    let manifest: serde_json::Value = serde_json::from_str(manifest_json)?;
    let identifiers = collect_manifest_resource_identifiers(&manifest);
    for id in identifiers {
        let path = base_dir.join(&id);
        if path.exists() && path.is_file() {
            let data = fs::read(&path)?;
            builder.add_resource(&id, Cursor::new(data))?;
        }
    }
    Ok(())
}

/// Helper function to sign a file with a manifest
#[allow(dead_code)]
pub fn sign_file_with_manifest(
    input_path: &Path,
    output_path: &Path,
    manifest_path: &Path,
) -> Result<()> {
    // Remove output file if it already exists
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    // Read the manifest JSON
    let manifest_json = fs::read_to_string(manifest_path)?;

    // Create the builder from JSON
    let mut builder = Builder::from_json(&manifest_json)?;

    // Use the manifest's directory as the base for resolving ingredient and resource paths
    let ingredients_base_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

    // Load any resources (e.g. claim_generator_info icon) referenced by identifier
    add_manifest_resources_from_dir(&mut builder, &manifest_json, ingredients_base_dir)?;

    // Process any ingredients_from_files that may be in the manifest
    process_ingredients_with_thumbnails(&mut builder, &manifest_json, ingredients_base_dir, false)?;

    // Use the same test signer approach as c2pa-rs tests
    let signer = test_signer();

    // Sign and embed
    builder.sign_file(&signer, input_path, output_path)?;

    Ok(())
}

/// Helper function to sign a file with a manifest that includes file-based ingredients
/// This processes ingredients with file_path fields
#[allow(dead_code)]
pub fn sign_file_with_manifest_and_ingredients(
    input_path: &Path,
    output_path: &Path,
    manifest_path: &Path,
    ingredients_base_dir: &Path,
) -> Result<()> {
    sign_file_with_manifest_and_ingredients_impl(
        input_path,
        output_path,
        manifest_path,
        ingredients_base_dir,
        false,
        false,
    )
}
#[allow(dead_code)]
pub fn sign_file_with_manifest_and_options(
    input_path: &Path,
    output_path: &Path,
    manifest_path: &Path,
    ingredients_base_dir: &Path,
    generate_asset_thumbnail: bool,
    generate_ingredient_thumbnails: bool,
) -> Result<()> {
    sign_file_with_manifest_and_ingredients_impl(
        input_path,
        output_path,
        manifest_path,
        ingredients_base_dir,
        generate_asset_thumbnail,
        generate_ingredient_thumbnails,
    )
}

/// Internal implementation for signing files with ingredients and thumbnails
#[allow(dead_code)]
fn sign_file_with_manifest_and_ingredients_impl(
    input_path: &Path,
    output_path: &Path,
    manifest_path: &Path,
    ingredients_base_dir: &Path,
    generate_asset_thumbnail: bool,
    generate_ingredient_thumbnails: bool,
) -> Result<()> {
    use std::io::Cursor;
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    // Read the manifest JSON
    let manifest_json = fs::read_to_string(manifest_path)?;

    // Create the builder from JSON
    let mut builder = Builder::from_json(&manifest_json)?;

    // Load any resources (e.g. claim_generator_info icon) referenced by identifier
    let manifest_base_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    add_manifest_resources_from_dir(&mut builder, &manifest_json, manifest_base_dir)?;

    // Process ingredients with file paths
    process_ingredients_with_thumbnails(
        &mut builder,
        &manifest_json,
        ingredients_base_dir,
        generate_ingredient_thumbnails,
    )?;

    // Generate thumbnail for the asset if requested
    if generate_asset_thumbnail {
        let mut input_file = fs::File::open(input_path)?;

        // Determine format from input file extension
        let input_extension = input_path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Input file has no extension"))?;

        let input_format = extension_to_mime(input_extension)
            .ok_or_else(|| anyhow::anyhow!("Unsupported input file format"))?;

        // Generate thumbnail
        let (thumb_format, thumbnail) = make_thumbnail_from_stream(input_format, &mut input_file)?;

        builder.set_thumbnail(&thumb_format, &mut Cursor::new(thumbnail))?;
    }

    // Use the same test signer approach as c2pa-rs tests
    let signer = test_signer();

    // Sign and embed
    builder.sign_file(&signer, input_path, output_path)?;

    Ok(())
}

/// Process ingredients from manifest JSON and add them to the builder with optional thumbnails
#[allow(dead_code)]
fn process_ingredients_with_thumbnails(
    builder: &mut Builder,
    manifest_json: &str,
    ingredients_base_dir: &Path,
    generate_thumbnails: bool,
) -> Result<()> {
    use serde_json::Value as JsonValue;
    use std::io::Seek;

    // Parse the manifest JSON to check for ingredients with file paths
    let manifest: JsonValue = serde_json::from_str(manifest_json)?;

    // Look for "ingredients_from_files" field
    if let Some(ingredients) = manifest
        .get("ingredients_from_files")
        .and_then(|v| v.as_array())
    {
        for ingredient_def in ingredients {
            // All entries in ingredients_from_files must have a file_path
            let file_path_str = ingredient_def
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing file_path in ingredient"))?;

            // Resolve the file path relative to the base directory
            let file_path = if Path::new(file_path_str).is_absolute() {
                PathBuf::from(file_path_str)
            } else {
                ingredients_base_dir.join(file_path_str)
            };

            if !file_path.exists() {
                anyhow::bail!("Ingredient file not found: {:?}", file_path);
            }

            // Load the ingredient file
            let mut source = fs::File::open(&file_path)?;

            // Determine format from file extension
            let extension = file_path
                .extension()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("Ingredient file has no extension"))?;

            let format = extension_to_mime(extension)
                .ok_or_else(|| anyhow::anyhow!("Unsupported ingredient file format"))?;

            // Create an Ingredient from the file
            let mut ingredient = Ingredient::from_stream(format, &mut source)?;

            // Set the title if provided in the manifest
            if let Some(title) = ingredient_def.get("title").and_then(|v| v.as_str()) {
                ingredient.set_title(title);
            }

            // Set the relationship if provided
            if let Some(rel) = ingredient_def.get("relationship").and_then(|v| v.as_str()) {
                let relationship = match rel.to_lowercase().as_str() {
                    "parentof" => Relationship::ParentOf,
                    "componentof" => Relationship::ComponentOf,
                    _ => anyhow::bail!("Invalid relationship type: {}", rel),
                };
                ingredient.set_relationship(relationship);
            }

            // Set the label (instance_id) if provided
            // This allows the ingredient to be referenced in actions by this label
            if let Some(label) = ingredient_def.get("label").and_then(|v| v.as_str()) {
                ingredient.set_instance_id(label);
            }

            // Set metadata if provided
            // This supports both standard C2PA AssertionMetadata fields and arbitrary custom fields
            if let Some(metadata_obj) = ingredient_def.get("metadata") {
                if let Some(metadata_map) = metadata_obj.as_object() {
                    use c2pa::assertions::AssertionMetadata;
                    let mut assertion_metadata = AssertionMetadata::new();

                    // Iterate through all key-value pairs in the metadata object
                    for (key, value) in metadata_map {
                        // Use set_field to add arbitrary key/value pairs
                        // This will work for custom fields like "com.adobe.repo.asset-id"
                        assertion_metadata = assertion_metadata.set_field(key, value.clone());
                    }

                    ingredient.set_metadata(assertion_metadata);
                }
            }

            // Generate thumbnail if requested and not already present
            if generate_thumbnails && ingredient.thumbnail_ref().is_none() {
                source.rewind()?;
                let (thumb_format, thumbnail) = make_thumbnail_from_stream(format, &mut source)?;
                ingredient.set_thumbnail(&thumb_format, thumbnail)?;
            }

            // Add the ingredient to the builder
            builder.add_ingredient(ingredient);
        }
    }

    Ok(())
}

/// Converts a file extension to a MIME type
#[allow(dead_code)]
fn extension_to_mime(extension: &str) -> Option<&'static str> {
    Some(match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "tiff" | "tif" => "image/tiff",
        "bmp" => "image/bmp",
        _ => return None,
    })
}

/// Generate a thumbnail from an image stream
/// Returns (format, thumbnail_bytes)
#[allow(dead_code)]
fn make_thumbnail_from_stream(format: &str, stream: &mut fs::File) -> Result<(String, Vec<u8>)> {
    use image::ImageFormat;
    use std::io::{BufReader, Cursor};

    // Determine image format from MIME type
    let img_format = match format {
        "image/jpeg" => ImageFormat::Jpeg,
        "image/png" => ImageFormat::Png,
        "image/gif" => ImageFormat::Gif,
        "image/bmp" => ImageFormat::Bmp,
        "image/tiff" => ImageFormat::Tiff,
        "image/webp" => ImageFormat::WebP,
        _ => ImageFormat::Jpeg, // Default to JPEG for unknown formats
    };

    // Wrap in BufReader for image loading
    let reader = BufReader::new(stream);

    // Load and resize the image
    let img = image::load(reader, img_format)?;

    const THUMBNAIL_SIZE: u32 = 256;
    let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);

    // Encode thumbnail to bytes (always use JPEG for thumbnails)
    let mut buf = Cursor::new(Vec::new());
    thumbnail.write_to(&mut buf, ImageFormat::Jpeg)?;

    Ok(("image/jpeg".to_string(), buf.into_inner()))
}

/// Create a test signer using Ed25519 (same as c2pa-rs test infrastructure)
/// This uses the Ed25519 certificates from c2pa-rs which pass all validation
fn test_signer() -> CallbackSigner {
    const CERTS: &[u8] = include_bytes!("../fixtures/certs/ed25519.pub");
    const PRIVATE_KEY: &[u8] = include_bytes!("../fixtures/certs/ed25519.pem");

    let ed_signer = |_context: *const (), data: &[u8]| ed_sign(data, PRIVATE_KEY);
    CallbackSigner::new(ed_signer, SigningAlg::Ed25519, CERTS)
        .set_context("test" as *const _ as *const ())
}

fn ed_sign(data: &[u8], private_key: &[u8]) -> c2pa::Result<Vec<u8>> {
    use c2pa::crypto::raw_signature::RawSignerError;
    use ed25519_dalek::{Signature, Signer, SigningKey};
    use pem::parse;

    // Parse the PEM data to get the private key
    let pem = parse(private_key).map_err(|e| c2pa::Error::OtherError(Box::new(e)))?;

    // For Ed25519, the key is 32 bytes long, so we skip the first 16 bytes of the PEM data
    let key_bytes = &pem.contents()[16..];
    let signing_key = SigningKey::try_from(key_bytes)
        .map_err(|e| RawSignerError::InternalError(e.to_string()))?;

    // Sign the data
    let signature: Signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

/// Helper function to verify a signed file has a valid manifest
#[allow(dead_code)]
pub fn verify_signed_file(file_path: &Path) -> Result<Reader> {
    let reader = Reader::from_file(file_path)?;

    // Ensure we have an active manifest
    assert!(
        reader.active_label().is_some(),
        "File should have an active manifest"
    );

    Ok(reader)
}

/// Helper to get all test image files
pub fn get_test_images() -> Vec<PathBuf> {
    let testfiles = testfiles_dir();
    vec![
        testfiles.join("Dog.jpg"),
        testfiles.join("Dog.png"),
        testfiles.join("Dog.webp"),
    ]
}

/// Check if a manifest has an asset thumbnail assertion
/// Note: Asset thumbnails are stored in the manifest JSON, not necessarily as assertions
#[allow(dead_code)]
pub fn has_asset_thumbnail(reader: &Reader, manifest_label: &str) -> bool {
    if let Some(manifest) = reader.get_manifest(manifest_label) {
        // Check the manifest JSON for thumbnail references
        // Asset thumbnails appear at the manifest level, while ingredient thumbnails
        // appear under ingredients
        let json = reader.json();

        // Parse the JSON to check for manifest-level thumbnail
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
            // Check if there's a manifest with a thumbnail field at the top level
            if let Some(manifests) = value.get("manifests").and_then(|m| m.as_object()) {
                for (_label, manifest_data) in manifests {
                    if manifest_data.get("thumbnail").is_some() {
                        return true;
                    }
                }
            }

            // Also check for thumbnail assertions (claim thumbnails)
            let assertions = manifest.assertions();
            if assertions
                .iter()
                .any(|a| a.label().starts_with("c2pa.thumbnail.claim"))
            {
                return true;
            }
        }

        false
    } else {
        false
    }
}

/// Check if any ingredients have thumbnails
#[allow(dead_code)]
pub fn has_ingredient_thumbnails(reader: &Reader, manifest_label: &str) -> bool {
    if let Some(manifest) = reader.get_manifest(manifest_label) {
        let ingredients = manifest.ingredients();
        ingredients.iter().any(|ing| ing.thumbnail_ref().is_some())
    } else {
        false
    }
}

/// Helper function to extract manifest from a signed file
#[allow(dead_code)]
pub fn extract_manifest_to_file(input_path: &Path, output_path: &Path) -> Result<()> {
    extract_manifest_impl(input_path, output_path, false)
}

/// Helper function to extract manifest from a signed file in JPEG Trust format
#[allow(dead_code)]
pub fn extract_manifest_to_file_jpt(input_path: &Path, output_path: &Path) -> Result<()> {
    extract_manifest_impl(input_path, output_path, true)
}

/// Internal implementation for extracting manifests
fn extract_manifest_impl(input_path: &Path, output_path: &Path, use_jpt: bool) -> Result<()> {
    // Remove output file if it already exists
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    // Get the manifest JSON based on format
    let manifest_json = if use_jpt {
        let mut jpt_reader = JpegTrustReader::from_file(input_path)?;

        // Compute asset hash for JPEG Trust format
        let _ = jpt_reader.compute_asset_hash_from_file(input_path);

        // Ensure there's an active manifest
        let _active_label = jpt_reader
            .inner()
            .active_label()
            .ok_or_else(|| anyhow::anyhow!("No active C2PA manifest found"))?;

        jpt_reader.json()
    } else {
        let reader = Reader::from_file(input_path)?;

        // Ensure there's an active manifest
        let _active_label = reader
            .active_label()
            .ok_or_else(|| anyhow::anyhow!("No active C2PA manifest found"))?;

        reader.json()
    };

    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Parse and re-serialize the JSON for pretty formatting
    let json_value: serde_json::Value = serde_json::from_str(&manifest_json)?;
    let pretty_json = serde_json::to_string_pretty(&json_value)?;

    fs::write(output_path, pretty_json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_exist() {
        assert!(testfiles_dir().exists(), "testfiles directory should exist");
        assert!(manifests_dir().exists(), "examples directory should exist");
        assert!(certs_dir().exists(), "test certs directory should exist");
    }

    #[test]
    fn test_images_exist() {
        for img in get_test_images() {
            assert!(img.exists(), "Test image should exist: {:?}", img);
        }
    }

    #[test]
    fn test_manifests_exist() {
        let simple = manifests_dir().join("simple_manifest.json");
        let full = manifests_dir().join("full_manifest.json");
        let asset_ref = manifests_dir().join("asset_ref_manifest.json");
        let asset_type = manifests_dir().join("asset_type_manifest.json");
        let cloud_data = manifests_dir().join("cloud_data_manifest.json");
        let depthmap_gdepth = manifests_dir().join("depthmap_gdepth_manifest.json");
        let external_reference = manifests_dir().join("external_reference_manifest.json");
        let actions_v2_edited = manifests_dir().join("actions_v2_edited_manifest.json");
        let actions_v2_translated = manifests_dir().join("actions_v2_translated_manifest.json");
        let actions_v2_redacted = manifests_dir().join("actions_v2_redacted_manifest.json");
        let actions_v2_cropped = manifests_dir().join("actions_v2_cropped_manifest.json");
        let actions_v2_filtered = manifests_dir().join("actions_v2_filtered_manifest.json");

        assert!(simple.exists(), "simple_manifest.json should exist");
        assert!(full.exists(), "full_manifest.json should exist");
        assert!(asset_ref.exists(), "asset_ref_manifest.json should exist");
        assert!(asset_type.exists(), "asset_type_manifest.json should exist");
        assert!(cloud_data.exists(), "cloud_data_manifest.json should exist");
        assert!(
            depthmap_gdepth.exists(),
            "depthmap_gdepth_manifest.json should exist"
        );
        assert!(
            external_reference.exists(),
            "external_reference_manifest.json should exist"
        );
        assert!(
            actions_v2_edited.exists(),
            "actions_v2_edited_manifest.json should exist"
        );
        assert!(
            actions_v2_translated.exists(),
            "actions_v2_translated_manifest.json should exist"
        );
        assert!(
            actions_v2_redacted.exists(),
            "actions_v2_redacted_manifest.json should exist"
        );
        assert!(
            actions_v2_cropped.exists(),
            "actions_v2_cropped_manifest.json should exist"
        );
        assert!(
            actions_v2_filtered.exists(),
            "actions_v2_filtered_manifest.json should exist"
        );
    }

    #[test]
    fn test_certs_exist() {
        let cert = certs_dir().join("es256_cert.pem");
        let key = certs_dir().join("es256_private.pem");

        assert!(cert.exists(), "Test certificate should exist");
        assert!(key.exists(), "Test private key should exist");
    }
}
