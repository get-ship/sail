//! The single reader of `sail.config.json`.
//!
//! [`SailConfig`] is the typed document (serde + schemars). Schema, unknown-key
//! refusal, and wrong-type refusal all fall out of that type. Adding a key means
//! adding a field with an owner — not a row in a string table.
//!
//! One walk-up, one parse. Every consumer — the compiler's `Policy` tier, theme
//! generate, Deck serve, and the floor's undeclared-stylesheet check — derives
//! from this resolved value. That is `inv-sail-one-policy-resolver`.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use schemars::schema::{
    InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec,
    SubschemaValidation,
};
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const CONFIG_FILE: &str = "sail.config.json";

/// Directory of the [`crate::project::Project`] that owns `from`.
///
/// Same walk as [`Project::discover`](crate::project::Project::discover):
/// nearest `sail.config.jsonc` / `.json` (jsonc wins in-dir). `routes/` is
/// not a tie-break.
pub fn owning_dir(from: &Path) -> Option<PathBuf> {
    crate::project::Project::discover(from).map(|p| p.root().to_path_buf())
}

/// Config filenames [`crate::project::Project::discover`] accepts. `jsonc`
/// wins when both exist in the same directory.
pub const CONFIG_FILES: &[&str] = &["sail.config.jsonc", "sail.config.json"];

/// True if `dir` itself contains a sail config file. Existence only —
/// does not read or parse. Project-root walkers use this so they do
/// not construct the filename (inv-sail-one-policy-resolver).
pub fn dir_has_config(dir: &Path) -> bool {
    CONFIG_FILES.iter().any(|name| dir.join(name).is_file())
}

pub use crate::schema::{
    ConfigKeyDoc, ConfigLayer, config_key_docs, emit_sail_config_schema, recognized_keys,
};

// ── Leaf enums ──────────────────────────────────────────────────────────────

/// Capability tier above the floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityPreset {
    Strict,
    Full,
    Repair,
    Off,
}

impl CapabilityPreset {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Full => "full",
            Self::Repair => "repair",
            Self::Off => "off",
        }
    }
}

/// Compose autofix preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AutofixSetting {
    #[serde(alias = "fix", alias = "on")]
    Repair,
    #[serde(alias = "error")]
    Strict,
    #[serde(alias = "false", alias = "0")]
    Off,
}

impl AutofixSetting {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Repair => "repair",
            Self::Strict => "strict",
            Self::Off => "off",
        }
    }
}

/// Layout finding severity profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LayoutSetting {
    Warn,
    Error,
}

impl LayoutSetting {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

/// Lint severity token (`error` | `warn` | `off`, plus historical aliases).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverity {
    Error,
    Warn,
    Off,
}

impl LintSeverity {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "error" => Some(Self::Error),
            "warn" | "warning" => Some(Self::Warn),
            "off" | "false" | "0" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Off => "off",
        }
    }
}

impl<'de> Deserialize<'de> for LintSeverity {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse(&s).ok_or_else(|| {
            de::Error::custom(format!(
                "unrecognized severity \"{s}\" (expected error|warn|off)"
            ))
        })
    }
}

/// Catalog allowlist: `"all"` or an array of theme ids.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ThemeEnabled {
    All,
    Ids(Vec<String>),
}

impl<'de> Deserialize<'de> for ThemeEnabled {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        match v {
            Value::String(s) if s == "all" => Ok(Self::All),
            Value::String(s) => Err(de::Error::custom(format!(
                "theme config: \"enabled\" string must be the literal \"all\" \
                 (got {s:?}); use an array of catalog ids for a thin allowlist"
            ))),
            Value::Array(arr) => {
                let mut ids = Vec::with_capacity(arr.len());
                for item in arr {
                    let Some(s) = item.as_str() else {
                        return Err(de::Error::custom(
                            "theme config: \"enabled\" array elements must all be strings \
                             (catalog theme ids)",
                        ));
                    };
                    if !s.is_empty() {
                        ids.push(s.to_string());
                    }
                }
                Ok(Self::Ids(ids))
            }
            other => Err(de::Error::custom(format!(
                "theme config: \"enabled\" must be an array of catalog ids or the \
                 string \"all\" (got {other})"
            ))),
        }
    }
}

impl JsonSchema for ThemeEnabled {
    fn schema_name() -> String {
        "ThemeEnabled".into()
    }

    fn json_schema(_gen: &mut schemars::r#gen::SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "catalog allowlist: omit/[] thin; \"all\" full; list filter".into(),
                ),
                ..Default::default()
            })),
            subschemas: Some(Box::new(SubschemaValidation {
                one_of: Some(vec![
                    Schema::Object(SchemaObject {
                        instance_type: Some(InstanceType::String.into()),
                        const_value: Some(json!("all")),
                        ..Default::default()
                    }),
                    Schema::Object(SchemaObject {
                        instance_type: Some(InstanceType::Array.into()),
                        array: Some(Box::new(schemars::schema::ArrayValidation {
                            items: Some(SingleOrVec::Single(Box::new(Schema::Object(
                                SchemaObject {
                                    instance_type: Some(InstanceType::String.into()),
                                    ..Default::default()
                                },
                            )))),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

// ── Nested objects ──────────────────────────────────────────────────────────

/// POLICY. App theme emit set. Owner: tokens generate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThemeConfig {
    /// Preferred default theme id (scaffold / host paint).
    #[serde(default)]
    pub default: Option<String>,
    /// Initial `mode` (prefer `light` when omitted at host).
    #[serde(default)]
    pub default_mode: Option<String>,
    /// Catalog theme allowlist. Omit → default sail; `[]` → none; `"all"` → full.
    #[serde(default)]
    pub enabled: Option<ThemeEnabled>,
}

/// POLICY. Global lint floor + per-rule tree. Owner: compiler / lint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LintConfig {
    /// Global lint severity floor (`off` | `warn` | `error`).
    #[serde(default)]
    pub severity: Option<LintSeverity>,
    /// Per-rule overrides. Owner: compiler `ResolvedPolicy`.
    #[serde(default)]
    pub rules: Option<BTreeMap<String, LintSeverity>>,
}

/// PRODUCT. Document defaults for SPA index.html. Owner: Deck / `SiteDocument`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    /// `index.html` `<title>`. Absent → `package.json#name`.
    #[serde(default)]
    pub title: Option<String>,
    /// `<meta name="description">`. Absent → omitted.
    #[serde(default)]
    pub description: Option<String>,
    /// `<html lang>`. Absent → `en`.
    #[serde(default)]
    pub lang: Option<String>,
}

/// POLICY. Deck bundler/serve (`AppServeConfig`). Owner: Deck. Nested only.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeckConfig {
    /// First-paint mode. Absent → `csr`.
    #[serde(default)]
    pub render: Option<DeckRender>,
    /// Host entry bootstrap (`entry.gen.ts`).
    #[serde(default)]
    pub entry: Option<DeckEntry>,
    /// URL-capture → `window.<k>` injections.
    #[serde(default)]
    pub inject_globals: Option<BTreeMap<String, String>>,
}

/// Deck `render` token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DeckRender {
    Csr,
    Ssr,
}

impl DeckRender {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csr => "csr",
            Self::Ssr => "ssr",
        }
    }
}

/// Host-owned entry bootstrap under `deck.entry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeckEntry {
    /// Module-relative specifier of the host's client factory.
    pub clients_module: String,
    /// Names spread into the `clients` map.
    pub expose_names: Vec<String>,
}

/// PRODUCT. Icon pack pin + role→glyph map. Nested unions stay in the compiler.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IconsConfig {
    pub fields: serde_json::Map<String, Value>,
}

impl JsonSchema for IconsConfig {
    fn schema_name() -> String {
        "IconsConfig".into()
    }

    fn json_schema(_gen: &mut schemars::r#gen::SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                additional_properties: Some(Box::new(Schema::Bool(true))),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "icon pack pin + role→glyph map (SPEC-ICONS-1). Author markup uses roles, never pack prefixes"
                        .into(),
                ),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

// ── Document ────────────────────────────────────────────────────────────────

/// Typed `sail.config.json(c)` document. Schema is generated from this type.
///
/// Parse metadata (`parse_ok`, unknown keys, type errors) is skipped by serde
/// and schemars — it is filled by [`SailConfig::from_json`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SailConfig {
    #[serde(default, skip)]
    #[schemars(skip)]
    parse_ok: bool,
    #[serde(default, skip)]
    #[schemars(skip)]
    unknown_keys: Vec<String>,
    #[serde(default, skip)]
    #[schemars(skip)]
    type_error: Option<String>,
    #[serde(default, skip)]
    #[schemars(skip)]
    present_keys: Vec<String>,

    /// POLICY. Select a capability tier above the floor in one word (`full` grants raw_containers; Style is ungated).
    ///
    /// Default: no tier — capabilities opt in individually.
    #[serde(default)]
    pub preset: Option<CapabilityPreset>,

    /// POLICY. Capability override: allow raw container elements instead of the layout keywords.
    ///
    /// Default: off.
    #[serde(default)]
    pub raw_containers: Option<bool>,

    /// POLICY. Hyphenated CEs in this project may spend Style slot form
    /// (`type({ size: xs })`, `bg({ clip: text })`, `filter({ blur: 4 })`).
    /// Routes stay tag-only. Product apps omit this and stay presets.
    /// Default: off.
    #[serde(default)]
    pub style_slots: Option<bool>,

    /// POLICY. Auto-fix sibling rhythm violations; false warns only.
    ///
    /// Default: true.
    #[serde(default)]
    pub compose_rhythm: Option<bool>,

    /// POLICY. Compose autofix preset: confluent fixes on / all error / pass silent.
    ///
    /// Default: repair.
    #[serde(default)]
    pub autofix: Option<AutofixSetting>,

    /// POLICY. The CSS files this package may ship; an undeclared stylesheet fails the floor check.
    ///
    /// Default: none.
    #[serde(default)]
    pub stylesheets: Option<Vec<String>>,

    /// POLICY. fetch/WebSocket/EventSource allowlist; `[]` grants nothing explicitly.
    ///
    /// Default: absent = full refusal.
    #[serde(default)]
    pub network: Option<Vec<String>>,

    /// POLICY. Opt out of controller ambient-authority confinement entirely.
    ///
    /// Default: confined.
    #[serde(default)]
    pub unconfined: Option<bool>,

    /// POLICY. App theme emit set: default id, optional mode, catalog allowlist. Owner: tokens generate.
    ///
    /// Default: no theme section — thin app emit (no catalog); package generate still full catalog.
    #[serde(default)]
    pub theme: Option<ThemeConfig>,

    /// PRODUCT. Icon pack pin + role→glyph map. Owner: tokens / compiler.
    ///
    /// Default: framework default pack lucide + core role table (no app overrides).
    #[serde(default)]
    pub icons: Option<IconsConfig>,

    /// POLICY. Promote gate-gap layout findings to Error so check/dev exit non-zero.
    ///
    /// Default: warn — taffy soft findings stay warnings; silent-clip stays Error.
    #[serde(default)]
    pub layout: Option<LayoutSetting>,

    /// POLICY. Enable experimental layout kinds (sub-min-tap-target) without full lint strict.
    ///
    /// Default: false.
    #[serde(default, rename = "layoutExperimental")]
    pub layout_experimental: Option<bool>,

    /// POLICY. Kill switch for text-metrics layout kind clipped-text (LAYOUT-5 default on; false disables).
    ///
    /// Default: absent → promoted default (on).
    #[serde(default, rename = "layoutTextMetrics")]
    pub layout_text_metrics: Option<bool>,

    /// POLICY. Every lint finding is Error at the build gate (Warn→Error); per-rule off still silences.
    ///
    /// Default: false.
    #[serde(default)]
    pub strict: Option<bool>,

    /// POLICY. Global lint severity floor + per-rule overrides. Owner: compiler `ResolvedPolicy`.
    ///
    /// Default: natural rule severities (no global floor, no per-rule tree).
    #[serde(default)]
    pub lint: Option<LintConfig>,

    /// POLICY. Deck bundler/serve settings. Nested only — top-level render/entry/injectGlobals refuse as unknown.
    ///
    /// Default: absent — fully conventional serve defaults (csr, no entry, no inject).
    #[serde(default)]
    pub deck: Option<DeckConfig>,

    /// PRODUCT. Application identity (title, description, lang). Owner: Deck `SiteDocument`.
    ///
    /// Default: absent — index.html title is package.json#name, lang=en, no description.
    #[serde(default)]
    pub app: Option<AppConfig>,
}

impl SailConfig {
    /// Parse config JSON text. Unparseable text yields an empty config
    /// (`parse_ok() == false`). Unknown top-level keys and wrong types are
    /// recorded on the value; [`crate`] callers refuse them loudly.
    pub fn from_json(text: &str) -> SailConfig {
        match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(map)) => Self::from_object(map),
            Ok(_) => SailConfig {
                parse_ok: true,
                type_error: Some("sail.config must be an object".into()),
                ..SailConfig::default()
            },
            Err(_) => SailConfig {
                parse_ok: false,
                ..SailConfig::default()
            },
        }
    }

    /// Parse JSON or JSONC body (`//` and `/* */` comments + trailing commas).
    pub fn from_jsonc(text: &str) -> SailConfig {
        SailConfig::from_json(&strip_jsonc_comments(text))
    }

    fn from_object(map: serde_json::Map<String, Value>) -> SailConfig {
        let recognized = crate::schema::recognized_key_set();
        let present_keys: Vec<String> = map.keys().cloned().collect();
        let unknown_keys: Vec<String> = present_keys
            .iter()
            .filter(|k| !recognized.contains(k.as_str()))
            .cloned()
            .collect();

        let mut cleaned = map.clone();
        for k in &unknown_keys {
            cleaned.remove(k);
        }

        match serde_json::from_value::<SailConfig>(Value::Object(cleaned)) {
            Ok(mut cfg) => {
                cfg.parse_ok = true;
                cfg.unknown_keys = unknown_keys;
                cfg.present_keys = present_keys;
                cfg
            }
            Err(e) => {
                let type_error = first_type_error(&map).unwrap_or_else(|| e.to_string());
                SailConfig {
                    parse_ok: true,
                    unknown_keys,
                    type_error: Some(type_error),
                    present_keys,
                    ..SailConfig::default()
                }
            }
        }
    }

    /// False iff the config body failed to parse as JSON at all (a malformed
    /// file, not merely an unrecognized key).
    pub fn parse_ok(&self) -> bool {
        self.parse_ok
    }

    /// Wrong-type refusal for a recognized key (like `"network": "https://…"`).
    pub fn type_error(&self) -> Option<&str> {
        self.type_error.as_deref()
    }

    /// The `"preset"` token, if present.
    pub fn preset(&self) -> Option<&str> {
        self.preset.map(CapabilityPreset::as_str)
    }

    /// Whether the config names `key` at all (any value/type).
    pub fn has_key(&self, key: &str) -> bool {
        self.present_keys.iter().any(|k| k == key)
    }

    /// Top-level keys present in the config that are not fields of [`SailConfig`].
    pub fn unknown_keys(&self) -> Vec<String> {
        self.unknown_keys.clone()
    }

    /// Declared `"network"` allowlist. `None` when the key is absent.
    /// Wrong-typed values are [`Self::type_error`] — never silently granted.
    pub fn network(&self) -> Option<&[String]> {
        self.network.as_deref()
    }

    /// Historical `network_key` shape used by `classify`.
    ///
    /// `None` = absent. `Some(Ok(_))` = well-typed. Wrong types surface as
    /// [`Self::type_error`] instead of `Some(Err(()))`.
    pub fn network_key(&self) -> Option<Result<Vec<String>, ()>> {
        self.network.as_ref().map(|v| Ok(v.clone()))
    }

    /// Declared `"unconfined"` capability. `None` when absent.
    pub fn unconfined(&self) -> Option<bool> {
        self.unconfined
    }

    /// Historical `unconfined_key` shape used by `classify`.
    pub fn unconfined_key(&self) -> Option<Result<bool, ()>> {
        self.unconfined.map(Ok)
    }

    /// The declared `"stylesheets"` set. Absent → empty.
    pub fn declared_stylesheets(&self) -> HashSet<String> {
        self.stylesheets
            .as_ref()
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Raw `"icons"` object when present (SPEC-ICONS-1). Typed role parse lives
    /// in `sail_compiler::icons`.
    pub fn icons_value(&self) -> Option<Value> {
        self.icons.as_ref().map(|m| Value::Object(m.fields.clone()))
    }

    /// `"layout": "error"` → strict layout gate. Absent / `"warn"` → soft.
    pub fn layout_is_error(&self) -> bool {
        matches!(self.layout, Some(LayoutSetting::Error))
    }

    /// S4.1 — `"layoutExperimental": true`.
    pub fn layout_experimental(&self) -> bool {
        matches!(self.layout_experimental, Some(true))
    }

    /// LAYOUT-5 — `"layoutTextMetrics": true` only when explicitly set.
    pub fn layout_text_metrics(&self) -> bool {
        matches!(self.layout_text_metrics, Some(true))
    }

    /// LAYOUT-5 — explicit `layoutTextMetrics` when set. `None` = promoted default (on).
    pub fn layout_text_metrics_opt(&self) -> Option<bool> {
        self.layout_text_metrics
    }

    /// Top-level `"strict": true`.
    pub fn strict(&self) -> bool {
        matches!(self.strict, Some(true))
    }

    /// Global lint severity floor from `"lint": { "severity": … }`.
    pub fn lint_severity_floor(&self) -> Option<&str> {
        self.lint
            .as_ref()
            .and_then(|l| l.severity)
            .map(LintSeverity::as_str)
    }

    /// Per-rule severity map from `"lint": { "rules": { … } }`.
    pub fn lint_rules(&self) -> Vec<(String, String)> {
        let Some(rules) = self.lint.as_ref().and_then(|l| l.rules.as_ref()) else {
            return Vec::new();
        };
        rules
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().to_string()))
            .collect()
    }

    /// Recognized top-level keys currently present (for NoSignal hints).
    pub fn present_recognized_keys(&self) -> Vec<String> {
        let recognized = crate::schema::recognized_key_set();
        self.present_keys
            .iter()
            .filter(|k| recognized.contains(k.as_str()))
            .cloned()
            .collect()
    }
}

fn first_type_error(map: &serde_json::Map<String, Value>) -> Option<String> {
    let bool_keys = [
        "raw_containers",
        "style_slots",
        "compose_rhythm",
        "unconfined",
        "strict",
        "layoutExperimental",
        "layoutTextMetrics",
    ];
    for key in bool_keys {
        if let Some(v) = map.get(key) {
            if !v.is_boolean() {
                return Some(format!("\"{key}\" must be a boolean"));
            }
        }
    }
    if let Some(v) = map.get("network") {
        if !matches!(v, Value::Array(a) if a.iter().all(Value::is_string)) {
            return Some("\"network\" must be an array of origin strings".into());
        }
    }
    if let Some(v) = map.get("stylesheets") {
        if !matches!(v, Value::Array(a) if a.iter().all(Value::is_string)) {
            return Some("\"stylesheets\" must be an array of strings".into());
        }
    }
    for key in ["theme", "lint", "deck", "app", "icons"] {
        if let Some(v) = map.get(key) {
            if !v.is_null() && !v.is_object() {
                return Some(format!("\"{key}\" must be an object"));
            }
        }
    }
    None
}

// ── Walk-up ─────────────────────────────────────────────────────────────────

/// A trailing clause naming the `sail.config.json` that could grant capability
/// `key` for a source at `path`. Does not read the file.
pub fn grant_hint(path: &Path, key: &str) -> String {
    match path.parent().and_then(owning_dir) {
        Some(dir) => format!(
            " Grant it in `{}` — `{{ \"{key}\": true }}`.",
            dir.join(CONFIG_FILE).display()
        ),
        None => String::new(),
    }
}

/// Load the sail config in `dir` itself (no walk-up). `jsonc` wins when both
/// exist. A present-but-unreadable file yields `None`.
pub fn load_in(dir: &Path) -> Option<(PathBuf, SailConfig)> {
    let path = crate::project::config_file_in(dir)?;
    let text = std::fs::read_to_string(&path).ok()?;
    let cfg = if path.extension().and_then(|e| e.to_str()) == Some("jsonc") {
        SailConfig::from_jsonc(&text)
    } else {
        SailConfig::from_json(&text)
    };
    Some((path, cfg))
}

/// Load the [`crate::project::Project`] root that owns `dir`.
///
/// One discover + one [`load_in`] — not a second walk with different stop
/// rules. A present-but-unparseable config resolves to an empty `SailConfig`
/// (`parse_ok() == false`); its presence is the package's declared intent.
pub fn resolve(dir: &Path) -> Option<(PathBuf, SailConfig)> {
    let project = crate::project::Project::discover(dir)?;
    load_in(project.root())
}

/// Product keys (`icons`, `theme`) on the same stack as [`resolve`].
///
/// Not a second walker — overlays fields on the one Project load.
pub fn resolve_product(from: &Path) -> Option<(PathBuf, SailConfig)> {
    resolve(from)
}

/// True when the Project config for `source_file` sets `"style_slots": true`.
///
/// Missing config or missing key is no grant. Routes still cannot Style.
pub fn style_slots_for(source_file: &Path) -> bool {
    resolve(source_file)
        .map(|(_, cfg)| cfg.style_slots == Some(true))
        .unwrap_or(false)
}

/// True when the Project config for `source_file` sets `"layout": "error"`.
///
/// Reads that field on the [`resolve`] stack. A missing key is not an
/// ancestor walk.
pub fn layout_is_error_for(source_file: &Path) -> bool {
    resolve(source_file)
        .map(|(_, cfg)| cfg.layout_is_error())
        .unwrap_or(false)
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
    fn wrong_type_strict_refuses() {
        let cfg = SailConfig::from_json(r#"{ "strict": 1 }"#);
        assert!(cfg.parse_ok());
        assert!(
            cfg.type_error()
                .is_some_and(|e| e.contains("strict") && e.contains("boolean")),
            "{:?}",
            cfg.type_error()
        );
    }

    #[test]
    fn unknown_sx_refuses() {
        let cfg = SailConfig::from_json(r#"{ "sx": true }"#);
        assert!(cfg.parse_ok());
        assert!(
            cfg.unknown_keys().iter().any(|k| k == "sx"),
            "{:?}",
            cfg.unknown_keys()
        );
    }

    #[test]
    fn layout_key_error_parses() {
        let cfg = SailConfig::from_json(r#"{ "layout": "error" }"#);
        assert!(cfg.layout_is_error());
        let soft = SailConfig::from_json(r#"{ "layout": "warn" }"#);
        assert!(!soft.layout_is_error());
        let absent = SailConfig::from_json(r#"{ "raw_containers": true }"#);
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
            r#"{ "deck": { "render": "ssr" }, "theme": { "default": "sail" } }"#,
        );
        assert!(ok.type_error().is_none(), "{:?}", ok.type_error());
        assert!(ok.unknown_keys().is_empty(), "nested deck is recognized");
        assert!(ok.has_key("deck"));
        assert_eq!(
            ok.deck.as_ref().and_then(|d| d.render),
            Some(DeckRender::Ssr)
        );

        let unknown_top =
            SailConfig::from_json(r#"{ "render": "ssr", "deck": { "render": "csr" } }"#);
        assert_eq!(unknown_top.unknown_keys(), vec!["render".to_string()]);
        assert_eq!(
            unknown_top.deck.as_ref().and_then(|d| d.render),
            Some(DeckRender::Csr)
        );
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
    fn layout_is_error_reads_the_project_config_not_an_ancestor() {
        let dir = tempfile::tempdir().expect("tempdir");
        let routes = dir.path().join("routes");
        std::fs::create_dir_all(&routes).expect("mkdir");
        std::fs::write(
            routes.join("sail.config.json"),
            r#"{ "raw_containers": false }"#,
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
            !layout_is_error_for(&page),
            "layout is a field on the Project load, not a second walk"
        );
        let owned = owning_dir(&page).expect("owning dir");
        let (resolved, _) = resolve(&page).expect("resolve");
        assert_eq!(owned, routes);
        assert_eq!(resolved.parent().unwrap(), routes.as_path());
    }

    #[test]
    fn layout_is_error_uses_nearest_project_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("routes")).unwrap();
        std::fs::write(
            dir.path().join("sail.config.jsonc"),
            r#"{ "layout": "error" }"#,
        )
        .unwrap();
        let page = dir.path().join("routes/page.sail");
        std::fs::write(&page, "<column></column>").unwrap();
        assert!(layout_is_error_for(&page));
    }

    #[test]
    fn owning_dir_and_resolve_agree_on_nested_package_config() {
        let tmp = tempfile::tempdir().unwrap();
        let app = tmp.path().join("hello");
        std::fs::create_dir_all(app.join("routes")).unwrap();
        std::fs::write(app.join("sail.config.jsonc"), "{}\n").unwrap();
        let foo = app.join("packages/foo");
        std::fs::create_dir_all(foo.join("src")).unwrap();
        std::fs::write(foo.join("sail.config.json"), "{}\n").unwrap();
        let src = foo.join("src/x.sail");
        std::fs::write(&src, "<p></p>\n").unwrap();

        let owned = owning_dir(&src).expect("owning");
        let (resolved_path, _) = resolve(&src).expect("resolve");
        assert_eq!(owned, foo, "owning_dir is the nested package");
        assert_eq!(
            resolved_path.parent().unwrap(),
            foo.as_path(),
            "resolve loads that same Project root"
        );
        assert_eq!(
            crate::project::Project::discover(&src).unwrap().root(),
            foo.as_path()
        );
    }
}
