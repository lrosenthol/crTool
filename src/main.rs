use anyhow::{Context, Result};
use c2pa::{
    create_signer, Builder, CallbackSigner, Ingredient, JpegTrustReader, Reader, Relationship,
    SigningAlg,
};
use clap::Parser;
use glob::glob;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};

/// C2PA Testfile Maker - Create and embed C2PA manifests into media assets
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the JSON manifest configuration file (not required in extract mode)
    #[arg(short, long, value_name = "FILE")]
    manifest: Option<PathBuf>,

    /// Path(s) to input media asset(s) (JPEG, PNG, etc.). Supports glob patterns (e.g., "*.jpg", "images/*.png")
    #[arg(value_name = "INPUT_FILE", required = true, num_args = 1..)]
    input: Vec<String>,

    /// Path to the output file or directory (required when processing single file, must be directory for multiple files)
    #[arg(short, long, value_name = "PATH")]
    output: PathBuf,

    /// Path to the certificate file (PEM format, not required in extract mode)
    #[arg(short, long, value_name = "FILE")]
    cert: Option<PathBuf>,

    /// Path to the private key file (PEM format, not required in extract mode)
    #[arg(short, long, value_name = "FILE")]
    key: Option<PathBuf>,

    /// Signing algorithm (es256, es384, es512, ps256, ps384, ps512, ed25519)
    /// If not specified, will be auto-detected from the certificate
    #[arg(short, long)]
    algorithm: Option<String>,

    /// Allow self-signed certificates (for testing/development only)
    #[arg(long, default_value = "false")]
    allow_self_signed: bool,

    /// Extract manifest from input file to JSON (read-only mode)
    #[arg(short, long, default_value = "false")]
    extract: bool,

    /// Use JPEG Trust format for extraction (only valid with --extract)
    #[arg(long, default_value = "false")]
    jpt: bool,

    /// Base directory for resolving relative ingredient file paths (defaults to manifest directory)
    #[arg(long, value_name = "DIR")]
    ingredients_dir: Option<PathBuf>,

    /// Generate thumbnail for the main asset
    #[arg(long, default_value = "false")]
    thumbnail_asset: bool,

    /// Generate thumbnails for ingredients
    #[arg(long, default_value = "false")]
    thumbnail_ingredients: bool,
}

/// Configuration for processing files with C2PA manifests
struct ProcessingConfig<'a> {
    manifest_json: &'a str,
    ingredients_base_dir: &'a Path,
    cert: &'a Path,
    key: &'a Path,
    signing_alg: SigningAlg,
    allow_self_signed: bool,
    thumbnail_asset: bool,
    thumbnail_ingredients: bool,
}

/// Expand glob patterns and collect matching file paths
fn expand_input_patterns(patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for pattern in patterns {
        let pattern_path = PathBuf::from(pattern);

        // Check if this is a literal path (not a glob pattern)
        if pattern_path.exists() {
            files.push(pattern_path);
        } else {
            // Try to expand as a glob pattern
            let matches: Vec<PathBuf> = glob(pattern)
                .context(format!("Invalid glob pattern: {}", pattern))?
                .filter_map(|entry: std::result::Result<PathBuf, glob::GlobError>| entry.ok())
                .collect();

            if matches.is_empty() {
                anyhow::bail!("No files match pattern: {}", pattern);
            }

            files.extend(matches);
        }
    }

    // Remove duplicates and sort for consistent processing order
    files.sort();
    files.dedup();

    Ok(files)
}

fn determine_output_path(input: &Path, output: &Path) -> Result<PathBuf> {
    if output.is_dir() {
        let filename = input.file_name().context("Input file has no filename")?;
        Ok(output.join(filename))
    } else {
        Ok(output.to_path_buf())
    }
}

/// Converts a file extension to a MIME type
fn extension_to_mime(extension: &str) -> Option<&'static str> {
    Some(match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "psd" => "image/vnd.adobe.photoshop",
        "tiff" | "tif" => "image/tiff",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "dng" => "image/dng",
        "heic" => "image/heic",
        "heif" => "image/heif",
        "avif" => "image/avif",
        "mp2" | "mpa" | "mpe" | "mpeg" | "mpg" | "mpv2" => "video/mpeg",
        "mp4" => "video/mp4",
        "mov" | "qt" => "video/quicktime",
        "m4a" => "audio/mp4",
        "mid" | "rmi" => "audio/mid",
        "mp3" => "audio/mpeg",
        "wav" => "audio/vnd.wav",
        "aif" | "aifc" | "aiff" => "audio/aiff",
        "ogg" => "audio/ogg",
        "pdf" => "application/pdf",
        "ai" => "application/postscript",
        _ => return None,
    })
}

/// Generate a thumbnail from an image stream
/// Returns (format, thumbnail_bytes)
fn make_thumbnail_from_stream(format: &str, stream: &mut fs::File) -> Result<(String, Vec<u8>)> {
    use image::ImageFormat;

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
    let img =
        image::load(reader, img_format).context("Failed to load image for thumbnail generation")?;

    const THUMBNAIL_SIZE: u32 = 256;
    let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);

    // Encode thumbnail to bytes (always use JPEG for thumbnails)
    let mut buf = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut buf, ImageFormat::Jpeg)
        .context("Failed to encode thumbnail")?;

    Ok(("image/jpeg".to_string(), buf.into_inner()))
}

/// Process ingredients from manifest JSON and add them to the builder
/// Helper function to load an ingredient from a file path
fn load_ingredient_from_file(file_path: &Path, generate_thumbnail: bool) -> Result<Ingredient> {
    if !file_path.exists() {
        anyhow::bail!("Ingredient file not found: {:?}", file_path);
    }

    println!("  Loading ingredient: {:?}", file_path);

    // Load the ingredient file
    let mut source = fs::File::open(file_path)
        .context(format!("Failed to open ingredient file: {:?}", file_path))?;

    // Determine format from file extension
    let extension = file_path
        .extension()
        .and_then(|s| s.to_str())
        .context(format!("Ingredient file has no extension: {:?}", file_path))?;

    let format = extension_to_mime(extension)
        .context(format!("Unsupported ingredient file format: {}", extension))?;

    // Create an Ingredient from the file
    let mut ingredient = Ingredient::from_stream(format, &mut source).context(format!(
        "Failed to create ingredient from file: {:?}",
        file_path
    ))?;

    // Generate thumbnail if requested and not already present
    if generate_thumbnail && ingredient.thumbnail_ref().is_none() {
        use std::io::Seek;
        source.rewind()?;
        let (thumb_format, thumbnail) = make_thumbnail_from_stream(format, &mut source)
            .context("Failed to generate thumbnail for ingredient")?;
        ingredient
            .set_thumbnail(&thumb_format, thumbnail)
            .context("Failed to set thumbnail for ingredient")?;
    }

    Ok(ingredient)
}

/// Returns the number of ingredients processed from files
fn process_ingredients(
    builder: &mut Builder,
    manifest_json: &str,
    ingredients_base_dir: &Path,
    generate_thumbnails: bool,
) -> Result<usize> {
    // Parse the manifest JSON to check for ingredients with file paths
    let manifest: JsonValue =
        serde_json::from_str(manifest_json).context("Failed to parse manifest JSON")?;

    let mut count = 0;

    // Look for "ingredients_from_files" field (detailed ingredient configuration)
    // This field allows loading ingredients from external files while still being able to
    // reference them in actions via an optional instance_id field
    if let Some(ingredients) = manifest
        .get("ingredients_from_files")
        .and_then(|v| v.as_array())
    {
        for ingredient_def in ingredients {
            // All entries in ingredients_from_files must have a file_path
            let file_path_str = ingredient_def
                .get("file_path")
                .and_then(|v| v.as_str())
                .context("Ingredient in ingredients_from_files must have a file_path field")?;

            count += 1;

            // Resolve the file path relative to the base directory
            let file_path = if Path::new(file_path_str).is_absolute() {
                PathBuf::from(file_path_str)
            } else {
                ingredients_base_dir.join(file_path_str)
            };

            let mut ingredient = load_ingredient_from_file(&file_path, generate_thumbnails)?;

            // Set the title if provided in the manifest
            if let Some(title) = ingredient_def.get("title").and_then(|v| v.as_str()) {
                ingredient.set_title(title);
            } else {
                // Use filename as title if not specified
                let filename = file_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown");
                ingredient.set_title(filename);
            }

            // Set the relationship if provided
            if let Some(rel) = ingredient_def.get("relationship").and_then(|v| v.as_str()) {
                let relationship = match rel.to_lowercase().as_str() {
                    "parentof" => Relationship::ParentOf,
                    "componentof" => Relationship::ComponentOf,
                    _ => {
                        anyhow::bail!("Invalid relationship type: {}", rel);
                    }
                };
                ingredient.set_relationship(relationship);
            }

            // Set the label (instance_id) if provided
            // This allows the ingredient to be referenced in actions by this label
            if let Some(label) = ingredient_def.get("label").and_then(|v| v.as_str()) {
                ingredient.set_instance_id(label);
            }

            // Add the ingredient to the builder
            builder.add_ingredient(ingredient);
        }
    }

    Ok(count)
}

fn parse_signing_algorithm(alg: &str) -> Result<SigningAlg> {
    match alg.to_lowercase().as_str() {
        "es256" => Ok(SigningAlg::Es256),
        "es384" => Ok(SigningAlg::Es384),
        "es512" => Ok(SigningAlg::Es512),
        "ps256" => Ok(SigningAlg::Ps256),
        "ps384" => Ok(SigningAlg::Ps384),
        "ps512" => Ok(SigningAlg::Ps512),
        "ed25519" => Ok(SigningAlg::Ed25519),
        _ => anyhow::bail!("Unsupported signing algorithm: {}", alg),
    }
}

/// Detect the signing algorithm from a certificate file
/// This examines the public key type and parameters to determine the appropriate algorithm
fn detect_signing_algorithm(cert_path: &Path) -> Result<SigningAlg> {
    use x509_parser::prelude::*;

    let cert_data = fs::read(cert_path).context("Failed to read certificate file")?;

    // Parse PEM
    let pem = ::pem::parse(&cert_data)
        .map_err(|e| anyhow::anyhow!("Failed to parse certificate PEM: {}", e))?;

    // Parse X.509 certificate
    let (_, cert) = X509Certificate::from_der(pem.contents())
        .map_err(|e| anyhow::anyhow!("Failed to parse X.509 certificate: {}", e))?;

    // Get the public key algorithm
    let public_key = cert.public_key();
    let alg_oid = &public_key.algorithm.algorithm;

    // Detect algorithm based on OID
    match alg_oid.to_id_string().as_str() {
        "1.2.840.10045.2.1" => {
            // EC Public Key - need to check the curve
            if let Some(params) = &public_key.algorithm.parameters {
                let curve_oid = params
                    .as_oid()
                    .map_err(|_| anyhow::anyhow!("Failed to parse curve OID"))?;

                match curve_oid.to_id_string().as_str() {
                    "1.2.840.10045.3.1.7" => Ok(SigningAlg::Es256), // P-256 (secp256r1)
                    "1.3.132.0.34" => Ok(SigningAlg::Es384),        // P-384 (secp384r1)
                    "1.3.132.0.35" => Ok(SigningAlg::Es512),        // P-521 (secp521r1)
                    other => anyhow::bail!("Unsupported EC curve OID: {}", other),
                }
            } else {
                anyhow::bail!("EC key missing curve parameters")
            }
        }
        "1.2.840.113549.1.1.1" => {
            // RSA - default to PS256
            // Could potentially examine key size to choose PS384/PS512, but PS256 is the most common
            Ok(SigningAlg::Ps256)
        }
        "1.3.101.112" => {
            // Ed25519
            Ok(SigningAlg::Ed25519)
        }
        other => anyhow::bail!("Unsupported public key algorithm OID: {}", other),
    }
}

/// Create a callback signer that bypasses certificate validation
/// This is useful for development/testing with self-signed certificates
fn create_callback_signer(
    cert_path: &Path,
    key_path: &Path,
    signing_alg: SigningAlg,
) -> Result<CallbackSigner> {
    // Read certificate and key files
    let cert_data = fs::read(cert_path).context("Failed to read certificate file")?;
    let key_data = fs::read(key_path).context("Failed to read private key file")?;

    // Create the callback signer based on the algorithm
    let signer = match signing_alg {
        SigningAlg::Ed25519 => {
            let ed_signer = move |_context: *const (), data: &[u8]| ed25519_sign(data, &key_data);
            CallbackSigner::new(ed_signer, signing_alg, cert_data)
        }
        SigningAlg::Es256 | SigningAlg::Es384 | SigningAlg::Es512 => {
            let es_signer = move |_context: *const (), data: &[u8]| ecdsa_sign(data, &key_data);
            CallbackSigner::new(es_signer, signing_alg, cert_data)
        }
        SigningAlg::Ps256 | SigningAlg::Ps384 | SigningAlg::Ps512 => {
            let ps_signer = move |_context: *const (), data: &[u8]| rsa_sign(data, &key_data);
            CallbackSigner::new(ps_signer, signing_alg, cert_data)
        }
    };

    Ok(signer)
}

/// Sign data using Ed25519
fn ed25519_sign(data: &[u8], private_key: &[u8]) -> c2pa::Result<Vec<u8>> {
    use c2pa::crypto::raw_signature::RawSignerError;
    use ed25519_dalek::{Signature, Signer, SigningKey};

    // Parse the PEM data to get the private key
    let pem = ::pem::parse(private_key).map_err(|e| c2pa::Error::OtherError(Box::new(e)))?;

    // For Ed25519, the key is 32 bytes long, so we skip the first 16 bytes of the PEM data
    let key_bytes = &pem.contents()[16..];
    let signing_key = SigningKey::try_from(key_bytes)
        .map_err(|e| RawSignerError::InternalError(e.to_string()))?;

    // Sign the data
    let signature: Signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

/// Sign data using ECDSA (ES256, ES384, ES512)
fn ecdsa_sign(data: &[u8], private_key: &[u8]) -> c2pa::Result<Vec<u8>> {
    use c2pa::crypto::raw_signature::RawSignerError;
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};
    use p256::pkcs8::DecodePrivateKey;

    // Parse the PEM data to get the private key
    let pem = ::pem::parse(private_key).map_err(|e| c2pa::Error::OtherError(Box::new(e)))?;

    let signing_key = SigningKey::from_pkcs8_der(pem.contents())
        .map_err(|e: p256::pkcs8::Error| RawSignerError::InternalError(e.to_string()))?;

    // Sign the data
    let signature: Signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

/// Sign data using RSA-PSS (PS256, PS384, PS512)
fn rsa_sign(data: &[u8], private_key: &[u8]) -> c2pa::Result<Vec<u8>> {
    use c2pa::crypto::raw_signature::RawSignerError;
    use rsa::pkcs1v15::SigningKey;
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::sha2::Sha256;
    use rsa::signature::{SignatureEncoding, Signer};
    use rsa::RsaPrivateKey;

    // Parse the PEM data to get the private key
    let pem = ::pem::parse(private_key).map_err(|e| c2pa::Error::OtherError(Box::new(e)))?;

    let private_key = RsaPrivateKey::from_pkcs8_der(pem.contents())
        .map_err(|e: rsa::pkcs8::Error| RawSignerError::InternalError(e.to_string()))?;

    let signing_key = SigningKey::<Sha256>::new(private_key);

    // Sign the data
    let signature = signing_key.sign(data);
    Ok(signature.to_vec())
}

/// Extract C2PA manifest from a file and save it as JSON
fn extract_manifest(input_path: &Path, output_path: &Path, use_jpt_format: bool) -> Result<()> {
    // Validate input file exists
    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {:?}", input_path);
    }

    println!("Extracting C2PA manifest...");
    println!("  Input: {:?}", input_path);
    if use_jpt_format {
        println!("  Format: JPEG Trust");
    }

    // Get the manifest JSON string based on format
    let manifest_json = if use_jpt_format {
        // Use JPEG Trust Reader
        let mut jpt_reader = JpegTrustReader::from_file(input_path).context(
            "Failed to read C2PA data from input file. The file may not contain a C2PA manifest.",
        )?;

        // Compute asset hash to include asset_info in the output
        if let Ok(hash) = jpt_reader.compute_asset_hash_from_file(input_path) {
            println!("  Asset hash computed: {}", hash);
        }

        // Get the active manifest
        let active_label = jpt_reader
            .inner()
            .active_label()
            .context("No active C2PA manifest found in the input file")?;

        println!("  Active manifest label: {}", active_label);

        jpt_reader.json()
    } else {
        // Use standard Reader
        let reader = Reader::from_file(input_path).context(
            "Failed to read C2PA data from input file. The file may not contain a C2PA manifest.",
        )?;

        // Get the active manifest
        let active_label = reader
            .active_label()
            .context("No active C2PA manifest found in the input file")?;

        println!("  Active manifest label: {}", active_label);

        reader.json()
    };

    // Determine the final output path
    let final_output_path = if output_path.is_dir() {
        // If output is a directory, create a filename based on the input
        let input_stem = input_path
            .file_stem()
            .context("Input file has no filename")?
            .to_str()
            .context("Invalid UTF-8 in filename")?;
        let suffix = if use_jpt_format {
            "_manifest_jpt.json"
        } else {
            "_manifest.json"
        };
        output_path.join(format!("{}{}", input_stem, suffix))
    } else {
        output_path.to_path_buf()
    };

    // Create output directory if it doesn't exist
    if let Some(parent) = final_output_path.parent() {
        fs::create_dir_all(parent).context("Failed to create output directory")?;
    }

    // Parse and re-serialize the JSON for pretty formatting
    let json_value: serde_json::Value =
        serde_json::from_str(&manifest_json).context("Failed to parse manifest JSON")?;
    let pretty_json = serde_json::to_string_pretty(&json_value).context("Failed to format JSON")?;

    fs::write(&final_output_path, pretty_json)
        .context("Failed to write manifest JSON to output file")?;

    println!("✓ Successfully extracted C2PA manifest");
    println!("  Output file: {:?}", final_output_path);

    Ok(())
}

/// Process a single input file with the manifest
fn process_single_file(
    input_path: &Path,
    output_path: &Path,
    config: &ProcessingConfig,
) -> Result<()> {
    println!("\n=== Processing: {:?} ===", input_path);

    // Validate input file exists
    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {:?}", input_path);
    }

    // Determine the output path
    let final_output_path = determine_output_path(input_path, output_path)?;

    // Create output directory if it doesn't exist
    if let Some(parent) = final_output_path.parent() {
        fs::create_dir_all(parent).context("Failed to create output directory")?;
    }

    // Remove existing output file if it exists (to avoid embedding failures)
    if final_output_path.exists() {
        fs::remove_file(&final_output_path).context("Failed to remove existing output file")?;
        println!(
            "  Note: Removed existing output file: {:?}",
            final_output_path
        );
    }

    println!("  Input: {:?}", input_path);
    println!("  Output: {:?}", final_output_path);

    // Create a builder from the JSON manifest
    let mut builder = Builder::from_json(config.manifest_json)
        .context("Failed to create builder from JSON manifest")?;

    // Process any ingredients with file paths
    let ingredient_count = process_ingredients(
        &mut builder,
        config.manifest_json,
        config.ingredients_base_dir,
        config.thumbnail_ingredients,
    )
    .context("Failed to process ingredients")?;

    if ingredient_count > 0 {
        println!("  Processed {} ingredient(s) from files", ingredient_count);
        if config.thumbnail_ingredients {
            println!("  Generated thumbnails for ingredients");
        }
    }

    // Generate thumbnail for the asset if requested
    if config.thumbnail_asset {
        println!("  Generating thumbnail for main asset...");
        let mut input_file = fs::File::open(input_path)
            .context("Failed to open input file for thumbnail generation")?;

        // Determine format from input file extension
        let input_extension = input_path
            .extension()
            .and_then(|s| s.to_str())
            .context("Input file has no extension")?;

        let input_format = extension_to_mime(input_extension)
            .context("Unsupported input file format for thumbnail")?;

        let (thumb_format, thumbnail) = make_thumbnail_from_stream(input_format, &mut input_file)
            .context("Failed to generate thumbnail for main asset")?;

        builder
            .set_thumbnail(&thumb_format, &mut Cursor::new(thumbnail))
            .context("Failed to set thumbnail for main asset")?;
    }

    // Sign and embed the manifest into the asset
    if config.allow_self_signed {
        // Use callback signer that bypasses certificate validation
        let signer = create_callback_signer(config.cert, config.key, config.signing_alg)
            .context("Failed to create callback signer")?;
        builder
            .sign_file(&signer, input_path, &final_output_path)
            .context("Failed to sign and embed manifest")?;
    } else {
        // Use standard signer with full certificate validation
        let signer = create_signer::from_files(
            config.cert.to_str().context("Invalid cert path")?,
            config.key.to_str().context("Invalid key path")?,
            config.signing_alg,
            None,
        )
        .context("Failed to create signer")?;
        builder
            .sign_file(&*signer, input_path, &final_output_path)
            .context("Failed to sign and embed manifest")?;
    }

    println!("✓ Successfully created and embedded C2PA manifest");
    println!("  Output file: {:?}", final_output_path);

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Expand glob patterns and collect all input files
    let input_files =
        expand_input_patterns(&cli.input).context("Failed to expand input file patterns")?;

    if input_files.is_empty() {
        anyhow::bail!("No input files specified");
    }

    // Validate input files exist
    for input_file in &input_files {
        if !input_file.exists() {
            anyhow::bail!("Input file does not exist: {:?}", input_file);
        }
    }

    println!("Found {} input file(s) to process", input_files.len());

    // Handle extract mode
    if cli.extract {
        // Validate --jpt can only be used with --extract
        if cli.jpt {
            println!("Using JPEG Trust format for extraction");
        }

        // Output must be a directory if processing multiple files
        if input_files.len() > 1 && !cli.output.is_dir() {
            anyhow::bail!(
                "Output must be a directory when extracting from multiple input files. Got: {:?}",
                cli.output
            );
        }

        // Process each file
        let mut success_count = 0;
        let mut error_count = 0;

        for input_file in &input_files {
            match extract_manifest(input_file, &cli.output, cli.jpt) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("Error processing {:?}: {}", input_file, e);
                    error_count += 1;
                }
            }
        }

        println!("\n=== Extraction Summary ===");
        println!("  Successful: {}", success_count);
        println!("  Failed: {}", error_count);
        println!("  Total: {}", input_files.len());

        if error_count > 0 {
            anyhow::bail!("{} file(s) failed to extract", error_count);
        }

        return Ok(());
    }

    // Validate --jpt can only be used with --extract
    if cli.jpt {
        anyhow::bail!("--jpt can only be used with --extract mode");
    }

    // Normal signing mode - validate required arguments
    let manifest = cli
        .manifest
        .context("--manifest is required when not in extract mode")?;
    let cert = cli
        .cert
        .context("--cert is required when not in extract mode")?;
    let key = cli
        .key
        .context("--key is required when not in extract mode")?;

    // Output must be a directory if processing multiple files
    if input_files.len() > 1 && !cli.output.is_dir() {
        anyhow::bail!(
            "Output must be a directory when processing multiple input files. Got: {:?}",
            cli.output
        );
    }

    // Read and parse the JSON manifest configuration
    let manifest_json =
        fs::read_to_string(&manifest).context("Failed to read manifest JSON file")?;

    // Determine the ingredients base directory
    // Use the provided ingredients_dir, or default to the manifest's parent directory
    let ingredients_base_dir = if let Some(ing_dir) = cli.ingredients_dir {
        ing_dir
    } else {
        manifest
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    println!("  Ingredients base directory: {:?}", ingredients_base_dir);

    // Auto-detect or parse signing algorithm
    let signing_alg = if let Some(alg_str) = &cli.algorithm {
        parse_signing_algorithm(alg_str)?
    } else {
        println!("Auto-detecting signing algorithm from certificate...");
        let detected = detect_signing_algorithm(&cert)?;
        println!("  Detected: {:?}", detected);
        detected
    };

    println!("Creating C2PA manifest(s)...");
    println!("  Algorithm: {:?}", signing_alg);
    if cli.allow_self_signed {
        println!("  Note: Allowing self-signed certificates (development mode)");
    }

    // Create processing configuration
    let config = ProcessingConfig {
        manifest_json: &manifest_json,
        ingredients_base_dir: &ingredients_base_dir,
        cert: &cert,
        key: &key,
        signing_alg,
        allow_self_signed: cli.allow_self_signed,
        thumbnail_asset: cli.thumbnail_asset,
        thumbnail_ingredients: cli.thumbnail_ingredients,
    };

    // Process each input file
    let mut success_count = 0;
    let mut error_count = 0;

    for input_file in &input_files {
        match process_single_file(input_file, &cli.output, &config) {
            Ok(_) => success_count += 1,
            Err(e) => {
                eprintln!("Error processing {:?}: {}", input_file, e);
                error_count += 1;
            }
        }
    }

    println!("\n=== Processing Summary ===");
    println!("  Successful: {}", success_count);
    println!("  Failed: {}", error_count);
    println!("  Total: {}", input_files.len());

    if error_count > 0 {
        anyhow::bail!("{} file(s) failed to process", error_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_signing_algorithm_ed25519() {
        // Test with the ed25519 test certificate
        let cert_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/certs/ed25519.pub");

        if cert_path.exists() {
            let result = detect_signing_algorithm(&cert_path);
            assert!(
                result.is_ok(),
                "Should successfully detect Ed25519 algorithm"
            );
            assert_eq!(result.unwrap(), SigningAlg::Ed25519);
        }
    }

    #[test]
    fn test_parse_signing_algorithm() {
        assert_eq!(parse_signing_algorithm("es256").unwrap(), SigningAlg::Es256);
        assert_eq!(parse_signing_algorithm("ES256").unwrap(), SigningAlg::Es256);
        assert_eq!(parse_signing_algorithm("es384").unwrap(), SigningAlg::Es384);
        assert_eq!(parse_signing_algorithm("es512").unwrap(), SigningAlg::Es512);
        assert_eq!(parse_signing_algorithm("ps256").unwrap(), SigningAlg::Ps256);
        assert_eq!(parse_signing_algorithm("ps384").unwrap(), SigningAlg::Ps384);
        assert_eq!(parse_signing_algorithm("ps512").unwrap(), SigningAlg::Ps512);
        assert_eq!(
            parse_signing_algorithm("ed25519").unwrap(),
            SigningAlg::Ed25519
        );

        assert!(parse_signing_algorithm("invalid").is_err());
    }
}
