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
        "column" => &["gap", "align", "direction"],
        // `direction` is a backward-compat alias for the old `<sui-stack
        // direction="row">` escape hatch (horizontal stack). Accepted on
        // row/column so legacy code keeps compiling through the keyword
        // migration. Misuse (`<column direction="row">` when you really
        // want `<row>`) is what `sui-legacy-layout` would flag, but the
        // modern author has no reason to reach for `direction` — they use
        // the keyword itself.
        "row" => &["gap", "align", "justify", "direction"],
        // `cols` is a fixed column count (1..=12); it conflicts with `min`
        // (the responsive `auto-fill minmax` axis) — `cols` wins if both
        // are present.
        "grid" => &["gap", "min", "flow", "cols"],
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
        // `direction` has NO default — the base class encodes the natural
        // direction (`.sui-col{flex-direction:column}`). Modifier only
        // fires when the author explicitly overrides.
        // `cols` also has NO default — when absent, `min` owns the column
        // track. Explicitly setting `cols` opts out of `min` (precedence:
        // cols wins).
        _ => return None,
    })
}

/// Valid values for an axis prop. `None` = the prop is free-form-invalid here.
///
/// `cols` is a small positive integer as a string (the closed set matches
/// what fits the language's grid vocabulary — 1..=12 covers author intent
/// without opening a `length-as-string` hole). Dynamic `cols={n()}` lowers
/// to a runtime class binding instead of going through this enum.
pub const COLS_VALUES: &[&str] = &["1","2","3","4","5","6","7","8","9","10","11","12"];
fn allowed(prop: &str) -> &'static [&'static str] {
    match prop {
        "gap" => &["0", "1", "2", "3", "4", "5", "6", "7", "8", "density"],
        "align" => &["start", "center", "end", "stretch", "baseline"],
        "justify" => &["start", "center", "end", "between", "around"],
        "min" => &["xs", "sm", "md", "lg"],
        "flow" => &["auto", "dense"],
        "cols" => COLS_VALUES,
        // `direction` is a flex-direction suffix on row/column. Same shape
        // as the underlying CSS property: `row` ↔ `flex-direction:row`,
        // `column` ↔ `flex-direction:column`.
        "direction" => &["row", "column"],
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
            let v = match value {
                "0" => "0".to_string(),
                // the density channel's row gap — rows/tiles resize with data-density
                "density" => "var(--density-row-gap)".to_string(),
                _ => format!("var(--space-{value})"),
            };
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
        // Fixed-N column grid: same shape as the JSX/CSS idiom, with minmax(0,1fr)
        // so children can shrink (defeats the "unguarded min-content" overflow
        // trap that haunts every hand-rolled grid-template-columns).
        "cols" => (
            format!("sui-gc-{value}"),
            format!(".sui-gc-{value}{{grid-template-columns:repeat({value},minmax(0,1fr))}}"),
        ),
        // `direction` overrides the underlying flex-direction — used when an
        // author reaches for `<column direction="row">` (the old sui-stack
        // idiom) and we want the markup to keep its visual meaning. The base
        // class already declares the default direction, so the modifier only
        // applies on a value mismatch (and any explicit direction ends up on
        // the same `.sui-{col,row}-{direction}` for predictable overrides).
        "direction" => (
            format!("sui-dir-{value}"),
            format!(".sui-dir-{value}{{flex-direction:{value}}}"),
        ),
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
        "cols" => "sui-gc-",
        "direction" => "sui-dir-",
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
    // Mirror the compiler-side precedence: `cols` wins over `min` on `<grid>`.
    // Both own `grid-template-columns`, so when both axes are present the
    // responsive one is dropped to avoid a CSS cascade / specificity tie.
    let has_cols = attrs.iter().any(|(n, _)| n == "cols");
    let skip: std::collections::HashSet<&str> = if tag == "grid" && has_cols {
        std::iter::once("min").collect()
    } else {
        std::collections::HashSet::new()
    };
    for &prop in tag_props(tag) {
        if skip.contains(prop) {
            continue;
        }
        // Some axes (e.g. `cols`, `direction`) have NO default — only an
        // explicit value lowers to a class+rule. Skip silently when the
        // author didn't set one; the base class already encodes the
        // natural state for the tag.
        let val = match attrs
            .iter()
            .find(|(n, _)| n == prop)
            .map(|(_, v)| v.as_str())
            .or_else(|| default_for(tag, prop))
        {
            Some(v) => v,
            None => continue,
        };
        if let Some((class, rule)) = modifier(prop, val) {
            classes.push(class);
            css.push(rule);
        }
    }
    Some((classes, css))
}

/// Lint: every layout element's axis props must be in-enum, must apply to the
/// tag, and must be static. Reads the raw source, scans the layout open tags.
///
/// Also enforces the no-silent-drop guarantee: any attribute on a layout
/// keyword that ISN'T a known axis prop on the tag AND ISN'T an HTML pass-
/// through is a compile error. A mistyped layout attribute would otherwise
/// flow through to the lowered `<div>` as a dead HTML attr with no signal.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for t in scan_open_tags(source) {
        let (line, col) = offset_to_line_col(source, t.start);
        let valid = tag_props(&t.tag);
        for (name, value) in &t.attrs {
            if is_passthrough_attr(name) {
                // Non-axis attrs (id, class, sx, on:click, data-*, a11y, …)
                // pass straight through to the div — nothing to validate.
                continue;
            }
            if !is_known_axis_prop(name) {
                findings.push(mk(
                    path,
                    line,
                    col,
                    format!(
                        "<{} {}=\"{}\"> — unknown attribute on layout keyword. {}",
                        t.tag,
                        name,
                        value,
                        unknown_attr_hint(&t.tag),
                    ),
                ));
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
                // layout. Lowered to a dynamic `class` binding with the full
                // rule set injected, so any runtime value matches. The value
                // is a runtime expression; we can't (and needn't) enum-check
                // it here.
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

/// Is `name` a layout-axis prop on ANY keyword? These get validated against
/// the tag-specific allow-list and the per-prop enum.
///
/// `cols` was added in the `job/sui-oss-grid` pass: it lives only on `<grid>`,
/// but listing it here lets the unknown-attr error name every axis prop the
/// user might have meant. `direction` is the legacy `<sui-stack
/// direction="row">` alias kept for migration.
fn is_known_axis_prop(name: &str) -> bool {
    matches!(
        name,
        "gap" | "align" | "justify" | "min" | "flow" | "cols" | "direction"
    )
}

pub fn is_passthrough_attr(name: &str) -> bool {
    // Universal attribute prefixes — namespaced attr families that ride through
    // to the div unchanged. The SUI compile pass treats these as opaque to
    // validation (the framework / browser own their semantics).
    if name.starts_with("data-")
        || name.starts_with("aria-")
        || name.starts_with("on")
            && name.as_bytes().get(2) == Some(&b':')
        // SUI directive prefixes — framework bindings that the compiler
        // understands (see `crates/sui/compiler/src/parse/template/attrs.rs`).
        // `bind:`, `class:`, `style:`, `attr:`, `prop:`, `transition:`, `in:`,
        // `out:`, `motion:`. Each is fully validated downstream; the layout
        // keyword lowering passes them through to the div unchanged.
        || name.starts_with("bind:")
        || name.starts_with("class:")
        || name.starts_with("style:")
        || name.starts_with("attr:")
        || name.starts_with("prop:")
        || name.starts_with("transition:")
        || name.starts_with("in:")
        || name.starts_with("out:")
        || name.starts_with("motion:")
    {
        return true;
    }
    matches!(
        name,
        "class"
            | "id"
            | "sx"
            | "title"
            | "role"
            | "tabindex"
            | "slot"
            | "hidden"
            | "draggable"
            | "translate"
            | "dir"
            | "lang"
            | "itemprop"
            | "itemscope"
            | "itemtype"
            | "itemref"
            | "itemid"
            | "inert"
            | "popover"
            | "autofocus"
            | "nonce"
            | "is"
            | "part"
            | "exportparts"
            | "shadowroot"
            | "form"
            | "name"
            | "value"
            | "checked"
            | "selected"
            | "disabled"
            | "readonly"
            | "required"
            | "placeholder"
            | "type"
            | "href"
            | "src"
            | "alt"
            | "for"
            | "target"
            | "rel"
            | "muted"
            | "loop"
            | "controls"
            | "autoplay"
            | "open"
            | "default"
            | "contenteditable"
            | "spellcheck"
            | "inputmode"
            | "enterkeyhint"
            | "maxlength"
            | "minlength"
            | "pattern"
    )
}

/// Message body for a `unknown attribute on layout keyword` finding. Points
/// the author at what the tag does accept.
fn unknown_attr_hint(tag: &str) -> String {
    let axes = tag_props(tag);
    if axes.is_empty() {
        format!("<{tag}> takes no axis props; only pass-through attrs are allowed.")
    } else {
        format!(
            "<{tag}> takes these axis props: {} — or any HTML/global pass-through (id, class, sx, data-*, aria-*, on:*, bind:*, class:*, style:*, attr:*, prop:*, transition:*, in:*, out:*, motion:*).",
            axes.join(", ")
        )
    }
}

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

    // --- `cols` axis on `<grid>` (m-sui-stdlib / job/sui-oss-grid) -----------

    #[test]
    fn accepts_grid_cols_static() {
        // The bug we closed: `<grid cols="7">` used to silently produce a
        // single-column broken grid. Now it lowers cleanly and lints clean.
        assert!(run(r#"<grid cols="7" gap="4">x</grid>"#).is_empty());
    }

    #[test]
    fn accepts_grid_cols_dynamic() {
        // A component parameterizing column count, the same shape as
        // `<row align={pick()}>`: a dynamic `class` binding with all 12 rules
        // pre-injected so any runtime value matches.
        assert!(run(r#"<grid cols={n()}>x</grid>"#).is_empty());
    }

    #[test]
    fn lowers_grid_cols_to_repeat_minmax() {
        let (classes, css) = lower("grid", &[("cols".into(), "7".into())]).unwrap();
        assert!(classes.contains(&"sui-grid".to_string()));
        assert!(classes.contains(&"sui-gc-7".to_string()));
        assert!(
            css.iter().any(|r| r.contains("grid-template-columns:repeat(7,minmax(0,1fr))")),
            "repeat(7,minmax(0,1fr)) rule present: {css:?}"
        );
        // Precedence: `cols` wins over `min` on `<grid>`. `lower()` mirrors
        // the compiler-side logic — when `cols` is present, the responsive
        // `min` axis is suppressed (no `sui-gm-*` class, no auto-fill rule).
        assert!(!classes.iter().any(|c| c.starts_with("sui-gm-")), "no min class when cols wins: {classes:?}");
    }

    #[test]
    fn rejects_off_enum_cols() {
        // 13 is outside the closed 1..=12 set; off-enum must surface as an
        // explicit finding so a too-large/col-zero typo doesn't silently
        // collapse to a broken grid.
        let m = run(r#"<grid cols="13">x</grid>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("13") && m[0].contains("not valid"), "got: {}", m[0]);
    }

    #[test]
    fn rejects_cols_on_a_tag_that_doesnt_take_it() {
        // `cols` is `<grid>`-only; `<column cols="5">` is a real author mistake
        // — say so.
        let m = run(r#"<column cols="5">x</column>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("<column>"), "got: {}", m[0]);
        assert!(m[0].contains("cols"), "names the bad attr: {}", m[0]);
    }

    // --- `direction` backward-compat (regression: was unknown-attr) ---------

    #[test]
    fn accepts_legacy_direction_idiom() {
        // `<column direction="row">` was the `<sui-stack direction="row">`
        // idiom; kept compiling through the keyword migration so existing
        // components don't break. The lint should accept it.
        assert!(run(r#"<column gap="2" direction="row">x</column>"#).is_empty());
        assert!(run(r#"<row gap="2" direction="column">x</row>"#).is_empty());
    }

    #[test]
    fn lowers_direction_to_flex_direction() {
        // Explicit override emits the `sui-dir-*` modifier.
        let (classes, css) = lower(
            "column",
            &[("direction".into(), "row".into())],
        )
        .unwrap();
        assert!(
            classes.contains(&"sui-dir-row".to_string()),
            "direction override class: {classes:?}"
        );
        assert!(
            css.iter().any(|r| r.contains("flex-direction:row")),
            "direction override rule: {css:?}"
        );
        // No default — no `sui-dir-column` on a plain `<column>`.
        let (classes, _) = lower("column", &[]).unwrap();
        assert!(
            !classes.iter().any(|c| c.starts_with("sui-dir-")),
            "no implicit direction class: {classes:?}"
        );
    }

    #[test]
    fn rejects_off_enum_direction() {
        let m = run(r#"<column direction="diagonal">x</column>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("diagonal"), "names the bad value: {}", m[0]);
    }

    // --- no silent drop ---------------------------------------------------

    #[test]
    fn unknown_layout_attribute_is_rejected() {
        // The original bug: `cols="7"` used to be treated as an unknown attr
        // that flowed through to the div as a dead HTML attribute with no
        // warning. Now any unrecognized attribute on a layout keyword is a
        // hard error.
        let m = run(r#"<grid bogus="x">x</grid>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("unknown"), "got: {}", m[0]);
        assert!(m[0].contains("bogus"), "names the bad attr: {}", m[0]);
    }

    #[test]
    fn recognized_html_passthroughs_still_pass() {
        // The same surface as `passes_through_non_axis_attrs`, but with a
        // wider vocabulary to lock in that we didn't regress the pass-through
        // set in the unknown-attr refactor.
        assert!(run(
            r#"<column id="x" title="t" role="region" tabindex="-1" hidden draggable="true" aria-label="x" data-y="1" on:click={go()}>x</column>"#
        ).is_empty());
    }

    #[test]
    fn unknown_attr_does_not_reach_lower_layout_as_a_passthrough() {
        // Lint and compiler share `is_passthrough_attr`; assert the source-of-
        // truth truth: the attribute vocab recognized as a pass-through on the
        // lint side is exactly what `lower_layout` would let ride through to
        // the div. If either side ever grows the set without the other, the
        // compiler stops agreeing with the lint — the silent-drop class of
        // regression is back.
        assert!(is_passthrough_attr("id"));
        assert!(is_passthrough_attr("data-x"));
        assert!(is_passthrough_attr("on:click"));
        assert!(is_passthrough_attr("aria-label"));
        assert!(!is_passthrough_attr("bogus"));
        assert!(!is_passthrough_attr("clss"));
        assert!(!is_passthrough_attr("colz"));
    }

    #[test]
    fn axis_prop_names_not_treated_as_passthroughs() {
        // `min`, `cols`, etc. are axis props; they MUST flow through
        // `is_known_axis_prop` for validation, not be short-circuited as
        // pass-throughs (which would silently hide `<column min="99">`).
        assert!(!is_passthrough_attr("cols"));
        assert!(!is_passthrough_attr("gap"));
        assert!(!is_passthrough_attr("align"));
        assert!(!is_passthrough_attr("justify"));
        assert!(!is_passthrough_attr("min"));
        assert!(!is_passthrough_attr("flow"));
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
