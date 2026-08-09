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

/// Config filenames the walk-up accepts. `jsonc` wins when both exist in the
/// same directory (app-level studio/sail-site convention). Capability Policy
/// still keys off the first config found; layout severity may walk further
/// (see [`layout_is_error_for`]).
pub const CONFIG_FILES: &[&str] = &["sail.config.jsonc", "sail.config.json"];

/// The top-level `sail.config.json` keys the parser recognizes. Anything else
/// present is a typo/unknown key — the compiler must refuse loudly rather
/// than silently ignore it (config-loud-strict: a `{"polcy":"strict"}` typo
/// used to fail closed with zero signal).
///
/// Layer discipline (SPEC-HARDENING-1 WI5 / SPEC-SAIL-CONFIG-1): every key is
/// POLICY (capability / serve / safety) or TASTE (product preference). No
/// behavior knobs that change language semantics under the author's feet.
/// See [`CONFIG_KEY_DOCS`] column "layer" via the 5th field on each row.
///
/// Bundler/serve settings live under the top-level **`deck`** object only
/// (`deck.render`, `deck.entry`, `deck.injectGlobals`). Top-level `render` /
/// `entry` / `injectGlobals` are unknown keys and refuse loudly — no dual-read.
pub const RECOGNIZED_KEYS: [&str; 14] = [
    "preset",
    "sx",
    "variants",
    "raw_containers",
    "compose_rhythm",
    "autofix",
    "stylesheets",
    "network",
    "unconfined",
    "theme",
    "layout",
    "policy",
    // Deck bundler/serve object (AppServeConfig) — POLICY layer.
    "deck",
    // Studio product preference (deep-link scheme) — TASTE layer.
    "editor",
];

/// Documentation rows for [`RECOGNIZED_KEYS`] — key, value type, default
/// when absent, one line of semantics, and layer (`POLICY` | `TASTE`).
/// The reference generator / RESULT audit renders this table; the test
/// below keeps it in lockstep with the recognized set.
pub const CONFIG_KEY_DOCS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "preset",
        "\"strict\" | \"full\" | \"repair\" | \"off\"",
        "no tier — capabilities opt in individually",
        "select a capability tier above the floor in one word",
        "POLICY",
    ),
    (
        "sx",
        "boolean",
        "off",
        "capability override: allow the sx surface beyond the floor allowance",
        "POLICY",
    ),
    (
        "variants",
        "boolean",
        "off",
        "capability override: allow CVA-in-props variance matrices (key→sx maps on props) and compound:",
        "POLICY",
    ),
    (
        "raw_containers",
        "boolean",
        "off",
        "capability override: allow raw container elements instead of the layout keywords",
        "POLICY",
    ),
    (
        "compose_rhythm",
        "boolean",
        "true",
        "auto-fix sibling rhythm violations; false warns only",
        "POLICY",
    ),
    (
        "autofix",
        "\"repair\" | \"strict\" | \"off\"",
        "repair",
        "compose autofix preset: confluent fixes on / all error / pass silent",
        "POLICY",
    ),
    (
        "stylesheets",
        "string[]",
        "none",
        "the CSS files this package may ship; an undeclared stylesheet fails the floor check",
        "POLICY",
    ),
    (
        "network",
        "string[] of origins",
        "absent = full refusal",
        "fetch/WebSocket/EventSource allowlist; [] grants nothing explicitly; a wrong-typed value fails closed",
        "POLICY",
    ),
    (
        "unconfined",
        "boolean",
        "confined",
        "opt out of controller ambient-authority confinement entirely; a wrong-typed value stays confined",
        "POLICY",
    ),
    (
        "theme",
        "{ default, defaultMode?, enabled: \"all\" | string[] }",
        "no theme section — thin app emit (no catalog); package generate still full catalog",
        "app theme emit set: default id, optional mode, catalog allowlist (omit/[] thin; \"all\" full; list filter). Project themes/ always emit",
        "POLICY",
    ),
    (
        "layout",
        "\"warn\" | \"error\"",
        "warn — taffy soft findings stay warnings; silent-clip stays Error",
        "promote gate-gap layout findings (main-axis overflow, collapsed box, height-chain) to Error so check/dev exit non-zero (studio strict profile)",
        "POLICY",
    ),
    (
        "policy",
        "{ layout.raw_containers?, lint.severity.<rule-id>? }",
        "registry defaults (see `sail-compiler policy`)",
        "typed compiler policy registry object; unknown keys are a hard error naming the closest valid key",
        "POLICY",
    ),
    (
        "deck",
        "{ render?: \"csr\"|\"ssr\", entry?: { clientsModule, exposeNames[] }, injectGlobals?: { [k]: template } }",
        "absent — fully conventional serve defaults (csr, no entry, no inject)",
        "Deck bundler/serve settings (SPEC-SAIL-CONFIG-1); nested only — top-level render/entry/injectGlobals refuse as unknown",
        "POLICY",
    ),
    (
        "editor",
        "string | { scheme, pathTemplate? }",
        "vscode (studio default when unset)",
        "studio deep-link preference for open-in-editor — product taste, not language behavior",
        "TASTE",
    ),
];

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
            Ok(raw) => SailConfig {
                raw,
                parse_ok: true,
            },
            Err(_) => SailConfig {
                raw: serde_json::Value::Null,
                parse_ok: false,
            },
        }
    }

    /// False iff the config body failed to parse as JSON at all (a malformed
    /// file, not merely an unrecognized key).
    pub fn parse_ok(&self) -> bool {
        self.parse_ok
    }

    /// The raw parsed JSON body — the single value every typed accessor reads.
    /// Callers that need a nested object (e.g. the compiler policy registry's
    /// `"policy"` target) use this rather than re-parsing the file.
    pub fn raw(&self) -> &serde_json::Value {
        &self.raw
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

    /// A string key (`"autofix"`, …), if present and a string.
    pub fn string_key(&self, key: &str) -> Option<String> {
        self.raw
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
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

    /// The declared `"unconfined"` capability — the whole-hog opt-out of the
    /// confined-by-default `sail-controller-no-ambient-authority` gate. `None`
    /// when the key is absent (today's default: confined). `Some(Err(()))`
    /// when present but not a bool — a type error the caller must fail closed
    /// on (stays confined), never silently grant. `Some(Ok(bool))` when
    /// present and well-typed.
    pub fn unconfined_key(&self) -> Option<Result<bool, ()>> {
        let v = self.raw.get("unconfined")?;
        match v.as_bool() {
            Some(b) => Some(Ok(b)),
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

    /// `"layout": "error"` → strict layout gate. Absent / `"warn"` / other → soft.
    /// Wrong-typed values are treated as soft (lenient extract).
    pub fn layout_is_error(&self) -> bool {
        matches!(
            self.raw.get("layout").and_then(|v| v.as_str()),
            Some(s) if s.eq_ignore_ascii_case("error")
        )
    }

    /// Parse JSON or JSONC body (// and /* */ comments + trailing commas).
    pub fn from_jsonc(text: &str) -> SailConfig {
        SailConfig::from_json(&strip_jsonc_comments(text))
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
/// `sail.config.json` / `sail.config.jsonc` found plus its parsed contents, or
/// `None` if none exists on the walk. A present-but-unparseable config
/// resolves to an empty `SailConfig` (`parse_ok() == false`) and stops the
/// walk — its presence is the package's declared intent, even if its body is
/// broken. The returned path is what a caller names in a loud
/// refusal/announcement message.
///
/// **Note:** Capability Policy still prefers pure `.json` package configs
/// (primitives packs, routes). App-level `.jsonc` may carry `theme`, `deck`,
/// and `editor` which are recognized ([`RECOGNIZED_KEYS`]) as POLICY/TASTE.
/// Serve keys nest under `deck` only. Layout severity still uses
/// [`layout_is_error_for`], which may walk past a capability config that has
/// no `"layout"` key up to an app-level profile.
pub fn resolve(dir: &Path) -> Option<(PathBuf, SailConfig)> {
    let mut cur = Some(dir);
    while let Some(d) = cur {
        // Capability Policy historically reads `.json` only. Prefer it so
        // routes/package packs keep winning over a co-located theme-only jsonc.
        let json = d.join(CONFIG_FILE);
        if let Ok(text) = std::fs::read_to_string(&json) {
            return Some((json, SailConfig::from_json(&text)));
        }
        cur = d.parent();
    }
    None
}

/// True when the nearest config that declares `"layout"` sets it to `"error"`.
///
/// Walks up from `source_file`'s parent, dual-reading `sail.config.jsonc` then
/// `sail.config.json` at each level. A config that omits `"layout"` does not
/// stop the walk — so `apps/studio/routes/sail.config.json` (capability keys
/// only) still inherits `apps/studio/sail.config.jsonc` `"layout": "error"`.
///
/// Lives in the single config reader module (not a second walk-up of policy
/// keys). Pure layout severity — never mutates capability Policy.
pub fn layout_is_error_for(source_file: &Path) -> bool {
    let mut cur = source_file.parent();
    while let Some(d) = cur {
        for name in CONFIG_FILES {
            let candidate = d.join(name);
            let Ok(text) = std::fs::read_to_string(&candidate) else {
                continue;
            };
            let cfg = if name.ends_with('c') {
                SailConfig::from_jsonc(&text)
            } else {
                SailConfig::from_json(&text)
            };
            if !cfg.parse_ok() {
                // Broken body at this path — keep walking; don't fail closed
                // into strict layout for a comment-only parse glitch.
                continue;
            }
            if cfg.has_key("layout") {
                return cfg.layout_is_error();
            }
        }
        cur = d.parent();
    }
    false
}

/// Std-only JSONC stripper (// + /* */ comments, trailing commas). Mirrors
/// deck/compiler helpers so lint stays free of those deps.
fn strip_jsonc_comments(input: &str) -> String {
    let mut stage = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if in_string {
            stage.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            stage.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc == '\n' {
                            stage.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }
        stage.push(c);
    }
    let src = stage.as_bytes();
    let mut cleaned: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    let mut in_string2 = false;
    let mut escape2 = false;
    while i < src.len() {
        let b = src[i];
        if in_string2 {
            cleaned.push(b);
            if escape2 {
                escape2 = false;
            } else if b == b'\\' {
                escape2 = true;
            } else if b == b'"' {
                in_string2 = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string2 = true;
            cleaned.push(b);
            i += 1;
            continue;
        }
        if b == b',' {
            let mut j = i + 1;
            while j < src.len()
                && (src[j] == b' ' || src[j] == b'\t' || src[j] == b'\r' || src[j] == b'\n')
            {
                j += 1;
            }
            if j < src.len() && (src[j] == b']' || src[j] == b'}') {
                i += 1;
                continue;
            }
        }
        cleaned.push(b);
        i += 1;
    }
    String::from_utf8(cleaned)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

#[cfg(test)]
mod config_doc_tests {
    use super::*;

    #[test]
    fn every_recognized_key_has_a_doc_row_and_no_extras() {
        let doc_keys: Vec<&str> = CONFIG_KEY_DOCS.iter().map(|(k, _, _, _, _)| *k).collect();
        assert_eq!(
            doc_keys, RECOGNIZED_KEYS,
            "CONFIG_KEY_DOCS must document exactly RECOGNIZED_KEYS, in the same order"
        );
    }

    #[test]
    fn every_key_is_policy_or_taste() {
        for (key, _, _, _, layer) in CONFIG_KEY_DOCS {
            assert!(
                *layer == "POLICY" || *layer == "TASTE",
                "key `{key}` layer must be POLICY or TASTE, got `{layer}`"
            );
        }
    }

    #[test]
    fn layout_key_error_parses() {
        let cfg = SailConfig::from_json(r#"{ "layout": "error" }"#);
        assert!(cfg.layout_is_error());
        let soft = SailConfig::from_json(r#"{ "layout": "warn" }"#);
        assert!(!soft.layout_is_error());
        let absent = SailConfig::from_json(r#"{ "sx": true }"#);
        assert!(!absent.layout_is_error());
    }

    /// SPEC-SAIL-CONFIG-1 C1: top-level serve keys are unknown after cutover.
    #[test]
    fn top_level_serve_keys_are_unknown() {
        for key in ["render", "entry", "injectGlobals"] {
            let body = format!(r#"{{ "{key}": "csr" }}"#);
            let cfg = SailConfig::from_json(&body);
            let unknown = cfg.unknown_keys();
            assert!(
                unknown.iter().any(|k| k == key),
                "top-level `{key}` must be unknown; got {unknown:?}"
            );
        }
    }

    /// Nested `deck` is recognized; leftover top-level `render` still refuses.
    #[test]
    fn deck_object_recognized_top_level_render_still_unknown() {
        let ok = SailConfig::from_json(r#"{ "deck": { "render": "ssr" }, "theme": { "default": "ship" } }"#);
        assert!(ok.unknown_keys().is_empty(), "nested deck is recognized");
        assert!(ok.has_key("deck"));

        let legacy = SailConfig::from_json(
            r#"{ "render": "ssr", "deck": { "render": "csr" } }"#,
        );
        assert_eq!(legacy.unknown_keys(), vec!["render".to_string()]);
    }

    #[test]
    fn layout_jsonc_strips_comments() {
        let cfg = SailConfig::from_jsonc(
            r#"{
            // studio strict
            "layout": "error",
          }"#,
        );
        assert!(cfg.parse_ok());
        assert!(cfg.layout_is_error());
    }

    #[test]
    fn layout_is_error_walks_past_capability_json_to_app_jsonc() {
        // routes/sail.config.json (no layout) + app sail.config.jsonc (layout:error)
        // must still resolve strict — studio shape.
        let dir = tempfile::tempdir().expect("tempdir");
        let routes = dir.path().join("routes");
        std::fs::create_dir_all(&routes).expect("mkdir");
        std::fs::write(
            routes.join("sail.config.json"),
            r#"{ "sx": true, "raw_containers": false }"#,
        )
        .expect("write routes config");
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{ "layout": "error", "theme": { "default": "studio" } }"#,
        )
        .expect("write app jsonc");
        let page = routes.join("page.sail");
        std::fs::write(&page, "<column></column>").expect("write page");
        assert!(
            layout_is_error_for(&page),
            "must inherit layout:error from parent jsonc"
        );
    }
}
