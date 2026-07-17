//! The single reader of `sail.config.json`.
//!
//! One walk-up, one parse. Every consumer — the compiler's `Policy` tier and
//! the floor's undeclared-stylesheet check — derives from this resolved value;
//! none re-walks the tree or re-parses the file. That is `inv-sail-one-policy-resolver`:
//! the effective config for a source is computed in exactly one place, enforced
//! by a uniqueness test (`sail-compiler/tests/config_single_reader.rs`).
//!
//! Extraction is lenient (field-by-field off a parsed `Value`), matching the
//! historical behavior: a wrong-typed or missing key is ignored, never a hard
//! failure — a config typo must not break a build.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "sail.config.json";

/// Walk up from `from` (inclusive) and return the first directory that contains
/// a `sail.config.json` — the package root that owns a source. The one place that
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

/// The top-level `sail.config.json` keys the parser recognizes. Anything else
/// present is a typo/unknown key — the compiler must refuse loudly rather
/// than silently ignore it (config-loud-strict: a `{"polcy":"strict"}` typo
/// used to fail closed with zero signal).
pub const RECOGNIZED_KEYS: [&str; 6] =
    ["preset", "sx", "variants", "raw_containers", "stylesheets", "network"];

/// The parsed contents of the nearest `sail.config.json` on the walk up. Holds
/// the raw JSON; typed accessors read individual fields leniently.
#[derive(Debug, Clone)]
pub struct SailConfig {
    raw: serde_json::Value,
    parse_ok: bool,
}

impl SailConfig {
    /// Parse config JSON text. Unparseable text yields an empty config — its
    /// presence on disk is the package's declared intent, and a broken body must
    /// not hard-fail a build (the accessors read fields leniently). `parse_ok`
    /// tracks whether the body actually parsed, so a caller that wants to
    /// REFUSE LOUDLY on an unparseable config (vs. one that parsed but carries
    /// no recognized signal) can tell the two apart.
    pub fn from_json(text: &str) -> SailConfig {
        match serde_json::from_str(text) {
            Ok(raw) => SailConfig { raw, parse_ok: true },
            Err(_) => SailConfig { raw: serde_json::Value::Null, parse_ok: false },
        }
    }

    /// False iff the config body failed to parse as JSON at all (a malformed
    /// file, not merely an unrecognized key).
    pub fn parse_ok(&self) -> bool {
        self.parse_ok
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

    /// Top-level keys present in the config that are NOT in [`RECOGNIZED_KEYS`]
    /// — typos (`"polcy"`) or made-up keys. A parse failure yields no keys
    /// (there's no object to inspect); callers check [`Self::parse_ok`] first.
    pub fn unknown_keys(&self) -> Vec<String> {
        self.raw
            .as_object()
            .map(|obj| {
                obj.keys()
                    .filter(|k| !RECOGNIZED_KEYS.contains(&k.as_str()))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The declared `"network"` per-origin allowlist — the fetch/WebSocket/
    /// EventSource capability grant. `None` when the key is absent entirely
    /// (today's default: full refusal). `Some(Err(()))` when the key is
    /// present but wrong-typed (not an array, or an array with a non-string
    /// element) — a type error the caller must fail closed on, never silently
    /// treat as granting. `Some(Ok(origins))` when present and well-typed;
    /// an empty array is a valid, explicit "grant nothing" declaration, not a
    /// type error.
    pub fn network_key(&self) -> Option<Result<Vec<String>, ()>> {
        let v = self.raw.get("network")?;
        match v.as_array() {
            Some(arr) => {
                let mut out = Vec::with_capacity(arr.len());
                for item in arr {
                    match item.as_str() {
                        Some(s) => out.push(s.to_string()),
                        None => return Some(Err(())),
                    }
                }
                Some(Ok(out))
            }
            None => Some(Err(())),
        }
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

/// A trailing clause naming the `sail.config.json` that could grant capability
/// `key` for a source at `path` — appended to a strict-tier rule's `suggest`
/// text so the refusal names WHERE to grant it, not just that it's refused
/// (config-loud-strict rule #3). Empty string when no config exists on the
/// walk (a synthetic/unrooted source, or a real gap — either way there is no
/// path to name). Does not read the file — `owning_dir` only checks
/// existence, so this is not a second `sail.config.json` reader.
pub fn grant_hint(path: &Path, key: &str) -> String {
    match path.parent().and_then(owning_dir) {
        Some(dir) => format!(
            " Grant it in `{}` — `{{ \"{key}\": true }}`.",
            dir.join(CONFIG_FILE).display()
        ),
        None => String::new(),
    }
}

/// Walk up from `dir` (inclusive) and return the path to the first
/// `sail.config.json` found plus its parsed contents, or `None` if none exists
/// on the walk. A present-but-unparseable config resolves to an empty
/// `SailConfig` (`parse_ok() == false`) and stops the walk — its presence is
/// the package's declared intent, even if its body is broken. The returned
/// path is what a caller names in a loud refusal/announcement message.
pub fn resolve(dir: &Path) -> Option<(PathBuf, SailConfig)> {
    let mut cur = Some(dir);
    while let Some(d) = cur {
        let candidate = d.join(CONFIG_FILE);
        match std::fs::read_to_string(&candidate) {
            Ok(text) => return Some((candidate, SailConfig::from_json(&text))),
            Err(_) => cur = d.parent(),
        }
    }
    None
}
