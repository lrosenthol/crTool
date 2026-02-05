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

// objc crate macros expand to cfg(cargo-clippy), which triggers unexpected_cfgs in security_scoped.
#![allow(unexpected_cfgs)]

use crtool::{
    default_schema_path, extract_jpt_manifest, validate_json_value, ManifestExtractionResult,
    ValidationResult,
};
use eframe::egui;
use egui_twemoji::EmojiLabel;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
mod macos_open_document;
mod security_scoped;

/// Convert a command-line argument to a file path. Handles macOS `file://` URLs
/// that the system may pass when opening via "Open With" or drop-on-icon.
fn arg_to_path(arg: &str) -> PathBuf {
    let arg = arg.trim();
    if let Some(path_str) = arg.strip_prefix("file://") {
        let path_str = path_str.trim_start_matches('/');
        let decoded = urlencoding::decode(path_str).unwrap_or(std::borrow::Cow::Borrowed(path_str));
        return PathBuf::from(decoded.as_ref());
    }
    PathBuf::from(arg)
}

fn main() -> Result<(), eframe::Error> {
    #[cfg(target_os = "macos")]
    macos_open_document::install_handler();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "C2PA Content Credential Tool",
        options,
        Box::new(|cc| {
            // Required for egui-twemoji to load and render color emoji (SVG/PNG)
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // Register Cocoa open-document handler (Dock drop, "Open With"). Must run after NSApp exists.
            #[cfg(target_os = "macos")]
            macos_open_document::install_cocoa_handler();
            // Open file from command line (e.g. "open -a crTool file.jpg" or drop on app icon)
            let initial_file = std::env::args().skip(1).find_map(|arg| {
                let path = arg_to_path(&arg);
                (path.is_file() && crtool::is_supported_asset_path(&path)).then_some(path)
            });
            #[cfg(target_os = "macos")]
            let initial_file = initial_file.or_else(macos_open_document::take_pending_file);
            Ok(Box::new(CrtoolApp::new_with_optional_file(initial_file)))
        }),
    )
}

struct CrtoolApp {
    /// Currently loaded file path
    selected_file: Option<PathBuf>,
    /// Extraction result
    extraction_result: Option<Result<ManifestExtractionResult, String>>,
    /// Validation result
    validation_result: Option<ValidationResult>,
    /// Whether to show the raw JSON
    show_raw_json: bool,
    /// Schema path (defaults to bundled schema)
    schema_path: PathBuf,
}

impl CrtoolApp {
    fn new() -> Self {
        Self::new_with_optional_file(None)
    }

    fn new_with_optional_file(initial_file: Option<PathBuf>) -> Self {
        let mut app = Self {
            selected_file: initial_file,
            extraction_result: None,
            validation_result: None,
            show_raw_json: false,
            schema_path: default_schema_path(),
        };
        if app.selected_file.is_some() {
            app.extract_and_validate();
        }
        app
    }

    fn extract_and_validate(&mut self) {
        if let Some(ref file_path) = self.selected_file {
            // On macOS, files opened via the system (drop-on-icon, Open With) require
            // security-scoped access before reading.
            let extract = || extract_jpt_manifest(file_path).map_err(|e| e.to_string());
            let result = {
                #[cfg(target_os = "macos")]
                {
                    security_scoped::with_security_scoped_access(file_path, extract)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    extract()
                }
            };

            match result {
                Ok(extract_result) => {
                    // Validate the extracted manifest
                    let validation =
                        validate_json_value(&extract_result.manifest_value, &self.schema_path)
                            .unwrap_or_else(|e| ValidationResult {
                                file_path: file_path.to_string_lossy().to_string(),
                                is_valid: false,
                                errors: vec![crtool::ValidationError {
                                    instance_path: "schema".to_string(),
                                    message: e.to_string(),
                                }],
                            });

                    self.validation_result = Some(validation);
                    self.extraction_result = Some(Ok(extract_result));
                }
                Err(e) => {
                    self.extraction_result = Some(Err(e));
                    self.validation_result = None;
                }
            }
        }
    }
}

impl Default for CrtoolApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for CrtoolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle files from macOS drop-on-icon / Open With (Apple Event queue)
        #[cfg(target_os = "macos")]
        for path in macos_open_document::drain_pending_files() {
            if crtool::is_supported_asset_path(&path) && path.is_file() {
                self.selected_file = Some(path);
                self.extract_and_validate();
                break;
            }
        }

        // Handle files dropped onto the window
        let dropped: Vec<egui::DroppedFile> = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            if let Some(path) = file.path.filter(|p| crtool::is_supported_asset_path(p)) {
                self.selected_file = Some(path);
                self.extract_and_validate();
                break; // use first supported file
            }
        }

        // Add menu bar with File and Edit menus
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📂 Open...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("C2PA-supported files", crtool::SUPPORTED_ASSET_EXTENSIONS)
                            .pick_file()
                        {
                            self.selected_file = Some(path);
                            self.extract_and_validate();
                        }
                        ui.close();
                    }

                    ui.add_enabled_ui(self.selected_file.is_some(), |ui| {
                        if ui.button("❌ Close").clicked() {
                            self.selected_file = None;
                            self.extraction_result = None;
                            self.validation_result = None;
                            ui.close();
                        }
                    });

                    ui.separator();

                    ui.add_enabled_ui(self.extraction_result.is_some(), |ui| {
                        if ui.button("💾 Save As...").clicked() {
                            if let Some(Ok(ref manifest)) = self.extraction_result {
                                // Generate default filename
                                let default_name = if let Some(ref path) = self.selected_file {
                                    let stem = path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("manifest");
                                    format!("{}-indicators.json", stem)
                                } else {
                                    "manifest-indicators.json".to_string()
                                };

                                if let Some(save_path) = rfd::FileDialog::new()
                                    .set_file_name(&default_name)
                                    .add_filter("JSON", &["json"])
                                    .save_file()
                                {
                                    if let Err(e) =
                                        std::fs::write(&save_path, &manifest.manifest_json)
                                    {
                                        eprintln!("Failed to save file: {}", e);
                                    }
                                }
                            }
                            ui.close();
                        }
                    });
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("📋 Copy").clicked() {
                        ctx.copy_text(get_selected_text(ctx));
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Select All").clicked() {
                        ui.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("C2PA Content Credential Tool");
            ui.separator();

            // File selection area
            ui.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new("📂 Select File").size(16.0))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("C2PA-supported files", crtool::SUPPORTED_ASSET_EXTENSIONS)
                        .pick_file()
                    {
                        self.selected_file = Some(path);
                        self.extract_and_validate();
                    }
                }

                if let Some(ref file_path) = self.selected_file {
                    ui.label(
                        egui::RichText::new(format!("File: {}", file_path.display())).size(15.0),
                    );
                }
            });

            ui.separator();

            // Display results
            if let Some(ref result) = self.extraction_result {
                match result {
                    Ok(manifest) => {
                        EmojiLabel::new(
                            egui::RichText::new("✅ Manifest Extracted Successfully").size(18.0),
                        )
                        .show(ui);

                        // Status items with icons and readable colors
                        ui.horizontal(|ui| {
                            EmojiLabel::new(
                                egui::RichText::new(format!(
                                    "📜  Active Manifest: {}",
                                    manifest.active_label
                                ))
                                .size(15.0)
                                .color(egui::Color32::from_rgb(200, 160, 50)), // Darker yellow/gold
                            )
                            .show(ui);
                        });

                        if let Some(ref hash) = manifest.asset_hash {
                            ui.horizontal(|ui| {
                                EmojiLabel::new(
                                    egui::RichText::new(format!("🔐 Asset Hash: {}", hash))
                                        .size(15.0)
                                        .color(egui::Color32::from_rgb(150, 200, 255)) // Medium blue
                                        .monospace(),
                                )
                                .show(ui);
                            });
                        }

                        // Trust status
                        if let Some(trust_status) = get_trust_status(&manifest.manifest_value) {
                            ui.horizontal(|ui| {
                                let (icon, color, text) = match trust_status.as_str() {
                                    "signingCredential.trusted" => {
                                        ("🔒", egui::Color32::from_rgb(80, 220, 120), "Trusted")
                                    }
                                    "signingCredential.untrusted" => {
                                        ("🔓", egui::Color32::from_rgb(255, 100, 100), "Untrusted")
                                    }
                                    _ => (
                                        "⚠️",
                                        egui::Color32::from_rgb(200, 200, 200),
                                        trust_status.as_str(),
                                    ),
                                };
                                EmojiLabel::new(
                                    egui::RichText::new(format!("{} Trust Status: {}", icon, text))
                                        .size(15.0)
                                        .color(color),
                                )
                                .show(ui);
                            });
                        }

                        ui.separator();

                        // Validation status with red/green colors
                        if let Some(ref validation) = self.validation_result {
                            if validation.is_valid {
                                EmojiLabel::new(
                                    egui::RichText::new(
                                        "✅ Manifest is valid according to JPEG Trust schema",
                                    )
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(80, 220, 120)),
                                )
                                .show(ui);
                            } else {
                                EmojiLabel::new(
                                    egui::RichText::new(format!(
                                        "❌ Validation failed ({} error(s))",
                                        validation.errors.len()
                                    ))
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(255, 100, 100)),
                                )
                                .show(ui);

                                ui.separator();

                                egui::ScrollArea::vertical()
                                    .id_salt("validation_errors")
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        EmojiLabel::new(
                                            egui::RichText::new("⚠️  Validation Errors:")
                                                .size(16.0),
                                        )
                                        .show(ui);
                                        for error in &validation.errors {
                                            ui.group(|ui| {
                                                EmojiLabel::new(
                                                    egui::RichText::new(format!(
                                                        "📍 Path: {}",
                                                        error.instance_path
                                                    ))
                                                    .size(14.0)
                                                    .color(egui::Color32::from_rgb(255, 200, 100)),
                                                )
                                                .show(ui);
                                                EmojiLabel::new(
                                                    egui::RichText::new(format!(
                                                        "❌ Error: {}",
                                                        error.message
                                                    ))
                                                    .size(14.0)
                                                    .color(egui::Color32::from_rgb(255, 150, 150)),
                                                )
                                                .show(ui);
                                            });
                                        }
                                    });
                            }
                        }

                        ui.separator();

                        // Toggle for raw JSON view
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.show_raw_json, "");
                            EmojiLabel::new(egui::RichText::new("📄 Show Raw JSON").size(15.0))
                                .show(ui);
                        });

                        if self.show_raw_json {
                            ui.separator();
                            EmojiLabel::new(egui::RichText::new("📋 Raw JSON:").size(17.0))
                                .show(ui);

                            egui::ScrollArea::vertical()
                                .id_salt("raw_json")
                                .show(ui, |ui| {
                                    // Syntax-highlighted JSON via egui_extras
                                    let theme =
                                        egui_extras::syntax_highlighting::CodeTheme::from_style(
                                            ui.style(),
                                        );
                                    egui_extras::syntax_highlighting::code_view_ui(
                                        ui,
                                        &theme,
                                        &manifest.manifest_json,
                                        "json",
                                    );
                                });
                        } else {
                            // Display structured manifest data
                            ui.separator();
                            EmojiLabel::new(egui::RichText::new("📊 Manifest Data:").size(17.0))
                                .show(ui);

                            egui::ScrollArea::vertical()
                                .id_salt("manifest_data")
                                .show(ui, |ui| {
                                    display_json_tree(ui, &manifest.manifest_value, 0);
                                });
                        }
                    }
                    Err(e) => {
                        EmojiLabel::new(
                            egui::RichText::new(format!("❌ Error: {}", e))
                                .size(15.0)
                                .color(egui::Color32::from_rgb(230, 80, 80)),
                        )
                        .show(ui);
                    }
                }
            } else if self.selected_file.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    EmojiLabel::new(
                        egui::RichText::new(
                            "👆 Select a C2PA-supported file (image, video, audio, or PDF) to extract its manifest",
                        )
                        .size(18.0),
                    )
                    .show(ui);
                });
            }
        });
    }
}

/// Helper function to get selected text from the context
fn get_selected_text(_ctx: &egui::Context) -> String {
    // egui handles text selection internally, this returns empty for now
    // The copy functionality works automatically when text is selected
    String::new()
}

/// Extract trust status from the manifest
fn get_trust_status(manifest_value: &serde_json::Value) -> Option<String> {
    // Try to find trust status in the first manifest's status.trust field
    manifest_value
        .get("manifests")?
        .as_array()?
        .first()?
        .get("status")?
        .get("trust")?
        .as_str()
        .map(|s| s.to_string())
}

/// Recursively display a JSON value as a tree
fn display_json_tree(ui: &mut egui::Ui, value: &serde_json::Value, depth: usize) {
    use serde_json::Value;

    let indent = "  ".repeat(depth);

    match value {
        Value::Object(map) => {
            for (key, val) in map {
                match val {
                    Value::Object(_) | Value::Array(_) => {
                        egui::CollapsingHeader::new(format!("{}{}", indent, key))
                            .default_open(depth < 2)
                            .show(ui, |ui| {
                                display_json_tree(ui, val, depth + 1);
                            });
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}{}: ", indent, key));
                            display_json_value(ui, val);
                        });
                    }
                }
            }
        }
        Value::Array(arr) => {
            for (idx, val) in arr.iter().enumerate() {
                match val {
                    Value::Object(_) | Value::Array(_) => {
                        egui::CollapsingHeader::new(format!("{}[{}]", indent, idx))
                            .default_open(depth < 2)
                            .show(ui, |ui| {
                                display_json_tree(ui, val, depth + 1);
                            });
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}[{}]: ", indent, idx));
                            display_json_value(ui, val);
                        });
                    }
                }
            }
        }
        _ => {
            display_json_value(ui, value);
        }
    }
}

/// Display a simple JSON value (not object or array) - using standard colors
fn display_json_value(ui: &mut egui::Ui, value: &serde_json::Value) {
    use serde_json::Value;

    match value {
        Value::String(s) => {
            ui.label(
                egui::RichText::new(format!("\"{}\"", s))
                    .color(egui::Color32::from_rgb(206, 145, 120)),
            );
        }
        Value::Number(n) => {
            ui.label(
                egui::RichText::new(n.to_string()).color(egui::Color32::from_rgb(181, 206, 168)),
            );
        }
        Value::Bool(b) => {
            ui.label(
                egui::RichText::new(b.to_string()).color(egui::Color32::from_rgb(86, 156, 214)),
            );
        }
        Value::Null => {
            ui.label(egui::RichText::new("null").color(egui::Color32::from_rgb(86, 156, 214)));
        }
        _ => {
            ui.label(value.to_string());
        }
    }
}
