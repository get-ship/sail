//! JSON Schema emit and key docs for [`crate::config::SailConfig`].
//!
//! Lives here so the types+load file stays the document and its readers.

use std::collections::HashSet;
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::{Value, json};

use crate::config::SailConfig;

/// Layer of a sail.config key (SPEC-HARDENING-1 / SPEC-SAIL-CONFIG-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConfigLayer {
    Policy,
    Product,
}

impl ConfigLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Policy => "POLICY",
            Self::Product => "PRODUCT",
        }
    }
}

fn layer_for(key: &str) -> ConfigLayer {
    match key {
        "app" | "icons" => ConfigLayer::Product,
        _ => ConfigLayer::Policy,
    }
}

/// Documentation row for the reference generator (`CONFIG_KEYS`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigKeyDoc {
    pub key: String,
    pub type_name: String,
    pub default: String,
    pub doc: String,
    pub layer: String,
}

fn schema_value() -> Value {
    serde_json::to_value(schemars::schema_for!(SailConfig)).expect("SailConfig schema")
}

/// Sort object keys so emit is stable whether or not a workspace crate
/// enabled `serde_json/preserve_order` (features unify; `-p sail-lint`
/// alone does not).
fn sort_keys(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = std::mem::take(map).into_iter().collect();
            for (_, v) in &mut entries {
                sort_keys(v);
            }
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            *map = entries.into_iter().collect();
        }
        Value::Array(items) => {
            for item in items {
                sort_keys(item);
            }
        }
        _ => {}
    }
}

pub(crate) fn recognized_key_set() -> &'static HashSet<String> {
    static KEYS: OnceLock<HashSet<String>> = OnceLock::new();
    KEYS.get_or_init(|| recognized_keys().into_iter().collect())
}

/// Top-level keys of [`SailConfig`], derived from the generated schema.
pub fn recognized_keys() -> Vec<String> {
    static KEYS: OnceLock<Vec<String>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let mut keys: Vec<String> = schema_value()
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        keys.sort();
        keys
    })
    .clone()
}

/// JSON Schema view of [`SailConfig`]. Refresh with `SAIL_WRITE_SCHEMA=1`.
pub fn emit_sail_config_schema() -> String {
    let mut schema = schema_value();
    if let Some(obj) = schema.as_object_mut() {
        obj.insert(
            "$schema".into(),
            json!("https://json-schema.org/draft/2020-12/schema"),
        );
        obj.insert(
            "$id".into(),
            json!("https://ship.dev/schemas/sail.config.schema.json"),
        );
        obj.insert("title".into(), json!("SailConfig"));
        obj.insert("type".into(), json!("object"));
        obj.insert("additionalProperties".into(), json!(false));
        if let Some(props) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
            for (key, prop) in props.iter_mut() {
                if let Some(p) = prop.as_object_mut() {
                    p.insert("x-sail-layer".into(), json!(layer_for(key).as_str()));
                }
            }
        }
    }
    sort_keys(&mut schema);
    let mut out = serde_json::to_string_pretty(&schema).expect("schema json");
    out.push('\n');
    out
}

/// Docs for every top-level key, derived from [`SailConfig`] field rustdoc.
pub fn config_key_docs() -> Vec<ConfigKeyDoc> {
    let schema = schema_value();
    let Some(props) = schema.get("properties").and_then(|p| p.as_object()) else {
        return Vec::new();
    };
    let mut keys: Vec<&String> = props.keys().collect();
    keys.sort();
    keys.into_iter()
        .map(|key| {
            let prop = &props[key];
            let description = prop
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .trim();
            let (doc, default) = split_doc_default(description);
            ConfigKeyDoc {
                key: key.clone(),
                type_name: type_name_of(prop),
                default,
                doc,
                layer: layer_for(key).as_str().to_string(),
            }
        })
        .collect()
}

fn split_doc_default(description: &str) -> (String, String) {
    // rustdoc: summary paragraph, then `Default: …`.
    let mut summary = String::new();
    let mut default = String::from("absent");
    for para in description.split("\n\n") {
        let p = para.trim();
        if let Some(rest) = p.strip_prefix("Default:") {
            default = rest.trim().trim_end_matches('.').to_string();
        } else if summary.is_empty() {
            summary = p.to_string();
        }
    }
    if summary.is_empty() {
        summary = description.to_string();
    }
    (summary, default)
}

fn type_name_of(prop: &Value) -> String {
    if let Some(r) = prop.get("$ref").and_then(|v| v.as_str()) {
        return r.rsplit('/').next().unwrap_or(r).to_string();
    }
    if let Some(any) = prop
        .get("anyOf")
        .or_else(|| prop.get("oneOf"))
        .and_then(|v| v.as_array())
    {
        let parts: Vec<String> = any.iter().map(type_name_of).collect();
        if !parts.is_empty() {
            return parts.join(" | ");
        }
    }
    match prop.get("type").and_then(|t| t.as_str()) {
        Some("boolean") => "boolean".into(),
        Some("string") => {
            if let Some(e) = prop.get("enum").and_then(|v| v.as_array()) {
                let vals: Vec<String> = e
                    .iter()
                    .filter_map(|x| x.as_str().map(|s| format!("\"{s}\"")))
                    .collect();
                if !vals.is_empty() {
                    return vals.join(" | ");
                }
            }
            "string".into()
        }
        Some("array") => {
            let item = prop
                .get("items")
                .map(type_name_of)
                .unwrap_or_else(|| "any".into());
            format!("{item}[]")
        }
        Some("object") => "object".into(),
        Some("integer") | Some("number") => "number".into(),
        _ => "any".into(),
    }
}

#[cfg(test)]
mod schema_tests {
    use super::*;

    #[test]
    fn committed_schema_matches_emit() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/schemas/sail.config.schema.json"
        );
        if std::env::var("SAIL_WRITE_SCHEMA").is_ok() {
            std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"), "/schemas")).unwrap();
            std::fs::write(path, emit_sail_config_schema()).unwrap();
        }
        let committed = std::fs::read_to_string(path).expect("committed schema");
        assert_eq!(
            committed,
            emit_sail_config_schema(),
            "sail.config.schema.json drifted — SAIL_WRITE_SCHEMA=1 cargo test -p sail-lint --lib committed_schema"
        );
    }

    #[test]
    fn schema_properties_are_the_recognized_keys() {
        let keys = recognized_keys();
        assert!(keys.contains(&"preset".into()));
        assert!(keys.contains(&"strict".into()));
        assert!(keys.contains(&"lint".into()));
        assert!(keys.contains(&"deck".into()));
        assert!(keys.contains(&"app".into()));
        assert!(keys.contains(&"icons".into()));
        assert!(keys.contains(&"theme".into()));
        assert!(keys.contains(&"raw_containers".into()));
        assert!(keys.contains(&"style_slots".into()));
        assert!(!keys.iter().any(|k| k == "policy"));
        assert!(!keys.iter().any(|k| k == "fonts"));
        assert!(!keys.iter().any(|k| k == "editor"));
        assert!(!keys.iter().any(|k| k == "sx"));
        assert!(!keys.iter().any(|k| k == "variants"));
        assert!(!keys.iter().any(|k| k == "compiler"));
        for row in config_key_docs() {
            assert!(
                row.layer == "POLICY" || row.layer == "PRODUCT",
                "key `{}` layer must be POLICY or PRODUCT, got `{}`",
                row.key,
                row.layer
            );
        }
    }

    #[test]
    fn icons_key_is_recognized_product() {
        let cfg = SailConfig::from_json(r#"{ "icons": { "pack": "lucide" } }"#);
        assert!(cfg.parse_ok());
        assert!(cfg.type_error().is_none(), "{:?}", cfg.type_error());
        assert!(cfg.unknown_keys().is_empty(), "{:?}", cfg.unknown_keys());
        assert!(cfg.has_key("icons"));
        assert_eq!(
            cfg.icons_value()
                .and_then(|v| v.get("pack").cloned())
                .and_then(|v| v.as_str().map(str::to_string)),
            Some("lucide".into())
        );
        let layer = config_key_docs()
            .into_iter()
            .find(|r| r.key == "icons")
            .map(|r| r.layer);
        assert_eq!(layer.as_deref(), Some("PRODUCT"));
    }

    #[test]
    fn fonts_and_editor_and_policy_are_unknown() {
        for key in ["fonts", "editor", "policy"] {
            let body = format!(r#"{{ "{key}": true }}"#);
            let cfg = SailConfig::from_json(&body);
            assert!(
                cfg.unknown_keys().iter().any(|k| k == key),
                "{key} must be unknown; got {:?}",
                cfg.unknown_keys()
            );
        }
    }
}
