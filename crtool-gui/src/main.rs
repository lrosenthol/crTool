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

mod app;
mod document;
mod manifest_ui;
mod tab_viewer;
mod util;

#[cfg(target_os = "macos")]
mod macos_open_document;
mod security_scoped;

use app::CrtoolApp;
use std::path::PathBuf;
use util::arg_to_path;

fn main() -> Result<(), eframe::Error> {
    #[cfg(target_os = "macos")]
    macos_open_document::install_handler();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "C2PA Content Credential Tool",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            #[cfg(target_os = "macos")]
            macos_open_document::install_cocoa_handler();
            let extraction_settings = util::gui_extraction_settings();

            let mut initial_files: Vec<PathBuf> = std::env::args()
                .skip(1)
                .filter_map(|arg| {
                    let path = arg_to_path(&arg);
                    (path.is_file() && crtool::is_supported_asset_path(&path)).then_some(path)
                })
                .collect();
            #[cfg(target_os = "macos")]
            initial_files.extend(macos_open_document::drain_pending_files());

            Ok(Box::new(CrtoolApp::new_with_optional_files(
                initial_files,
                extraction_settings,
            )))
        }),
    )
}
