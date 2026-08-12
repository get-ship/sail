//! Compiler policy registry — one typed table is the single source of truth.
//!
//! Skill, MCP, RPC, and the `sail-compiler policy` CLI are projections of this
//! module. Knobs are decided by the feature that owns them; this module ships
//! only the **mechanism** plus two proof families:
//!
//! - `layout.raw_containers` (bool, default `false`)
//! - `lint.severity.<rule-id>` (per-rule tree of `error` | `warn` | `off`)
//!
//! Plus the SPEC-LINTS-1 §3 operator surface (S3.2), resolved here so one
//! [`ResolvedPolicy`] carries the full lint gate dial:
//!
//! - top-level `"strict": true` — every lint finding → Error at the build gate
//! - top-level `"lint": { "severity": "error"|"warn"|"off", "rules": { … } }`
//!   — global floor + per-rule tree (merged with nested `policy.lint.severity.*`)
//!
//! No `platforms` field — one platform today. Scope is always `project`.
//!
//! ## Config surface
//!
//! Registry knobs live under the top-level `"policy"` object of
//! `sail.config.json(c)`:
//!
//! ```json
//! {
//!   "policy": {
//!     "layout": { "raw_containers": true },
//!     "lint": { "severity": { "sail-for-key": "warn" } }
//!   }
//! }
//! ```
//!
//! SPEC-LINTS-1 §3 also accepts the product shape at the top level (dual-read
//! with the nested tree — both land in [`ResolvedPolicy::lint_severities`]):
//!
//! ```jsonc
//! {
//!   "strict": true,
//!   "lint": {
//!     "severity": "error",
//!     "rules": { "sail-no-free-aria": "off" }
//!   }
//! }
//! ```
//!
//! Legacy top-level `"raw_containers"` still feeds the same knob (no behavior
//! change for existing projects). Unknown keys under `"policy"` are a hard
//! error that names the closest valid key. Compiler/parse errors are never
//! severity-configurable through this surface.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::check::lint_overrides::{LintOverride, OverrideSeverity};

// ── Schema ──────────────────────────────────────────────────────────────────

/// Value type of a registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyType {
    Bool,
    Enum,
    Path,
    List,
    /// Namespace tree: every lint rule gets a severity under `lint.severity.`.
    PerRuleTree,
}

impl PolicyType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Enum => "enum",
            Self::Path => "path",
            Self::List => "list",
            Self::PerRuleTree => "per-rule-tree",
        }
    }
}

/// Stability of a registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyStatus {
    Stable,
    Staging,
}

impl PolicyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Staging => "staging",
        }
    }
}

/// One static registry entry (the documentation row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicyEntry {
    /// Dotted name, e.g. `layout.raw_containers`, or tree pattern
    /// `lint.severity.<rule-id>`.
    pub name: &'static str,
    pub ty: PolicyType,
    /// JSON-serializable default encoded as a static string for `Copy`:
    /// `"true"` / `"false"` / `"null"` / quoted string. Serialized via
    /// [`PolicyEntry::default_value`].
    pub default_json: &'static str,
    /// Always `project` today.
    pub scope: &'static str,
    pub description: &'static str,
    pub status: PolicyStatus,
}

impl PolicyEntry {
    /// Parsed default as a JSON value.
    pub fn default_value(&self) -> Value {
        serde_json::from_str(self.default_json).unwrap_or(Value::Null)
    }
}

/// The static table — proof-of-mechanism knobs only. Do not add feature knobs
/// here; each feature owns its own knob decisions.
pub static REGISTRY: &[PolicyEntry] = &[
    PolicyEntry {
        name: "layout.raw_containers",
        ty: PolicyType::Bool,
        default_json: "false",
        scope: "project",
        description: "Allow raw <div>/<span> containers instead of layout keywords",
        status: PolicyStatus::Stable,
    },
    PolicyEntry {
        name: "lint.severity.<rule-id>",
        ty: PolicyType::PerRuleTree,
        default_json: "null",
        scope: "project",
        description: "Per-rule severity override (error|warn|off) under the lint.severity. namespace",
        status: PolicyStatus::Stable,
    },
];

/// Prefix for the lint severity tree.
pub const LINT_SEVERITY_PREFIX: &str = "lint.severity.";

/// Canonical name of the raw-containers knob.
pub const RAW_CONTAINERS_KEY: &str = "layout.raw_containers";

// ── Resolved values ─────────────────────────────────────────────────────────

/// Severity tokens accepted under `lint.severity.<rule-id>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverityValue {
    Error,
    Warn,
    Off,
}

impl LintSeverityValue {
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

    pub fn to_override(self) -> OverrideSeverity {
        match self {
            Self::Error => OverrideSeverity::Error,
            Self::Warn => OverrideSeverity::Warn,
            Self::Off => OverrideSeverity::Off,
        }
    }
}

/// Fully resolved policy values for a project (registry defaults + config).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResolvedPolicy {
    /// Effective `layout.raw_containers`. Always populated after full resolve
    /// (defaults to the registry default `false` when unset).
    pub raw_containers: bool,
    /// Whether `layout.raw_containers` was **explicitly** set by the `"policy"`
    /// object (not merely defaulted). Callers that layer presets / legacy
    /// top-level keys consult this so a policy-only lint.severity tree does
    /// not clobber `preset: full`.
    pub raw_containers_explicit: bool,
    /// Explicit per-rule severity overrides from `policy.lint.severity.*` and
    /// top-level `lint.rules.*` (dual-read; one merged tree).
    pub lint_severities: BTreeMap<String, LintSeverityValue>,
    /// Top-level `"strict": true` — promote every remaining lint finding to
    /// Error at the build gate. Per-rule `off` still silences.
    pub strict: bool,
    /// Global lint severity floor from top-level `"lint": { "severity": … }`.
    /// `Some(Error)` alone promotes Warn→Error (same gate effect as `strict`
    /// for promotion; does not force every rule on). `None` = natural severities.
    pub lint_severity_floor: Option<LintSeverityValue>,
    /// Source path of the config that supplied values, when known.
    pub config_path: Option<std::path::PathBuf>,
}

impl ResolvedPolicy {
    /// Registry defaults only (no config).
    pub fn defaults() -> Self {
        Self {
            raw_containers: false,
            raw_containers_explicit: false,
            lint_severities: BTreeMap::new(),
            strict: false,
            lint_severity_floor: None,
            config_path: None,
        }
    }

    /// True when the build gate must promote remaining lint findings from
    /// Warn → Error (`strict: true` or global floor `error`).
    pub fn promotes_lint_warn_to_error(&self) -> bool {
        self.strict || self.lint_severity_floor == Some(LintSeverityValue::Error)
    }

    /// Lint overrides implied by the severity tree (empty when none set).
    pub fn lint_overrides(&self) -> Vec<LintOverride> {
        self.lint_severities
            .iter()
            .map(|(rule, sev)| LintOverride {
                rule: rule.clone(),
                severity: sev.to_override(),
            })
            .collect()
    }
}

// ── Errors ──────────────────────────────────────────────────────────────────

/// Failure while deserializing / validating a `"policy"` object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyError {
    pub message: String,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PolicyError {}

// ── Lookup helpers ──────────────────────────────────────────────────────────

/// Static registry entry by exact name (tree pattern included).
pub fn entry_by_name(name: &str) -> Option<&'static PolicyEntry> {
    REGISTRY.iter().find(|e| e.name == name)
}

/// Valid concrete key names for closest-match suggestions.
///
/// Includes the fixed knobs plus every `lint.severity.<rule-id>` expansion.
pub fn valid_keys() -> Vec<String> {
    let mut keys = vec![RAW_CONTAINERS_KEY.to_string()];
    for id in sail_lint::all_rule_ids() {
        keys.push(format!("{LINT_SEVERITY_PREFIX}{id}"));
    }
    keys
}

/// Closest valid key for an unknown dotted name (Levenshtein).
pub fn closest_key(unknown: &str) -> Option<String> {
    let keys = valid_keys();
    let mut best: Option<(usize, &str)> = None;
    for k in &keys {
        let d = levenshtein(unknown, k);
        let max_d = (k.len().max(unknown.len()) / 3).max(2);
        if d <= max_d && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, k.as_str()));
        }
    }
    best.map(|(_, s)| s.to_string())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

// ── Deserialization ─────────────────────────────────────────────────────────

/// Flatten a nested `"policy"` JSON object into dotted keys.
///
/// `{ "layout": { "raw_containers": true } }` → `layout.raw_containers = true`
/// `{ "lint": { "severity": { "sail-for-key": "warn" } } }` →
/// `lint.severity.sail-for-key = "warn"`
fn flatten_policy_object(v: &Value, prefix: &str, out: &mut BTreeMap<String, Value>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                match child {
                    Value::Object(_) => flatten_policy_object(child, &path, out),
                    other => {
                        out.insert(path, other.clone());
                    }
                }
            }
        }
        other => {
            if !prefix.is_empty() {
                out.insert(prefix.to_string(), other.clone());
            }
        }
    }
}

/// Is `key` a known registry key (fixed or lint.severity.<valid-rule>)?
fn is_known_key(key: &str) -> bool {
    if key == RAW_CONTAINERS_KEY {
        return true;
    }
    if let Some(rule) = key.strip_prefix(LINT_SEVERITY_PREFIX) {
        if rule.is_empty() || rule.contains('.') {
            return false;
        }
        // Accept any non-empty rule id shape; unknown rule ids are forward-
        // compatible (same as LintOverride — a typo has no effect at gate).
        // The registry still documents the tree; we only reject malformed
        // prefixes / unknown *top-level* policy branches.
        return !rule.is_empty();
    }
    false
}

/// Validate a flattened key is under a known family; return a hard error with
/// closest valid key when not.
fn validate_key(key: &str) -> Result<(), PolicyError> {
    if is_known_key(key) {
        return Ok(());
    }
    // Also reject intermediate-only paths that shouldn't appear as leaves
    // (e.g. bare "layout" with a non-object was already leaf-flattened).
    let suggestion = closest_key(key);
    let msg = match suggestion {
        Some(s) => format!("unknown policy key \"{key}\" (closest valid key: \"{s}\")"),
        None => format!("unknown policy key \"{key}\""),
    };
    Err(PolicyError { message: msg })
}

/// Parse the `"policy"` object from a config JSON value into a [`ResolvedPolicy`].
///
/// Returns `Ok(None)` when the key is absent. Wrong-typed `"policy"` (not an
/// object) and unknown keys are hard errors.
pub fn resolve_policy_object(raw: &Value) -> Result<Option<ResolvedPolicy>, PolicyError> {
    let Some(policy_val) = raw.get("policy") else {
        return Ok(None);
    };
    if policy_val.is_null() {
        return Ok(None);
    }
    if !policy_val.is_object() {
        return Err(PolicyError {
            message: "\"policy\" must be an object".to_string(),
        });
    }

    let mut flat = BTreeMap::new();
    flatten_policy_object(policy_val, "", &mut flat);

    let mut resolved = ResolvedPolicy::defaults();
    for (key, val) in &flat {
        validate_key(key)?;
        if key == RAW_CONTAINERS_KEY {
            let Some(b) = val.as_bool() else {
                return Err(PolicyError {
                    message: format!("\"{RAW_CONTAINERS_KEY}\" must be a boolean"),
                });
            };
            resolved.raw_containers = b;
            resolved.raw_containers_explicit = true;
            continue;
        }
        if let Some(rule) = key.strip_prefix(LINT_SEVERITY_PREFIX) {
            let Some(s) = val.as_str() else {
                return Err(PolicyError {
                    message: format!("\"{key}\" must be a severity string (error|warn|off)"),
                });
            };
            let Some(sev) = LintSeverityValue::parse(s) else {
                return Err(PolicyError {
                    message: format!(
                        "unrecognized severity \"{s}\" for \"{key}\" (expected error|warn|off)"
                    ),
                });
            };
            resolved.lint_severities.insert(rule.to_string(), sev);
            continue;
        }
        // validate_key should have rejected anything else
        return Err(PolicyError {
            message: format!("unknown policy key \"{key}\""),
        });
    }
    Ok(Some(resolved))
}

/// Resolve policy for a source file: registry defaults, then the nearest
/// `sail.config.json(c)`'s `"policy"` object, then legacy top-level
/// `"raw_containers"` when the nested key is absent.
///
/// Does **not** apply styling presets (`full`/`strict`) — those stay in
/// [`crate::check::policy::Policy`]. This returns only registry-governed knobs.
pub fn resolve_for_source(source_file: &Path) -> Result<ResolvedPolicy, PolicyError> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = source_file;
        return Ok(ResolvedPolicy::defaults());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut resolved = ResolvedPolicy::defaults();
        let Some(parent) = source_file.parent() else {
            return Ok(resolved);
        };
        let Some((config_path, cfg)) = sail_lint::config::resolve(parent) else {
            return Ok(resolved);
        };
        resolved.config_path = Some(config_path);
        let applied = apply_config_to_resolved(&cfg)?;
        resolved.raw_containers = applied.raw_containers;
        resolved.raw_containers_explicit = applied.raw_containers_explicit;
        resolved.lint_severities = applied.lint_severities;
        resolved.strict = applied.strict;
        resolved.lint_severity_floor = applied.lint_severity_floor;
        Ok(resolved)
    }
}

/// Merge a nested `"policy"` object (and legacy top-level `raw_containers`)
/// plus SPEC-LINTS-1 top-level `"strict"` / `"lint"` into a [`ResolvedPolicy`].
///
/// Used by `check::policy::classify` so capability tier + registry share one
/// parse of the config body.
///
/// Precedence for `layout.raw_containers`:
/// 1. Explicit `policy.layout.raw_containers` (`raw_containers_explicit = true`)
/// 2. Legacy top-level `"raw_containers"` (explicit flag stays false — the
///    capability-tier loop still owns the signal line)
/// 3. Registry default (`false`)
///
/// Precedence for per-rule severities (dual-read):
/// 1. Nested `policy.lint.severity.<rule>` (seeded first)
/// 2. Top-level `lint.rules.<rule>` overlays (SPEC product shape wins on clash)
pub fn apply_config_to_resolved(
    cfg: &sail_lint::config::SailConfig,
) -> Result<ResolvedPolicy, PolicyError> {
    let mut resolved = ResolvedPolicy::defaults();
    if let Some(from_obj) = resolve_policy_object(cfg.raw())? {
        resolved.lint_severities = from_obj.lint_severities;
        if from_obj.raw_containers_explicit {
            resolved.raw_containers = from_obj.raw_containers;
            resolved.raw_containers_explicit = true;
        } else if let Some(b) = cfg.bool_key("raw_containers") {
            resolved.raw_containers = b;
        }
    } else if let Some(b) = cfg.bool_key("raw_containers") {
        resolved.raw_containers = b;
    }

    // SPEC-LINTS-1 §3 — top-level strict + lint floor/rules.
    resolved.strict = cfg.strict();
    if let Some(floor) = cfg.lint_severity_floor() {
        // Token already canonicalized by SailConfig; parse is infallible here.
        resolved.lint_severity_floor = LintSeverityValue::parse(floor);
    }
    for (rule, sev_str) in cfg.lint_rules() {
        let Some(sev) = LintSeverityValue::parse(&sev_str) else {
            return Err(PolicyError {
                message: format!(
                    "unrecognized severity \"{sev_str}\" for lint.rules.{rule} (expected error|warn|off)"
                ),
            });
        };
        // Top-level lint.rules overlays nested policy.lint.severity (SPEC path).
        resolved.lint_severities.insert(rule, sev);
    }

    Ok(resolved)
}

// ── Shared serialization (CLI / MCP / axum) ─────────────────────────────────

/// One registry row as JSON — shared by every projection.
fn entry_to_json(e: &PolicyEntry) -> Value {
    json!({
        "name": e.name,
        "type": e.ty.as_str(),
        "default": e.default_value(),
        "scope": e.scope,
        "description": e.description,
        "status": e.status.as_str(),
    })
}

/// `list` payload: registry with types / defaults / descriptions.
///
/// Shape is identical across CLI JSON mode, MCP `sail_policy` list, and the
/// deck-axum policy endpoint.
pub fn serialize_list() -> Value {
    json!({
        "mode": "list",
        "entries": REGISTRY.iter().map(entry_to_json).collect::<Vec<_>>(),
    })
}

/// `get` payload: registry rows + resolved values.
pub fn serialize_get(resolved: &ResolvedPolicy) -> Value {
    let mut values = serde_json::Map::new();
    values.insert(
        RAW_CONTAINERS_KEY.to_string(),
        Value::Bool(resolved.raw_containers),
    );
    for (rule, sev) in &resolved.lint_severities {
        values.insert(
            format!("{LINT_SEVERITY_PREFIX}{rule}"),
            Value::String(sev.as_str().to_string()),
        );
    }
    json!({
        "mode": "get",
        "entries": REGISTRY.iter().map(entry_to_json).collect::<Vec<_>>(),
        "values": Value::Object(values),
        "configPath": resolved.config_path.as_ref().map(|p| p.display().to_string()),
    })
}

/// Unified entry used by MCP / axum / tests: `mode` is `"list"` or `"get"`.
///
/// For `get`, pass an optional project directory (defaults = registry defaults
/// when no config is found).
pub fn serialize_policy(mode: &str, project_dir: Option<&Path>) -> Result<Value, PolicyError> {
    match mode {
        "list" => Ok(serialize_list()),
        "get" => {
            let resolved = match project_dir {
                Some(dir) => resolve_for_source(&dir.join("_probe.sail"))?,
                None => ResolvedPolicy::defaults(),
            };
            Ok(serialize_get(&resolved))
        }
        other => Err(PolicyError {
            message: format!("unknown policy mode \"{other}\" (expected list|get)"),
        }),
    }
}

/// Human table for `sail-compiler policy` (docs projection — no markdown file).
pub fn format_registry_table() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{:<32} {:<14} {:<10} {:<10} {}\n",
        "NAME", "TYPE", "DEFAULT", "STATUS", "DESCRIPTION"
    ));
    out.push_str(&format!(
        "{:<32} {:<14} {:<10} {:<10} {}\n",
        "----", "----", "-------", "------", "-----------"
    ));
    for e in REGISTRY {
        let default = match e.default_value() {
            Value::Null => "(natural)".to_string(),
            Value::Bool(b) => b.to_string(),
            other => other.to_string(),
        };
        out.push_str(&format!(
            "{:<32} {:<14} {:<10} {:<10} {}\n",
            e.name,
            e.ty.as_str(),
            default,
            e.status.as_str(),
            e.description
        ));
    }
    out
}

/// Skill-facing summary of resolved registry values (agent-facing truth).
pub fn format_skill_section(resolved: &ResolvedPolicy) -> String {
    let mut md = String::new();
    md.push_str("## Compiler policy (registry)\n\n");
    md.push_str("Resolved from the compiler policy registry (`sail-compiler policy`).\n\n");
    md.push_str(&format!(
        "- `layout.raw_containers`: **{}**\n",
        resolved.raw_containers
    ));
    if resolved.lint_severities.is_empty() {
        md.push_str("- `lint.severity.*`: _(no overrides — rules use natural severity)_\n");
    } else {
        md.push_str("- `lint.severity.*`:\n");
        for (rule, sev) in &resolved.lint_severities {
            md.push_str(&format!("  - `{rule}`: **{}**\n", sev.as_str()));
        }
    }
    md.push('\n');
    md
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn registry_has_exactly_the_proof_knobs() {
        let names: Vec<&str> = REGISTRY.iter().map(|e| e.name).collect();
        assert_eq!(
            names,
            vec!["layout.raw_containers", "lint.severity.<rule-id>"]
        );
        assert!(!REGISTRY.iter().any(|e| {
            // platforms field deliberately absent from schema
            let v = entry_to_json(e);
            v.get("platforms").is_some()
        }));
    }

    #[test]
    fn unknown_key_names_closest_valid() {
        let raw = json!({
            "policy": {
                "layout": { "raw_containrs": true }
            }
        });
        let err = resolve_policy_object(&raw).expect_err("typo must fail");
        assert!(
            err.message.contains("raw_containrs"),
            "must name the unknown key: {}",
            err.message
        );
        assert!(
            err.message.contains("layout.raw_containers"),
            "must name closest valid key: {}",
            err.message
        );
    }

    #[test]
    fn unknown_lint_family_key_suggests_severity_tree() {
        let raw = json!({
            "policy": {
                "lint": { "severty": { "sail-for-key": "warn" } }
            }
        });
        // flattens to lint.severty.sail-for-key
        let err = resolve_policy_object(&raw).expect_err("typo must fail");
        assert!(err.message.contains("closest valid key"), "{}", err.message);
    }

    #[test]
    fn raw_containers_and_two_lint_severities_round_trip() {
        let raw = json!({
            "policy": {
                "layout": { "raw_containers": true },
                "lint": {
                    "severity": {
                        "sail-for-key": "warn",
                        "sail-compose-rhythm": "off"
                    }
                }
            }
        });
        let resolved = resolve_policy_object(&raw).expect("ok").expect("some");
        assert!(resolved.raw_containers);
        assert_eq!(
            resolved.lint_severities.get("sail-for-key"),
            Some(&LintSeverityValue::Warn)
        );
        assert_eq!(
            resolved.lint_severities.get("sail-compose-rhythm"),
            Some(&LintSeverityValue::Off)
        );

        // serialize_get round-trips values
        let payload = serialize_get(&resolved);
        assert_eq!(payload["mode"], "get");
        assert_eq!(payload["values"]["layout.raw_containers"], true);
        assert_eq!(payload["values"]["lint.severity.sail-for-key"], "warn");
        assert_eq!(
            payload["values"]["lint.severity.sail-compose-rhythm"],
            "off"
        );
    }

    #[test]
    fn jsonc_policy_object_round_trip_via_sail_config() {
        let text = r#"{
            // proof knobs
            "policy": {
                "layout": { "raw_containers": false },
                "lint": {
                    "severity": {
                        "sail-for-key": "error",
                        "sail-no-sx": "off"
                    }
                }
            }
        }"#;
        let cfg = sail_lint::config::SailConfig::from_jsonc(text);
        assert!(cfg.parse_ok());
        let resolved = apply_config_to_resolved(&cfg).expect("resolve");
        assert!(!resolved.raw_containers);
        assert_eq!(
            resolved.lint_severities.get("sail-for-key"),
            Some(&LintSeverityValue::Error)
        );
        assert_eq!(
            resolved.lint_severities.get("sail-no-sx"),
            Some(&LintSeverityValue::Off)
        );
    }

    #[test]
    fn legacy_top_level_raw_containers_still_reads() {
        let cfg = sail_lint::config::SailConfig::from_json(r#"{"raw_containers": true}"#);
        let resolved = apply_config_to_resolved(&cfg).expect("resolve");
        assert!(resolved.raw_containers);
        assert!(resolved.lint_severities.is_empty());
    }

    #[test]
    fn nested_policy_wins_over_legacy_top_level() {
        let cfg = sail_lint::config::SailConfig::from_json(
            r#"{"raw_containers": true, "policy": {"layout": {"raw_containers": false}}}"#,
        );
        let resolved = apply_config_to_resolved(&cfg).expect("resolve");
        // Nested object present → it is the authority for registry knobs.
        assert!(!resolved.raw_containers);
    }

    #[test]
    fn list_and_get_share_entry_shape() {
        let list = serialize_list();
        let get = serialize_get(&ResolvedPolicy::defaults());
        assert_eq!(list["entries"], get["entries"]);
        assert_eq!(list["mode"], "list");
        assert_eq!(get["mode"], "get");
        assert!(get.get("values").is_some());
    }

    #[test]
    fn serialize_policy_rejects_bad_mode() {
        let err = serialize_policy("write", None).expect_err("no write path");
        assert!(err.message.contains("list|get"));
    }

    #[test]
    fn lint_overrides_from_severity_tree() {
        let mut r = ResolvedPolicy::defaults();
        r.lint_severities
            .insert("sail-for-key".into(), LintSeverityValue::Warn);
        let o = r.lint_overrides();
        assert_eq!(o.len(), 1);
        assert_eq!(o[0].rule, "sail-for-key");
        assert_eq!(o[0].severity, OverrideSeverity::Warn);
    }

    #[test]
    fn format_registry_table_names_proof_knobs() {
        let t = format_registry_table();
        assert!(t.contains("layout.raw_containers"));
        assert!(t.contains("lint.severity.<rule-id>"));
        assert!(t.contains("bool"));
        assert!(t.contains("per-rule-tree"));
    }
}
