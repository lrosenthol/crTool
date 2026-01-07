use anyhow::{Context, Result};
use c2pa::{create_signer, Builder, CallbackSigner, SigningAlg};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

/// C2PA Testfile Maker - Create and embed C2PA manifests into media assets
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the JSON manifest configuration file
    #[arg(short, long, value_name = "FILE")]
    manifest: PathBuf,

    /// Path to the input media asset (JPEG, PNG, etc.)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Path to the output file or directory
    #[arg(short, long, value_name = "PATH")]
    output: PathBuf,

    /// Path to the certificate file (PEM format)
    #[arg(short, long, value_name = "FILE")]
    cert: PathBuf,

    /// Path to the private key file (PEM format)
    #[arg(short, long, value_name = "FILE")]
    key: PathBuf,

    /// Signing algorithm (es256, es384, es512, ps256, ps384, ps512, ed25519)
    #[arg(short, long, default_value = "es256")]
    algorithm: String,

    /// Allow self-signed certificates (for testing/development only)
    #[arg(long, default_value = "false")]
    allow_self_signed: bool,
}

fn determine_output_path(input: &Path, output: &Path) -> Result<PathBuf> {
    if output.is_dir() {
        let filename = input.file_name().context("Input file has no filename")?;
        Ok(output.join(filename))
    } else {
        Ok(output.to_path_buf())
    }
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

/// Sign data using ECDSA (ES256, ES384, ES512)
fn ecdsa_sign(data: &[u8], private_key: &[u8]) -> c2pa::Result<Vec<u8>> {
    use c2pa::crypto::raw_signature::RawSignerError;
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};
    use p256::pkcs8::DecodePrivateKey;
    use pem::parse;

    // Parse the PEM data to get the private key
    let pem = parse(private_key).map_err(|e| c2pa::Error::OtherError(Box::new(e)))?;

    let signing_key = SigningKey::from_pkcs8_der(pem.contents())
        .map_err(|e: p256::pkcs8::Error| RawSignerError::InternalError(e.to_string()))?;

    // Sign the data
    let signature: Signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

/// Sign data using RSA-PSS (PS256, PS384, PS512)
fn rsa_sign(data: &[u8], private_key: &[u8]) -> c2pa::Result<Vec<u8>> {
    use c2pa::crypto::raw_signature::RawSignerError;
    use pem::parse;
    use rsa::pkcs1v15::SigningKey;
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::sha2::Sha256;
    use rsa::signature::{SignatureEncoding, Signer};
    use rsa::RsaPrivateKey;

    // Parse the PEM data to get the private key
    let pem = parse(private_key).map_err(|e| c2pa::Error::OtherError(Box::new(e)))?;

    let private_key = RsaPrivateKey::from_pkcs8_der(pem.contents())
        .map_err(|e: rsa::pkcs8::Error| RawSignerError::InternalError(e.to_string()))?;

    let signing_key = SigningKey::<Sha256>::new(private_key);

    // Sign the data
    let signature = signing_key.sign(data);
    Ok(signature.to_vec())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Read and parse the JSON manifest configuration
    let manifest_json =
        fs::read_to_string(&cli.manifest).context("Failed to read manifest JSON file")?;

    // Validate input file exists
    if !cli.input.exists() {
        anyhow::bail!("Input file does not exist: {:?}", cli.input);
    }

    // Determine the output path
    let output_path = determine_output_path(&cli.input, &cli.output)?;

    // Create output directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).context("Failed to create output directory")?;
    }

    // Remove existing output file if it exists (to avoid embedding failures)
    if output_path.exists() {
        fs::remove_file(&output_path).context("Failed to remove existing output file")?;
        println!("  Note: Removed existing output file: {:?}", output_path);
    }

    // Parse signing algorithm
    let signing_alg = parse_signing_algorithm(&cli.algorithm)?;

    println!("Creating C2PA manifest...");
    println!("  Input: {:?}", cli.input);
    println!("  Output: {:?}", output_path);
    println!("  Algorithm: {}", cli.algorithm);
    if cli.allow_self_signed {
        println!("  Note: Allowing self-signed certificates (development mode)");
    }

    // Create a builder from the JSON manifest
    let mut builder = Builder::from_json(&manifest_json)
        .context("Failed to create builder from JSON manifest")?;

    // Sign and embed the manifest into the asset
    if cli.allow_self_signed {
        // Use callback signer that bypasses certificate validation
        let signer = create_callback_signer(&cli.cert, &cli.key, signing_alg)
            .context("Failed to create callback signer")?;
        builder
            .sign_file(&signer, &cli.input, &output_path)
            .context("Failed to sign and embed manifest")?;
    } else {
        // Use standard signer with full certificate validation
        let signer = create_signer::from_files(
            cli.cert.to_str().context("Invalid cert path")?,
            cli.key.to_str().context("Invalid key path")?,
            signing_alg,
            None,
        )
        .context("Failed to create signer")?;
        builder
            .sign_file(&*signer, &cli.input, &output_path)
            .context("Failed to sign and embed manifest")?;
    }

    println!("✓ Successfully created and embedded C2PA manifest");
    println!("  Output file: {:?}", output_path);

    Ok(())
}
