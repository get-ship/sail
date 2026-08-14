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

/// True if `dir` itself contains a sail config file. Existence only —
/// does not read or parse. Project-root walkers use this so they do
/// not construct the filename (inv-sail-one-policy-resolver).
pub fn dir_has_config(dir: &Path) -> bool {
    CONFIG_FILES.iter().any(|name| dir.join(name).is_file())
}

/// The top-level `sail.config.json` keys the parser recognizes. Anything else
/// present is a typo/unknown key — the compiler must refuse loudly rather
/// than silently ignore it (config-loud-strict: a `{"polcy":"strict"}` typo
/// used to fail closed with zero signal).
///
/// Layer discipline (SPEC-HARDENING-1 WI5 / SPEC-SAIL-CONFIG-1): every key is
/// POLICY (capability / serve / safety), TASTE (product preference), or
/// PRODUCT (design-system resources — icons, fonts). No behavior knobs that
/// change language semantics under the author's feet.
/// See [`CONFIG_KEY_DOCS`] column "layer" via the 5th field on each row.
///
/// Bundler/serve settings live under the top-level **`deck`** object only
/// (`deck.render`, `deck.entry`, `deck.injectGlobals`). Top-level `render` /
/// `entry` / `injectGlobals` are unknown keys and refuse loudly — no dual-read.
///
/// Lint severity floor + top-level `strict` (SPEC-LINTS-1 §3 / S3.2):
/// - `"strict": true` — every lint finding is Error at the build gate
/// - `"lint": { "severity", "rules" }` — global floor + per-rule tree (dual-read
///   with nested `policy.lint.severity.*`)
pub const RECOGNIZED_KEYS: [&str; 20] = [
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
    // SPEC-ICONS-1 / SPEC-SAIL-CONFIG-1 C2 — product icon packs + roles.
    "icons",
    // SPEC-FONTS-1 — role→face pins (Fontsource / vendored). Sibling of icons.
    "fonts",
    "layout",
    // SPEC-LAYOUT-DIAG-1 / S4.1 — experimental layout finding kinds.
    "layoutExperimental",
    "layoutTextMetrics",
    "policy",
    // SPEC-LINTS-1 §3 — global lint severity floor + strict gate (S3.2).
    "strict",
    "lint",
    // Deck bundler/serve object (AppServeConfig) — POLICY layer.
    "deck",
    // Studio product preference (deep-link scheme) — TASTE layer.
    "editor",
];

/// Documentation rows for [`RECOGNIZED_KEYS`] — key, value type, default
/// when absent, one line of semantics, and layer (`POLICY` | `TASTE` | `PRODUCT`).
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
        "icons",
        "{ pack?: string | { id, version? }, version?, packs?: { [id]: { version } }, roles?: { [role]: glyph | \"pack:glyph\" | { pack, glyph } } }",
        "framework default pack lucide + core role table (no app overrides)",
        "icon pack pin + role→glyph map (SPEC-ICONS-1). Author markup uses roles, never pack prefixes",
        "PRODUCT",
    ),
    (
        "fonts",
        "{ roles: { <role>: { source, id, version? } } }",
        "omitted — vendored @ship/tokens FONTS + theme --font-*-app (no regression)",
        "pin font roles (body, heading, mono, display, app-custom) to Fontsource package ids or local faces; unused families do not ship",
        "PRODUCT",
    ),
    (
        "layout",
        "\"warn\" | \"error\"",
        "warn — taffy soft findings stay warnings; silent-clip stays Error",
        "promote gate-gap layout findings (main-axis overflow, collapsed box, height-chain) to Error so check/dev exit non-zero (studio strict profile)",
        "POLICY",
    ),
    (
        "layoutExperimental",
        "boolean",
        "false",
        "enable experimental layout kinds (unintended-overflow, clipped-text) without full lint strict (SPEC-LAYOUT-DIAG-1 / S4.1)",
        "POLICY",
    ),
    (
        "layoutTextMetrics",
        "boolean",
        "false",
        "enable text-metrics-dependent layout kind clipped-text (noisy until takumi-grade metrics; S4.1)",
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
        "strict",
        "boolean",
        "false",
        "every lint finding is Error at the build gate (Warn→Error); per-rule off still silences; never downgrades compiler errors",
        "POLICY",
    ),
    (
        "lint",
        "{ severity?: \"off\"|\"warn\"|\"error\", rules?: { <rule-id>: \"off\"|\"warn\"|\"error\" } }",
        "natural rule severities (no global floor, no per-rule tree)",
        "global lint severity floor + per-rule overrides (SPEC-LINTS-1 §3); dual-read with policy.lint.severity.*",
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

/// One `fonts.roles.<role>` pin (SPEC-FONTS-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontRoleBinding {
    pub role: String,
    pub source: String,
    pub id: String,
    pub version: Option<String>,
}

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

    /// The declared `"stylesheets"` set — the CSS files the components package is allowed to
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

    /// Raw `"icons"` object when present (SPEC-ICONS-1). Typed parse lives in
    /// `sail_compiler::icons` — this crate only exposes the value so there is
    /// still one disk reader.
    pub fn icons_value(&self) -> Option<&serde_json::Value> {
        self.raw.get("icons")
    }

    /// Raw `"fonts"` object when present (SPEC-FONTS-1).
    pub fn fonts_value(&self) -> Option<&serde_json::Value> {
        self.raw.get("fonts")
    }

    /// Role→face bindings from `"fonts".roles`. Absent / wrong-typed → empty.
    /// Each binding is `(role, source, id, version)`.
    pub fn fonts_roles(&self) -> Vec<FontRoleBinding> {
        let Some(roles) = self
            .raw
            .get("fonts")
            .and_then(|v| v.get("roles"))
            .and_then(|v| v.as_object())
        else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (role, spec) in roles {
            let Some(obj) = spec.as_object() else {
                continue;
            };
            let source = obj
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("fontsource")
                .to_string();
            let Some(id) = obj.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            let version = obj
                .get("version")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            out.push(FontRoleBinding {
                role: role.clone(),
                source,
                id: id.to_string(),
                version,
            });
        }
        out
    }

    /// `"layout": "error"` → strict layout gate. Absent / `"warn"` / other → soft.
    /// Wrong-typed values are treated as soft (lenient extract).
    pub fn layout_is_error(&self) -> bool {
        matches!(
            self.raw.get("layout").and_then(|v| v.as_str()),
            Some(s) if s.eq_ignore_ascii_case("error")
        )
    }

    /// S4.1 / SPEC-LAYOUT-DIAG-1 — top-level `"layoutExperimental": true`
    /// enables experimental layout kinds (`unintended-overflow`,
    /// `clipped-text`) without full lint `strict`. Absent / wrong-typed → false.
    pub fn layout_experimental(&self) -> bool {
        matches!(
            self.raw.get("layoutExperimental").and_then(|v| v.as_bool()),
            Some(true)
        )
    }

    /// S4.1 — top-level `"layoutTextMetrics": true` enables text-metrics
    /// kinds (`clipped-text`). Absent / wrong-typed → false.
    pub fn layout_text_metrics(&self) -> bool {
        matches!(self.layout_text_metrics_opt(), Some(true))
    }

    /// LAYOUT-5 — explicit `layoutTextMetrics` when set. `None` means the
    /// promoted default (on) should apply.
    pub fn layout_text_metrics_opt(&self) -> Option<bool> {
        self.raw.get("layoutTextMetrics").and_then(|v| v.as_bool())
    }

    /// Top-level `"strict": true` — every lint finding is Error at the build
    /// gate (SPEC-LINTS-1 §3). Absent / wrong-typed / `false` → `false`.
    /// Does **not** downgrade compiler/parse errors; per-rule `off` still
    /// silences individual lints (kill switch).
    pub fn strict(&self) -> bool {
        matches!(self.raw.get("strict").and_then(|v| v.as_bool()), Some(true))
    }

    /// Global lint severity floor from top-level `"lint": { "severity": … }`.
    ///
    /// Returns a canonical token `"error"` | `"warn"` | `"off"` when the key is
    /// present and a recognized severity string; `None` when absent or
    /// wrong-typed / unrecognized (lenient extract — never hard-fails a build
    /// for a lint floor typo; unknown top-level keys are the loud path).
    ///
    /// Lives on [`SailConfig`] (lint crate) so consumers do not depend on
    /// compiler severity enums. The compiler maps the token onto
    /// `LintSeverityValue` in `apply_config_to_resolved`.
    pub fn lint_severity_floor(&self) -> Option<&str> {
        let s = self
            .raw
            .get("lint")
            .and_then(|lint| lint.get("severity"))
            .and_then(|v| v.as_str())?;
        match s.trim().to_ascii_lowercase().as_str() {
            "error" => Some("error"),
            "warn" | "warning" => Some("warn"),
            "off" | "false" | "0" => Some("off"),
            _ => None,
        }
    }

    /// Per-rule severity map from top-level `"lint": { "rules": { … } }`.
    /// Empty when absent or wrong-typed. Values are the raw severity strings
    /// as authored; the compiler parses them into `LintSeverityValue`.
    pub fn lint_rules(&self) -> Vec<(String, String)> {
        let Some(rules) = self
            .raw
            .get("lint")
            .and_then(|lint| lint.get("rules"))
            .and_then(|v| v.as_object())
        else {
            return Vec::new();
        };
        rules
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect()
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
/// (primitives packs, routes). App-level `.jsonc` may carry `theme`, `icons`,
/// `deck`, and `editor` which are recognized ([`RECOGNIZED_KEYS`]) as
/// POLICY / TASTE / PRODUCT. Serve keys nest under `deck` only. Layout
/// severity still uses [`layout_is_error_for`], which may walk past a
/// capability config that has no `"layout"` key up to an app-level profile.
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

/// Nearest app-level config for product keys (`icons`, `theme`).
///
/// Prefers `sail.config.jsonc` then `.json` at each directory. A capability-only
/// package config (no `icons`) does not stop the walk — app jsonc further up
/// still wins. Broken bodies are skipped.
pub fn resolve_product(from: &Path) -> Option<(PathBuf, SailConfig)> {
    let mut cur = if from.is_file() {
        from.parent()
    } else {
        Some(from)
    };
    let mut first_ok: Option<(PathBuf, SailConfig)> = None;
    while let Some(d) = cur {
        for name in CONFIG_FILES {
            let path = d.join(name);
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let cfg = if name.ends_with('c') {
                SailConfig::from_jsonc(&text)
            } else {
                SailConfig::from_json(&text)
            };
            if !cfg.parse_ok() {
                continue;
            }
            if cfg.has_key("icons") {
                return Some((path, cfg));
            }
            if first_ok.is_none() {
                first_ok = Some((path, cfg));
            }
        }
        cur = d.parent();
    }
    first_ok
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
    fn every_key_is_policy_taste_or_product() {
        for (key, _, _, _, layer) in CONFIG_KEY_DOCS {
            assert!(
                *layer == "POLICY" || *layer == "TASTE" || *layer == "PRODUCT",
                "key `{key}` layer must be POLICY, TASTE, or PRODUCT, got `{layer}`"
            );
        }
    }

    #[test]
    fn icons_key_is_recognized_product() {
        let cfg = SailConfig::from_json(r#"{ "icons": { "pack": "lucide" } }"#);
        assert!(cfg.unknown_keys().is_empty(), "{:?}", cfg.unknown_keys());
        assert!(cfg.has_key("icons"));
        assert_eq!(
            cfg.icons_value()
                .and_then(|v| v.get("pack"))
                .and_then(|v| v.as_str()),
            Some("lucide")
        );
        let layer = CONFIG_KEY_DOCS
            .iter()
            .find(|(k, _, _, _, _)| *k == "icons")
            .map(|r| r.4);
        assert_eq!(layer, Some("PRODUCT"));
    }

    #[test]
    fn fonts_key_is_recognized_product() {
        let cfg = SailConfig::from_json(
            r#"{ "fonts": { "roles": { "body": { "source": "fontsource", "id": "@fontsource-variable/dm-sans", "version": "5.3.0" } } } }"#,
        );
        assert!(cfg.unknown_keys().is_empty(), "{:?}", cfg.unknown_keys());
        assert!(cfg.has_key("fonts"));
        let roles = cfg.fonts_roles();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].role, "body");
        assert_eq!(roles[0].id, "@fontsource-variable/dm-sans");
        assert_eq!(roles[0].version.as_deref(), Some("5.3.0"));
        let layer = CONFIG_KEY_DOCS
            .iter()
            .find(|(k, _, _, _, _)| *k == "fonts")
            .map(|r| r.4);
        assert_eq!(layer, Some("PRODUCT"));
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
        let ok = SailConfig::from_json(
            r#"{ "deck": { "render": "ssr" }, "theme": { "default": "ship" } }"#,
        );
        assert!(ok.unknown_keys().is_empty(), "nested deck is recognized");
        assert!(ok.has_key("deck"));

        let unknown_top =
            SailConfig::from_json(r#"{ "render": "ssr", "deck": { "render": "csr" } }"#);
        assert_eq!(unknown_top.unknown_keys(), vec!["render".to_string()]);
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
