//! `sui-layout` — the language-level layout elements.
//!
//! `row`, `column`, `grid`, `center`, `stack` are compile-time keywords, not
//! web components. The preprocess pass lowers each to a plain `<div>` + atomic
//! classes, and injects the deduped CSS those classes need into the component's
//! `<style>` — the same "class + injected stylesheet" mechanism `sx` uses. No
//! shadow DOM, no custom-element upgrade: layout disappears into the language.
//!
//! Faithful successor to the `<sui-stack>` / `<sui-cluster>` / `<sui-grid>` /
//! `<sui-center>` primitives — the hard-won opinions ride along in the lowering:
//!   - `min-width: 0` on the box AND its direct children, so a layout nested in
//!     a flex/grid parent can shrink instead of forcing overflow (the structural
//!     fix those primitives shipped, now free at the language level).
//!   - gap is a spacing token (`gap="4"` → `var(--space-4)`), never a raw value.
//!   - `grid` keeps the responsive `auto-fill minmax` + per-child span opt-in.
//!
//! Every axis value comes from a CLOSED enum; off-enum (`gap="99"`,
//! `align="middle"`) is a `Severity::Error`, so a typo can't silently degrade to
//! an unstyled div — the same no-silent-failure contract as `sx`.
//!
//! Naming: `column` = vertical flow (was `sui-stack`), `row` = horizontal flow
//! that wraps (was `sui-cluster`), `grid`/`center` unchanged, and `stack` is the
//! z-axis overlay (children layered) — the one genuinely new box.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-layout";

/// The layout keywords the preprocess lowers to `<div>`.
pub const TAGS: &[&str] = &["row", "column", "grid", "center", "stack"];

pub fn is_layout_tag(tag: &str) -> bool {
    TAGS.contains(&tag)
}

/// Which axis props each tag understands. Used by both the lowering (to know
/// what to read) and the lint (to reject props that don't apply to a tag).
pub fn tag_props(tag: &str) -> &'static [&'static str] {
    match tag {
        "column" => &["gap", "align"],
        "row" => &["gap", "align", "justify"],
        "grid" => &["gap", "min", "flow"],
        "center" => &["gap"],
        "stack" => &["align", "justify"],
        _ => &[],
    }
}

/// The base structural class for a tag (`.sui-col` etc.). Its CSS is
/// direction/display only — every tunable (gap/align/justify) rides an explicit
/// atomic class so exactly one rule owns each property (no specificity games).
fn base_class(tag: &str) -> &'static str {
    match tag {
        "row" => "sui-row",
        "column" => "sui-col",
        "grid" => "sui-grid",
        "center" => "sui-center",
        "stack" => "sui-stackz",
        _ => "",
    }
}

/// The CSS rule(s) for a base class — structure + the `min-width:0` child guard.
fn base_css(tag: &str) -> Vec<String> {
    match tag {
        "column" => vec![
            ".sui-col{display:flex;flex-direction:column;min-width:0}".into(),
            ".sui-col>*{min-width:0}".into(),
        ],
        "row" => vec![
            ".sui-row{display:flex;flex-wrap:wrap;min-width:0}".into(),
            ".sui-row>*{min-width:0}".into(),
        ],
        "grid" => vec![
            ".sui-grid{display:grid;min-width:0}".into(),
            ".sui-grid>*{grid-column:span 1;grid-row:span 1;min-width:0}".into(),
            ".sui-grid>*[data-col-span='2']{grid-column:span 2}".into(),
            ".sui-grid>*[data-col-span='3']{grid-column:span 3}".into(),
            ".sui-grid>*[data-col-span='4']{grid-column:span 4}".into(),
            ".sui-grid>*[data-row-span='2']{grid-row:span 2}".into(),
            ".sui-grid>*[data-row-span='3']{grid-row:span 3}".into(),
            ".sui-grid>*[data-row-span='4']{grid-row:span 4}".into(),
            "@media(max-width:720px){.sui-grid>*[data-col-span]{grid-column:span 1}.sui-grid>*[data-row-span]{grid-row:span 1}}".into(),
        ],
        "center" => vec![
            ".sui-center{display:grid;place-items:center;place-content:center;width:100%}".into(),
        ],
        "stack" => vec![
            ".sui-stackz{display:grid}".into(),
            ".sui-stackz>*{grid-area:1/1}".into(),
        ],
        _ => vec![],
    }
}

/// Default value for an axis prop on a tag (mirrors the old primitives' defaults).
fn default_for(tag: &str, prop: &str) -> Option<&'static str> {
    Some(match (tag, prop) {
        ("column", "gap") => "4",
        ("row", "gap") => "3",
        ("grid", "gap") => "3",
        ("center", "gap") => "0",
        ("column", "align") => "stretch",
        ("row", "align") => "center",
        ("stack", "align") => "stretch",
        ("row", "justify") | ("stack", "justify") => "start",
        ("grid", "min") => "md",
        ("grid", "flow") => "dense",
        _ => return None,
    })
}

/// Valid values for an axis prop. `None` = the prop is free-form-invalid here.
fn allowed(prop: &str) -> &'static [&'static str] {
    match prop {
        "gap" => &["0", "1", "2", "3", "4", "5", "6", "7", "8"],
        "align" => &["start", "center", "end", "stretch", "baseline"],
        "justify" => &["start", "center", "end", "between", "around"],
        "min" => &["xs", "sm", "md", "lg"],
        "flow" => &["auto", "dense"],
        _ => &[],
    }
}

/// The atomic class + CSS rule for one resolved `prop:value`. `None` if the
/// value is off-enum (the lint reports it; the lowering just skips it).
fn modifier(prop: &str, value: &str) -> Option<(String, String)> {
    if !allowed(prop).contains(&value) {
        return None;
    }
    Some(match prop {
        "gap" => {
            let v = if value == "0" { "0".to_string() } else { format!("var(--space-{value})") };
            (format!("sui-g-{value}"), format!(".sui-g-{value}{{gap:{v}}}"))
        }
        "align" => {
            let v = match value {
                "start" => "flex-start",
                "end" => "flex-end",
                other => other, // center | stretch | baseline
            };
            (format!("sui-ai-{value}"), format!(".sui-ai-{value}{{align-items:{v}}}"))
        }
        "justify" => {
            let v = match value {
                "start" => "flex-start",
                "end" => "flex-end",
                "between" => "space-between",
                "around" => "space-around",
                other => other, // center
            };
            (format!("sui-jc-{value}"), format!(".sui-jc-{value}{{justify-content:{v}}}"))
        }
        "min" => {
            let rem = match value {
                "xs" => "16rem",
                "sm" => "18rem",
                "lg" => "24rem",
                _ => "20rem", // md
            };
            (
                format!("sui-gm-{value}"),
                format!(".sui-gm-{value}{{grid-template-columns:repeat(auto-fill,minmax({rem},1fr))}}"),
            )
        }
        "flow" => (format!("sui-gf-{value}"), format!(".sui-gf-{value}{{grid-auto-flow:{value}}}")),
        _ => return None,
    })
}

/// The base structural class + CSS for a tag (no axis modifiers). The div always
/// gets this; axes ride on top via [`lower_axis`].
pub fn base(tag: &str) -> Option<(String, Vec<String>)> {
    if !is_layout_tag(tag) {
        return None;
    }
    Some((base_class(tag).to_string(), base_css(tag)))
}

/// The atomic-class PREFIX for an axis (`align` → `sui-ai-`), so a dynamic value
/// can lower to `'<prefix>' + (expr)`.
fn axis_prefix(prop: &str) -> Option<&'static str> {
    Some(match prop {
        "gap" => "sui-g-",
        "align" => "sui-ai-",
        "justify" => "sui-jc-",
        "min" => "sui-gm-",
        "flow" => "sui-gf-",
        _ => return None,
    })
}

/// One resolved axis for the lowering.
pub enum AxisLower {
    /// A concrete class + its one CSS rule.
    Static { class: String, rule: String },
    /// A dynamic `class` fragment (`'sui-ai-' + (expr)`) plus EVERY rule for the
    /// axis, so whatever the expression evaluates to at runtime has a match.
    Dynamic { class_expr: String, rules: Vec<String> },
}

/// Resolve one axis prop for a tag. `attr` is the raw attribute value if present
/// (`Some("center")` static, `Some("{expr}")` dynamic) or `None` (use the tag's
/// default). Returns `None` if the prop doesn't apply or the static value is
/// off-enum (the lint surfaces that separately).
pub fn lower_axis(tag: &str, prop: &str, attr: Option<&str>) -> Option<AxisLower> {
    match attr {
        Some(v) if v.starts_with('{') => {
            let inner = &v[1..v.len().saturating_sub(1)];
            let prefix = axis_prefix(prop)?;
            let rules = allowed(prop)
                .iter()
                .filter_map(|val| modifier(prop, val).map(|(_, r)| r))
                .collect();
            Some(AxisLower::Dynamic { class_expr: format!("'{prefix}' + ({inner})"), rules })
        }
        other => {
            let val = other.or_else(|| default_for(tag, prop))?;
            let (class, rule) = modifier(prop, val)?;
            Some(AxisLower::Static { class, rule })
        }
    }
}

/// Lower one layout element to (classes, css-rules). `attrs` is the element's
/// resolved axis props (already parsed out of the tag). Off-enum values are
/// dropped here (the lint surfaces them); the base class always applies so the
/// box still lays out.
pub fn lower(tag: &str, attrs: &[(String, String)]) -> Option<(Vec<String>, Vec<String>)> {
    if !is_layout_tag(tag) {
        return None;
    }
    let mut classes = vec![base_class(tag).to_string()];
    let mut css = base_css(tag);
    for &prop in tag_props(tag) {
        let val = attrs
            .iter()
            .find(|(n, _)| n == prop)
            .map(|(_, v)| v.as_str())
            .or_else(|| default_for(tag, prop))?;
        if let Some((class, rule)) = modifier(prop, val) {
            classes.push(class);
            css.push(rule);
        }
    }
    Some((classes, css))
}

/// Lint: every layout element's axis props must be in-enum, must apply to the
/// tag, and must be static. Reads the raw source, scans the layout open tags.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for t in scan_open_tags(source) {
        let (line, col) = offset_to_line_col(source, t.start);
        let valid = tag_props(&t.tag);
        for (name, value) in &t.attrs {
            // Non-axis attrs (id, class, sx, on:click, data-*) pass straight
            // through to the div — nothing to validate.
            if !matches!(name.as_str(), "gap" | "align" | "justify" | "min" | "flow") {
                continue;
            }
            if !valid.contains(&name.as_str()) {
                findings.push(mk(
                    path,
                    line,
                    col,
                    format!("<{}> has no `{name}` — it takes: {}", t.tag, valid.join(", ")),
                ));
                continue;
            }
            if value.starts_with('{') {
                // Dynamic axis (`align={expr}`) — a component parameterizing its
                // layout. Lowered to a dynamic `class` binding with the full rule
                // set injected, so any runtime value matches. The value is a
                // runtime expression; we can't (and needn't) enum-check it here.
                continue;
            }
            if !allowed(name).contains(&value.as_str()) {
                let mut m = format!(
                    "<{} {name}=\"{value}\"> — `{value}` is not valid. allowed: {}",
                    t.tag,
                    allowed(name).join(", ")
                );
                if name == "gap" && (value.ends_with("px") || value.chars().all(|c| c.is_ascii_digit())) {
                    m.push_str(" (gap is a spacing token 0–8, not a raw length)");
                }
                findings.push(mk(path, line, col, m));
            }
        }
    }
    findings
}

/// A parsed layout open tag: the byte span of `<tag …>` and its attributes.
/// Shared by the lint (line/col) and the preprocess lowering (splice the span).
pub struct OpenTag {
    pub tag: String,
    /// `(name, value)` — value is `""` for a boolean attr and keeps its braces
    /// (`{expr}`) for a dynamic one, so callers can tell static from dynamic.
    pub attrs: Vec<(String, String)>,
    pub start: usize,
    pub end: usize,
    pub self_closing: bool,
}

/// Scan every `<row|column|grid|center|stack …>` open tag, skipping HTML
/// comments. Byte-safe (advances full UTF-8 chars). Recognizes the tag only at
/// a word boundary so `<rowlike>` / `<columns>` are left alone.
pub fn scan_open_tags(source: &str) -> Vec<OpenTag> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if bytes[i] == b'<' {
            if let Some(tag) = TAGS.iter().find(|&&t| {
                source[i + 1..].starts_with(t)
                    && matches!(
                        bytes.get(i + 1 + t.len()),
                        Some(b) if b.is_ascii_whitespace() || *b == b'>' || *b == b'/'
                    )
            }) {
                if let Some(parsed) = parse_open_tag(source, i, tag) {
                    let end = parsed.end;
                    out.push(parsed);
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    out
}

/// Parse a single `<tag …>` starting at `start` (the `<`). Returns None if the
/// tag never closes.
fn parse_open_tag(source: &str, start: usize, tag: &str) -> Option<OpenTag> {
    let bytes = source.as_bytes();
    let mut i = start + 1 + tag.len();
    let mut attrs = Vec::new();
    let mut self_closing = false;
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        if bytes[i] == b'>' {
            i += 1;
            break;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'>') {
            self_closing = true;
            i += 2;
            break;
        }
        // attribute name
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'>'
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let name = source[name_start..i].to_string();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let vstart = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                value = source[vstart..i].to_string();
                i += 1; // past the closing quote
            } else if i < bytes.len() && bytes[i] == b'{' {
                // dynamic `{expr}` — keep braces so callers see it's dynamic.
                let vstart = i;
                let mut depth = 0;
                while i < bytes.len() {
                    match bytes[i] {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                i += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                value = source[vstart..i].to_string();
            } else {
                let vstart = i;
                while i < bytes.len()
                    && !bytes[i].is_ascii_whitespace()
                    && bytes[i] != b'>'
                    && bytes[i] != b'/'
                {
                    i += 1;
                }
                value = source[vstart..i].to_string();
            }
        }
        if !name.is_empty() {
            attrs.push((name, value));
        }
    }
    Some(OpenTag { tag: tag.to_string(), attrs, start, end: i, self_closing })
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, b) in source.bytes().enumerate() {
        if i == offset {
            break;
        }
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<String> {
        check(Path::new("t.sui"), src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn lowers_column_with_defaults() {
        let (classes, _css) = lower("column", &[]).unwrap();
        assert_eq!(classes, vec!["sui-col", "sui-g-4", "sui-ai-stretch"]);
    }

    #[test]
    fn lowers_row_with_explicit_axes() {
        let (classes, css) = lower(
            "row",
            &[("gap".into(), "2".into()), ("justify".into(), "between".into())],
        )
        .unwrap();
        assert!(classes.contains(&"sui-row".to_string()));
        assert!(classes.contains(&"sui-g-2".to_string()));
        assert!(classes.contains(&"sui-jc-between".to_string()));
        assert!(css.iter().any(|r| r.contains("justify-content:space-between")));
        assert!(css.iter().any(|r| r.contains(".sui-row>*{min-width:0}")));
    }

    #[test]
    fn accepts_valid_tags() {
        assert!(run(r#"<column gap="4" align="center"><row gap="2" justify="between">x</row></column>"#).is_empty());
        assert!(run(r#"<grid min="lg" gap="4" flow="auto">x</grid>"#).is_empty());
    }

    #[test]
    fn rejects_off_enum_gap() {
        let m = run(r#"<column gap="99">x</column>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("spacing token"), "{}", m[0]);
    }

    #[test]
    fn rejects_prop_not_on_tag() {
        // center takes only gap; justify is meaningless there.
        assert_eq!(run(r#"<center justify="between">x</center>"#).len(), 1);
    }

    #[test]
    fn accepts_dynamic_axis() {
        // A component parameterizing its layout — lowered to a dynamic `class`
        // binding (see preprocess::lower_layout), not rejected.
        assert!(run(r#"<row align={x() ? 'start' : 'end'}>x</row>"#).is_empty());
    }

    #[test]
    fn passes_through_non_axis_attrs() {
        // id / class / sx / on:click / data-* are fine on a layout element.
        assert!(run(r#"<column id="hero" sx="p:4" on:click={go()} data-x="1">x</column>"#).is_empty());
    }

    #[test]
    fn ignores_non_layout_similar_tags() {
        assert!(scan_open_tags("<rows><columns><rowish>").is_empty());
    }
}

fn mk(path: &Path, line: usize, col: usize, message: String) -> Finding {
    Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Error,
        message,
        suggest: Some(
            "Layout keywords: row/column (flex), grid, center, stack (z-overlay). \
             gap is 0–8 (spacing tokens); align/justify take the flex enums."
                .to_string(),
        ),
    }
}
