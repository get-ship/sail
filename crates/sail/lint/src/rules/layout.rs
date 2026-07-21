//! `sail-layout` — the language-level layout elements.
//!
//! `row`, `column`, `grid`, `center`, `stack` are compile-time keywords, not
//! web components. The preprocess pass lowers each to a plain `<div>` + atomic
//! classes, and injects the deduped CSS those classes need into the component's
//! `<style>` — the same "class + injected stylesheet" mechanism `sx` uses. No
//! shadow DOM, no custom-element upgrade: layout disappears into the language.
//!
//! Faithful successor to the `<sail-stack>` / `<sail-cluster>` / `<sail-grid>` /
//! `<sail-center>` primitives — the hard-won opinions ride along in the lowering:
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
//! Naming: `column` = vertical flow (was `sail-stack`), `row` = horizontal flow
//! that wraps (was `sail-cluster`), `grid`/`center` unchanged, and `stack` is the
//! z-axis overlay (children layered) — the one genuinely new box.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-layout";

/// The layout keywords the preprocess lowers to `<div>`.
pub const TAGS: &[&str] = &["row", "column", "grid", "center", "stack"];

pub fn is_layout_tag(tag: &str) -> bool {
    TAGS.contains(&tag)
}

/// Which axis props each tag understands. Used by both the lowering (to know
/// what to read) and the lint (to reject props that don't apply to a tag).
pub fn tag_props(tag: &str) -> &'static [&'static str] {
    match tag {
        // `inline` opts the layout box into `display: inline-flex` / `inline-grid`
        // — the shrink-to-fit counterpart to the (default) block-level layout.
        // It is a flag, not an enum: presence is the whole vocabulary. The
        // language knows the pattern (an icon wrapper that hug-wraps; the
        // charts' inline hosts) but it had no element to express it; sx's
        // `inline-flex` / `inline-grid` refusal redirects here, so this prop
        // is what makes that suggestion buildable.
        // `justify` distributes the MAIN (vertical) axis of a column — the
        // symmetric counterpart to `<row justify>`. A full-height column with
        // `justify="between"` pins its first child to the top and its last to
        // the bottom (a sidebar's logo/nav split). It has NO default: a plain
        // `<column>` keeps `justify-content`'s CSS initial (flex-start), so the
        // axis only fires when the author asks — no output churn on existing
        // columns.
        "column" => &["gap", "align", "justify", "direction", "inline"],
        // `direction` is a backward-compat alias for the old `<sail-stack
        // direction="row">` escape hatch (horizontal stack). Accepted on
        // row/column so legacy code keeps compiling through the keyword
        // migration. Misuse (`<column direction="row">` when you really
        // want `<row>`) is what `sail-legacy-layout` would flag, but the
        // modern author has no reason to reach for `direction` — they use
        // the keyword itself.
        // `nowrap` is a boolean axis: a `<row>` wraps by default (it absorbed
        // `sail-cluster`), and `<row nowrap>` is the single-line variant —
        // toolbars, split panes, anything that must NOT reflow onto a second
        // line. Presence is the whole vocabulary (like `inline`); it layers a
        // `flex-wrap:nowrap` override on the base and composes with `inline`.
        "row" => &["gap", "align", "justify", "direction", "inline", "nowrap"],
        // `cols` is a fixed column count (1..=12); it conflicts with `min`
        // (the responsive `auto-fill minmax` axis) — `cols` wins if both
        // are present. `rows` is the orthogonal explicit ROW count
        // (`grid-template-rows`); it owns a different property than
        // `cols`/`min`/`tracks`, so it composes freely with all of them.
        "grid" => &["gap", "min", "flow", "cols", "rows", "tracks", "inline"],
        "center" => &["gap", "inline"],
        "stack" => &["align", "justify", "inline"],
        _ => &[],
    }
}

/// The base structural class for a tag, block (default) or inline (`<row inline>`).
/// Its CSS is direction/display only — every tunable (gap/align/justify) rides an
/// explicit atomic class so exactly one rule owns each property (no specificity
/// games). The inline variant swaps `display:flex|grid` for `inline-flex|inline-grid`,
/// preserving the `min-width:0` child guard so an inline layout nested in a
/// block parent can still shrink instead of forcing overflow.
///
/// `inline` and the block form are mutually exclusive — exactly one base class
/// ends up on the lowered div, and exactly one base CSS rule is injected.
/// They are NOT modifiers of each other: a `<row>` and a `<row inline>` are
/// different elements, not the same element with a flag.
fn base_class(tag: &str) -> &'static str {
    match tag {
        "row" => "sail-row",
        "column" => "sail-col",
        "grid" => "sail-grid",
        "center" => "sail-center",
        "stack" => "sail-stackz",
        _ => "",
    }
}
fn inline_base_class(tag: &str) -> &'static str {
    match tag {
        "row" => "sail-row-i",
        "column" => "sail-col-i",
        "grid" => "sail-grid-i",
        "center" => "sail-center-i",
        "stack" => "sail-stackz-i",
        _ => "",
    }
}

/// The CSS rule(s) for a base class — structure + the `min-width:0` child guard.
fn base_css(tag: &str) -> Vec<String> {
    match tag {
        "column" => vec![
            ".sail-col{display:flex;flex-direction:column;min-width:0}".into(),
            ".sail-col>*{min-width:0}".into(),
        ],
        "row" => vec![
            ".sail-row{display:flex;flex-wrap:wrap;min-width:0}".into(),
            ".sail-row>*{min-width:0}".into(),
        ],
        "grid" => vec![
            ".sail-grid{display:grid;min-width:0}".into(),
            ".sail-grid>*{grid-column:span 1;grid-row:span 1;min-width:0}".into(),
            ".sail-grid>*[data-col-span='2']{grid-column:span 2}".into(),
            ".sail-grid>*[data-col-span='3']{grid-column:span 3}".into(),
            ".sail-grid>*[data-col-span='4']{grid-column:span 4}".into(),
            ".sail-grid>*[data-row-span='2']{grid-row:span 2}".into(),
            ".sail-grid>*[data-row-span='3']{grid-row:span 3}".into(),
            ".sail-grid>*[data-row-span='4']{grid-row:span 4}".into(),
            "@media(max-width:720px){.sail-grid>*[data-col-span]{grid-column:span 1}.sail-grid>*[data-row-span]{grid-row:span 1}}".into(),
        ],
        "center" => vec![
            ".sail-center{display:grid;place-items:center;place-content:center;width:100%}".into(),
        ],
        "stack" => vec![
            ".sail-stackz{display:grid}".into(),
            // position:relative on every layer: without it, a positioned child
            // (any skinned component) paints ABOVE later static siblings and
            // the stack's DOM order stops meaning front-to-back. All-positioned
            // restores document-order painting — layering by construction.
            ".sail-stackz>*{grid-area:1/1;position:relative}".into(),
        ],
        _ => vec![],
    }
}
/// The inline counterpart to [`base_css`] — same min-width:0 child guard, same
/// axis property ownership, only the `display` value differs. `inline-flex` /
/// `inline-grid` make the layout box shrink-to-fit its content so it can sit
/// inside text or alongside other inline siblings without forcing a new line.
fn inline_base_css(tag: &str) -> Vec<String> {
    match tag {
        "column" => vec![
            ".sail-col-i{display:inline-flex;flex-direction:column;min-width:0}".into(),
            ".sail-col-i>*{min-width:0}".into(),
        ],
        "row" => vec![
            ".sail-row-i{display:inline-flex;flex-wrap:wrap;min-width:0}".into(),
            ".sail-row-i>*{min-width:0}".into(),
        ],
        "grid" => vec![
            ".sail-grid-i{display:inline-grid;min-width:0}".into(),
            ".sail-grid-i>*{grid-column:span 1;grid-row:span 1;min-width:0}".into(),
            ".sail-grid-i>*[data-col-span='2']{grid-column:span 2}".into(),
            ".sail-grid-i>*[data-col-span='3']{grid-column:span 3}".into(),
            ".sail-grid-i>*[data-col-span='4']{grid-column:span 4}".into(),
            ".sail-grid-i>*[data-row-span='2']{grid-row:span 2}".into(),
            ".sail-grid-i>*[data-row-span='3']{grid-row:span 3}".into(),
            ".sail-grid-i>*[data-row-span='4']{grid-row:span 4}".into(),
            "@media(max-width:720px){.sail-grid-i>*[data-col-span]{grid-column:span 1}.sail-grid-i>*[data-row-span]{grid-row:span 1}}".into(),
        ],
        // `<center inline>` drops the `width:100%` block default — an inline
        // center hug-wraps its content like an inline-flex equivalent of
        // place-content:center.
        "center" => vec![
            ".sail-center-i{display:inline-grid;place-items:center;place-content:center}".into(),
        ],
        "stack" => vec![
            ".sail-stackz-i{display:inline-grid}".into(),
            ".sail-stackz-i>*{grid-area:1/1;position:relative}".into(),
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
        // direction (`.sail-col{flex-direction:column}`). Modifier only
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
        // `evenly` (space-evenly) completes the main-axis distribution set:
        // `between` puts all extra space BETWEEN items, `evenly` puts equal
        // space around AND between (evenly-spaced nav items / toolbar). It is
        // the last standard `justify-content` distribution the enum was
        // missing.
        "justify" => &["start", "center", "end", "between", "around", "evenly"],
        "min" => &["xs", "sm", "md", "lg"],
        // validated by shape, not enum — see is_tracks_var
        "tracks" => &[],
        "flow" => &["auto", "dense"],
        "cols" => COLS_VALUES,
        // `rows` shares the closed 1..=12 set with `cols`: an explicit
        // grid-template-rows track count.
        "rows" => COLS_VALUES,
        // `direction` is a flex-direction suffix on row/column. Same shape
        // as the underlying CSS property: `row` ↔ `flex-direction:row`,
        // `column` ↔ `flex-direction:column`.
        "direction" => &["row", "column"],
        _ => &[],
    }
}

/// The atomic class + CSS rule for one resolved `prop:value`. `None` if the
/// value is off-enum (the lint reports it; the lowering just skips it).
/// A `tracks` value is a custom-property name (`--x`): the engine owns the
/// template string and writes it via `style:--x`; the language consumes it.
pub fn is_tracks_var(value: &str) -> bool {
    value.starts_with("--")
        && value.len() > 2
        && value[2..]
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn modifier(prop: &str, value: &str) -> Option<(String, String)> {
    if prop == "tracks" {
        if !is_tracks_var(value) {
            return None;
        }
        let slug: String = value[2..].chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        return Some((
            format!("sail-gt-{slug}"),
            format!(".sail-gt-{slug}{{grid-template-columns:var({value})}}"),
        ));
    }
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
            (format!("sail-g-{value}"), format!(".sail-g-{value}{{gap:{v}}}"))
        }
        "align" => {
            let v = match value {
                "start" => "flex-start",
                "end" => "flex-end",
                other => other, // center | stretch | baseline
            };
            (format!("sail-ai-{value}"), format!(".sail-ai-{value}{{align-items:{v}}}"))
        }
        "justify" => {
            let v = match value {
                "start" => "flex-start",
                "end" => "flex-end",
                "between" => "space-between",
                "around" => "space-around",
                "evenly" => "space-evenly",
                other => other, // center
            };
            (format!("sail-jc-{value}"), format!(".sail-jc-{value}{{justify-content:{v}}}"))
        }
        "min" => {
            let rem = match value {
                "xs" => "16rem",
                "sm" => "18rem",
                "lg" => "24rem",
                _ => "20rem", // md
            };
            (
                format!("sail-gm-{value}"),
                format!(".sail-gm-{value}{{grid-template-columns:repeat(auto-fill,minmax({rem},1fr))}}"),
            )
        }
        "flow" => (format!("sail-gf-{value}"), format!(".sail-gf-{value}{{grid-auto-flow:{value}}}")),
        // Fixed-N column grid: same shape as the JSX/CSS idiom, with minmax(0,1fr)
        // so children can shrink (defeats the "unguarded min-content" overflow
        // trap that haunts every hand-rolled grid-template-columns).
        "cols" => (
            format!("sail-gc-{value}"),
            format!(".sail-gc-{value}{{grid-template-columns:repeat({value},minmax(0,1fr))}}"),
        ),
        // Fixed-N row grid: the row-axis mirror of `cols`. `minmax(0,1fr)` so
        // children can shrink below their content height (the same overflow
        // guard `cols` applies on the column axis). Owns `grid-template-rows`
        // — orthogonal to `cols`/`min`/`tracks`, which own the column axis.
        "rows" => (
            format!("sail-gr-{value}"),
            format!(".sail-gr-{value}{{grid-template-rows:repeat({value},minmax(0,1fr))}}"),
        ),
        // `direction` overrides the underlying flex-direction — used when an
        // author reaches for `<column direction="row">` (the old sail-stack
        // idiom) and we want the markup to keep its visual meaning. The base
        // class already declares the default direction, so the modifier only
        // applies on a value mismatch (and any explicit direction ends up on
        // the same `.sail-{col,row}-{direction}` for predictable overrides).
        "direction" => (
            format!("sail-dir-{value}"),
            format!(".sail-dir-{value}{{flex-direction:{value}}}"),
        ),
        _ => return None,
    })
}
/// The base structural class + CSS for a tag (no axis modifiers). The div always
/// gets this; axes ride on top via [`lower_axis`].
///
/// `inline` selects the inline variant (the `<row inline>`-style hug-wrap
/// element): the base class changes (`sail-row` → `sail-row-i`) and the
/// `display` value in CSS swaps to `inline-flex` / `inline-grid`. Everything
/// else — the `min-width:0` guard, axis property ownership — is identical,
/// because an inline layout is still a layout, not a different kind of box.
pub fn base(tag: &str, inline: bool) -> Option<(String, Vec<String>)> {
    if !is_layout_tag(tag) {
        return None;
    }
    Some((
        if inline {
            inline_base_class(tag).to_string()
        } else {
            base_class(tag).to_string()
        },
        if inline {
            inline_base_css(tag)
        } else {
            base_css(tag)
        },
    ))
}

/// The atomic-class PREFIX for an axis (`align` → `sail-ai-`), so a dynamic value
/// can lower to `'<prefix>' + (expr)`.
fn axis_prefix(prop: &str) -> Option<&'static str> {
    Some(match prop {
        "gap" => "sail-g-",
        "align" => "sail-ai-",
        "justify" => "sail-jc-",
        "min" => "sail-gm-",
        "flow" => "sail-gf-",
        "cols" => "sail-gc-",
        "rows" => "sail-gr-",
        "direction" => "sail-dir-",
        _ => return None,
    })
}

/// One resolved axis for the lowering.
pub enum AxisLower {
    /// A concrete class + its one CSS rule.
    Static { class: String, rule: String },
    /// A dynamic `class` fragment (`'sail-ai-' + (expr)`) plus EVERY rule for the
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
///
/// `inline` (presence of the boolean axis) swaps the base class + CSS for
/// the inline variant BEFORE axis modifiers are layered on, so axis props
/// (gap/align/justify/cols/min/flow/direction) compose cleanly with the
/// inline box — the same axes, just hung on a shrink-to-fit container.
pub fn lower(tag: &str, attrs: &[(String, String)]) -> Option<(Vec<String>, Vec<String>)> {
    if !is_layout_tag(tag) {
        return None;
    }
    // `inline` is a boolean axis: presence selects the inline base. It has
    // no class modifier of its own — the swap happens at the base, not as
    // an `sail-*` atomic riding on top of the block base.
    let inline = attrs.iter().any(|(n, _)| n == "inline");
    let mut classes = vec![if inline {
        inline_base_class(tag).to_string()
    } else {
        base_class(tag).to_string()
    }];
    let mut css = if inline {
        inline_base_css(tag)
    } else {
        base_css(tag)
    };
    // `nowrap` is a boolean axis (row only): it layers a `flex-wrap:nowrap`
    // override on top of the base's `flex-wrap:wrap`. Injected AFTER the base
    // rule so source order gives it the win (single-class selectors, so no
    // specificity game). Composes with `inline` — `<row inline nowrap>` gets
    // the inline base plus this override.
    if tag == "row" && attrs.iter().any(|(n, _)| n == "nowrap") {
        classes.push("sail-nowrap".to_string());
        css.push(".sail-nowrap{flex-wrap:nowrap}".to_string());
    }
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
        // `inline` is handled at the base, `nowrap` as a boolean override
        // above — neither is an enum-valued axis modifier.
        if prop == "inline" || prop == "nowrap" {
            continue;
        }
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
                //
                // Boolean axes (`inline`, `nowrap`) are NOT dynamic — their
                // meaning is structural (`display:flex` vs `inline-flex`;
                // `flex-wrap:wrap` vs `nowrap`), not a property value, so the
                // language pins them at compile time. A `inline={cond}` /
                // `nowrap={cond}` reading is almost always an authoring
                // mistake (the author wanted to swap element variants).
                if is_boolean_axis(name) {
                    let (block, flag) = boolean_axis_variants(name, &t.tag);
                    findings.push(mk(path, line, col, format!(
                        "<{}> {name}={{…}} — `{name}` is a structural axis, not a value. \
                         Pick one element: {block} or {flag}.",
                        t.tag
                    )));
                    continue;
                }
                continue;
            }
            if is_boolean_axis(name) {
                // Boolean axis — presence is the entire vocabulary. Any
                // explicit value (`inline="yes"`, `nowrap="true"`, …) is a
                // contract violation that the rejection explains.
                if !value.is_empty() {
                    findings.push(mk(path, line, col, format!(
                        "<{} {name}=\"{value}\"> — `{name}` is a flag, not a value. \
                         Write `<{} {name}>` (presence-only).",
                        t.tag, t.tag
                    )));
                }
                continue;
            }
            if name == "tracks" && is_tracks_var(&value) {
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
/// `cols` was added in the `job/sail-oss-grid` pass: it lives only on `<grid>`,
/// but listing it here lets the unknown-attr error name every axis prop the
/// user might have meant. `direction` is the legacy `<sail-stack
/// direction="row">` alias kept for migration. `inline` is a boolean axis
/// (no value) — present on row/column/grid/center/stack — that swaps the
/// base layout box from `display:flex|grid` to `inline-flex|inline-grid`.
/// The boolean axes — presence-only flags with no value (`<row inline>`,
/// `<row nowrap>`). They swap structure rather than taking an enum value, so
/// the lint rejects both a dynamic reading (`nowrap={x}`) and an explicit
/// value (`nowrap="yes"`).
fn is_boolean_axis(name: &str) -> bool {
    matches!(name, "inline" | "nowrap")
}

/// Prose for a boolean axis's two element variants: `(default/block form,
/// flag form)`. Feeds the "pick one element" refusal so the author sees the
/// two concrete elements the flag chooses between, not an abstract rule.
fn boolean_axis_variants(name: &str, tag: &str) -> (String, String) {
    match name {
        "inline" => (format!("<{tag}> (block)"), format!("<{tag} inline> (hug-wrap)")),
        "nowrap" => (format!("<{tag}> (wraps)"), format!("<{tag} nowrap> (single line)")),
        _ => (format!("<{tag}>"), format!("<{tag} {name}>")),
    }
}

fn is_known_axis_prop(name: &str) -> bool {
    matches!(
        name,
        "gap" | "align" | "justify" | "min" | "flow" | "cols" | "rows" | "direction" | "tracks"
            | "inline" | "nowrap"
    )
}

pub fn is_passthrough_attr(name: &str) -> bool {
    // Universal attribute prefixes — namespaced attr families that ride through
    // to the div unchanged. The SAIL compile pass treats these as opaque to
    // validation (the framework / browser own their semantics).
    if name.starts_with("data-")
        || name.starts_with("aria-")
        || name.starts_with("on")
            && name.as_bytes().get(2) == Some(&b':')
        // SAIL directive prefixes — framework bindings that the compiler
        // understands (see `crates/sail/compiler/src/parse/template/attrs.rs`).
        // `bind:`, `use:`, `attr:`, `prop:`, `motion:`. Each is fully
        // validated downstream; the layout keyword lowering passes them
        // through to the div unchanged. `class:` / `style:` are retired
        // and intentionally absent here.
        || name.starts_with("bind:")
        || name.starts_with("use:")
        || name.starts_with("attr:")
        || name.starts_with("prop:")
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
            | "placement"
            | "popovertarget"
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
            "<{tag}> takes these axis props: {} — or any HTML/global pass-through (id, sx, data-*, aria-*, on:*, bind:*, attr:*, prop:*, motion:*).",
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
        check(Path::new("t.sail"), src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn lowers_column_with_defaults() {
        let (classes, _css) = lower("column", &[]).unwrap();
        assert_eq!(classes, vec!["sail-col", "sail-g-4", "sail-ai-stretch"]);
    }

    #[test]
    fn lowers_row_with_explicit_axes() {
        let (classes, css) = lower(
            "row",
            &[("gap".into(), "2".into()), ("justify".into(), "between".into())],
        )
        .unwrap();
        assert!(classes.contains(&"sail-row".to_string()));
        assert!(classes.contains(&"sail-g-2".to_string()));
        assert!(classes.contains(&"sail-jc-between".to_string()));
        assert!(css.iter().any(|r| r.contains("justify-content:space-between")));
        assert!(css.iter().any(|r| r.contains(".sail-row>*{min-width:0}")));
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
        assert!(run(r#"<column id="hero" sx="p-4" on:click={go()} data-x="1">x</column>"#).is_empty());
    }

    #[test]
    fn ignores_non_layout_similar_tags() {
        assert!(scan_open_tags("<rows><columns><rowish>").is_empty());
    }

    // --- `cols` axis on `<grid>` (m-sail-stdlib / job/sail-oss-grid) -----------

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
        assert!(classes.contains(&"sail-grid".to_string()));
        assert!(classes.contains(&"sail-gc-7".to_string()));
        assert!(
            css.iter().any(|r| r.contains("grid-template-columns:repeat(7,minmax(0,1fr))")),
            "repeat(7,minmax(0,1fr)) rule present: {css:?}"
        );
        // Precedence: `cols` wins over `min` on `<grid>`. `lower()` mirrors
        // the compiler-side logic — when `cols` is present, the responsive
        // `min` axis is suppressed (no `sail-gm-*` class, no auto-fill rule).
        assert!(!classes.iter().any(|c| c.starts_with("sail-gm-")), "no min class when cols wins: {classes:?}");
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
        // `<column direction="row">` was the `<sail-stack direction="row">`
        // idiom; kept compiling through the keyword migration so existing
        // components don't break. The lint should accept it.
        assert!(run(r#"<column gap="2" direction="row">x</column>"#).is_empty());
        assert!(run(r#"<row gap="2" direction="column">x</row>"#).is_empty());
    }

    #[test]
    fn lowers_direction_to_flex_direction() {
        // Explicit override emits the `sail-dir-*` modifier.
        let (classes, css) = lower(
            "column",
            &[("direction".into(), "row".into())],
        )
        .unwrap();
        assert!(
            classes.contains(&"sail-dir-row".to_string()),
            "direction override class: {classes:?}"
        );
        assert!(
            css.iter().any(|r| r.contains("flex-direction:row")),
            "direction override rule: {css:?}"
        );
        // No default — no `sail-dir-column` on a plain `<column>`.
        let (classes, _) = lower("column", &[]).unwrap();
        assert!(
            !classes.iter().any(|c| c.starts_with("sail-dir-")),
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
        assert!(!is_passthrough_attr("inline"));
    }

    // --- `inline` axis (the hug-wrap counterpart to the block layout) ------

    #[test]
    fn accepts_inline_on_every_layout_tag() {
        // `inline` is a boolean axis, present on row/column/grid/center/stack.
        // Any of those with `<tag inline>` is a valid layout element — the
        // compiler swaps the base class for the `-i` variant and the `display`
        // value for `inline-flex`/`inline-grid`. No value; presence is the
        // vocabulary.
        assert!(run(r#"<row inline>x</row>"#).is_empty());
        assert!(run(r#"<column inline>x</column>"#).is_empty());
        assert!(run(r#"<grid inline>x</grid>"#).is_empty());
        assert!(run(r#"<center inline>x</center>"#).is_empty());
        assert!(run(r#"<stack inline>x</stack>"#).is_empty());
    }

    #[test]
    fn lowers_row_inline_to_inline_flex_base_class() {
        // The swap happens at the base, not as a class modifier: `<row inline>`
        // lowers to `sail-row-i` (and NOT `sail-row sail-row-i`). The block base
        // and the inline base are mutually exclusive — exactly one base rule
        // is injected, exactly one base class is on the div. A duplicate would
        // be a specificity tie that nothing can resolve.
        let (classes, css) = lower("row", &[("inline".into(), "".into())]).unwrap();
        assert_eq!(classes[0], "sail-row-i", "base swapped to inline variant: {classes:?}");
        assert!(
            !classes.iter().any(|c| c == "sail-row"),
            "no block base leaked into inline layout: {classes:?}"
        );
        assert!(
            css.iter().any(|r| r.starts_with(".sail-row-i{display:inline-flex")),
            "inline-flex rule is the one we ship: {css:?}"
        );
        assert!(
            css.iter().any(|r| r == ".sail-row-i>*{min-width:0}"),
            "the min-width:0 child guard survives the inline swap: {css:?}"
        );
        assert!(
            !css.iter().any(|r| r.starts_with(".sail-row{display:flex")),
            "no block-flex rule when inline is set: {css:?}"
        );
    }

    #[test]
    fn lowers_column_inline_with_default_axes() {
        // Default axis props still apply on `<column inline>` — the swap is
        // orthogonal to axis selection. `gap="4"` and `align="stretch"` are
        // the column defaults and ride on top of the inline base.
        let (classes, _css) = lower("column", &[("inline".into(), "".into())]).unwrap();
        assert_eq!(classes[0], "sail-col-i");
        assert!(classes.contains(&"sail-g-4".to_string()));
        assert!(classes.contains(&"sail-ai-stretch".to_string()));
    }

    #[test]
    fn lowers_grid_inline_to_inline_grid_base_class() {
        // Grid mirrors the same swap (`display:grid` → `inline-grid`). The
        // child span/data-* selectors carry over identically — the inline
        // base class has its own block of rules (`.sail-grid-i>` not
        // `.sail-grid>`) so no rule collides across the two bases.
        let (classes, css) = lower("grid", &[("inline".into(), "".into())]).unwrap();
        assert_eq!(classes[0], "sail-grid-i");
        assert!(
            css.iter().any(|r| r.starts_with(".sail-grid-i{display:inline-grid")),
            "inline-grid base rule: {css:?}"
        );
        assert!(
            css.iter().any(|r| r == ".sail-grid-i>*[data-col-span='2']{grid-column:span 2}"),
            "data-col-span rule carries to the inline base: {css:?}"
        );
    }

    #[test]
    fn lowers_stack_inline_to_inline_grid_with_overlay() {
        // Stack keeps its z-axis overlay (`*{grid-area:1/1}`) on the inline
        // variant — it is the same logical element (children layered), just
        // shrink-to-fit at the box level.
        let (classes, css) = lower("stack", &[("inline".into(), "".into())]).unwrap();
        assert_eq!(classes[0], "sail-stackz-i");
        assert!(
            css.iter().any(|r| r.contains(".sail-stackz-i>*{grid-area:1/1;position:relative}")),
            "z-overlay child rule (all layers positioned, so DOM order IS paint order) \
             survives the swap: {css:?}"
        );
    }

    #[test]
    fn center_inline_drops_block_width_default() {
        // `<center>` defaults to `width:100%` (block-level); the inline
        // variant drops that — an inline center hug-wraps its content.
        // Without the drop, the inline center would force a 100%-wide inline
        // box, which is incoherent (inline boxes take their content width).
        let (_classes, css) = lower("center", &[("inline".into(), "".into())]).unwrap();
        assert!(
            !css.iter().any(|r| r.contains("width:100%")),
            "no width:100% on inline center: {css:?}"
        );
    }

    #[test]
    fn rejects_inline_with_a_value() {
        // Boolean axes do not take values. `<row inline="yes">` is a contract
        // violation — the rejection spells the rule out so an author who
        // mistyped doesn't lose an afternoon.
        let m = run(r#"<row inline="yes">x</row>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("flag, not a value"), "got: {}", m[0]);
    }

    #[test]
    fn rejects_dynamic_inline() {
        // `<row inline={cond}>` is rejected — `inline` is a structural axis,
        // not a property value, so the language pins it at compile time.
        // A dynamic read is almost always an authoring mistake (the author
        // wanted to swap element variants), and the rejection names the
        // fixed choice the language offers.
        let m = run(r#"<row inline={open()}>x</row>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("structural axis"), "got: {}", m[0]);
    }

    #[test]
    fn inline_does_not_leak_to_an_unknown_attr_finding() {
        // Sanity: `inline` is in `is_known_axis_prop`, so it should NEVER
        // surface as an unknown-attr finding. If it does, the base swap
        // silently fails to fire and the lowered div is `display:flex` — the
        // the exact regression that would defeat this feature.
        let m = run(r#"<row inline>x</row>"#);
        assert!(
            !m.iter().any(|s| s.contains("unknown attribute")),
            "`inline` is a known axis prop: {m:?}"
        );
    }

    // --- `<row nowrap>` boolean axis (job/layout-pass known gap #1) --------

    #[test]
    fn accepts_row_nowrap() {
        // A non-wrapping row — the single-line variant a toolbar / split pane
        // needs. Presence-only, like `inline`.
        assert!(run(r#"<row nowrap>x</row>"#).is_empty());
        assert!(run(r#"<row nowrap gap="2" align="center">x</row>"#).is_empty());
    }

    #[test]
    fn lowers_row_nowrap_to_flex_wrap_override() {
        let (classes, css) = lower("row", &[("nowrap".into(), "".into())]).unwrap();
        assert!(classes.contains(&"sail-nowrap".to_string()), "nowrap class: {classes:?}");
        assert!(
            css.iter().any(|r| r == ".sail-nowrap{flex-wrap:nowrap}"),
            "nowrap override rule: {css:?}"
        );
        // The rule is injected AFTER the base `.sail-row{…flex-wrap:wrap…}`
        // so source order gives nowrap the win.
        let base_i = css.iter().position(|r| r.starts_with(".sail-row{")).unwrap();
        let nowrap_i = css.iter().position(|r| r == ".sail-nowrap{flex-wrap:nowrap}").unwrap();
        assert!(nowrap_i > base_i, "nowrap must follow the base rule: {css:?}");
    }

    #[test]
    fn nowrap_composes_with_inline() {
        // `<row inline nowrap>` — inline base swap AND the wrap override,
        // both applied.
        let (classes, css) =
            lower("row", &[("inline".into(), "".into()), ("nowrap".into(), "".into())]).unwrap();
        assert_eq!(classes[0], "sail-row-i", "inline base: {classes:?}");
        assert!(classes.contains(&"sail-nowrap".to_string()), "nowrap class: {classes:?}");
        assert!(css.iter().any(|r| r == ".sail-nowrap{flex-wrap:nowrap}"));
    }

    #[test]
    fn plain_row_has_no_nowrap_class() {
        // No implicit override — a plain `<row>` keeps its default wrap.
        let (classes, _) = lower("row", &[]).unwrap();
        assert!(!classes.iter().any(|c| c == "sail-nowrap"), "{classes:?}");
    }

    #[test]
    fn rejects_nowrap_on_a_tag_that_doesnt_take_it() {
        // `nowrap` is `<row>`-only — a column doesn't wrap, a grid isn't flex.
        let m = run(r#"<column nowrap>x</column>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("<column>") && m[0].contains("nowrap"), "got: {}", m[0]);
    }

    #[test]
    fn rejects_nowrap_with_a_value() {
        let m = run(r#"<row nowrap="yes">x</row>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("flag, not a value"), "got: {}", m[0]);
    }

    #[test]
    fn rejects_dynamic_nowrap() {
        let m = run(r#"<row nowrap={cond()}>x</row>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("structural axis"), "got: {}", m[0]);
    }

    // --- `<grid rows="N">` explicit row count (job/layout-pass known gap #2) -

    #[test]
    fn accepts_grid_rows_static_and_dynamic() {
        assert!(run(r#"<grid rows="3" gap="4">x</grid>"#).is_empty());
        assert!(run(r#"<grid rows={n()}>x</grid>"#).is_empty());
    }

    #[test]
    fn lowers_grid_rows_to_repeat_minmax() {
        let (classes, css) = lower("grid", &[("rows".into(), "3".into())]).unwrap();
        assert!(classes.contains(&"sail-gr-3".to_string()), "rows class: {classes:?}");
        assert!(
            css.iter().any(|r| r.contains("grid-template-rows:repeat(3,minmax(0,1fr))")),
            "rows track rule: {css:?}"
        );
    }

    #[test]
    fn rows_composes_with_cols() {
        // A fixed N×M grid — `cols` owns the columns, `rows` owns the rows;
        // they never collide (different CSS properties), so unlike `cols`/`min`
        // there is no precedence to resolve.
        let (classes, _css) =
            lower("grid", &[("cols".into(), "4".into()), ("rows".into(), "3".into())]).unwrap();
        assert!(classes.contains(&"sail-gc-4".to_string()), "cols kept: {classes:?}");
        assert!(classes.contains(&"sail-gr-3".to_string()), "rows kept: {classes:?}");
    }

    #[test]
    fn rejects_off_enum_rows() {
        let m = run(r#"<grid rows="13">x</grid>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("13") && m[0].contains("not valid"), "got: {}", m[0]);
    }

    #[test]
    fn rejects_rows_on_a_tag_that_doesnt_take_it() {
        let m = run(r#"<row rows="3">x</row>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("<row>") && m[0].contains("rows"), "got: {}", m[0]);
    }

    // --- `justify` on `<column>` + `justify="evenly"` ----------------------

    #[test]
    fn accepts_justify_on_column() {
        // The column's MAIN (vertical) axis — a full-height column pinning
        // first-to-top / last-to-bottom.
        assert!(run(r#"<column justify="between">x</column>"#).is_empty());
    }

    #[test]
    fn lowers_column_justify_and_has_no_default() {
        let (classes, css) = lower("column", &[("justify".into(), "between".into())]).unwrap();
        assert!(classes.contains(&"sail-jc-between".to_string()), "{classes:?}");
        assert!(css.iter().any(|r| r.contains("justify-content:space-between")), "{css:?}");
        // NO default — a plain `<column>` emits no justify class (no output
        // churn on existing columns).
        let (classes, _) = lower("column", &[]).unwrap();
        assert!(!classes.iter().any(|c| c.starts_with("sail-jc-")), "{classes:?}");
    }

    #[test]
    fn accepts_and_lowers_justify_evenly() {
        assert!(run(r#"<row justify="evenly">x</row>"#).is_empty());
        let (classes, css) = lower("row", &[("justify".into(), "evenly".into())]).unwrap();
        assert!(classes.contains(&"sail-jc-evenly".to_string()), "{classes:?}");
        assert!(css.iter().any(|r| r.contains("justify-content:space-evenly")), "{css:?}");
    }

    #[test]
    fn the_compound_refusal_now_recommends_a_buildable_element() {
        // THE MISSION: a refusal must point at something the author can
        // actually write. Before this job, `<div sx="inline-flex">` produced
        // a suggestion (`<row inline>`) that did not compile — the compiler
        // recommended an unbuildable element. This test pins both halves:
        // the suggestion is still emitted, and the suggested element is now
        // in the layout vocabulary.
        //
        // We assert via the lowerer's per-tag prop list (the source of truth
        // the compiler consults) — if `<row>` ever loses `inline`, the
        // suggestion goes stale again and the test catches it.
        assert!(
            tag_props("row").contains(&"inline"),
            "<row> must still take `inline` so the sx refusal's suggestion compiles"
        );
        assert!(
            tag_props("grid").contains(&"inline"),
            "<grid> must still take `inline` so the sx refusal's suggestion compiles"
        );
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
