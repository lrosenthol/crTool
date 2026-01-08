use anyhow::{Context, Result};
use c2pa::{create_signer, Builder, CallbackSigner, Reader, SigningAlg};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

/// C2PA Testfile Maker - Create and embed C2PA manifests into media assets
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the JSON manifest configuration file (not required in extract mode)
    #[arg(short, long, value_name = "FILE")]
    manifest: Option<PathBuf>,

    /// Path to the input media asset (JPEG, PNG, etc.)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Path to the output file or directory
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
fn extract_manifest(input_path: &Path, output_path: &Path) -> Result<()> {
    // Validate input file exists
    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {:?}", input_path);
    }

    println!("Extracting C2PA manifest...");
    println!("  Input: {:?}", input_path);

    // Create a Reader from the input file
    let reader = Reader::from_file(input_path).context(
        "Failed to read C2PA data from input file. The file may not contain a C2PA manifest.",
    )?;

    // Get the active manifest
    let active_label = reader
        .active_label()
        .context("No active C2PA manifest found in the input file")?;

    println!("  Active manifest label: {}", active_label);

    // Get the manifest JSON string
    let manifest_json = reader.json();

    // Determine the final output path
    let final_output_path = if output_path.is_dir() {
        // If output is a directory, create a filename based on the input
        let input_stem = input_path
            .file_stem()
            .context("Input file has no filename")?
            .to_str()
            .context("Invalid UTF-8 in filename")?;
        output_path.join(format!("{}_manifest.json", input_stem))
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate input file exists
    if !cli.input.exists() {
        anyhow::bail!("Input file does not exist: {:?}", cli.input);
    }

    // Handle extract mode
    if cli.extract {
        return extract_manifest(&cli.input, &cli.output);
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

    // Read and parse the JSON manifest configuration
    let manifest_json =
        fs::read_to_string(&manifest).context("Failed to read manifest JSON file")?;

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

    // Auto-detect or parse signing algorithm
    let signing_alg = if let Some(alg_str) = &cli.algorithm {
        parse_signing_algorithm(alg_str)?
    } else {
        println!("Auto-detecting signing algorithm from certificate...");
        let detected = detect_signing_algorithm(&cert)?;
        println!("  Detected: {:?}", detected);
        detected
    };

    println!("Creating C2PA manifest...");
    println!("  Input: {:?}", cli.input);
    println!("  Output: {:?}", output_path);
    println!("  Algorithm: {:?}", signing_alg);
    if cli.allow_self_signed {
        println!("  Note: Allowing self-signed certificates (development mode)");
    }

    // Create a builder from the JSON manifest
    let mut builder = Builder::from_json(&manifest_json)
        .context("Failed to create builder from JSON manifest")?;

    // Sign and embed the manifest into the asset
    if cli.allow_self_signed {
        // Use callback signer that bypasses certificate validation
        let signer = create_callback_signer(&cert, &key, signing_alg)
            .context("Failed to create callback signer")?;
        builder
            .sign_file(&signer, &cli.input, &output_path)
            .context("Failed to sign and embed manifest")?;
    } else {
        // Use standard signer with full certificate validation
        let signer = create_signer::from_files(
            cert.to_str().context("Invalid cert path")?,
            key.to_str().context("Invalid key path")?,
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
