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

mod batch;
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
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use test_case::handle_create_test;

// ─── Logger ──────────────────────────────────────────────────────────────────

/// Output manager: writes progress to stdout (unless quiet) and optionally to a log file.
pub struct Logger {
    quiet: bool,
    log_writer: Option<BufWriter<std::fs::File>>,
}

impl Logger {
    pub fn new(quiet: bool, log_path: Option<&std::path::Path>) -> Result<Self> {
        let log_writer = if let Some(path) = log_path {
            let file = std::fs::File::create(path)
                .with_context(|| format!("Failed to create log file: {}", path.display()))?;
            eprintln!("📝 Logging to: {}", path.display());
            Some(BufWriter::new(file))
        } else {
            None
        };
        Ok(Self { quiet, log_writer })
    }

    /// Print informational message to stdout (suppressed by --quiet) and log file.
    pub fn info(&mut self, msg: &str) {
        if !self.quiet {
            println!("{msg}");
        }
        if let Some(w) = &mut self.log_writer {
            let _ = writeln!(w, "{msg}");
        }
    }

    /// Print error message to stderr (never suppressed) and log file.
    pub fn error(&mut self, msg: &str) {
        eprintln!("{msg}");
        if let Some(w) = &mut self.log_writer {
            let _ = writeln!(w, "ERROR: {msg}");
        }
    }
}

// ─── CLI definition ───────────────────────────────────────────────────────────

/// Content Credential Tool - Create and embed C2PA manifests into media assets
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to a test case JSON file (C2PA validator test case schema).
    /// Reads all signing configuration (manifest, cert, key, algorithm, TSA URL) from the file.
    /// Use with -o to specify the output file path.
    #[arg(short = 't', long = "create-test", value_name = "FILE")]
    create_test: Option<PathBuf>,

    /// Path(s) to input media asset(s). Supported: avi, avif, c2pa, dng, gif, heic, heif,
    /// jpg/jpeg, m4a, mov, mp3, mp4, pdf, png, svg, tiff, wav, webp.
    /// Supports glob patterns (e.g., "*.jpg", "images/*.png")
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

    /// Enable trust list validation: load the official C2PA trust list and the Content
    /// Credentials interim trust list for certificate validation during extract/read
    #[arg(long, default_value = "false")]
    trust: bool,

    /// Path to the YAML asset profile for profile evaluation. When combined with --extract,
    /// evaluates the extracted crJSON. When used alone, treats input files as crJSON indicators.
    #[arg(long, value_name = "FILE")]
    profile: Option<PathBuf>,

    /// Output format for the profile evaluation report (json or yaml)
    #[arg(long, value_enum, default_value_t = ReportFormat::Json)]
    report_format: ReportFormat,

    /// Path to a batch JSON file — runs multiple commands in sequence
    #[arg(short = 'b', long = "batch", value_name = "FILE")]
    batch: Option<PathBuf>,

    /// Suppress progress output (errors are still shown on stderr)
    #[arg(short = 'q', long = "quiet", default_value = "false")]
    quiet: bool,

    /// Write all progress output to a log file (in addition to stdout)
    #[arg(short = 'l', long = "log", value_name = "FILE")]
    log: Option<PathBuf>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Expand glob patterns and collect matching file paths.
pub fn expand_input_patterns(patterns: &[String]) -> Result<Vec<PathBuf>> {
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

// ─── Core execution ───────────────────────────────────────────────────────────

/// Execute a parsed CLI command. Called from both normal mode and batch mode.
pub fn run_cli(cli: Cli, logger: &mut Logger) -> Result<()> {
    // Handle --create-test mode before anything else (no positional input required)
    if let Some(test_case_path) = &cli.create_test {
        let output = cli
            .output
            .context("--output is required when using --create-test mode")?;

        if cli.input.is_empty() {
            // No CLI inputs: let handle_create_test use inputAsset from JSON (or error)
            return handle_create_test(test_case_path, None, &output);
        }

        let input_files =
            expand_input_patterns(&cli.input).context("Failed to expand input file patterns")?;

        if input_files.len() > 1 && !output.is_dir() {
            anyhow::bail!(
                "Output must be a directory when creating test assets from multiple input files. Got: {:?}",
                output
            );
        }

        let mut success_count = 0u32;
        let mut error_count = 0u32;

        for input_file in &input_files {
            logger.info(&format!("  📄 Processing: {} ...", input_file.display()));
            match handle_create_test(test_case_path, Some(input_file), &output) {
                Ok(_) => {
                    logger.info("     ✅ Done");
                    success_count += 1;
                }
                Err(e) => {
                    logger.error(&format!("     ❌ Error: {e}"));
                    error_count += 1;
                }
            }
        }

        if input_files.len() > 1 {
            logger.info(&format!(
                "\n📊 Test Asset Creation: {success_count} succeeded, {error_count} failed, {} total",
                input_files.len()
            ));
        }

        if error_count > 0 {
            anyhow::bail!("{error_count} file(s) failed to create test asset");
        }

        return Ok(());
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

    logger.info(&format!(
        "🚀 Processing {} input file(s)",
        input_files.len()
    ));

    // ── Validate mode ─────────────────────────────────────────────────────────
    if cli.validate {
        let schema_path = crtool::crjson_schema_path();
        return validate_json_files(&input_files, &schema_path, "crJSON");
    }

    // ── Standalone profile evaluation mode: --profile without --extract ───────
    if standalone_eval {
        let profile_path = cli.profile.as_ref().unwrap();
        let mut success_count = 0u32;
        let mut error_count = 0u32;

        logger.info("=== Profile Evaluation ===");

        for input_file in &input_files {
            logger.info(&format!("  📄 Processing: {} ...", input_file.display()));
            match run_profile_evaluation(input_file, profile_path, cli.report_format) {
                Ok(_) => {
                    logger.info("     ✅ Done");
                    success_count += 1;
                }
                Err(e) => {
                    logger.error(&format!("     ❌ Error: {e}"));
                    error_count += 1;
                }
            }
        }

        logger.info(&format!(
            "\n📊 Evaluation Summary: {success_count} succeeded, {error_count} failed, {} total",
            input_files.len()
        ));

        if error_count > 0 {
            anyhow::bail!("{error_count} file(s) failed evaluation");
        }

        return Ok(());
    }

    // ── Extract mode ──────────────────────────────────────────────────────────
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

        let mut success_count = 0u32;
        let mut error_count = 0u32;

        for input_file in &input_files {
            logger.info(&format!("  📄 Processing: {} ...", input_file.display()));
            match extract_manifest(input_file, &output, &extraction_settings) {
                Ok(crjson_path) => {
                    logger.info("     ✅ Done");
                    success_count += 1;
                    if let Some(profile_path) = &cli.profile {
                        if let Err(e) =
                            run_profile_evaluation(&crjson_path, profile_path, cli.report_format)
                        {
                            logger.error(&format!(
                                "     ⚠️  Profile evaluation failed for {}: {e}",
                                crjson_path.display()
                            ));
                        }
                    }
                }
                Err(e) => {
                    logger.error(&format!("     ❌ Error: {e}"));
                    error_count += 1;
                }
            }
        }

        logger.info(&format!(
            "\n📊 Extraction Summary: {success_count} succeeded, {error_count} failed, {} total",
            input_files.len()
        ));

        if error_count > 0 {
            anyhow::bail!("{error_count} file(s) failed to extract");
        }

        return Ok(());
    }

    anyhow::bail!(
        "No operation specified. Use --create-test FILE to create a test asset, \
        --extract to extract a manifest, --validate to validate JSON files, or \
        --batch FILE to run a batch of commands."
    );
}

// ─── Entry point ──────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut logger = Logger::new(cli.quiet, cli.log.as_deref())?;

    // ── Batch mode ────────────────────────────────────────────────────────────
    if let Some(batch_path) = &cli.batch.clone() {
        return batch::run_batch(batch_path, &mut logger);
    }

    run_cli(cli, &mut logger)
}
