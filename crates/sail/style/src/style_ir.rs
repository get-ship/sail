//! **INTERNAL leaf IR** for Style paint-op emit — not an author surface.
//!
//! Product paint is Style (`style({ box, paint, text, animate })`).
//! Utility-string sx is a hard residual error (`is_residual_utility_sx`).
//! This module is residual detect plus a token-value helper so Style setters
//! can lower a closed property + token to CSS.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Serialize;

use crate::prefixes as sx_rules;
use crate::style_vocab::{self, Resolved};
use crate::tokens;

// ─── IR ──────────────────────────────────────────────────────────────────────

/// How a value is classified in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SxValue {
    /// Design-token name or path (`primary`, `space-2`, `space.2`, `color.primary`).
    Token(String),
    /// CSS keyword / physics value (`center`, `solid`, `hidden` via bare).
    Keyword(String),
    /// Explicit raw CSS escape (`raw:#3366ff`, `raw:13px`). Only with `raw:`.
    Raw(String),
    /// Runtime custom-property read: `(--name)` → `var(--name)`.
    VarRef(String),
}

/// One style declaration in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SxDecl {
    /// Closed prefixes: `hover`, `focus`, `md`/`tablet`, `before`, `group-hover`, …
    pub prefixes: Vec<String>,
    /// Closed property name (`bg`, `px`, `rounded`, …) or bare atom name (`hidden`).
    pub prop: String,
    /// Value; `None` only for bare atoms (`hidden`, `truncate`, `italic`).
    pub value: Option<SxValue>,
    /// Negation (`-m: 4`).
    pub negative: bool,
    /// True when this was a bare atom with no value.
    pub bare: bool,
    /// Source fragment (for diagnostics).
    pub source: String,
}

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SxError {
    UnknownProp {
        prop: String,
        source: String,
    },
    UnknownAtom {
        atom: String,
        source: String,
    },
    BadValue {
        prop: String,
        value: String,
        source: String,
    },
    RawWithoutEscape {
        value: String,
        source: String,
    },
    Refused {
        atom: String,
        instead: String,
        source: String,
    },
    EmptyValue {
        prop: String,
        source: String,
    },
    UnknownPrefix {
        prefix: String,
        source: String,
    },
}

impl SxError {
    pub fn message(&self) -> String {
        match self {
            SxError::UnknownProp { prop, .. } => {
                format!("sx unknown property `{prop}` — not in the closed property set")
            }
            SxError::UnknownAtom { atom, .. } => {
                format!("sx `{atom}` — not in the vocabulary")
            }
            SxError::BadValue { prop, value, .. } => {
                format!("sx `{prop}: {value}` — value is not a token or accepted keyword")
            }
            SxError::RawWithoutEscape { value, .. } => {
                format!(
                    "sx raw color/length `{value}` requires `raw:` escape \
                     (e.g. `bg: raw:{value}`) — token values only by default"
                )
            }
            SxError::Refused { atom, instead, .. } => {
                format!("sx `{atom}` is not sx — use {instead}")
            }
            SxError::EmptyValue { prop, .. } => {
                format!("sx property `{prop}` is missing a value")
            }
            SxError::UnknownPrefix { prefix, .. } => {
                format!("sx unknown prefix `{prefix}:`")
            }
        }
    }

    pub fn source(&self) -> &str {
        match self {
            SxError::UnknownProp { source, .. }
            | SxError::UnknownAtom { source, .. }
            | SxError::BadValue { source, .. }
            | SxError::RawWithoutEscape { source, .. }
            | SxError::Refused {
                atom: _,
                instead: _,
                source,
            }
            | SxError::EmptyValue { source, .. }
            | SxError::UnknownPrefix { source, .. } => source,
        }
    }
}

// ─── Property aliases (lead-B names → vocab utility names) ───────────────────

/// Author-facing property names that map onto an existing vocab head.
/// Value disambiguation (shared heads like `text` / `font` / `border`) still
/// runs through `style_vocab::resolve` after the atom is built.
const PROP_ALIASES: &[(&str, &str)] = &[
    ("color", "text"),       // text color (color-scale values)
    ("font-size", "text"),   // type scale
    ("font-weight", "font"), // weight scale
    ("font-family", "font"), // font scale
    ("radius", "rounded"),
    ("bg-color", "bg"),
];

fn alias_prop(prop: &str) -> &str {
    PROP_ALIASES
        .iter()
        .find(|(a, _)| *a == prop)
        .map(|(_, v)| *v)
        .unwrap_or(prop)
}

/// Is `name` a closed property (vocab utility head, bare atom, or alias)?
pub fn is_property(name: &str) -> bool {
    let n = name.strip_prefix('-').unwrap_or(name);
    if PROP_ALIASES.iter().any(|(a, _)| *a == n) {
        return true;
    }
    let names = style_vocab::names();
    names.iter().any(|k| k == n)
}

/// Documented closed property table (canonical props + aliases).
pub fn property_table() -> Vec<PropertyMeta> {
    let mut out: BTreeMap<String, PropertyMeta> = BTreeMap::new();
    for (name, values) in style_vocab::vocabulary() {
        out.insert(
            name.clone(),
            PropertyMeta {
                name: name.clone(),
                alias_of: None,
                values_sample: values.into_iter().take(24).collect(),
            },
        );
    }
    for (alias, target) in PROP_ALIASES {
        let values_sample = out
            .get(*target)
            .map(|m| m.values_sample.clone())
            .unwrap_or_default();
        out.entry((*alias).to_string())
            .or_insert_with(|| PropertyMeta {
                name: (*alias).to_string(),
                alias_of: Some((*target).to_string()),
                values_sample,
            });
    }
    out.into_values().collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct PropertyMeta {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias_of: Option<String>,
    pub values_sample: Vec<String>,
}

// ─── Value helpers ───────────────────────────────────────────────────────────

/// Looks like a raw color that must use `raw:`.
pub fn looks_like_raw_color(v: &str) -> bool {
    let v = v.trim();
    v.starts_with('#')
        || starts_with_fn(v, "rgb")
        || starts_with_fn(v, "rgba")
        || starts_with_fn(v, "hsl")
        || starts_with_fn(v, "hsla")
        || starts_with_fn(v, "oklch")
        || starts_with_fn(v, "oklab")
        || starts_with_fn(v, "hwb")
}

/// Looks like a raw length/unit that must use `raw:` on scale props.
pub fn looks_like_raw_length(v: &str) -> bool {
    let v = v.trim();
    if v == "0" {
        return false;
    }
    let bytes = v.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    // digit…unit
    if !bytes[0].is_ascii_digit() && bytes[0] != b'.' && bytes[0] != b'-' {
        return false;
    }
    v.ends_with("px")
        || v.ends_with("rem")
        || v.ends_with("em")
        || v.ends_with("vh")
        || v.ends_with("vw")
        || v.ends_with("dvh")
        || v.ends_with("dvw")
        || v.ends_with('%')
        || v.ends_with("ch")
        || v.ends_with("ex")
        || v.ends_with("pt")
        || v.ends_with("cm")
        || v.ends_with("mm")
        || v.ends_with("in")
}

fn starts_with_fn(v: &str, name: &str) -> bool {
    v.len() > name.len()
        && v[..name.len()].eq_ignore_ascii_case(name)
        && v.as_bytes().get(name.len()) == Some(&b'(')
}

/// Dual value resolve: path (`space.2`, `color.primary`) or cssName (`space-2`, `primary`).
/// Returns candidate atom-value suffixes to try against the vocab.
pub fn value_candidates(value: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let push = |out: &mut Vec<String>, s: String| {
        if !s.is_empty() && !out.iter().any(|x| x == &s) {
            out.push(s);
        }
    };

    push(&mut out, value.to_string());

    // Registry v2 path → cssName (when J1 paths are present).
    if value.contains('.') {
        if let Some(css) = tokens::path_to_css_name(value) {
            push(&mut out, css.to_string());
        }
    }

    let kebab = value.replace('.', "-");
    push(&mut out, kebab.clone());

    // Last path segment: space.2 → 2, radius.sm → sm, color.primary → primary
    if let Some((_, last)) = value.rsplit_once('.') {
        push(&mut out, last.to_string());
    }

    // Known scale-prefix strip on kebab / resolved cssName forms.
    let strip_sources: Vec<String> = out.clone();
    for src in strip_sources {
        for prefix in [
            "space-",
            "radius-",
            "text-",
            "shadow-",
            "leading-",
            "tracking-",
            "duration-",
            "ease-",
            "font-weight-",
            "z-",
            "color-",
            "control-h-",
            "icon-size-",
        ] {
            if let Some(rest) = src.strip_prefix(prefix) {
                push(&mut out, rest.to_string());
            }
        }
    }

    out
}

// ─── Lower ───────────────────────────────────────────────────────────────────

/// Vocab atom key for a decl (without prefixes) — token-value helper for emit.
pub fn decl_atom(decl: &SxDecl) -> String {
    if decl.bare || decl.value.is_none() {
        let mut a = String::new();
        if decl.negative {
            a.push('-');
        }
        a.push_str(&decl.prop);
        return a;
    }
    let Some(value) = decl.value.as_ref() else {
        // Guarded above; keep the impossible path total instead of panicking.
        return alias_prop(&decl.prop).to_string();
    };
    match value {
        SxValue::Raw(_) => {
            // Raw does not go through atom resolve; atom key is synthetic.
            format!("{}-raw", alias_prop(&decl.prop))
        }
        SxValue::VarRef(name) => {
            let prop = alias_prop(&decl.prop);
            let mut a = String::new();
            if decl.negative {
                a.push('-');
            }
            a.push_str(prop);
            a.push_str("-(--");
            a.push_str(name);
            a.push(')');
            a
        }
        SxValue::Token(v) | SxValue::Keyword(v) => {
            let prop = alias_prop(&decl.prop);
            // Try candidates until one resolves.
            for cand in value_candidates(v) {
                let mut a = String::new();
                if decl.negative {
                    a.push('-');
                }
                a.push_str(prop);
                a.push('-');
                a.push_str(&cand);
                if matches!(style_vocab::resolve(&a), Resolved::Ok(_)) {
                    return a;
                }
            }
            // Fallback: first kebab candidate (for class naming / errors).
            let mut a = String::new();
            if decl.negative {
                a.push('-');
            }
            a.push_str(prop);
            a.push('-');
            a.push_str(&v.replace('.', "-"));
            a
        }
    }
}

/// CSS declarations for one IR decl. `None` if it cannot lower (unknown / bad).
pub fn lower_decl(decl: &SxDecl) -> Option<Vec<(String, String)>> {
    if let Some(SxValue::Raw(css_val)) = &decl.value {
        // Map property → CSS props via resolving a dummy accepted atom if possible,
        // else known alias css list from vocab by probing.
        return lower_raw(decl, css_val);
    }
    let atom = decl_atom(decl);
    match style_vocab::resolve(&atom) {
        Resolved::Ok(decls) => Some(decls),
        _ => None,
    }
}

fn lower_raw(decl: &SxDecl, css_val: &str) -> Option<Vec<(String, String)>> {
    let prop = alias_prop(&decl.prop);
    // Probe vocab for the CSS property list by finding any accepted value.
    // Fall back to identity mapping for known simple props.
    let css_props = prop_css_properties(prop)?;
    Some(
        css_props
            .into_iter()
            .map(|p| (p, css_val.to_string()))
            .collect(),
    )
}

fn prop_css_properties(prop: &str) -> Option<Vec<String>> {
    // From vocab table: pick first utility with this name that has css list.
    for (name, _) in style_vocab::vocabulary() {
        if name != prop {
            continue;
        }
        // Resolve any sample to get css — use a well-known probe per prop.
        break;
    }
    // Probe common *scale* values first so we learn CSS props for the utility head.
    // Put color/size tokens before keywords like `none` — bare composites
    // (`bg-none` → background-image) must not win over `bg-primary` (background-color)
    // when resolving raw: paint for `bg`.
    let probes = [
        "primary",
        "foreground",
        "muted",
        "0",
        "1",
        "2",
        "4",
        "sm",
        "md",
        "lg",
        "full",
        "auto",
        "solid",
        "hidden",
        "transparent",
        "currentColor",
        "center",
        "body",
        "semibold",
        "pointer",
        "none",
    ];
    for p in probes {
        let atom = format!("{prop}-{p}");
        // Skip bare exact atoms that are not `{prop}-{value}` utilities (e.g. bg-none bare).
        if matches!(style_vocab::resolve(p), Resolved::Ok(_)) && p == "none" && prop == "bg" {
            continue;
        }
        if let Resolved::Ok(decls) = style_vocab::resolve(&atom) {
            // Prefer color/background-color over background-image for `bg` raw paint.
            if prop == "bg"
                && decls.iter().any(|(k, _)| k == "background-image")
                && !decls
                    .iter()
                    .any(|(k, _)| k == "background-color" || k == "background")
            {
                continue;
            }
            return Some(decls.into_iter().map(|(k, _)| k).collect());
        }
    }
    // Bare?
    if let Resolved::Ok(decls) = style_vocab::resolve(prop) {
        return Some(decls.into_iter().map(|(k, _)| k).collect());
    }
    // Hardcoded fallbacks for aliases / common props.
    let fallback: &[(&str, &[&str])] = &[
        ("bg", &["background-color"]),
        ("text", &["color"]),
        ("color", &["color"]),
        ("p", &["padding"]),
        ("px", &["padding-inline"]),
        ("py", &["padding-block"]),
        ("m", &["margin"]),
        ("mx", &["margin-inline"]),
        ("my", &["margin-block"]),
        ("w", &["width"]),
        ("h", &["height"]),
        ("min-h", &["min-height"]),
        ("min-w", &["min-width"]),
        ("max-h", &["max-height"]),
        ("max-w", &["max-width"]),
        ("rounded", &["border-radius"]),
        ("shadow", &["box-shadow"]),
        ("opacity", &["opacity"]),
        ("border", &["border-color"]),
        ("leading", &["line-height"]),
        ("tracking", &["letter-spacing"]),
        ("font", &["font-family"]),
    ];
    fallback
        .iter()
        .find(|(n, _)| *n == prop)
        .map(|(_, ps)| ps.iter().map(|s| (*s).to_string()).collect())
}

/// Validate one decl against vocab / raw rules.
pub fn validate_decl(decl: &SxDecl) -> Result<(), SxError> {
    // Prefix check
    for p in &decl.prefixes {
        if !sx_rules::is_prefix(p) {
            return Err(SxError::UnknownPrefix {
                prefix: p.clone(),
                source: decl.source.clone(),
            });
        }
    }

    if let Some(SxValue::Raw(_)) = &decl.value {
        // raw: is the escape hatch — property must still be closed.
        if !is_property(&decl.prop) {
            return Err(SxError::UnknownProp {
                prop: decl.prop.clone(),
                source: decl.source.clone(),
            });
        }
        if prop_css_properties(alias_prop(&decl.prop)).is_none() {
            return Err(SxError::UnknownProp {
                prop: decl.prop.clone(),
                source: decl.source.clone(),
            });
        }
        return Ok(());
    }

    let atom = decl_atom(decl);
    match style_vocab::resolve(&atom) {
        Resolved::Ok(_) => Ok(()),
        Resolved::Refused { instead } => Err(SxError::Refused {
            atom,
            instead,
            source: decl.source.clone(),
        }),
        Resolved::BadValue { name, .. } => {
            let value = match &decl.value {
                Some(SxValue::Token(v) | SxValue::Keyword(v)) => v.clone(),
                Some(SxValue::VarRef(v)) => format!("(--{v})"),
                _ => String::new(),
            };
            // If it looks raw, prefer the raw diagnostic.
            if looks_like_raw_color(&value) || looks_like_raw_length(&value) {
                return Err(SxError::RawWithoutEscape {
                    value: value.clone(),
                    source: decl.source.clone(),
                });
            }
            Err(SxError::BadValue {
                prop: name,
                value,
                source: decl.source.clone(),
            })
        }
        Resolved::Unknown => {
            if decl.bare {
                Err(SxError::UnknownAtom {
                    atom,
                    source: decl.source.clone(),
                })
            } else if !is_property(&decl.prop) {
                Err(SxError::UnknownProp {
                    prop: decl.prop.clone(),
                    source: decl.source.clone(),
                })
            } else {
                let value = match &decl.value {
                    Some(SxValue::Token(v) | SxValue::Keyword(v)) => v.clone(),
                    _ => String::new(),
                };
                Err(SxError::BadValue {
                    prop: decl.prop.clone(),
                    value,
                    source: decl.source.clone(),
                })
            }
        }
    }
}

// ─── Emit helpers (class name + rule body for preprocess) ────────────────────

/// Sanitized atomic class name for a decl.
pub fn class_name(decl: &SxDecl) -> String {
    let atom = decl_atom(decl);
    let sanitized: String = atom.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if decl.prefixes.is_empty() {
        format!("s-{sanitized}")
    } else {
        let sp: String = decl
            .prefixes
            .iter()
            .flat_map(|s| s.chars())
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        format!("s-{sp}-{sanitized}")
    }
}

/// Prefix/media wiring for one decl.
pub struct EmitSelector {
    pub media: String,
    pub ancestor: String,
    pub pseudo: String,
}

pub fn emit_selector(decl: &SxDecl) -> Option<EmitSelector> {
    let mut media = String::new();
    let mut ancestor = String::new();
    let mut pseudo = String::new();
    for s in &decl.prefixes {
        if let Some(m) = sx_rules::condition_media(s) {
            media = m.to_string();
        } else if let Some(base) = s.strip_prefix("group-") {
            let p = sx_rules::state_pseudo(base)
                .map(str::to_string)
                .or_else(|| sx_rules::state_data(base))
                .or_else(|| sx_rules::group_attr_presence(base).map(|n| format!("[{n}]")))?;
            ancestor = format!(".sail-grp{p} ");
        } else if let Some(pe) = sx_rules::pseudo_element_prefix(s) {
            pseudo.push_str(pe);
        } else if let Some(p) = sx_rules::state_pseudo(s) {
            pseudo.push_str(p);
        } else if let Some(p) = sx_rules::state_data(s) {
            pseudo.push_str(&p);
        } else {
            return None;
        }
    }
    Some(EmitSelector {
        media,
        ancestor,
        pseudo,
    })
}

/// Lower a Style expression (product path) into class list + CSS rules.
///
/// Utility-string sx is **not** accepted — returns an Err carrying the Style
/// diagnostics so callers can surface them as compile errors. Sole architecture:
/// chain + match/when/compound/bind.
///
/// `env` resolves scope Style consts (`Style::Ref`).
/// Old `sx_ir` parse/emit remains as internal leaf helper for paint-op emit only.
pub fn lower_sx_string(
    val: &str,
    env: &crate::StyleEnv,
    css: &mut String,
    seen: &mut std::collections::HashSet<String>,
) -> Result<String, Vec<crate::StyleDiagnostic>> {
    crate::emit::lower_style_source(val, env, css, seen)
}

/// Product Style lower + extracted motion attaches (`.enter` / `.exit` / …).
pub fn lower_sx_string_with_motion(
    val: &str,
    env: &crate::StyleEnv,
    css: &mut String,
    seen: &mut std::collections::HashSet<String>,
) -> Result<(String, Vec<(String, String)>), Vec<crate::StyleDiagnostic>> {
    crate::emit::lower_style_source_with_motion(val, env, css, seen)
}

/// Parse-only residual detection for preprocess/lint (utility string product).
pub fn is_residual_utility_sx(val: &str) -> bool {
    crate::parse::looks_like_utility_string_product(val.trim())
}

// ─── Agent schema ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AgentSchema {
    pub version: u32,
    pub form: &'static str,
    pub properties: Vec<PropertyMeta>,
    pub aliases: BTreeMap<&'static str, &'static str>,
    pub prefixes: PrefixSchema,
    pub value_kinds: &'static [&'static str],
    pub tokens: TokenSchema,
    pub notes: &'static [&'static str],
}

#[derive(Debug, Serialize)]
pub struct PrefixSchema {
    pub states: Vec<&'static str>,
    pub conditions: Vec<&'static str>,
    pub pseudo_elements: Vec<&'static str>,
    pub extra: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct TokenSchema {
    pub count: usize,
    pub colors_count: usize,
    pub sample_names: Vec<&'static str>,
    pub resolve: &'static str,
}

/// Agent-facing schema snippet derived from the property table + registry.
pub fn agent_schema() -> AgentSchema {
    let props = property_table();
    let mut names: Vec<&'static str> = tokens::registry().iter().copied().collect();
    names.sort_unstable();
    let sample: Vec<&'static str> = names.iter().copied().take(40).collect();

    AgentSchema {
        version: 0,
        form: "residual detect + token-value helper — not an author surface",
        properties: props,
        aliases: PROP_ALIASES.iter().copied().collect(),
        prefixes: PrefixSchema {
            states: sx_rules::STATE_PREFIXES.iter().map(|(n, _)| *n).collect(),
            conditions: sx_rules::CONDITION_PREFIXES
                .iter()
                .map(|(n, _)| *n)
                .collect(),
            pseudo_elements: vec!["before", "after", "empty"],
            extra: vec!["group", "group-<state>:", "state-<value>:"],
        },
        value_kinds: &["token", "keyword", "raw", "var-ref"],
        tokens: TokenSchema {
            count: names.len(),
            colors_count: tokens::color_values().len(),
            sample_names: sample,
            resolve: "path (dotted → kebab) or cssName (flat registry name); dual resolve",
        },
        notes: &[
            "Closed property set only — do not invent utilities.",
            "Token values from registry; keywords for display/position physics.",
            "Raw colors/lengths require raw: prefix.",
            "States use existing hover:/focus:/… prefixes mapped into IR.",
            "Static compile-time only — no dynamic style={…}.",
            "tone/size are not sx properties (context attrs / variant axes).",
        ],
    }
}

/// Pretty JSON for the agent schema snippet.
pub fn agent_schema_json() -> String {
    // Manual serialize to avoid requiring serde_json pretty in all builds —
    // serde_json is already a dep.
    serde_json::to_string_pretty(&agent_schema()).unwrap_or_else(|_| "{}".into())
}

/// Cached schema JSON (for tests / tooling).
pub fn agent_schema_json_cached() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(agent_schema_json).as_str()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(prop: &str, value: &str) -> SxDecl {
        SxDecl {
            prefixes: Vec::new(),
            prop: prop.to_string(),
            value: Some(SxValue::Token(value.to_string())),
            negative: false,
            bare: false,
            source: format!("{prop} {value}"),
        }
    }

    fn raw(prop: &str, css: &str) -> SxDecl {
        SxDecl {
            prefixes: Vec::new(),
            prop: prop.to_string(),
            value: Some(SxValue::Raw(css.to_string())),
            negative: false,
            bare: false,
            source: format!("{prop} raw:{css}"),
        }
    }

    #[test]
    fn color_alias_atom() {
        assert_eq!(decl_atom(&tok("color", "primary")), "text-primary");
    }

    #[test]
    fn lower_px_space() {
        let d = lower_decl(&tok("px", "2")).expect("lower");
        assert_eq!(d, vec![("padding-inline".into(), "var(--space-2)".into())]);
    }

    #[test]
    fn lower_path_space_dot() {
        let d = lower_decl(&tok("px", "space.2")).expect("lower space.2");
        assert_eq!(d, vec![("padding-inline".into(), "var(--space-2)".into())]);
    }

    #[test]
    fn lower_bg() {
        let d = lower_decl(&tok("bg", "primary")).expect("lower bg");
        assert_eq!(
            d,
            vec![("background-color".into(), "var(--primary)".into())]
        );
    }

    #[test]
    fn lower_rounded_path() {
        let d = lower_decl(&tok("rounded", "radius.sm")).expect("rounded");
        assert_eq!(d[0].0, "border-radius");
        assert_eq!(d[0].1, "var(--radius-sm)");
    }

    #[test]
    fn lower_raw_emits_css_literal() {
        let d = lower_decl(&raw("bg", "#3366ff")).expect("raw");
        assert_eq!(d, vec![("background-color".into(), "#3366ff".into())]);
    }

    #[test]
    fn validate_rejects_bad_token() {
        assert!(validate_decl(&tok("bg", "not-a-real-token-zz")).is_err());
    }

    #[test]
    fn lower_bare_hidden() {
        let decl = SxDecl {
            prefixes: Vec::new(),
            prop: "hidden".into(),
            value: None,
            negative: false,
            bare: true,
            source: "hidden".into(),
        };
        let d = lower_decl(&decl).unwrap();
        assert_eq!(d, vec![("display".into(), "none".into())]);
    }

    #[test]
    fn class_name_stable() {
        assert_eq!(class_name(&tok("px", "2")), "s-px2");
        let hover = SxDecl {
            prefixes: vec!["hover".into()],
            prop: "bg".into(),
            value: Some(SxValue::Token("primary".into())),
            negative: false,
            bare: false,
            source: "hover bg primary".into(),
        };
        assert_eq!(class_name(&hover), "s-hover-bgprimary");
    }

    #[test]
    fn lower_sx_string_emits_style_chain_css() {
        // Product path: Style value language only.
        let mut css = String::new();
        let mut seen = std::collections::HashSet::new();
        let env = std::collections::HashMap::new();
        let classes = lower_sx_string(
            "style({ bg(primary), box: pad({ x: 2 }) })",
            &env,
            &mut css,
            &mut seen,
        )
        .expect("valid Style spine lowers");
        assert!(classes.contains("s-"), "{classes}");
        assert!(
            css.contains("var(--primary)") || css.contains("background"),
            "{css}"
        );
        assert!(
            css.contains("var(--space-2)") || css.contains("padding"),
            "{css}"
        );
    }

    #[test]
    fn lower_sx_string_rejects_utility_string_product() {
        let mut css = String::new();
        let mut seen = std::collections::HashSet::new();
        let env = std::collections::HashMap::new();
        let result = lower_sx_string("bg: primary px: 2", &env, &mut css, &mut seen);
        assert!(
            result.is_err() && css.is_empty(),
            "utility-string product must error and not emit: {result:?} css={css}"
        );
        assert!(is_residual_utility_sx("bg-primary px-2"));
        assert!(is_residual_utility_sx("bg: primary px: 2"));
    }

    #[test]
    fn lower_sx_string_resolves_style_const_env() {
        let mut env = std::collections::HashMap::new();
        env.insert("filled".into(), crate::parse("style({ bg(primary) })").0);
        let mut css = String::new();
        let mut seen = std::collections::HashSet::new();
        let classes =
            lower_sx_string("filled", &env, &mut css, &mut seen).expect("Style const ref lowers");
        assert!(classes.contains("s-"), "{classes}");
        assert!(
            css.contains("var(--primary)") || css.contains("background"),
            "{css}"
        );
    }

    #[test]
    fn agent_schema_has_properties_and_tokens() {
        let s = agent_schema();
        assert!(s.properties.len() > 20);
        assert!(s.tokens.count > 10);
        assert!(s.aliases.contains_key("color"));
        let j = agent_schema_json();
        assert!(j.contains("\"properties\""));
        assert!(j.contains("\"tokens\""));
    }

    #[test]
    fn value_candidates_dual_resolve() {
        let c = value_candidates("space.2");
        assert!(c.iter().any(|x| x == "2"));
        assert!(c.iter().any(|x| x == "space-2"));
        let c = value_candidates("radius.sm");
        assert!(c.iter().any(|x| x == "sm"));
    }

    #[test]
    fn property_table_documents_closed_set() {
        let t = property_table();
        assert!(t.iter().any(|m| m.name == "bg"));
        assert!(t.iter().any(|m| m.name == "px"));
        assert!(t.iter().any(|m| m.name == "color"));
    }

    #[test]
    fn font_size_alias() {
        let d = lower_decl(&tok("font-size", "xs")).expect("font-size");
        assert_eq!(d[0].0, "font-size");
        assert!(d[0].1.contains("--text-xs"), "{}", d[0].1);
    }

    #[test]
    fn var_ref_lowers() {
        let decl = SxDecl {
            prefixes: Vec::new(),
            prop: "w".into(),
            value: Some(SxValue::VarRef("box-w".into())),
            negative: false,
            bare: false,
            source: "w (--box-w)".into(),
        };
        assert_eq!(decl.value, Some(SxValue::VarRef("box-w".into())));
        let d = lower_decl(&decl).expect("var ref");
        assert_eq!(d[0].1, "var(--box-w)");
    }
}

// debug removed

#[cfg(test)]
mod schema_write {
    #[test]
    fn write_agent_schema_file() {
        let json = super::agent_schema_json();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("sx-schema.gen.json");
        std::fs::write(&path, json).expect("write schema");
        assert!(path.exists());
    }
}
