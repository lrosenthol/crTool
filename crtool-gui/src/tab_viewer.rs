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

//! egui_dock TabViewer: tab title and content for each document.

use crate::document::{self, DocumentTab};
use eframe::egui;
use egui_dock::TabViewer;

/// TabViewer for the dock: shows document title and content per tab.
pub(crate) struct CrtoolTabViewer;

impl TabViewer for CrtoolTabViewer {
    type Tab = DocumentTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let name = tab
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| tab.file_path.to_string_lossy().into_owned());
        name.into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        document::show_document_tab_ui(ui, tab);
    }
}
