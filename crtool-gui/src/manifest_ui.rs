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

//! Manifest introspection and ingredient tree display for the document tab UI.

use eframe::egui;

/// Extract generator name from manifest JSON for the active manifest.
pub(crate) fn get_generator_name(
    manifest_json: &serde_json::Value,
    active_label: &str,
) -> Option<String> {
    let get_from_claim = |claim: &serde_json::Value| -> Option<String> {
        if let Some(name) = claim.get("claim_generator_info") {
            if name.is_string() {
                return name.as_str().map(|s| s.to_string());
            }
            if let Some(obj) = name.as_object() {
                if let Some(n) = obj.get("name").and_then(|v| v.as_str()) {
                    return Some(n.to_string());
                }
            }
        }
        if let Some(name) = claim.get("claimGenerator") {
            if name.is_string() {
                return name.as_str().map(|s| s.to_string());
            }
            if let Some(obj) = name.as_object() {
                if let Some(n) = obj.get("name").and_then(|v| v.as_str()) {
                    return Some(n.to_string());
                }
            }
        }
        None
    };

    let manifests = manifest_json.get("manifests").and_then(|v| v.as_array());
    let manifest_val = manifests
        .and_then(|arr| {
            arr.iter().find(|m| {
                m.get("label")
                    .and_then(|l| l.as_str())
                    .map(|lbl| lbl == active_label)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(manifest_json);

    manifest_val
        .get("claim.v2")
        .and_then(get_from_claim)
        .or_else(|| manifest_val.get("claim").and_then(get_from_claim))
        .or_else(|| get_from_claim(manifest_val))
}

/// Get "Issued by" name and date from the active manifest's signature.
/// Subject is read from `signature.certificateInfo.subject` (new schema); falls back to
/// `signature.subject` for older crJSON. Date is from `signature.timeStampInfo.timestamp`
/// (new schema); falls back to top-level `signature.timestamp` for older crJSON.
pub(crate) fn get_signature_issued_info(
    manifest_value: &serde_json::Value,
    active_label: &str,
) -> Option<(String, String)> {
    let active_manifest = manifest_value
        .get("manifests")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|m| m.get("label").and_then(|v| v.as_str()) == Some(active_label))
        })?;
    let sig = active_manifest.get("signature")?.as_object()?;
    let subject = sig
        .get("certificateInfo")
        .and_then(|ci| ci.get("subject"))
        .or_else(|| sig.get("subject"));
    let name = subject
        .and_then(|s| s.get("CN").or_else(|| s.get("cn")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            subject
                .and_then(|s| s.get("O").or_else(|| s.get("o")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "—".to_string());
    let date = sig
        .get("timeStampInfo")
        .and_then(|ts| ts.get("timestamp"))
        .or_else(|| sig.get("timestamp"))
        .and_then(|v| v.as_str())
        .and_then(format_rfc3339_date)
        .unwrap_or_else(|| "—".to_string());
    Some((name, date))
}

/// Get timestamp presence and TSA certificate authority name from the active manifest's signature.
/// Returns (timestamp_present, tsa_authority_name). When `signature.timeStampInfo` exists,
/// the authority name is taken from `timeStampInfo.certificateInfo.issuer` (CN or O).
pub(crate) fn get_timestamp_info(
    manifest_value: &serde_json::Value,
    active_label: &str,
) -> (bool, Option<String>) {
    let sig = manifest_value
        .get("manifests")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|m| m.get("label").and_then(|v| v.as_str()) == Some(active_label))
        })
        .and_then(|m| m.get("signature"))
        .and_then(|s| s.as_object());
    let Some(sig) = sig else {
        return (false, None);
    };
    let ts_info = match sig.get("timeStampInfo").and_then(|t| t.as_object()) {
        Some(t) => t,
        None => return (false, None),
    };
    let issuer = ts_info
        .get("certificateInfo")
        .and_then(|ci| ci.get("issuer"));
    let name = issuer
        .and_then(|s| s.get("CN").or_else(|| s.get("cn")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            issuer
                .and_then(|s| s.get("O").or_else(|| s.get("o")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });
    (true, name)
}

/// Derive trust status from a manifest's `validationResults` (success/failure arrays).
/// New schema: each manifest has its own validationResults; trust is inferred from status codes.
fn trust_from_validation_results(vr: &serde_json::Value) -> Option<String> {
    let vr = vr.as_object()?;
    let has_code = |key: &str, code: &str| -> bool {
        vr.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .any(|e| e.get("code").and_then(|c| c.as_str()) == Some(code))
            })
            .unwrap_or(false)
    };
    if has_code("failure", "signingCredential.untrusted") {
        return Some("signingCredential.untrusted".to_string());
    }
    if has_code("success", "signingCredential.trusted") {
        return Some("signingCredential.trusted".to_string());
    }
    None
}

/// Extract trust status from the active manifest (manifests[] entry matching active_label).
/// Uses that manifest's validationResults (success/failure codes); falls back to status.trust for legacy crJSON.
pub(crate) fn get_trust_status(
    manifest_value: &serde_json::Value,
    active_label: &str,
) -> Option<String> {
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
        .get("validationResults")
        .and_then(trust_from_validation_results)
        .or_else(|| {
            active_manifest
                .get("status")
                .and_then(|s| s.get("trust"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        })
}

/// One validation failure entry from validationResults (code + optional url/explanation).
#[derive(Clone, Debug)]
pub(crate) struct ValidationFailureEntry {
    pub(crate) code: String,
    pub(crate) explanation: Option<String>,
    pub(crate) url: Option<String>,
    /// When from ingredientDeltas, e.g. "Ingredient: …"
    pub(crate) source: Option<String>,
}

/// Collect validation failure entries for the active manifest. Uses the manifest record's
/// `validationResults.failure` and `ingredientDeltas[].validationDeltas.failure`. Falls back to
/// document-level `validationResults.activeManifest` / `ingredientDeltas` for legacy crJSON.
/// Excludes signingCredential.untrusted (shown as trust status).
pub(crate) fn get_validation_failures(
    manifest_value: &serde_json::Value,
    active_label: &str,
) -> Vec<ValidationFailureEntry> {
    const UNTRUSTED_CODE: &str = "signingCredential.untrusted";

    let mut out = Vec::new();

    let push_entries = |out: &mut Vec<ValidationFailureEntry>,
                        arr: Option<&serde_json::Value>,
                        source: Option<String>| {
        let arr = match arr.and_then(|v| v.as_array()) {
            Some(a) => a,
            None => return,
        };
        for entry in arr {
            let obj = match entry.as_object() {
                Some(o) => o,
                None => continue,
            };
            let code = match obj.get("code").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => continue,
            };
            if code == UNTRUSTED_CODE {
                continue;
            }
            out.push(ValidationFailureEntry {
                code: code.to_string(),
                explanation: obj
                    .get("explanation")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                url: obj.get("url").and_then(|v| v.as_str()).map(String::from),
                source: source.clone(),
            });
        }
    };

    // New schema: per-manifest validationResults (statusCodes) and ingredientDeltas
    let active_manifest = manifest_value
        .get("manifests")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|m| m.get("label").and_then(|v| v.as_str()) == Some(active_label))
        });

    if let Some(am) = active_manifest {
        if let Some(vr) = am.get("validationResults").and_then(|v| v.as_object()) {
            push_entries(&mut out, vr.get("failure"), None);
        }
        if let Some(deltas) = am.get("ingredientDeltas").and_then(|v| v.as_array()) {
            for delta in deltas {
                let uri = delta
                    .get("ingredientAssertionURI")
                    .and_then(|v| v.as_str())
                    .map(|s| format!("Ingredient: {}", s));
                let vd = delta.get("validationDeltas").and_then(|v| v.as_object());
                if let Some(vd) = vd {
                    push_entries(&mut out, vd.get("failure"), uri);
                }
            }
        }
    }

    // Fallback: legacy document-level validationResults (activeManifest + ingredientDeltas)
    if out.is_empty() {
        let vr = manifest_value
            .get("validationResults")
            .and_then(|v| v.as_object());
        if let Some(vr) = vr {
            if let Some(am) = vr.get("activeManifest").and_then(|v| v.as_object()) {
                push_entries(&mut out, am.get("failure"), None);
            }
            if let Some(deltas) = vr.get("ingredientDeltas").and_then(|v| v.as_array()) {
                for delta in deltas {
                    let uri = delta
                        .get("ingredientAssertionURI")
                        .and_then(|v| v.as_str())
                        .map(|s| format!("Ingredient: {}", s));
                    let vd = delta.get("validationDeltas").and_then(|v| v.as_object());
                    if let Some(vd) = vd {
                        push_entries(&mut out, vd.get("failure"), uri);
                    }
                }
            }
        }
    }

    out
}

/// Collect validation failure entries from a single manifest record (its validationResults and
/// ingredientDeltas). Used for ingredient tree nodes so each ingredient shows the matching
/// manifest's validation. Excludes signingCredential.untrusted.
pub(crate) fn get_validation_failures_for_manifest(
    manifest_obj: &serde_json::Value,
) -> Vec<ValidationFailureEntry> {
    const UNTRUSTED_CODE: &str = "signingCredential.untrusted";
    let mut out = Vec::new();
    let push_entries = |out: &mut Vec<ValidationFailureEntry>,
                        arr: Option<&serde_json::Value>,
                        source: Option<String>| {
        let arr = match arr.and_then(|v| v.as_array()) {
            Some(a) => a,
            None => return,
        };
        for entry in arr {
            let obj = match entry.as_object() {
                Some(o) => o,
                None => continue,
            };
            let code = match obj.get("code").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => continue,
            };
            if code == UNTRUSTED_CODE {
                continue;
            }
            out.push(ValidationFailureEntry {
                code: code.to_string(),
                explanation: obj
                    .get("explanation")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                url: obj.get("url").and_then(|v| v.as_str()).map(String::from),
                source: source.clone(),
            });
        }
    };
    if let Some(vr) = manifest_obj
        .get("validationResults")
        .and_then(|v| v.as_object())
    {
        push_entries(&mut out, vr.get("failure"), None);
    }
    if let Some(deltas) = manifest_obj
        .get("ingredientDeltas")
        .and_then(|v| v.as_array())
    {
        for delta in deltas {
            let uri = delta
                .get("ingredientAssertionURI")
                .and_then(|v| v.as_str())
                .map(|s| format!("Ingredient: {}", s));
            let vd = delta.get("validationDeltas").and_then(|v| v.as_object());
            if let Some(vd) = vd {
                push_entries(&mut out, vd.get("failure"), uri);
            }
        }
    }
    out
}

/// Recursively display manifest → ingredients tree in the given UI.
pub(crate) fn display_manifest_ingredient_tree(
    ui: &mut egui::Ui,
    manifest_value: &serde_json::Value,
    active_label: &str,
) {
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
        let (claim_type, claim_gen, claim_gen_info) = manifest_claim_info(active_manifest);
        if let Some(ct) = claim_type {
            ui.label(
                egui::RichText::new(format!("Claim type: {}", ct))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(64, 64, 64)),
            );
        }
        if let Some(ref gen) = claim_gen {
            ui.label(
                egui::RichText::new(format!("Claim generator: {}", gen))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(64, 64, 64)),
            );
        }
        if let Some(ref info) = claim_gen_info {
            ui.label(
                egui::RichText::new(format!("Claim generator info: {}", info))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(64, 64, 64)),
            );
        }
        if let Some(label) = active_manifest.get("label").and_then(|v| v.as_str()) {
            ui.label(
                egui::RichText::new(format!("Label: {}", label))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(64, 64, 64)),
            );
        }
        let ingredients = collect_ingredients_from_manifest(active_manifest);
        if let Some(dst) = manifest_digital_source_type(active_manifest) {
            ui.label(
                egui::RichText::new(format!("Digital source type: {}", dst))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(64, 64, 64)),
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
                            .color(egui::Color32::from_rgb(64, 64, 64)),
                        );
                        break;
                    }
                }
            }
        }
        if let Some(trust) = trust_status_from_manifest(active_manifest) {
            let (icon, color) = match trust.as_str() {
                "signingCredential.trusted" => ("🔒", egui::Color32::from_rgb(0, 100, 0)),
                "signingCredential.untrusted" => ("🔓", egui::Color32::from_rgb(255, 100, 100)),
                _ => ("", egui::Color32::from_rgb(64, 64, 64)),
            };
            let text = if icon.is_empty() {
                format!("Trust: {}", trust)
            } else {
                format!("Trust: {} {}", icon, trust)
            };
            ui.label(egui::RichText::new(text).size(12.0).color(color));
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

// --- Private helpers ---

fn collect_ingredients_from_manifest(manifest_obj: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut out = Vec::new();
    let obj = match manifest_obj.as_object() {
        Some(o) => o,
        None => return out,
    };
    for key in ["claim.v2", "claim"] {
        if let Some(claim) = obj.get(key) {
            if let Some(arr) = claim.get("ingredients").and_then(|v| v.as_array()) {
                for ing in arr {
                    out.push(ing);
                }
            }
        }
    }
    if let Some(arr) = obj.get("ingredients").and_then(|v| v.as_array()) {
        for ing in arr {
            out.push(ing);
        }
    }
    if let Some(assertions) = obj.get("assertions").and_then(|v| v.as_object()) {
        for (key, val) in assertions {
            if !key.contains("ingredient") {
                continue;
            }
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
    out
}

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

fn manifest_digital_source_type(manifest_obj: &serde_json::Value) -> Option<String> {
    let try_actions_array = |actions: &serde_json::Value| -> Option<String> {
        let arr = actions.as_array()?;
        for act in arr {
            if act.get("action").and_then(|v| v.as_str()) != Some("c2pa.created") {
                continue;
            }
            let url = act.get("digitalSourceType").and_then(|v| v.as_str())?;
            return Some(url.split('/').rfind(|s| !s.is_empty())?.to_string());
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

/// Trust status for a manifest (used for both root and ingredient tree nodes).
/// Uses the manifest's validationResults (success/failure); falls back to status.trust for legacy.
fn trust_status_from_manifest(manifest_obj: &serde_json::Value) -> Option<String> {
    manifest_obj
        .get("validationResults")
        .and_then(trust_from_validation_results)
        .or_else(|| {
            manifest_obj
                .get("status")
                .and_then(|s| s.get("trust"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        })
}

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
        _ => egui::Color32::from_rgb(64, 64, 64),
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

fn ingredient_node_details(
    ui: &mut egui::Ui,
    manifest_value: &serde_json::Value,
    ingredient: &serde_json::Value,
) {
    let gray = egui::Color32::from_rgb(64, 64, 64);
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
    if let Some(nested) = nested_manifest_for_ingredient(manifest_value, ingredient) {
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
        if let Some(dst) = manifest_digital_source_type(nested) {
            ui.label(
                egui::RichText::new(format!("Digital source type: {}", dst))
                    .size(small)
                    .color(gray),
            );
        }
        if let Some(trust) = trust_status_from_manifest(nested) {
            let (icon, color) = match trust.as_str() {
                "signingCredential.trusted" => ("🔒", egui::Color32::from_rgb(0, 100, 0)),
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
        let failures = get_validation_failures_for_manifest(nested);
        if !failures.is_empty() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Validation failures (this manifest):")
                    .size(small)
                    .color(egui::Color32::from_rgb(255, 150, 150)),
            );
            for entry in &failures {
                let line = if let Some(ref src) = entry.source {
                    format!("  {} — {}", src, entry.code)
                } else {
                    format!("  {}", entry.code)
                };
                ui.label(
                    egui::RichText::new(line)
                        .size(small)
                        .color(egui::Color32::from_rgb(255, 120, 120)),
                );
                if let Some(ref ex) = entry.explanation {
                    ui.label(
                        egui::RichText::new(format!("    {}", ex))
                            .size(small - 1.0)
                            .color(gray),
                    );
                }
            }
        }
    }
}

fn format_rfc3339_date(s: &str) -> Option<String> {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let date_part = s.split('T').next()?;
    let mut parts = date_part.split('-');
    let year: u32 = parts.next()?.parse().ok()?;
    let month: usize = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || day == 0 || day > 31 {
        return None;
    }
    Some(format!("{} {}, {}", MONTHS[month - 1], day, year))
}
