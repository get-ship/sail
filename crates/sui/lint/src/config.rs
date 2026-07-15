//! The single reader of `sui.config.json`.
//!
//! One walk-up, one parse. Every consumer — the compiler's `Policy` tier and
//! the floor's undeclared-stylesheet check — derives from this resolved value;
//! none re-walks the tree or re-parses the file. That is `inv-sui-one-policy-resolver`:
//! the effective config for a source is computed in exactly one place, enforced
//! by a uniqueness test (`sui-compiler/tests/config_single_reader.rs`).
//!
//! Extraction is lenient (field-by-field off a parsed `Value`), matching the
//! historical behavior: a wrong-typed or missing key is ignored, never a hard
//! failure — a config typo must not break a build.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "sui.config.json";

/// Walk up from `from` (inclusive) and return the first directory that contains
/// a `sui.config.json` — the package root that owns a source. The one place that
/// knows the config's filename; callers that need the owning dir (not its
/// contents) use this instead of re-hardcoding the name.
pub fn owning_dir(from: &Path) -> Option<PathBuf> {
    let mut cur = Some(from);
    while let Some(d) = cur {
        if d.join(CONFIG_FILE).is_file() {
            return Some(d.to_path_buf());
        }
        cur = d.parent();
    }
    None
}

/// The parsed contents of the nearest `sui.config.json` on the walk up. Holds
/// the raw JSON; typed accessors read individual fields leniently.
#[derive(Debug, Clone)]
pub struct SuiConfig {
    raw: serde_json::Value,
}

impl SuiConfig {
    /// Parse config JSON text. Unparseable text yields an empty config — its
    /// presence on disk is the package's declared intent, and a broken body must
    /// not hard-fail a build (the accessors read fields leniently).
    pub fn from_json(text: &str) -> SuiConfig {
        SuiConfig {
            raw: serde_json::from_str(text).unwrap_or(serde_json::Value::Null),
        }
    }

    /// The `"preset"` string, if present and a string.
    pub fn preset(&self) -> Option<&str> {
        self.raw.get("preset").and_then(|v| v.as_str())
    }

    /// Whether the config names `key` at all (any value/type).
    pub fn has_key(&self, key: &str) -> bool {
        self.raw.get(key).is_some()
    }

    /// A boolean capability override (`"sx"`, `"variants"`, `"raw_containers"`),
    /// if present and a bool.
    pub fn bool_key(&self, key: &str) -> Option<bool> {
        self.raw.get(key).and_then(|v| v.as_bool())
    }

    /// The declared `"stylesheets"` set — the CSS files the pack is allowed to
    /// ship. Absent/wrong-typed → empty.
    pub fn declared_stylesheets(&self) -> HashSet<String> {
        self.raw
            .get("stylesheets")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Walk up from `dir` (inclusive) and return the first `sui.config.json`'s
/// parsed contents, or `None` if none exists on the walk. A present-but-
/// unparseable config resolves to an empty `SuiConfig` and stops the walk — its
/// presence is the package's declared intent, even if its body is broken.
pub fn resolve(dir: &Path) -> Option<SuiConfig> {
    let mut cur = Some(dir);
    while let Some(d) = cur {
        match std::fs::read_to_string(d.join(CONFIG_FILE)) {
            Ok(text) => return Some(SuiConfig::from_json(&text)),
            Err(_) => cur = d.parent(),
        }
    }
    None
}
