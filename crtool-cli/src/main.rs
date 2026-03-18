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

mod extraction;
mod processing;
mod profile;
mod test_case;

use anyhow::{Context, Result};
use clap::Parser;
use crtool::SUPPORTED_ASSET_EXTENSIONS;
use extraction::{extract_manifest, extraction_settings, validate_json_files};
use glob::glob;
use profile::{run_profile_evaluation, ReportFormat};
use std::path::PathBuf;
use test_case::handle_create_test;

/// Content Credential Tool - Create and embed C2PA manifests into media assets
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to a test case JSON file (C2PA validator test case schema).
    /// Reads all signing configuration (manifest, cert, key, algorithm, TSA URL) from the file.
    /// Use with -o to specify the output file path.
    #[arg(short = 't', long = "create-test", value_name = "FILE")]
    create_test: Option<PathBuf>,

    /// Path(s) to input media asset(s). Supported: avi, avif, c2pa, dng, gif, heic, heif, jpg/jpeg, m4a, mov, mp3, mp4, pdf, png, svg, tiff, wav, webp. Supports glob patterns (e.g., "*.jpg", "images/*.png")
    #[arg(value_name = "INPUT_FILE", required = false, num_args = 0..)]
    input: Vec<String>,

    /// Path to the output file or directory (not required in validate mode)
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Extract manifest from input file to JSON (read-only mode; outputs crJSON)
    #[arg(short, long, default_value = "false")]
    extract: bool,

    /// Validate JSON files against the crJSON schema
    #[arg(short = 'v', long, default_value = "false")]
    validate: bool,

    /// Enable trust list validation: load the official C2PA trust list and the Content Credentials interim trust list for certificate validation during extract/read
    #[arg(long, default_value = "false")]
    trust: bool,

    /// Path to the YAML asset profile for profile evaluation. When combined with --extract,
    /// evaluates the extracted crJSON. When used alone, treats input files as crJSON indicators.
    #[arg(long, value_name = "FILE")]
    profile: Option<PathBuf>,

    /// Output format for the profile evaluation report (json or yaml)
    #[arg(long, value_enum, default_value_t = ReportFormat::Json)]
    report_format: ReportFormat,
}

/// Expand glob patterns and collect matching file paths.
fn expand_input_patterns(patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for pattern in patterns {
        let pattern_path = PathBuf::from(pattern);

        if pattern_path.exists() {
            files.push(pattern_path);
        } else {
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

    files.sort();
    files.dedup();

    Ok(files)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --create-test mode before anything else (no positional input required)
    if let Some(test_case_path) = &cli.create_test {
        let output = cli
            .output
            .context("--output is required when using --create-test mode")?;
        return handle_create_test(test_case_path, &output);
    }

    // All other modes require at least one input file
    if cli.input.is_empty() {
        anyhow::bail!(
            "No input files specified. Use --create-test to create a test asset from a test \
            case JSON file, or provide input file(s) for extract/validate/profile modes."
        );
    }

    let extraction_settings =
        extraction_settings(cli.trust).context("Failed to prepare extraction settings")?;

    let input_files =
        expand_input_patterns(&cli.input).context("Failed to expand input file patterns")?;

    if input_files.is_empty() {
        anyhow::bail!("No input files found matching the specified pattern(s)");
    }

    for input_file in &input_files {
        if !input_file.exists() {
            anyhow::bail!("Input file does not exist: {:?}", input_file);
        }
    }

    let standalone_eval = cli.profile.is_some() && !cli.extract && !cli.validate;
    if !cli.validate && !standalone_eval {
        let unsupported: Vec<_> = input_files
            .iter()
            .filter(|p| !crtool::is_supported_asset_path(p))
            .collect();
        if !unsupported.is_empty() {
            anyhow::bail!(
                "Unsupported file format(s). The following file(s) have extensions not supported \
                by C2PA: {:?}. Supported extensions: {}.",
                unsupported.iter().map(|p| p.as_path()).collect::<Vec<_>>(),
                SUPPORTED_ASSET_EXTENSIONS.join(", ")
            );
        }
    }

    println!("Found {} input file(s) to process", input_files.len());

    // Handle validation mode (validates against crJSON schema)
    if cli.validate {
        let schema_path = crtool::crjson_schema_path();
        return validate_json_files(&input_files, &schema_path, "crJSON");
    }

    // Handle standalone profile evaluation mode: --profile without --extract
    if standalone_eval {
        let profile_path = cli.profile.as_ref().unwrap();
        let mut success_count = 0;
        let mut error_count = 0;

        println!("=== Profile Evaluation ===");

        for input_file in &input_files {
            match run_profile_evaluation(input_file, profile_path, cli.report_format) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("Error evaluating {:?}: {}", input_file, e);
                    error_count += 1;
                }
            }
        }

        println!("\n=== Evaluation Summary ===");
        println!("  Successful: {}", success_count);
        println!("  Failed: {}", error_count);
        println!("  Total: {}", input_files.len());

        if error_count > 0 {
            anyhow::bail!("{} file(s) failed evaluation", error_count);
        }

        return Ok(());
    }

    // Handle extract mode (always outputs crJSON)
    if cli.extract {
        let output = cli
            .output
            .context("--output is required when using --extract mode")?;

        if input_files.len() > 1 && !output.is_dir() {
            anyhow::bail!(
                "Output must be a directory when extracting from multiple input files. Got: {:?}",
                output
            );
        }

        let mut success_count = 0;
        let mut error_count = 0;

        for input_file in &input_files {
            match extract_manifest(input_file, &output, &extraction_settings) {
                Ok(crjson_path) => {
                    success_count += 1;
                    if let Some(profile_path) = &cli.profile {
                        if let Err(e) =
                            run_profile_evaluation(&crjson_path, profile_path, cli.report_format)
                        {
                            eprintln!(
                                "Warning: profile evaluation failed for {:?}: {}",
                                crjson_path, e
                            );
                        }
                    }
                }
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

    anyhow::bail!(
        "No operation specified. Use --create-test FILE to create a test asset, \
        --extract to extract a manifest, or --validate to validate JSON files."
    );
}
