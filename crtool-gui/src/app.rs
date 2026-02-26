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

//! Main application: dock state, menu bar, and central panel (welcome or DockArea).

use crate::document::{self, DocumentTab};
use crate::tab_viewer::CrtoolTabViewer;
use crate::util;
use crtool::{crjson_schema_path, is_supported_asset_path};
use eframe::egui;
use egui_dock::{DockArea, DockState, Style};
use egui_twemoji::EmojiLabel;
use std::path::PathBuf;

/// Main app state: multi-document dock and schema path.
pub(crate) struct CrtoolApp {
    /// Multi-document dock state (tabs can be undocked into separate windows).
    pub(crate) dock_state: DockState<DocumentTab>,
    /// Schema path for validation (shared).
    pub(crate) schema_path: PathBuf,
}

impl CrtoolApp {
    pub(crate) fn new() -> Self {
        Self::new_with_optional_files(Vec::new())
    }

    pub(crate) fn new_with_optional_files(initial_files: Vec<PathBuf>) -> Self {
        let mut app = Self {
            dock_state: DockState::new(Vec::new()),
            schema_path: crjson_schema_path(),
        };
        app.add_documents(initial_files);
        app
    }

    /// Open one or more files as new tabs (focus goes to the last opened).
    pub(crate) fn add_documents(&mut self, paths: Vec<PathBuf>) {
        let schema_path = self.schema_path.clone();
        for path in paths {
            if !path.is_file() || !is_supported_asset_path(&path) {
                continue;
            }
            let tab = document::load_document(path, &schema_path);
            self.dock_state.push_to_focused_leaf(tab);
        }
    }

    /// Returns the location of the currently focused tab for Close / Save As. None if no tabs.
    pub(crate) fn focused_tab_location(
        &self,
    ) -> Option<(
        egui_dock::SurfaceIndex,
        egui_dock::NodeIndex,
        egui_dock::TabIndex,
    )> {
        self.dock_state
            .focused_leaf()
            .and_then(|(surface, node_index)| {
                let tree = &self.dock_state[surface];
                let node = &tree[node_index];
                let leaf = node.get_leaf()?;
                Some((surface, node_index, leaf.active))
            })
    }
}

impl Default for CrtoolApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for CrtoolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut paths_to_open: Vec<PathBuf> = Vec::new();

        #[cfg(target_os = "macos")]
        paths_to_open.extend(
            crate::macos_open_document::drain_pending_files()
                .into_iter()
                .filter(|p| p.is_file() && is_supported_asset_path(p)),
        );

        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            if let Some(path) = file.path.filter(|p| is_supported_asset_path(p)) {
                paths_to_open.push(path);
            }
        }

        if !paths_to_open.is_empty() {
            self.add_documents(paths_to_open);
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📂 Open...").clicked() {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter("C2PA-supported files", crtool::SUPPORTED_ASSET_EXTENSIONS)
                            .pick_files()
                        {
                            self.add_documents(paths);
                        }
                        ui.close();
                    }

                    let has_tabs = self.dock_state.iter_all_tabs().next().is_some();
                    let focused = self.focused_tab_location();

                    ui.add_enabled_ui(focused.is_some(), |ui| {
                        if ui.button("❌ Close").clicked() {
                            if let Some(loc) = focused {
                                self.dock_state.remove_tab(loc);
                            }
                            ui.close();
                        }
                    });

                    ui.add_enabled_ui(has_tabs, |ui| {
                        if ui.button("❌ Close All").clicked() {
                            self.dock_state.retain_tabs(|_| false);
                            ui.close();
                        }
                    });

                    ui.separator();

                    ui.add_enabled_ui(focused.is_some(), |ui| {
                        if ui.button("💾 Save As...").clicked() {
                            if let Some((_, tab)) = self.dock_state.find_active_focused() {
                                if let Ok(ref manifest) = tab.extraction_result {
                                    let default_name = tab
                                        .file_path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .map(|s| format!("{}-indicators.json", s))
                                        .unwrap_or_else(|| "manifest-indicators.json".to_string());
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
                            }
                            ui.close();
                        }
                    });
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("📋 Copy").clicked() {
                        ctx.copy_text(util::get_selected_text(ctx));
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Select All").clicked() {
                        ui.close();
                    }
                });
            });
        });

        let has_any_tabs = self.dock_state.iter_all_tabs().next().is_some();
        let mut tab_viewer = CrtoolTabViewer;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("C2PA Content Credential Tool");
            ui.separator();

            if !has_any_tabs {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    EmojiLabel::new(
                        egui::RichText::new(
                            "👆 Open one or more C2PA-supported files (image, video, audio, or PDF) to extract manifests. \
                             You can drag files onto this window or use Open below.",
                        )
                        .size(16.0),
                    )
                    .show(ui);
                    ui.add_space(20.0);
                    if ui
                        .button(egui::RichText::new("📂 Select File(s)...").size(16.0))
                        .clicked()
                    {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter("C2PA-supported files", crtool::SUPPORTED_ASSET_EXTENSIONS)
                            .pick_files()
                        {
                            self.add_documents(paths);
                        }
                    }
                });
            } else {
                let style = Style::from_egui(ui.style().as_ref());
                DockArea::new(&mut self.dock_state)
                    .style(style)
                    .show_inside(ui, &mut tab_viewer);
            }
        });
    }
}
