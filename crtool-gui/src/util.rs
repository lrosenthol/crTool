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
    apply_trust_settings, C2PA_TRUST_ANCHORS_URL, INTERIM_ALLOWED_LIST_URL,
    INTERIM_TRUST_ANCHORS_URL, INTERIM_TRUST_CONFIG_URL,
};
use eframe::egui;
use egui_code_editor::Syntax;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Syntax definition for JSON (keywords true/false/null) for the code editor.
pub(crate) fn json_syntax() -> Syntax {
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

/// Convert a command-line argument to a file path. Handles macOS `file://` URLs
/// that the system may pass when opening via "Open With" or drop-on-icon.
pub(crate) fn arg_to_path(arg: &str) -> PathBuf {
    let arg = arg.trim();
    if let Some(path_str) = arg.strip_prefix("file://") {
        let path_str = path_str.trim_start_matches('/');
        let decoded = urlencoding::decode(path_str).unwrap_or(std::borrow::Cow::Borrowed(path_str));
        return PathBuf::from(decoded.as_ref());
    }
    PathBuf::from(arg)
}

/// Load C2PA and Content Credentials trust lists and apply for validation. Runs at GUI startup.
pub(crate) fn load_trust_lists_for_gui() {
    let client = match reqwest::blocking::Client::builder()
        .user_agent("crTool-gui/1.0")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Trust lists: failed to create HTTP client: {}", e);
            return;
        }
    };
    let fetch = |url: &str| -> Option<String> {
        client
            .get(url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.text())
            .map_err(|e| eprintln!("Trust lists: failed to fetch {}: {}", url, e))
            .ok()
    };
    let c2pa_anchors = match fetch(C2PA_TRUST_ANCHORS_URL) {
        Some(s) => s,
        None => return,
    };
    let interim_anchors = match fetch(INTERIM_TRUST_ANCHORS_URL) {
        Some(s) => s,
        None => return,
    };
    let trust_anchors = format!(
        "{}\n{}",
        c2pa_anchors.trim_end(),
        interim_anchors.trim_end()
    );
    let allowed_list = fetch(INTERIM_ALLOWED_LIST_URL);
    let trust_config = fetch(INTERIM_TRUST_CONFIG_URL);
    if let Err(e) = apply_trust_settings(
        &trust_anchors,
        allowed_list.as_deref().map(|s| s.trim()),
        trust_config.as_deref().map(|s| s.trim()),
    ) {
        eprintln!("Trust lists: failed to apply settings: {}", e);
    }
}

/// Helper to get selected text from the context (for Edit → Copy).
pub(crate) fn get_selected_text(_ctx: &egui::Context) -> String {
    String::new()
}
