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

use crtool::{
    default_schema_path, extract_jpt_manifest, validate_json_value, ManifestExtractionResult,
    ValidationResult,
};
use eframe::egui;
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "C2PA Content Credential Tool",
        options,
        Box::new(|_cc| Ok(Box::new(CrtoolApp::default()))),
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
        Self {
            selected_file: None,
            extraction_result: None,
            validation_result: None,
            show_raw_json: false,
            schema_path: default_schema_path(),
        }
    }

    fn extract_and_validate(&mut self) {
        if let Some(ref file_path) = self.selected_file {
            // Extract manifest
            match extract_jpt_manifest(file_path) {
                Ok(result) => {
                    // Validate the extracted manifest
                    let validation = validate_json_value(&result.manifest_value, &self.schema_path)
                        .unwrap_or_else(|e| ValidationResult {
                            file_path: file_path.to_string_lossy().to_string(),
                            is_valid: false,
                            errors: vec![crtool::ValidationError {
                                instance_path: "schema".to_string(),
                                message: e.to_string(),
                            }],
                        });

                    self.validation_result = Some(validation);
                    self.extraction_result = Some(Ok(result));
                }
                Err(e) => {
                    self.extraction_result = Some(Err(e.to_string()));
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("C2PA Content Credential Tool");
            ui.separator();

            // File selection area
            ui.horizontal(|ui| {
                if ui.button("📂 Select Image File").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["jpg", "jpeg", "png", "webp"])
                        .pick_file()
                    {
                        self.selected_file = Some(path);
                        self.extract_and_validate();
                    }
                }

                if let Some(ref file_path) = self.selected_file {
                    ui.label(format!("File: {}", file_path.display()));
                }
            });

            ui.separator();

            // Display results
            if let Some(ref result) = self.extraction_result {
                match result {
                    Ok(manifest) => {
                        ui.heading("✓ Manifest Extracted Successfully");

                        ui.horizontal(|ui| {
                            ui.label(format!("Active Label: {}", manifest.active_label));
                            if let Some(ref hash) = manifest.asset_hash {
                                ui.label(format!("Asset Hash: {}", hash));
                            }
                        });

                        ui.separator();

                        // Validation status
                        if let Some(ref validation) = self.validation_result {
                            if validation.is_valid {
                                ui.colored_label(
                                    egui::Color32::GREEN,
                                    "✓ Manifest is valid according to JPEG Trust schema",
                                );
                            } else {
                                ui.colored_label(
                                    egui::Color32::RED,
                                    format!(
                                        "✗ Validation failed ({} error(s))",
                                        validation.errors.len()
                                    ),
                                );

                                ui.separator();

                                egui::ScrollArea::vertical()
                                    .id_salt("validation_errors")
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        ui.heading("Validation Errors:");
                                        for error in &validation.errors {
                                            ui.group(|ui| {
                                                ui.label(format!("Path: {}", error.instance_path));
                                                ui.label(format!("Error: {}", error.message));
                                            });
                                        }
                                    });
                            }
                        }

                        ui.separator();

                        // Toggle for raw JSON view
                        ui.checkbox(&mut self.show_raw_json, "Show Raw JSON");

                        if self.show_raw_json {
                            ui.separator();
                            ui.heading("Raw JSON:");

                            egui::ScrollArea::vertical()
                                .id_salt("raw_json")
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut manifest.manifest_json.as_str())
                                            .font(egui::TextStyle::Monospace)
                                            .code_editor()
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                        } else {
                            // Display structured manifest data
                            ui.separator();
                            ui.heading("Manifest Data:");

                            egui::ScrollArea::vertical()
                                .id_salt("manifest_data")
                                .show(ui, |ui| {
                                    display_json_tree(ui, &manifest.manifest_value, 0);
                                });
                        }
                    }
                    Err(e) => {
                        ui.colored_label(egui::Color32::RED, format!("✗ Error: {}", e));
                    }
                }
            } else if self.selected_file.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("👆 Select an image file to extract its C2PA manifest");
                });
            }
        });
    }
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

/// Display a simple JSON value (not object or array)
fn display_json_value(ui: &mut egui::Ui, value: &serde_json::Value) {
    use serde_json::Value;

    match value {
        Value::String(s) => {
            ui.label(egui::RichText::new(format!("\"{}\"", s)).color(egui::Color32::LIGHT_GREEN));
        }
        Value::Number(n) => {
            ui.label(egui::RichText::new(n.to_string()).color(egui::Color32::LIGHT_BLUE));
        }
        Value::Bool(b) => {
            ui.label(egui::RichText::new(b.to_string()).color(egui::Color32::LIGHT_YELLOW));
        }
        Value::Null => {
            ui.label(egui::RichText::new("null").color(egui::Color32::GRAY));
        }
        _ => {
            ui.label(value.to_string());
        }
    }
}
