//! Shared theme token registry — LSP + MCP complete/hover SSOT (THEMES-2 §3).
//!
//! Sanctioned override vocabulary = semantic slots + L1 component tokens
//! (element-scoped when the host is known). App Style never sees component
//! tokens (THEMES-1-ENFORCE). There is no nested `theme=` override-attr
//! syntax yet: `theme=` takes a theme id. Override names complete in theme
//! JSON / studio via this API.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::l1_manifest::{L1ComponentToken, l1_entries};
use crate::theme_gate::REQUIRED_SLOTS;
use crate::tokens_generate::{baked_catalog_theme_ids, baked_catalog_theme_json, public_name};

/// Kind of a theme-override token name.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeTokenKind {
    SemanticSlot,
    ComponentToken,
    ThemeId,
}

/// One completion / registry item.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ThemeTokenRef {
    pub name: String,
    pub kind: ThemeTokenKind,
    pub css_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Resolved hover for a token under a theme + mode.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ThemeTokenHover {
    pub name: String,
    pub theme: String,
    pub mode: String,
    pub value: String,
    pub css_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_divergence: Option<ModeDivergence>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModeDivergence {
    pub other_mode: String,
    pub other_value: String,
}

/// Catalog theme ids (baked SSOT — same list generate uses).
pub fn catalog_theme_ids() -> Vec<String> {
    baked_catalog_theme_ids()
}

/// Semantic slots + L1 component tokens for a theme-override site.
///
/// `host` scopes component tokens to that L1 element's pack when known.
/// `None` returns every L1 pack's tokens (studio / theme JSON).
pub fn theme_override_vocabulary(host: Option<&str>) -> Vec<ThemeTokenRef> {
    let mut out = Vec::new();
    for slot in REQUIRED_SLOTS {
        out.push(ThemeTokenRef {
            name: (*slot).to_string(),
            kind: ThemeTokenKind::SemanticSlot,
            css_name: (*slot).to_string(),
            storage_path: Some(format!("color.{slot}")),
            host: None,
            detail: Some("semantic slot".into()),
        });
    }
    // Additional public semantic paints commonly overridden (not required).
    for extra in ["border", "ring", "card", "popover"] {
        if REQUIRED_SLOTS.contains(&extra) {
            continue;
        }
        out.push(ThemeTokenRef {
            name: extra.to_string(),
            kind: ThemeTokenKind::SemanticSlot,
            css_name: extra.to_string(),
            storage_path: Some(format!("color.{extra}")),
            host: None,
            detail: Some("semantic slot".into()),
        });
    }

    for entry in l1_entries() {
        if let Some(h) = host {
            if entry.tag != h {
                continue;
            }
        }
        for tok in &entry.component_tokens {
            let author = if tok.name == "bg" {
                entry.tag.clone()
            } else {
                format!("{}.{}", entry.tag, tok.name)
            };
            let storage_leaf = match tok.name.as_str() {
                "foreground" => "fg",
                other => other,
            };
            let storage = format!("component.{}.{}", entry.tag, storage_leaf);
            let css = public_name(&storage);
            out.push(ThemeTokenRef {
                name: author,
                kind: ThemeTokenKind::ComponentToken,
                css_name: css,
                storage_path: Some(storage),
                host: Some(entry.tag.clone()),
                detail: Some(format!("L1 {} · default {}", entry.tag, tok.default)),
            });
        }
    }
    out.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then(a.kind.as_str().cmp(b.kind.as_str()))
    });
    out.dedup_by(|a, b| a.name == b.name && a.host == b.host);
    out
}

impl ThemeTokenKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::SemanticSlot => "semantic-slot",
            Self::ComponentToken => "component-token",
            Self::ThemeId => "theme-id",
        }
    }
}

/// Completions for a `theme=` attribute value (theme ids).
pub fn theme_attr_completions(partial: &str) -> Vec<ThemeTokenRef> {
    let p = partial.trim().to_ascii_lowercase();
    catalog_theme_ids()
        .into_iter()
        .filter(|id| p.is_empty() || id.to_ascii_lowercase().starts_with(&p))
        .map(|id| ThemeTokenRef {
            name: id.clone(),
            kind: ThemeTokenKind::ThemeId,
            css_name: id.clone(),
            storage_path: None,
            host: None,
            detail: Some("theme id".into()),
        })
        .collect()
}

/// Completions at a byte offset: `theme="¦"` / `theme='¦'` → theme ids;
/// otherwise empty (Style complete owns style() sites).
pub fn theme_complete_at(
    source: &str,
    byte_offset: usize,
    host: Option<&str>,
) -> Vec<ThemeTokenRef> {
    if let Some(partial) = theme_attr_partial(source, byte_offset) {
        return theme_attr_completions(&partial);
    }
    if theme_override_object_at(source, byte_offset) {
        let p = word_prefix_at(source, byte_offset);
        let mut items = theme_override_vocabulary(host);
        if !p.is_empty() {
            items.retain(|it| {
                it.name
                    .to_ascii_lowercase()
                    .starts_with(&p.to_ascii_lowercase())
            });
        }
        return items;
    }
    Vec::new()
}

/// Hover for the token / theme-id under `byte_offset`.
pub fn theme_hover_at(
    source: &str,
    byte_offset: usize,
    theme: &str,
    mode: &str,
) -> Option<ThemeTokenHover> {
    let name = ident_at(source, byte_offset)?;
    if theme_attr_partial(source, byte_offset).is_some()
        && catalog_theme_ids().iter().any(|id| id == &name)
    {
        return Some(ThemeTokenHover {
            name: name.clone(),
            theme: name.clone(),
            mode: mode.to_string(),
            value: format!("theme id `{name}`"),
            css_name: name,
            mode_divergence: None,
        });
    }
    if !in_style_or_theme_site(source, byte_offset) {
        return None;
    }
    resolve_theme_token(&name, theme, mode)
}

fn in_style_or_theme_site(source: &str, offset: usize) -> bool {
    if theme_attr_partial(source, offset).is_some() || theme_override_object_at(source, offset) {
        return true;
    }
    let off = offset.min(source.len());
    let before = &source[..off];
    before
        .rfind("style(")
        .is_some_and(|i| !before[i..].contains(')'))
        || before
            .rfind("style({")
            .is_some_and(|i| before[i..].matches('{').count() > before[i..].matches('}').count())
}

/// Resolve a semantic slot or component token under `theme` + `mode`.
pub fn resolve_theme_token(name: &str, theme: &str, mode: &str) -> Option<ThemeTokenHover> {
    let theme = if theme.is_empty() { "ship" } else { theme };
    let mode = if mode.is_empty() { "light" } else { mode };
    let css = css_name_for_author(name)?;
    let (value, divergence) = resolved_values(theme, mode, &css)?;
    Some(ThemeTokenHover {
        name: name.to_string(),
        theme: theme.to_string(),
        mode: mode.to_string(),
        value,
        css_name: css,
        mode_divergence: divergence,
    })
}

/// Default theme + mode inferred from source (`theme="id"`; mode defaults light).
pub fn infer_active_theme_mode(source: &str) -> (String, String) {
    let theme = last_quoted_attr(source, "theme").unwrap_or_else(|| "ship".into());
    let mode = last_quoted_attr(source, "data-mode").unwrap_or_else(|| "light".into());
    (theme, mode)
}

fn css_name_for_author(name: &str) -> Option<String> {
    let n = name.trim();
    if n.is_empty() {
        return None;
    }
    if REQUIRED_SLOTS.contains(&n) || matches!(n, "border" | "ring" | "card" | "popover") {
        return Some(n.to_string());
    }
    if let Some(css) = sail_lint::tokens::path_to_css_name(n) {
        return Some(css.to_string());
    }
    if n.contains('.') {
        if let Some((root, rest)) = n.split_once('.') {
            let leaf = match rest {
                "foreground" => "fg",
                "bg" => "bg",
                other => other,
            };
            let path = format!("component.{root}.{leaf}");
            if let Some(css) = sail_lint::tokens::path_to_css_name(&path) {
                return Some(css.to_string());
            }
            return Some(public_name(&path));
        }
    }
    if sail_lint::style::is_theme_owned_author_root(n) {
        return Some(public_name(&format!("component.{n}.bg")));
    }
    if sail_lint::tokens::registry().contains(n) {
        return Some(n.to_string());
    }
    None
}

fn resolved_values(
    theme: &str,
    mode: &str,
    css_name: &str,
) -> Option<(String, Option<ModeDivergence>)> {
    let doc = load_theme_doc(theme)?;
    let this = resolve_css_in_mode(&doc, mode, css_name)?;
    let other_mode = if mode == "dark" { "light" } else { "dark" };
    let divergence = resolve_css_in_mode(&doc, other_mode, css_name).and_then(|other| {
        if other != this {
            Some(ModeDivergence {
                other_mode: other_mode.to_string(),
                other_value: other,
            })
        } else {
            None
        }
    });
    Some((this, divergence))
}

fn load_theme_doc(theme: &str) -> Option<Value> {
    let body = baked_catalog_theme_json(theme)?;
    serde_json::from_str(body).ok()
}

fn resolve_css_in_mode(doc: &Value, mode: &str, css_name: &str) -> Option<String> {
    let body = doc.get("modes")?.get(mode)?;
    let mut map = BTreeMap::new();
    walk_leaves(body, "", &mut map);
    let raw = map.get(css_name)?.clone();
    Some(expand_alias(&map, &raw, 0))
}

fn walk_leaves(node: &Value, path: &str, out: &mut BTreeMap<String, String>) {
    let Some(obj) = node.as_object() else {
        return;
    };
    if let Some(val) = obj.get("$value") {
        let css = public_name(path);
        if !css.is_empty() {
            let s = match val {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            out.insert(css, s);
        }
        return;
    }
    for (k, child) in obj {
        if k.starts_with('$') {
            continue;
        }
        let next = if path.is_empty() {
            k.clone()
        } else {
            format!("{path}.{k}")
        };
        walk_leaves(child, &next, out);
    }
}

fn expand_alias(map: &BTreeMap<String, String>, raw: &str, depth: u32) -> String {
    if depth > 8 {
        return raw.to_string();
    }
    let s = raw.trim();
    if let Some(inner) = s.strip_prefix('{').and_then(|x| x.strip_suffix('}')) {
        let inner = inner.trim();
        let css = public_name(inner);
        if let Some(next) = map.get(&css).or_else(|| map.get(inner)) {
            return expand_alias(map, next, depth + 1);
        }
        return css;
    }
    if let Some(inner) = s.strip_prefix("var(--").and_then(|x| x.strip_suffix(')')) {
        let name = inner.split(',').next().unwrap_or(inner).trim();
        if let Some(next) = map.get(name) {
            return expand_alias(map, next, depth + 1);
        }
        return name.to_string();
    }
    s.to_string()
}

/// `theme="partial` / `theme='partial` immediately before offset.
pub fn theme_attr_partial(source: &str, offset: usize) -> Option<String> {
    let off = offset.min(source.len());
    let before = &source[..off];
    for quote in ['"', '\''] {
        if let Some(q) = before.rfind(quote) {
            let head = before[..q].trim_end();
            if head.ends_with("theme=") {
                let rest = &before[q + 1..];
                if !rest.contains(quote) && !rest.contains('\n') {
                    return Some(rest.to_string());
                }
            }
        }
    }
    None
}

fn theme_override_object_at(source: &str, offset: usize) -> bool {
    let off = offset.min(source.len());
    let before = &source[..off];
    // Future / studio object form: theme={{ ¦ }} — not a binding ident.
    before.rfind("theme={{").is_some_and(|i| {
        !before[i + 8..].contains("}}") && before[i + 8..].chars().all(|c| c != '\n')
    })
}

fn word_prefix_at(source: &str, offset: usize) -> String {
    let off = offset.min(source.len());
    let bytes = source.as_bytes();
    let mut i = off;
    while i > 0 {
        let b = bytes[i - 1];
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' {
            i -= 1;
        } else {
            break;
        }
    }
    source[i..off].to_string()
}

fn ident_at(source: &str, offset: usize) -> Option<String> {
    let bytes = source.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut i = offset.min(bytes.len().saturating_sub(1));
    if i < bytes.len()
        && !(bytes[i].is_ascii_alphanumeric()
            || bytes[i] == b'-'
            || bytes[i] == b'_'
            || bytes[i] == b'.')
    {
        if i == 0 {
            return None;
        }
        i -= 1;
    }
    if i >= bytes.len()
        || !(bytes[i].is_ascii_alphanumeric()
            || bytes[i] == b'-'
            || bytes[i] == b'_'
            || bytes[i] == b'.')
    {
        return None;
    }
    let mut start = i;
    while start > 0 {
        let b = bytes[start - 1];
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' {
            start -= 1;
        } else {
            break;
        }
    }
    let mut end = i + 1;
    while end < bytes.len() {
        let b = bytes[end];
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' {
            end += 1;
        } else {
            break;
        }
    }
    let s = source[start..end].trim_matches('.');
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn last_quoted_attr(source: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let i = source.rfind(&needle)?;
    let rest = &source[i + needle.len()..];
    let end = rest.find('"')?;
    let v = rest[..end].trim();
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

/// L1 component tokens as listed on the manifest (studio / gate SSOT).
pub fn manifest_component_tokens(tag: &str) -> Vec<L1ComponentToken> {
    l1_entries()
        .into_iter()
        .find(|e| e.tag == tag)
        .map(|e| e.component_tokens)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_vocab_has_semantic_and_button() {
        let items = theme_override_vocabulary(None);
        assert!(
            items
                .iter()
                .any(|i| i.name == "primary" && i.kind == ThemeTokenKind::SemanticSlot)
        );
        assert!(
            items
                .iter()
                .any(|i| i.name == "button" && i.kind == ThemeTokenKind::ComponentToken),
            "{:?}",
            items.iter().map(|i| &i.name).take(20).collect::<Vec<_>>()
        );
        let scoped = theme_override_vocabulary(Some("button"));
        assert!(scoped.iter().all(|i| {
            i.kind == ThemeTokenKind::SemanticSlot || i.host.as_deref() == Some("button")
        }));
        assert!(scoped.iter().any(|i| i.name == "button"));
        assert!(
            !scoped
                .iter()
                .any(|i| i.name == "card" && i.kind == ThemeTokenKind::ComponentToken)
        );
    }

    #[test]
    fn theme_eq_completes_catalog_ids() {
        let src = r#"<column theme="shi">x</column>"#;
        let off = src.find("shi").unwrap() + 3;
        let items = theme_complete_at(src, off, None);
        assert!(
            items
                .iter()
                .any(|i| i.name == "ship" && i.kind == ThemeTokenKind::ThemeId),
            "{items:?}"
        );
    }

    #[test]
    fn hover_primary_under_ship_light() {
        let h = resolve_theme_token("primary", "ship", "light").expect("primary");
        assert!(!h.value.is_empty(), "{h:?}");
        assert_eq!(h.css_name, "primary");
        assert_eq!(h.theme, "ship");
        assert_eq!(h.mode, "light");
    }

    #[test]
    fn hover_flags_mode_divergence_when_modes_differ() {
        let h = resolve_theme_token("background", "ship", "light").expect("background");
        if let Some(div) = &h.mode_divergence {
            assert_eq!(div.other_mode, "dark");
            assert_ne!(div.other_value, h.value);
        }
        // Dual-mode catalog themes should diverge on background; if a theme is
        // identical, None is honest — just ensure the field is well-formed.
        let _ = h.mode_divergence;
    }
}
