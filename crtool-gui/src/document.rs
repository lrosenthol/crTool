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

//! Document tab state and UI: one loaded file per tab (manifest, validation, tree, raw JSON).

use crate::manifest_ui::{
    display_manifest_ingredient_tree, get_generator_name, get_signature_issued_info,
    get_timestamp_info, get_trust_status, get_validation_failures, ValidationFailureEntry,
};
use crate::util;
use crtool::{
    extract_crjson_manifest_with_settings, validate_json_value, ManifestExtractionResult, Settings,
    ValidationResult,
};
use eframe::egui;
use egui_code_editor::{CodeEditor, ColorTheme};
use egui_json_tree::{DefaultExpand, JsonTree};
use egui_twemoji::EmojiLabel;
use std::path::{Path, PathBuf};

/// Width of the draggable resize handle between the two columns (px).
const RESIZE_HANDLE_WIDTH: f32 = 6.0;
/// Minimum fraction of total width for each column (so neither collapses).
const MIN_PANEL_RATIO: f32 = 0.15;
const MAX_PANEL_RATIO: f32 = 0.85;

/// Per-document state for each tab in the dock.
#[derive(Clone)]
pub(crate) struct DocumentTab {
    /// Loaded file path
    pub(crate) file_path: PathBuf,
    /// Extraction result (Ok with manifest or Err with message)
    pub(crate) extraction_result: Result<ManifestExtractionResult, String>,
    /// Validation result when extraction succeeded
    pub(crate) validation_result: Option<ValidationResult>,
    /// Whether to show the raw JSON view
    show_raw_json: bool,
    /// Buffer for raw JSON view (refreshed from manifest each frame)
    raw_json_buffer: String,
    /// Split ratio for left/right panels (0..1)
    split_ratio: f32,
}

/// Load one document from disk and return a DocumentTab. Uses security-scoped access on macOS when needed.
/// Uses the given Settings for extraction so trust validation is applied consistently (no thread-local reliance).
pub(crate) fn load_document(
    file_path: PathBuf,
    schema_path: &Path,
    extraction_settings: &Settings,
) -> DocumentTab {
    let extract = || {
        extract_crjson_manifest_with_settings(&file_path, extraction_settings)
            .map_err(|e| e.to_string())
    };
    let result = {
        #[cfg(target_os = "macos")]
        {
            crate::security_scoped::with_security_scoped_access(&file_path, extract)
        }
        #[cfg(not(target_os = "macos"))]
        {
            extract()
        }
    };

    let (extraction_result, validation_result) = match result {
        Ok(extract_result) => {
            let validation = validate_json_value(&extract_result.manifest_value, schema_path)
                .unwrap_or_else(|e| ValidationResult {
                    file_path: file_path.to_string_lossy().to_string(),
                    is_valid: false,
                    errors: vec![crtool::ValidationError {
                        instance_path: "schema".to_string(),
                        message: e.to_string(),
                    }],
                });
            (Ok(extract_result), Some(validation))
        }
        Err(e) => (Err(e), None),
    };

    DocumentTab {
        file_path,
        extraction_result,
        validation_result,
        show_raw_json: false,
        raw_json_buffer: String::new(),
        split_ratio: 0.5,
    }
}

/// Renders one validation failure entry (code, optional explanation, url, source).
fn show_validation_failure_entry(ui: &mut egui::Ui, entry: &ValidationFailureEntry) {
    ui.group(|ui| {
        if let Some(ref source) = entry.source {
            EmojiLabel::new(
                egui::RichText::new(format!("📍 {}", source))
                    .size(14.0)
                    .color(egui::Color32::from_rgb(255, 200, 100)),
            )
            .show(ui);
        }
        EmojiLabel::new(
            egui::RichText::new(format!("❌ Code: {}", entry.code))
                .size(14.0)
                .color(egui::Color32::from_rgb(255, 150, 150)),
        )
        .show(ui);
        if let Some(ref explanation) = entry.explanation {
            EmojiLabel::new(
                egui::RichText::new(format!("   {}", explanation))
                    .size(14.0)
                    .color(egui::Color32::from_rgb(64, 64, 64)),
            )
            .show(ui);
        }
        if let Some(ref url) = entry.url {
            EmojiLabel::new(
                egui::RichText::new(format!("   URL: {}", url))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(64, 64, 64)),
            )
            .show(ui);
        }
    });
}

/// Renders one document tab: manifest info, validation, raw JSON toggle, and manifest/tree panels.
pub(crate) fn show_document_tab_ui(ui: &mut egui::Ui, tab: &mut DocumentTab) {
    let manifest = match &tab.extraction_result {
        Ok(m) => m.clone(),
        Err(e) => {
            EmojiLabel::new(
                egui::RichText::new(format!("❌ Error: {}", e))
                    .size(15.0)
                    .color(egui::Color32::from_rgb(230, 80, 80)),
            )
            .show(ui);
            return;
        }
    };

    ui.horizontal(|ui| {
        EmojiLabel::new(
            egui::RichText::new(format!("📜 Active Manifest: {}", manifest.active_label))
                .size(15.0)
                .color(egui::Color32::from_rgb(200, 160, 50)),
        )
        .show(ui);
    });

    let (name, date) = get_signature_issued_info(&manifest.manifest_value, &manifest.active_label)
        .unwrap_or_else(|| ("—".to_string(), "—".to_string()));
    ui.horizontal(|ui| {
        EmojiLabel::new(
            egui::RichText::new(format!("📝 Issued by: {} on {}", name, date))
                .size(15.0)
                .color(egui::Color32::from_rgb(100, 120, 140)),
        )
        .show(ui);
    });

    let (timestamp_present, tsa_authority) =
        get_timestamp_info(&manifest.manifest_value, &manifest.active_label);
    let timestamp_text = if timestamp_present {
        let ca = tsa_authority.as_deref().unwrap_or("—");
        format!("🕐 Timestamp: Yes — {}", ca)
    } else {
        "🕐 Timestamp: No".to_string()
    };
    ui.horizontal(|ui| {
        EmojiLabel::new(
            egui::RichText::new(timestamp_text)
                .size(15.0)
                .color(egui::Color32::from_rgb(100, 120, 140)),
        )
        .show(ui);
    });

    let generator = get_generator_name(&manifest.manifest_value, &manifest.active_label)
        .unwrap_or_else(|| "—".to_string());
    ui.horizontal(|ui| {
        EmojiLabel::new(
            egui::RichText::new(format!("🛠️ App or device used: {}", generator))
                .size(15.0)
                .color(egui::Color32::from_rgb(100, 120, 140)),
        )
        .show(ui);
    });

    if let Some(trust_status) = get_trust_status(&manifest.manifest_value, &manifest.active_label) {
        ui.horizontal(|ui| {
            let (icon, color, text) = match trust_status.as_str() {
                "signingCredential.trusted" => {
                    ("🔒", egui::Color32::from_rgb(0, 100, 0), "Trusted")
                }
                "signingCredential.untrusted" => {
                    ("🚫", egui::Color32::from_rgb(255, 100, 100), "Untrusted")
                }
                _ => (
                    "⚠️",
                    egui::Color32::from_rgb(64, 64, 64),
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

    if let Some(ref validation) = tab.validation_result {
        let manifest_failures =
            get_validation_failures(&manifest.manifest_value, &manifest.active_label);
        let has_schema_errors = !validation.errors.is_empty();
        let has_manifest_failures = !manifest_failures.is_empty();

        if validation.is_valid && !has_manifest_failures {
            EmojiLabel::new(
                egui::RichText::new("✅ Manifest is valid!")
                    .size(15.0)
                    .color(egui::Color32::from_rgb(0, 100, 0)),
            )
            .show(ui);
        } else {
            let total_errors = validation.errors.len() + manifest_failures.len();
            EmojiLabel::new(
                egui::RichText::new(format!("❌ Validation failed ({} error(s))", total_errors))
                    .size(15.0)
                    .color(egui::Color32::from_rgb(255, 100, 100)),
            )
            .show(ui);

            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("validation_errors")
                .max_height(200.0)
                .show(ui, |ui| {
                    if has_schema_errors {
                        EmojiLabel::new(
                            egui::RichText::new("⚠️  Schema validation errors:").size(16.0),
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
                                    egui::RichText::new(format!("❌ Error: {}", error.message))
                                        .size(14.0)
                                        .color(egui::Color32::from_rgb(255, 150, 150)),
                                )
                                .show(ui);
                            });
                        }
                        if has_manifest_failures {
                            ui.add_space(8.0);
                        }
                    }
                    if has_manifest_failures {
                        EmojiLabel::new(
                            egui::RichText::new(
                                "⚠️ Manifest validation failures (validationResults):",
                            )
                            .size(16.0),
                        )
                        .show(ui);
                        for entry in &manifest_failures {
                            show_validation_failure_entry(ui, entry);
                        }
                    }
                });
        }
    }

    ui.separator();

    ui.horizontal(|ui| {
        ui.checkbox(&mut tab.show_raw_json, "");
        EmojiLabel::new(
            egui::RichText::new("Show Raw JSON (replaces tree and manifest data)").size(15.0),
        )
        .show(ui);
    });

    if tab.show_raw_json {
        ui.separator();
        EmojiLabel::new(egui::RichText::new("📋 Raw JSON:").size(17.0)).show(ui);

        tab.raw_json_buffer = manifest.manifest_json.clone();
        let mut editor = CodeEditor::default()
            .id_source("raw_json")
            .with_rows(28)
            .with_ui_fontsize(ui)
            .with_theme(ColorTheme::AYU)
            .with_syntax(util::json_syntax())
            .with_numlines(false)
            .vscroll(true);
        editor.show(ui, &mut tab.raw_json_buffer);
    } else {
        ui.separator();
        let fill_height = ui.available_height();
        let total_width = ui.available_width();
        let content_width = (total_width - RESIZE_HANDLE_WIDTH).max(0.0);
        let left_width = content_width * tab.split_ratio;
        let right_width = content_width - left_width;

        ui.horizontal(|ui| {
            let left_response = ui.allocate_ui_with_layout(
                egui::vec2(left_width, fill_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_size(egui::vec2(left_width, fill_height));
                    EmojiLabel::new(egui::RichText::new("📊 Manifest Data").size(16.0)).show(ui);
                    egui::ScrollArea::vertical()
                        .id_salt("manifest_data")
                        .show(ui, |ui| {
                            ui.set_min_width((left_width - 16.0).max(0.0));
                            JsonTree::new("manifest-data-tree", &manifest.manifest_value)
                                .default_expand(DefaultExpand::ToLevel(2))
                                .show(ui);
                        });
                },
            );

            let left_rect = left_response.response.rect;
            let resize_rect = egui::Rect::from_min_size(
                left_rect.right_top(),
                egui::vec2(RESIZE_HANDLE_WIDTH, fill_height),
            );
            let resize_response = ui
                .push_id("manifest_tree_resize", |ui| {
                    ui.allocate_rect(resize_rect, egui::Sense::drag())
                })
                .inner;
            if resize_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if resize_response.dragged() {
                if let Some(pos) = ui.ctx().input(|i| i.pointer.latest_pos()) {
                    let new_ratio = (pos.x - left_rect.left()) / content_width;
                    tab.split_ratio = new_ratio.clamp(MIN_PANEL_RATIO, MAX_PANEL_RATIO);
                }
            }
            let painter = ui.painter_at(resize_rect);
            let line_x = resize_rect.center().x;
            painter.line_segment(
                [
                    egui::pos2(line_x, resize_rect.top()),
                    egui::pos2(line_x, resize_rect.bottom()),
                ],
                egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.fg_stroke.color),
            );

            ui.allocate_ui_with_layout(
                egui::vec2(right_width, fill_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_size(egui::vec2(right_width, fill_height));
                    EmojiLabel::new(
                        egui::RichText::new("🌳 Manifest & Ingredients Tree").size(16.0),
                    )
                    .show(ui);
                    egui::ScrollArea::vertical()
                        .id_salt("tree_view")
                        .show(ui, |ui| {
                            display_manifest_ingredient_tree(
                                ui,
                                &manifest.manifest_value,
                                &manifest.active_label,
                            );
                        });
                },
            );
        });
    }
}
