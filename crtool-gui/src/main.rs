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
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use egui_json_tree::{DefaultExpand, JsonTree};
use egui_twemoji::EmojiLabel;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Syntax definition for JSON (keywords true/false/null) for the code editor.
fn json_syntax() -> Syntax {
    Syntax {
        language: "json",
        case_sensitive: true,
        comment: "",
        comment_multiline: ["", ""],
        hyperlinks: BTreeSet::new(),
        keywords: BTreeSet::from(["false", "null", "true"]),
        types: BTreeSet::new(),
        special: BTreeSet::new(),
    }
}

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

/// Width of the draggable resize handle between the two columns (px).
const RESIZE_HANDLE_WIDTH: f32 = 6.0;

/// Minimum fraction of total width for each column (so neither collapses).
const MIN_PANEL_RATIO: f32 = 0.15;
const MAX_PANEL_RATIO: f32 = 0.85;

struct CrtoolApp {
    /// Currently loaded file path
    selected_file: Option<PathBuf>,
    /// Extraction result
    extraction_result: Option<Result<ManifestExtractionResult, String>>,
    /// Validation result
    validation_result: Option<ValidationResult>,
    /// Whether to show the raw JSON (replaces tree + manifest data when on)
    show_raw_json: bool,
    /// Buffer for raw JSON view (read-only; refreshed from manifest each frame)
    raw_json_buffer: String,
    /// Schema path (defaults to bundled schema)
    schema_path: PathBuf,
    /// Fraction of (content width minus resize handle) for the left panel (0.5 = 50%). Used for side-by-side view.
    split_ratio: f32,
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
            raw_json_buffer: String::new(),
            schema_path: default_schema_path(),
            split_ratio: 0.5,
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

                        // Trust status (from active manifest, not index 0)
                        if let Some(trust_status) =
                            get_trust_status(&manifest.manifest_value, &manifest.active_label)
                        {
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
                                        egui::Color32::from_rgb(128, 128, 128),
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

                        // Toggle: Raw JSON replaces both Tree and Manifest Data views when on
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.show_raw_json, "");
                            EmojiLabel::new(
                                egui::RichText::new("📄 Show Raw JSON (replaces tree and manifest data)")
                                    .size(15.0),
                            )
                            .show(ui);
                        });

                        if self.show_raw_json {
                            ui.separator();
                            EmojiLabel::new(egui::RichText::new("📋 Raw JSON:").size(17.0))
                                .show(ui);

                            // Read-only: refresh buffer from manifest each frame so edits are discarded
                            self.raw_json_buffer = manifest.manifest_json.clone();
                            let mut editor = CodeEditor::default()
                                .id_source("raw_json")
                                .with_rows(28)
                                .with_ui_fontsize(ui)
                                .with_theme(ColorTheme::AYU)
                                .with_syntax(json_syntax())
                                .with_numlines(false)
                                .vscroll(true);
                            editor.show(ui, &mut self.raw_json_buffer);
                        } else {
                            // Side by side: Manifest Data (left), resizer, Tree (right); 50/50 by default, resizable via drag
                            ui.separator();
                            let fill_height = ui.available_height();
                            let total_width = ui.available_width();
                            let content_width = (total_width - RESIZE_HANDLE_WIDTH).max(0.0);
                            let left_width = content_width * self.split_ratio;
                            let right_width = content_width - left_width;

                            ui.horizontal(|ui| {
                                // Left: Manifest Data — claim full left_width so layout advances correctly (egui advances by child min_rect)
                                let left_response = ui.allocate_ui_with_layout(
                                    egui::vec2(left_width, fill_height),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_min_size(egui::vec2(left_width, fill_height));
                                        EmojiLabel::new(
                                            egui::RichText::new("📊 Manifest Data").size(16.0),
                                        )
                                        .show(ui);
                                        egui::ScrollArea::vertical()
                                            .id_salt("manifest_data")
                                            .show(ui, |ui| {
                                                // Force content width to match panel so tree fills on init (not only after resize)
                                                ui.set_min_width((left_width - 16.0).max(0.0));
                                                JsonTree::new("manifest-data-tree", &manifest.manifest_value)
                                                    .default_expand(DefaultExpand::ToLevel(2))
                                                    .show(ui);
                                            });
                                    },
                                );

                                // Resize handle: stable id so drag is tracked across frames; draggable divider
                                let left_rect = left_response.response.rect;
                                let resize_rect = egui::Rect::from_min_size(
                                    left_rect.right_top(),
                                    egui::vec2(RESIZE_HANDLE_WIDTH, fill_height),
                                );
                                let resize_response = ui.push_id("manifest_tree_resize", |ui| {
                                    ui.allocate_rect(resize_rect, egui::Sense::drag())
                                })
                                .inner;
                                if resize_response.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                                }
                                if resize_response.dragged() {
                                    if let Some(pos) = ui.ctx().input(|i| i.pointer.latest_pos()) {
                                        let panels_left = left_rect.left();
                                        let new_ratio = (pos.x - panels_left) / content_width;
                                        self.split_ratio =
                                            new_ratio.clamp(MIN_PANEL_RATIO, MAX_PANEL_RATIO);
                                    }
                                }
                                // Visible divider line
                                let painter = ui.painter_at(resize_rect);
                                let line_x = resize_rect.center().x;
                                painter.line_segment(
                                    [
                                        egui::pos2(line_x, resize_rect.top()),
                                        egui::pos2(line_x, resize_rect.bottom()),
                                    ],
                                    egui::Stroke::new(
                                        1.0,
                                        ui.visuals().widgets.noninteractive.fg_stroke.color,
                                    ),
                                );

                                // Right: Tree view — claim full right_width so it gets exactly the remaining space
                                ui.allocate_ui_with_layout(
                                    egui::vec2(right_width, fill_height),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_min_size(egui::vec2(right_width, fill_height));
                                        EmojiLabel::new(
                                            egui::RichText::new("🌳 Manifest & Ingredients Tree")
                                                .size(16.0),
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

/// Collect ingredients from a manifest object (claim or full manifest).
/// Handles: claim.ingredients, claim.v2.ingredients, assertions with c2pa.ingredient*.
fn collect_ingredients_from_manifest(manifest_obj: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut out = Vec::new();
    let obj = match manifest_obj.as_object() {
        Some(o) => o,
        None => return out,
    };

    // Claim-level ingredients (claim.v2 or claim)
    for key in ["claim.v2", "claim"] {
        if let Some(claim) = obj.get(key) {
            if let Some(arr) = claim.get("ingredients").and_then(|v| v.as_array()) {
                for ing in arr {
                    out.push(ing);
                }
            }
        }
    }

    // Top-level ingredients (flat manifest format)
    if let Some(arr) = obj.get("ingredients").and_then(|v| v.as_array()) {
        for ing in arr {
            out.push(ing);
        }
    }

    // Assertions: c2pa.ingredient, c2pa.ingredient.v3, etc.
    if let Some(assertions) = obj.get("assertions").and_then(|v| v.as_object()) {
        for (key, val) in assertions {
            if key.contains("ingredient") {
                if let Some(data) = val.get("data") {
                    if let Some(arr) = data.as_array() {
                        for ing in arr {
                            out.push(ing);
                        }
                    } else if data.get("relationship").is_some()
                        || data.get("title").is_some()
                        || data.get("instanceID").is_some()
                    {
                        out.push(data);
                    }
                } else if val.get("relationship").is_some()
                    || val.get("title").is_some()
                    || val.get("instanceID").is_some()
                {
                    out.push(val);
                }
            }
        }
    }

    out
}

/// Find a manifest in the document by label (or instance_id). Used to resolve nested ingredients.
fn find_manifest_by_label<'a>(
    manifest_value: &'a serde_json::Value,
    label: &str,
) -> Option<&'a serde_json::Value> {
    let arr = manifest_value.get("manifests")?.as_array()?;
    arr.iter().find(|m| {
        m.get("label").and_then(|v| v.as_str()) == Some(label)
            || m.get("claim.v2")
                .or_else(|| m.get("claim"))
                .and_then(|c| c.get("instanceID").or_else(|| c.get("instance_id")))
                .and_then(|v| v.as_str())
                == Some(label)
    })
}

/// Get nested manifest for an ingredient (if this ingredient is a C2PA asset, its manifest may be in manifests[]).
/// Resolves by active_manifest / activeManifest first, then instance_id / documentID / label.
fn nested_manifest_for_ingredient<'a>(
    manifest_value: &'a serde_json::Value,
    ingredient: &serde_json::Value,
) -> Option<&'a serde_json::Value> {
    let mut labels_to_try: Vec<&str> = Vec::new();
    if let Some(s) = ingredient.get("active_manifest").and_then(|v| v.as_str()) {
        labels_to_try.push(s);
    }
    if let Some(s) = ingredient.get("activeManifest").and_then(|v| v.as_str()) {
        labels_to_try.push(s);
    }
    if let Some(s) = ingredient
        .get("activeManifest")
        .and_then(|o| o.get("uri"))
        .and_then(|v| v.as_str())
    {
        labels_to_try.push(s);
    }
    for key in ["instanceID", "instance_id", "documentID", "label"] {
        if let Some(s) = ingredient.get(key).and_then(|v| v.as_str()) {
            labels_to_try.push(s);
        }
    }
    for label in labels_to_try {
        if let Some(m) = find_manifest_by_label(manifest_value, label) {
            return Some(m);
        }
    }
    None
}

/// Extract digital source type from a manifest: look for a c2pa.actions or c2pa.actions.v2 assertion
/// (it has an "actions" array); find an action with "action": "c2pa.created" and "digitalSourceType";
/// return the last segment of the digitalSourceType URL.
fn manifest_digital_source_type(manifest_obj: &serde_json::Value) -> Option<String> {
    let try_actions_array = |actions: &serde_json::Value| -> Option<String> {
        let arr = actions.as_array()?;
        for act in arr {
            if act.get("action").and_then(|v| v.as_str()) != Some("c2pa.created") {
                continue;
            }
            let url = act.get("digitalSourceType").and_then(|v| v.as_str())?;
            return Some(url.split('/').filter(|s| !s.is_empty()).last()?.to_string());
        }
        None
    };

    let try_assertions_obj = |assertions: &serde_json::Value| -> Option<String> {
        let obj = assertions.as_object()?;
        for key in ["c2pa.actions.v2", "c2pa.actions"] {
            let assertion = obj.get(key)?;
            if let Some(actions) = assertion.get("actions") {
                if let Some(s) = try_actions_array(actions) {
                    return Some(s);
                }
            }
        }
        None
    };

    let try_assertions_any = |assertions: &serde_json::Value| -> Option<String> {
        if let Some(s) = try_assertions_obj(assertions) {
            return Some(s);
        }
        if let Some(arr) = assertions.as_array() {
            for a in arr {
                let label = a.get("label").and_then(|v| v.as_str())?;
                if label != "c2pa.actions" && label != "c2pa.actions.v2" {
                    continue;
                }
                let data = a.get("data")?;
                if let Some(actions) = data.get("actions") {
                    if let Some(s) = try_actions_array(actions) {
                        return Some(s);
                    }
                }
            }
        }
        None
    };

    if let Some(assertions) = manifest_obj.get("assertions") {
        if let Some(s) = try_assertions_any(assertions) {
            return Some(s);
        }
    }
    if let Some(claim) = manifest_obj
        .get("claim.v2")
        .or_else(|| manifest_obj.get("claim"))
    {
        if let Some(assertions) = claim.get("assertions") {
            if let Some(s) = try_assertions_any(assertions) {
                return Some(s);
            }
        }
    }
    None
}

/// Extract claim type ("claim.v2" or "claim"), claim_generator string, and formatted claim_generator_info from a manifest object.
fn manifest_claim_info(
    manifest_obj: &serde_json::Value,
) -> (Option<&'static str>, Option<String>, Option<String>) {
    let (claim_type, claim_obj) = if manifest_obj.get("claim.v2").is_some() {
        (Some("claim.v2"), manifest_obj.get("claim.v2"))
    } else if manifest_obj.get("claim").is_some() {
        (Some("claim"), manifest_obj.get("claim"))
    } else {
        (None, None)
    };

    let claim = match claim_obj {
        Some(c) => c,
        None => {
            // Flat format: claim_generator_info at top level
            let cgi = format_claim_generator_info(manifest_obj.get("claim_generator_info"));
            return (None, None, cgi);
        }
    };

    let gen = claim
        .get("claim_generator")
        .or_else(|| claim.get("claimGenerator"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let cgi = format_claim_generator_info(
        claim
            .get("claim_generator_info")
            .or_else(|| manifest_obj.get("claim_generator_info")),
    );
    (claim_type, gen, cgi)
}

/// Format claim_generator_info (array or object) as a short string for display.
fn format_claim_generator_info(cgi: Option<&serde_json::Value>) -> Option<String> {
    let cgi = cgi?;
    let arr = cgi.as_array();
    let objs: Vec<&serde_json::Value> = if let Some(a) = arr {
        a.iter().collect()
    } else if cgi.get("name").is_some() || cgi.get("version").is_some() {
        return Some(format_one_cgi_entry(cgi));
    } else {
        return None;
    };
    if objs.is_empty() {
        return None;
    }
    let parts: Vec<String> = objs.iter().map(|o| format_one_cgi_entry(o)).collect();
    Some(parts.join("; "))
}

fn format_one_cgi_entry(entry: &serde_json::Value) -> String {
    let name = entry
        .get("name")
        .or_else(|| entry.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("—");
    let version = entry.get("version").and_then(|v| v.as_str()).unwrap_or("");
    if version.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", name, version)
    }
}

/// Get trust status from a manifest object (status.trust).
fn trust_status_from_manifest(manifest_obj: &serde_json::Value) -> Option<String> {
    manifest_obj
        .get("status")
        .and_then(|s| s.get("trust"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

/// Get title or identifier for an ingredient or manifest
fn ingredient_display_name(ing: &serde_json::Value) -> String {
    ing.get("title")
        .or_else(|| ing.get("dc:title"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            ing.get("instanceID")
                .or_else(|| ing.get("instance_id"))
                .or_else(|| ing.get("documentID"))
                .or_else(|| ing.get("label"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "(unnamed)".to_string())
}

/// Recursively display manifest → ingredients tree. Root = active manifest, children = ingredients
/// with relationship (parentOf, componentOf, inputOf); recurse into nested ingredients.
fn display_manifest_ingredient_tree(
    ui: &mut egui::Ui,
    manifest_value: &serde_json::Value,
    active_label: &str,
) {
    // Resolve active manifest: either from manifests[] or use root as single claim
    let active_manifest = manifest_value
        .get("manifests")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|m| m.get("label").and_then(|v| v.as_str()) == Some(active_label))
        })
        .or_else(|| {
            if manifest_value.get("claim_generator_info").is_some()
                || manifest_value.get("title").is_some()
            {
                Some(manifest_value)
            } else {
                None
            }
        });

    let active_manifest = match active_manifest {
        Some(m) => m,
        None => {
            ui.colored_label(
                egui::Color32::from_rgb(200, 100, 100),
                "Could not find active manifest in document.",
            );
            return;
        }
    };

    let root_title = active_manifest
        .get("claim.v2")
        .or_else(|| active_manifest.get("claim"))
        .and_then(|c| c.get("title").or_else(|| c.get("dc:title")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            active_manifest
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| active_label.to_string());

    egui::CollapsingHeader::new(
        egui::RichText::new(format!("📜 Active Manifest: {}", root_title))
            .size(15.0)
            .color(egui::Color32::from_rgb(200, 160, 50)),
    )
    .default_open(true)
    .show(ui, |ui| {
        // Root manifest details: claim type, claim generator, claim_generator_info, label
        let (claim_type, claim_gen, claim_gen_info) = manifest_claim_info(active_manifest);
        if let Some(ct) = claim_type {
            ui.label(
                egui::RichText::new(format!("Claim type: {}", ct))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(128, 128, 128)),
            );
        }
        if let Some(ref gen) = claim_gen {
            ui.label(
                egui::RichText::new(format!("Claim generator: {}", gen))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(128, 128, 128)),
            );
        }
        if let Some(ref info) = claim_gen_info {
            ui.label(
                egui::RichText::new(format!("Claim generator info: {}", info))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(128, 128, 128)),
            );
        }
        if let Some(label) = active_manifest.get("label").and_then(|v| v.as_str()) {
            ui.label(
                egui::RichText::new(format!("Label: {}", label))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(128, 128, 128)),
            );
        }
        if let Some(trust) = trust_status_from_manifest(active_manifest) {
            let (icon, color) = match trust.as_str() {
                "signingCredential.trusted" => ("🔒", egui::Color32::from_rgb(80, 220, 120)),
                "signingCredential.untrusted" => ("🔓", egui::Color32::from_rgb(255, 100, 100)),
                _ => ("", egui::Color32::from_rgb(128, 128, 128)),
            };
            let text = if icon.is_empty() {
                format!("Trust: {}", trust)
            } else {
                format!("Trust: {} {}", icon, trust)
            };
            ui.label(egui::RichText::new(text).size(12.0).color(color));
        }
        let ingredients = collect_ingredients_from_manifest(active_manifest);
        if let Some(dst) = manifest_digital_source_type(active_manifest) {
            ui.label(
                egui::RichText::new(format!("Digital source type: {}", dst))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(128, 128, 128)),
            );
        } else {
            for ing in &ingredients {
                if let Some(nested) = nested_manifest_for_ingredient(manifest_value, ing) {
                    if let Some(dst) = manifest_digital_source_type(nested) {
                        ui.label(
                            egui::RichText::new(format!(
                                "Digital source type: {} (from ingredient manifest)",
                                dst
                            ))
                            .size(12.0)
                            .color(egui::Color32::from_rgb(128, 128, 128)),
                        );
                        break;
                    }
                }
            }
        }
        ui.add_space(4.0);
        if ingredients.is_empty() {
            ui.label("(no ingredients)");
            return;
        }
        for ing in ingredients {
            render_ingredient_node(ui, manifest_value, ing, 0);
        }
    });
}

/// Render one ingredient node with relationship (parentOf/componentOf/inputOf), details, and optional nested ingredients
fn render_ingredient_node(
    ui: &mut egui::Ui,
    manifest_value: &serde_json::Value,
    ingredient: &serde_json::Value,
    depth: usize,
) {
    let relationship = ingredient
        .get("relationship")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let name = ingredient_display_name(ingredient);
    let indent = "  ".repeat(depth);

    let badge_color = match relationship {
        "parentOf" => egui::Color32::from_rgb(100, 180, 255),
        "componentOf" => egui::Color32::from_rgb(120, 220, 120),
        "inputOf" => egui::Color32::from_rgb(255, 200, 100),
        _ => egui::Color32::from_rgb(128, 128, 128),
    };

    let nested_manifest = nested_manifest_for_ingredient(manifest_value, ingredient);
    let nested_ingredients: Vec<_> = nested_manifest
        .map(|m| collect_ingredients_from_manifest(m))
        .unwrap_or_default();
    let has_nested = !nested_ingredients.is_empty();

    let header_text = format!("{}[{}] {}", indent, relationship, name);

    if has_nested {
        egui::CollapsingHeader::new(
            egui::RichText::new(header_text)
                .size(14.0)
                .color(badge_color),
        )
        .default_open(true)
        .show(ui, |ui| {
            ingredient_node_details(ui, manifest_value, ingredient);
            ui.add_space(4.0);
            for ing in &nested_ingredients {
                render_ingredient_node(ui, manifest_value, ing, depth + 1);
            }
        });
    } else {
        egui::CollapsingHeader::new(
            egui::RichText::new(header_text)
                .size(14.0)
                .color(badge_color),
        )
        .default_open(true)
        .show(ui, |ui| {
            ingredient_node_details(ui, manifest_value, ingredient);
        });
    }
}

/// Show title, format, instance_id/label, active manifest, claim generator info, claim type, and trust status for an ingredient.
fn ingredient_node_details(
    ui: &mut egui::Ui,
    manifest_value: &serde_json::Value,
    ingredient: &serde_json::Value,
) {
    let gray = egui::Color32::from_rgb(128, 128, 128);
    let small = 12.0f32;
    if let Some(s) = ingredient
        .get("title")
        .or_else(|| ingredient.get("dc:title"))
        .and_then(|v| v.as_str())
    {
        ui.label(
            egui::RichText::new(format!("Title: {}", s))
                .size(small)
                .color(gray),
        );
    }
    if let Some(s) = ingredient.get("format").and_then(|v| v.as_str()) {
        ui.label(
            egui::RichText::new(format!("Format: {}", s))
                .size(small)
                .color(gray),
        );
    }
    let id = ingredient
        .get("instanceID")
        .or_else(|| ingredient.get("instance_id"))
        .or_else(|| ingredient.get("documentID"))
        .or_else(|| ingredient.get("label"))
        .and_then(|v| v.as_str());
    if let Some(s) = id {
        ui.label(
            egui::RichText::new(format!("Instance ID: {}", s))
                .size(small)
                .color(gray),
        );
    }
    if let Some(label) = ingredient
        .get("active_manifest")
        .and_then(|v| v.as_str())
        .or_else(|| ingredient.get("activeManifest").and_then(|v| v.as_str()))
    {
        ui.label(
            egui::RichText::new(format!("Active manifest: {}", label))
                .size(small)
                .color(gray),
        );
    } else if ingredient
        .get("activeManifest")
        .and_then(|v| v.as_object())
        .is_some()
    {
        if let Some(uri) = ingredient
            .get("activeManifest")
            .and_then(|o| o.get("uri"))
            .and_then(|v| v.as_str())
        {
            ui.label(
                egui::RichText::new(format!("Active manifest: {}", uri))
                    .size(small)
                    .color(gray),
            );
        }
    }
    // Claim generator / claim type / digital source type from nested manifest (if this ingredient is a C2PA asset)
    if let Some(nested) = nested_manifest_for_ingredient(manifest_value, ingredient) {
        if let Some(dst) = manifest_digital_source_type(nested) {
            ui.label(
                egui::RichText::new(format!("Digital source type: {}", dst))
                    .size(small)
                    .color(gray),
            );
        }
        let (claim_type, claim_gen, claim_gen_info) = manifest_claim_info(nested);
        if let Some(ct) = claim_type {
            ui.label(
                egui::RichText::new(format!("Claim type: {}", ct))
                    .size(small)
                    .color(gray),
            );
        }
        if let Some(ref gen) = claim_gen {
            ui.label(
                egui::RichText::new(format!("Claim generator: {}", gen))
                    .size(small)
                    .color(gray),
            );
        }
        if let Some(ref info) = claim_gen_info {
            ui.label(
                egui::RichText::new(format!("Claim generator info: {}", info))
                    .size(small)
                    .color(gray),
            );
        }
        // Trust status for ingredient (from nested manifest)
        if let Some(trust) = trust_status_from_manifest(nested) {
            let (icon, color) = match trust.as_str() {
                "signingCredential.trusted" => ("🔒", egui::Color32::from_rgb(80, 220, 120)),
                "signingCredential.untrusted" => ("🔓", egui::Color32::from_rgb(255, 100, 100)),
                _ => ("", gray),
            };
            let text = if icon.is_empty() {
                format!("Trust: {}", trust)
            } else {
                format!("Trust: {} {}", icon, trust)
            };
            ui.label(egui::RichText::new(text).size(small).color(color));
        } else {
            ui.label(
                egui::RichText::new("Trust: — (no status)")
                    .size(small)
                    .color(gray),
            );
        }
    }
}

/// Extract trust status from the active manifest (identified by label, not array index).
fn get_trust_status(manifest_value: &serde_json::Value, active_label: &str) -> Option<String> {
    let active_manifest = manifest_value
        .get("manifests")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|m| m.get("label").and_then(|v| v.as_str()) == Some(active_label))
        })
        .or_else(|| {
            if manifest_value.get("claim_generator_info").is_some()
                || manifest_value.get("title").is_some()
            {
                Some(manifest_value)
            } else {
                None
            }
        })?;
    active_manifest
        .get("status")?
        .get("trust")?
        .as_str()
        .map(|s| s.to_string())
}
