//! **INTERNAL leaf IR** for Style paint-op emit — not an author surface.
//!
//! Product paint is Style (`style.…` chains, `match` / `when` / `compound`).
//! Utility-string sx is a hard residual error at parse + lint. This module
//! remains so Style setters can lower atoms → CSS via the closed property
//! table + token registry.

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
    /// Negation (`-m: 4` / legacy `-m-4`).
    pub negative: bool,
    /// True when this was a bare atom with no value.
    pub bare: bool,
    /// Source fragment (for diagnostics).
    pub source: String,
}

/// Parsed sx attribute body.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SxProgram {
    pub decls: Vec<SxDecl>,
    /// Bare `group` ancestor marker present.
    pub group: bool,
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

/// Refused heads (`gap`, `flex`, …) so structured `gap: 2` parses into IR and
/// surfaces the same "use layout element" diagnostic as legacy `gap-2`.
fn is_refused_head(name: &str) -> bool {
    let n = name.strip_prefix('-').unwrap_or(name);
    style_vocab::refusals().iter().any(|(k, _)| k == n)
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

fn parse_value_token(raw: &str) -> Result<SxValue, SxError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(SxError::EmptyValue {
            prop: String::new(),
            source: raw.to_string(),
        });
    }
    // raw: escape
    if let Some(rest) = raw.strip_prefix("raw:") {
        if rest.is_empty() {
            return Err(SxError::EmptyValue {
                prop: "raw".into(),
                source: raw.to_string(),
            });
        }
        return Ok(SxValue::Raw(rest.to_string()));
    }
    // (--name) var ref
    if let Some(inner) = strip_var_ref(raw) {
        return Ok(SxValue::VarRef(inner.to_string()));
    }
    // Naked raw color → error (must use raw:)
    if looks_like_raw_color(raw) {
        return Err(SxError::RawWithoutEscape {
            value: raw.to_string(),
            source: raw.to_string(),
        });
    }
    // Naked raw length → error
    if looks_like_raw_length(raw) {
        return Err(SxError::RawWithoutEscape {
            value: raw.to_string(),
            source: raw.to_string(),
        });
    }
    Ok(SxValue::Token(raw.to_string()))
}

fn strip_var_ref(value: &str) -> Option<&str> {
    if !value.starts_with("(--") || !value.ends_with(')') {
        return None;
    }
    let inner = &value[3..value.len() - 1];
    if inner.is_empty()
        || !inner
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }
    Some(inner)
}

// ─── Parser (dual: structured + legacy) ──────────────────────────────────────

/// Parse an sx attribute body into IR. Accepts both structured (`bg: primary`)
/// and legacy atom (`bg-primary`) forms in the same string.
///
/// On hard token errors (raw color without `raw:`, empty value), valid decls
/// still appear in the program; errors are returned alongside so lower can
/// emit the good half and lint can report the bad half.
pub fn parse(input: &str) -> Result<SxProgram, Vec<SxError>> {
    let (prog, errors) = parse_lenient(input);
    if errors.is_empty() {
        Ok(prog)
    } else {
        Err(errors)
    }
}

/// Always returns a program plus any per-token errors.
pub fn parse_lenient(input: &str) -> (SxProgram, Vec<SxError>) {
    let mut prog = SxProgram::default();
    let mut errors: Vec<SxError> = Vec::new();
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0usize;

    while i < tokens.len() {
        let t = tokens[i];
        if t == "group" {
            prog.group = true;
            i += 1;
            continue;
        }

        match try_structured(&tokens, i) {
            StructuredTry::Ok { decl, next } => {
                prog.decls.push(decl);
                i = next;
            }
            StructuredTry::Err { err, next } => {
                errors.push(err);
                i = next;
            }
            StructuredTry::NotStructured => {
                // Legacy atom form (incl. prefix:atom).
                match parse_utility_atom(t) {
                    Ok(decl) => prog.decls.push(decl),
                    Err(e) => errors.push(e),
                }
                i += 1;
            }
        }
    }

    (prog, errors)
}

enum StructuredTry {
    Ok { decl: SxDecl, next: usize },
    Err { err: SxError, next: usize },
    NotStructured,
}

/// Attempt to read a structured `prop: value` (with optional prefixes) starting
/// at `tokens[i]`.
fn try_structured(tokens: &[&str], i: usize) -> StructuredTry {
    let t = tokens[i];
    // Need a colon somewhere after optional prefixes for structured form.
    if !t.contains(':') {
        return StructuredTry::NotStructured;
    }

    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() < 2 {
        return StructuredTry::NotStructured;
    }

    // Consume leading closed prefixes.
    let mut pi = 0usize;
    let mut prefixes: Vec<String> = Vec::new();
    while pi + 1 < parts.len() && sx_rules::is_prefix(parts[pi]) {
        // Guard: a property name that is also somehow a prefix must not be
        // swallowed. None of our props are prefixes today.
        prefixes.push(parts[pi].to_string());
        pi += 1;
    }

    let rest = &parts[pi..];
    // Structured requires prop:…  (at least two segments after prefixes).
    // Legacy `hover:bg-primary` leaves rest = ["bg-primary"] → not structured.
    if rest.len() < 2 {
        return StructuredTry::NotStructured;
    }

    let mut prop = rest[0];
    if prop.is_empty() {
        return StructuredTry::NotStructured;
    }

    let mut negative = false;
    if let Some(p) = prop.strip_prefix('-') {
        negative = true;
        prop = p;
    }

    // Property must be closed (or a known refusal head). Otherwise fall through
    // to legacy so `hover:bg-primary` stays an atom, not a failed structure.
    if !is_property(prop) && !is_refused_head(prop) {
        return StructuredTry::NotStructured;
    }

    // Value: remaining colon segments joined, or next whitespace token if empty.
    let mut value_raw = rest[1..].join(":");
    let mut next = i + 1;
    if value_raw.is_empty() {
        if i + 1 >= tokens.len() {
            return StructuredTry::Err {
                err: SxError::EmptyValue {
                    prop: prop.to_string(),
                    source: t.to_string(),
                },
                next: i + 1,
            };
        }
        // Next token is the value (and must not look like another prop: start
        // unless it's raw: / a plain token).
        value_raw = tokens[i + 1].to_string();
        next = i + 2;
    }

    let source = if next == i + 2 {
        format!("{t} {value_raw}")
    } else {
        t.to_string()
    };

    // Unknown prefix already filtered by is_prefix walk; any non-prefix junk
    // before prop would have stopped the walk and left a non-property head.
    let value = match parse_value_token(&value_raw) {
        Ok(v) => v,
        Err(mut e) => {
            // Fill prop into empty-value errors.
            if let SxError::EmptyValue {
                prop: ref mut p,
                source: ref mut s,
            } = e
            {
                if p.is_empty() {
                    *p = prop.to_string();
                }
                *s = source.clone();
            } else if let SxError::RawWithoutEscape {
                source: ref mut s, ..
            } = e
            {
                *s = source.clone();
            }
            return StructuredTry::Err { err: e, next };
        }
    };

    StructuredTry::Ok {
        decl: SxDecl {
            prefixes,
            prop: prop.to_string(),
            value: Some(value),
            negative,
            bare: false,
            source,
        },
        next,
    }
}

fn parse_utility_atom(pair: &str) -> Result<SxDecl, SxError> {
    // Split closed prefix chain, remainder is atom (may contain `:` only if
    // value was something weird — legacy atoms use `-`).
    let parts: Vec<&str> = pair.split(':').collect();
    let mut i = 0usize;
    while i < parts.len().saturating_sub(1) && sx_rules::is_prefix(parts[i]) {
        i += 1;
    }
    if i > 0 && i >= parts.len() {
        return Err(SxError::UnknownAtom {
            atom: pair.to_string(),
            source: pair.to_string(),
        });
    }
    // If we stopped mid-chain on an unknown prefix, flag it.
    if i < parts.len().saturating_sub(1) && parts[i].contains('-') == false {
        // e.g. print:bg-primary — print is not a prefix and not consumed.
        // Fall through: whole remainder is atom (will fail resolve).
    }
    let prefixes: Vec<String> = parts[..i].iter().map(|s| (*s).to_string()).collect();
    // Reject unknown prefix: if something before the atom is not a known
    // prefix and we didn't consume it… actually we only consume known ones.
    // `print:bg-primary` → prefixes=[], atom=`print:bg-primary` via join?
    // Wait: i=0 because print is not prefix; atom = full pair with colons.
    let atom = if i == 0 {
        pair.to_string()
    } else {
        parts[i..].join(":")
    };

    if atom.is_empty() {
        return Err(SxError::UnknownAtom {
            atom: pair.to_string(),
            source: pair.to_string(),
        });
    }

    // Unknown prefix detection: pair has `:` but first segment isn't prefix
    // and result looks like `unknown:atom`.
    if i == 0 && pair.contains(':') {
        if let Some((head, _)) = pair.split_once(':') {
            if !head.is_empty() && !sx_rules::is_prefix(head) && !head.starts_with("state-") {
                // Could be structured that failed property check — report unknown prefix
                // only when the head isn't a property either.
                if !is_property(head) {
                    return Err(SxError::UnknownPrefix {
                        prefix: head.to_string(),
                        source: pair.to_string(),
                    });
                }
            }
        }
    }

    let (negative, body) = match atom.strip_prefix('-') {
        Some(rest) if !rest.is_empty() => (true, rest),
        _ => (false, atom.as_str()),
    };

    // Bare atom (no value dash) or name-value.
    // We don't fully re-parse name/value here — lower uses the atom string.
    let (prop, value) = split_legacy_name_value(body);
    let bare = value.is_none();

    Ok(SxDecl {
        prefixes,
        prop,
        value,
        negative,
        bare,
        source: pair.to_string(),
    })
}

/// Best-effort split of a legacy body into prop + value for IR display.
fn split_legacy_name_value(body: &str) -> (String, Option<SxValue>) {
    // Longest vocab name match (same as style_vocab::resolve).
    let names = style_vocab::names();
    let mut best: Option<(&str, &str)> = None;
    for name in names.iter() {
        if let Some(rest) = body.strip_prefix(name.as_str()) {
            if let Some(value) = rest.strip_prefix('-') {
                if best.map(|(b, _)| name.len() > b.len()).unwrap_or(true) {
                    best = Some((name.as_str(), value));
                }
            }
        }
    }
    if let Some((name, value)) = best {
        let v = if let Some(inner) = strip_var_ref(value) {
            SxValue::VarRef(inner.to_string())
        } else {
            SxValue::Token(value.to_string())
        };
        return (name.to_string(), Some(v));
    }
    // Bare atom
    (body.to_string(), None)
}

// ─── Lower ───────────────────────────────────────────────────────────────────

/// Build the legacy atom string for a structured/legacy decl (without prefixes).
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

/// Validate one decl against vocab / raw rules. Returns Ok(()) or a soft error
/// list (unknown prop already caught at parse for structured; legacy resolves here).
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

/// Validate a whole program (parse must have succeeded).
pub fn validate_program(prog: &SxProgram) -> Vec<SxError> {
    let mut errs = Vec::new();
    for d in &prog.decls {
        if let Err(e) = validate_decl(d) {
            errs.push(e);
        }
    }
    errs
}

/// Parse + validate in one shot. Used by lint.
pub fn check_sx_string(input: &str) -> Vec<SxError> {
    match parse(input) {
        Ok(prog) => validate_program(&prog),
        Err(errs) => {
            // Also try to validate any partial… parse returns only errors.
            errs
        }
    }
}

// ─── Emit helpers (class name + rule body for preprocess) ────────────────────

/// Sanitized atomic class name for a decl (matches legacy lower_sx_value shape).
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

/// Prefix/media wiring for one decl — same model as legacy preprocess.
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
        form: "string — `prop: value` (structured) or legacy `prop-value` atoms; same IR",
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

    #[test]
    fn parse_structured_basic() {
        let p = parse("bg: primary px: 2").unwrap();
        assert_eq!(p.decls.len(), 2);
        assert_eq!(p.decls[0].prop, "bg");
        assert_eq!(p.decls[0].value, Some(SxValue::Token("primary".into())));
        assert_eq!(p.decls[1].prop, "px");
        assert_eq!(p.decls[1].value, Some(SxValue::Token("2".into())));
    }

    #[test]
    fn parse_structured_attached_value() {
        let p = parse("bg:primary px:2").unwrap();
        assert_eq!(p.decls.len(), 2);
        assert_eq!(p.decls[0].value, Some(SxValue::Token("primary".into())));
    }

    #[test]
    fn parse_structured_with_prefix() {
        let p = parse("hover:bg: muted focus:bg: primary").unwrap();
        assert_eq!(p.decls[0].prefixes, vec!["hover"]);
        assert_eq!(p.decls[0].prop, "bg");
        assert_eq!(p.decls[0].value, Some(SxValue::Token("muted".into())));
        assert_eq!(p.decls[1].prefixes, vec!["focus"]);
    }

    #[test]
    fn parse_dotted_token_paths() {
        let p = parse("px: space.2 rounded: radius.sm bg: primary").unwrap();
        assert_eq!(p.decls[0].value, Some(SxValue::Token("space.2".into())));
        assert_eq!(p.decls[1].value, Some(SxValue::Token("radius.sm".into())));
    }

    #[test]
    fn parse_utility_atoms() {
        let p = parse("bg-primary px-2 hover:bg-muted").unwrap();
        assert_eq!(p.decls.len(), 3);
        assert_eq!(p.decls[0].prop, "bg");
        assert_eq!(p.decls[2].prefixes, vec!["hover"]);
    }

    #[test]
    fn dual_parse_mixed() {
        let p = parse("bg: primary px-2 rounded: sm").unwrap();
        assert_eq!(p.decls.len(), 3);
        assert_eq!(p.decls[0].prop, "bg");
        assert_eq!(p.decls[1].prop, "px");
        assert_eq!(p.decls[2].prop, "rounded");
    }

    #[test]
    fn parse_color_alias() {
        let p = parse("color: primary").unwrap();
        assert_eq!(p.decls[0].prop, "color");
        let atom = decl_atom(&p.decls[0]);
        assert_eq!(atom, "text-primary");
    }

    #[test]
    fn parse_raw_escape() {
        let p = parse("bg: raw:#3366ff").unwrap();
        assert_eq!(p.decls[0].value, Some(SxValue::Raw("#3366ff".into())));
    }

    #[test]
    fn reject_raw_color_without_escape() {
        let err = parse("bg: #3366ff").unwrap_err();
        assert!(matches!(err[0], SxError::RawWithoutEscape { .. }));
        assert!(err[0].message().contains("raw:"));
    }

    #[test]
    fn reject_raw_length_without_escape() {
        let err = parse("p: 16px").unwrap_err();
        assert!(matches!(err[0], SxError::RawWithoutEscape { .. }));
    }

    #[test]
    fn reject_unknown_prop() {
        // Structured with unknown prop falls through; `foo: bar` has head `foo`
        // which is not a prefix → unknown prefix error from legacy path.
        let err = parse("notaprop: primary").unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn lower_structured_px_space() {
        let p = parse("px: 2").unwrap();
        let d = lower_decl(&p.decls[0]).expect("lower");
        assert_eq!(d, vec![("padding-inline".into(), "var(--space-2)".into())]);
    }

    #[test]
    fn lower_structured_path_space_dot() {
        let p = parse("px: space.2").unwrap();
        let d = lower_decl(&p.decls[0]).expect("lower space.2");
        assert_eq!(d, vec![("padding-inline".into(), "var(--space-2)".into())]);
    }

    #[test]
    fn lower_structured_bg() {
        let p = parse("bg: primary").unwrap();
        let d = lower_decl(&p.decls[0]).expect("lower bg");
        assert_eq!(
            d,
            vec![("background-color".into(), "var(--primary)".into())]
        );
    }

    #[test]
    fn lower_legacy_still_works() {
        let p = parse("bg-primary p-4").unwrap();
        assert!(lower_decl(&p.decls[0]).is_some());
        assert!(lower_decl(&p.decls[1]).is_some());
    }

    #[test]
    fn lower_rounded_path() {
        let p = parse("rounded: radius.sm").unwrap();
        let d = lower_decl(&p.decls[0]).expect("rounded");
        assert_eq!(d[0].0, "border-radius");
        assert_eq!(d[0].1, "var(--radius-sm)");
    }

    #[test]
    fn lower_raw_emits_css_literal() {
        let p = parse("bg: raw:#3366ff").unwrap();
        let d = lower_decl(&p.decls[0]).expect("raw");
        assert_eq!(d, vec![("background-color".into(), "#3366ff".into())]);
    }

    #[test]
    fn validate_rejects_bad_token() {
        let p = parse("bg: not-a-real-token-zz").unwrap();
        let errs = validate_program(&p);
        assert!(!errs.is_empty());
    }

    #[test]
    fn bare_hidden_legacy() {
        let p = parse("hidden").unwrap();
        assert!(p.decls[0].bare);
        let d = lower_decl(&p.decls[0]).unwrap();
        assert_eq!(d, vec![("display".into(), "none".into())]);
    }

    #[test]
    fn group_marker() {
        let p = parse("group hover:bg: muted").unwrap();
        assert!(p.group);
        assert_eq!(p.decls[0].prefixes, vec!["hover"]);
    }

    #[test]
    fn class_name_stable() {
        let p = parse("px: 2").unwrap();
        assert_eq!(class_name(&p.decls[0]), "s-px2");
        let p = parse("hover:bg: primary").unwrap();
        assert_eq!(class_name(&p.decls[0]), "s-hover-bgprimary");
    }

    #[test]
    fn lower_sx_string_emits_style_chain_css() {
        // Product path: Style value language only.
        let mut css = String::new();
        let mut seen = std::collections::HashSet::new();
        let env = std::collections::HashMap::new();
        let classes = lower_sx_string(
            "style({ paint: bg(primary), box: padX(2) })",
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
        // Residual product forms must not emit dual-path CSS.
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
        env.insert(
            "filled".into(),
            crate::parse("style({ paint: bg(primary) })").0,
        );
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
    fn check_sx_string_reject_raw() {
        let errs = check_sx_string("bg: #fff p: 4");
        assert!(
            errs.iter()
                .any(|e| matches!(e, SxError::RawWithoutEscape { .. }))
        );
    }

    #[test]
    fn font_size_alias() {
        let p = parse("font-size: xs").unwrap();
        let d = lower_decl(&p.decls[0]).expect("font-size");
        assert_eq!(d[0].0, "font-size");
        assert!(d[0].1.contains("--text-xs"), "{}", d[0].1);
    }

    #[test]
    fn var_ref_structured() {
        let p = parse("w: (--box-w)").unwrap();
        assert_eq!(p.decls[0].value, Some(SxValue::VarRef("box-w".into())));
        let d = lower_decl(&p.decls[0]).expect("var ref");
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
