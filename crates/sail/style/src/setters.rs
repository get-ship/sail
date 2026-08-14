//! CSS lower helpers and clash emit.
//!
//! Product names live on `vocab.rs`. Concern / axis emit lives on
//! `concerns.rs`. Plains emit lives on `plains.rs`. This module lowers
//! after `find_vocab` says Plain. CSS `when` targets live in `sail-style-css`.

use crate::scales;

use super::ir::{PaintArg, PaintOp, StyleDiagnostic};
use crate::plains::{PlainSpec, find_plain};

/// Plains emit after vocab confirms Plain. Not a name atlas.
fn emit_plain(name: &str) -> Option<&'static PlainSpec> {
    let row = crate::vocab::find_vocab(name)?;
    if row.tag != crate::vocab::VocabTag::Plain {
        return None;
    }
    find_plain(row.name)
}

/// Unique closest product setter for typo correction (confluent only).
/// Returns `None` when no candidate is close enough or two candidates tie.
///
/// Ranking: lower Levenshtein first; then prefer names that are a prefix of the
/// typo (or vice versa) so `padd` → `pad` beats `padX` at the same distance;
/// then shorter absolute length delta. Exact ties remain non-confluent.
pub fn closest_unique_setter(typo: &str) -> Option<&'static str> {
    let typo_l = typo.to_ascii_lowercase();
    // score: (distance, prefix_penalty 0|1, len_delta)
    let mut best: Option<(usize, u8, usize, &'static str)> = None;
    let mut tied = false;
    for row in crate::vocab::vocab_rows() {
        let name = row.name;
        let name_l = name.to_ascii_lowercase();
        let cap = std::cmp::max(2, name.len() / 2 + 1);
        let d = setter_levenshtein(&typo_l, &name_l, cap + 1);
        if d > cap || d == 0 {
            continue;
        }
        let prefix_ok = typo_l.starts_with(&name_l) || name_l.starts_with(&typo_l);
        let prefix_penalty: u8 = if prefix_ok { 0 } else { 1 };
        let len_delta = typo_l.len().abs_diff(name_l.len());
        let score = (d, prefix_penalty, len_delta, name);
        match best {
            None => {
                best = Some(score);
                tied = false;
            }
            Some(b) => {
                let b_key = (b.0, b.1, b.2);
                let s_key = (score.0, score.1, score.2);
                if s_key < b_key {
                    best = Some(score);
                    tied = false;
                } else if s_key == b_key {
                    tied = true;
                }
            }
        }
    }
    if tied {
        return None;
    }
    best.map(|(_, _, _, n)| n)
}

fn setter_levenshtein(a: &str, b: &str, cap: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len().min(cap);
    }
    if b.is_empty() {
        return a.len().min(cap);
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        let mut row_min = curr[0];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            if curr[j] < row_min {
                row_min = curr[j];
            }
        }
        if row_min > cap {
            return cap;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()].min(cap)
}

/// Paint slots THIS op writes. Concern ops key on `(concern, kind)` so a
/// base stroke and a focus ring coexist; two ring calls in one scope clash.
/// Markers still key on CSS longhands.
pub fn op_slots(op: &PaintOp) -> Vec<&'static str> {
    if crate::concerns::find_concern(&op.setter).is_some() {
        return crate::concerns::concern_clash_keys(op).unwrap_or_default();
    }
    let Some(spec) = emit_plain(&op.setter) else {
        return Vec::new();
    };
    // Composite border: position 0 = width slots, position 1 = color slots.
    if op.args.len() == 2 && is_border_composite(spec.name) {
        return slots_part(spec, "width")
            .into_iter()
            .chain(slots_part(spec, "color"))
            .collect();
    }
    if is_border_composite(spec.name) {
        return match op.first_arg() {
            // bind / bound group paths resolve through the color entry.
            PaintArg::Bind(_) | PaintArg::MemberPath { .. } | PaintArg::Slots(_) => {
                slots_part(spec, "color")
            }
            PaintArg::Token(tok) => {
                let bare = strip_neg(tok).1;
                if scales::contains("border-width", bare)
                    || (spec.name == "border" && bare == "control-stroke")
                {
                    slots_part(spec, "width")
                } else if matches!(
                    bare,
                    "solid" | "dashed" | "dotted" | "double" | "hidden" | "none"
                ) {
                    slots_part(spec, "style")
                } else {
                    slots_part(spec, "color")
                }
            }
        };
    }
    match spec.name {
        "outline" => {
            return match op.first_arg() {
                PaintArg::Bind(_) | PaintArg::MemberPath { .. } | PaintArg::Slots(_) => {
                    vec!["outline-color"]
                }
                PaintArg::Token(tok) => {
                    let bare = strip_neg(tok).1;
                    if bare == "ring-width" || scales::contains("border-width", bare) {
                        vec!["outline-width"]
                    } else if matches!(bare, "none" | "solid" | "dashed" | "dotted" | "double") {
                        vec!["outline-style"]
                    } else {
                        vec!["outline-color"]
                    }
                }
            };
        }
        "ring" => {
            return match op.first_arg() {
                PaintArg::Token(tok) if strip_neg(tok).1 == "none" => vec!["outline-style"],
                _ => vec!["outline-color"],
            };
        }
        "bg" => {
            return match op.first_arg() {
                PaintArg::Token(tok) if strip_neg(tok).1 == "none" => vec!["background-image"],
                _ => vec!["background-color"],
            };
        }
        _ => {}
    }
    spec.slots.to_vec()
}

/// The subset of a setter's slots matching one part (`width` / `color` /
/// `style`) — slot longhands carry the part name (`border-left-color`).
fn slots_part(spec: &PlainSpec, part: &str) -> Vec<&'static str> {
    spec.slots
        .iter()
        .copied()
        .filter(|s| s.contains(part))
        .collect()
}

/// Validate paint arg against setter table; fail closed.
pub fn validate_op(op: &PaintOp) -> Result<(), StyleDiagnostic> {
    if let Some(d) = crate::concerns::retired_diag(&op.setter, &op.source) {
        return Err(d);
    }
    if let Some(concern) = crate::concerns::find_concern(&op.setter) {
        return crate::concerns::validate_concern_op(concern, op);
    }
    let Some(spec) = emit_plain(&op.setter) else {
        return Err(StyleDiagnostic::UnknownSetter {
            setter: op.setter.clone(),
            source: op.source.clone(),
            suggestion: closest_unique_setter(&op.setter).map(|s| s.to_string()),
        });
    };
    // Marker setters with empty arg. `truncate` is retired — type.wrap.
    if matches!(
        spec.name,
        "group" | "scrollbarThin" | "isolate" | "srOnly" | "controlFrame" | "controlLift" | "press"
    ) {
        return Ok(());
    }
    // Composite positional setters: style.border(1, border) = width + color.
    if op.args.len() > 1 {
        return validate_composite_op(spec, op);
    }
    match op.first_arg() {
        PaintArg::Bind(_) => Ok(()),
        // Bound paths validated at parse; emit expands before lower.
        PaintArg::MemberPath { .. } => Ok(()),
        PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: op.first_arg().display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg && !spec.negatable {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
            // SPEC-D1 / THEMES-1-ENFORCE: component surface public names are
            // emit labels only — not Style author Tokens. Host units use
            // dotted MemberPath. App Style gets a theme-owned teaching error.
            if spec.scale == Some("color")
                && crate::component_surfaces::is_component_public_name(bare)
            {
                return Err(StyleDiagnostic::ThemeOwnedToken {
                    name: tok.clone(),
                    source: op.source.clone(),
                });
            }
            if spec.keywords.contains(&bare) {
                return Ok(());
            }
            if let Some(scale) = spec.scale {
                if scales::contains(scale, bare) {
                    return Ok(());
                }
                if scale == "color"
                    || scale == "font"
                    || scale == "weight"
                    || scale == "text"
                    || scale == "duration"
                    || scale == "ease"
                    || scale == "space"
                    || scale == "size"
                    || scale == "z"
                    || scale == "shadow"
                    || scale == "leading"
                    || scale == "tracking"
                    || scale == "radius"
                {
                    if lower_op_css(op).is_ok() {
                        return Ok(());
                    }
                }
            } else if lower_op_css(op).is_ok() {
                return Ok(());
            }
            if lower_op_css(op).is_ok() {
                return Ok(());
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Lower a PaintOp to CSS declarations via sx vocab (table-driven).
pub fn lower_op_css(op: &PaintOp) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    if let Some(d) = crate::concerns::retired_diag(&op.setter, &op.source) {
        return Err(d);
    }
    if let Some(concern) = crate::concerns::find_concern(&op.setter) {
        return crate::concerns::lower_concern_op(concern, op);
    }
    let Some(spec) = emit_plain(&op.setter) else {
        return Err(StyleDiagnostic::UnknownSetter {
            setter: op.setter.clone(),
            source: op.source.clone(),
            suggestion: closest_unique_setter(&op.setter).map(|s| s.to_string()),
        });
    };
    // group: host marker only — no CSS declarations.
    if spec.name == "group" {
        return Ok(Vec::new());
    }
    // Phase 3 gap keys — special-case CSS (filter longhands, clip/mask, 3D, cq).
    if let Some(pairs) = lower_gap_key_css(spec.name, op) {
        return pairs;
    }
    // isolation stacking root.
    if spec.name == "isolate" {
        return Ok(vec![("isolation".into(), "isolate".into())]);
    }
    // scrollbar-thin bare composite (width + color vars).
    if spec.name == "scrollbarThin" {
        return Ok(vec![
            ("scrollbar-width".into(), "thin".into()),
            (
                "scrollbar-color".into(),
                "var(--sx-scrollbar-thumb, var(--border)) var(--sx-scrollbar-track, transparent)"
                    .into(),
            ),
        ]);
    }
    // truncate bare composite.
    if spec.name == "truncate" {
        return Ok(vec![
            ("overflow".into(), "hidden".into()),
            ("text-overflow".into(), "ellipsis".into()),
            ("white-space".into(), "nowrap".into()),
        ]);
    }
    // sr-only a11y clip pack.
    if spec.name == "srOnly" {
        return Ok(vec![
            ("position".into(), "absolute".into()),
            ("width".into(), "1px".into()),
            ("height".into(), "1px".into()),
            ("padding".into(), "0".into()),
            ("margin".into(), "-1px".into()),
            ("overflow".into(), "hidden".into()),
            ("clip-path".into(), "inset(50%)".into()),
            ("white-space".into(), "nowrap".into()),
            ("border-width".into(), "0".into()),
        ]);
    }
    // Composite border: width + color in one call (style.border(1, border)).
    if op.args.len() > 1 {
        if op.args.len() != 2 {
            return Err(StyleDiagnostic::BadArity {
                setter: op.setter.clone(),
                got: op.args.len(),
                source: op.source.clone(),
            });
        }
        if is_border_composite(spec.name) {
            let mut out = lower_border_width_arg(spec, op, &op.args[0])?;
            out.extend(lower_border_color_arg(spec, op, &op.args[1])?);
            return Ok(out);
        }
        if spec.name == "colorDeepen" {
            return lower_color_deepen_arg(spec, op, &op.args[0], &op.args[1]);
        }
        if spec.name == "colorMix" {
            return lower_color_mix_wash_arg(spec, op, &op.args[0], &op.args[1]);
        }
        return Err(StyleDiagnostic::BadArity {
            setter: op.setter.clone(),
            got: op.args.len(),
            source: op.source.clone(),
        });
    }
    let arg = op.first_arg();
    // visibility keywords.
    if spec.name == "visibility" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css = match bare_t {
                "hidden" => "hidden",
                "visible" => "visible",
                "collapse" => "collapse",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            return Ok(vec![("visibility".into(), css.into())]);
        }
    }
    // align() is vertical-align. type({ align }) is text-align.
    if spec.name == "align" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if !spec.keywords.contains(&bare_t) {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare_t.into(),
                    source: op.source.clone(),
                });
            }
            return Ok(vec![("vertical-align".into(), bare_t.into())]);
        }
    }
    // aspect-ratio map.
    if spec.name == "aspect" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css = match bare_t {
                "square" => "1 / 1",
                "video" => "16 / 9",
                "auto" => "auto",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            return Ok(vec![("aspect-ratio".into(), css.into())]);
        }
    }
    // object-fit.
    if spec.name == "objectFit" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if !spec.keywords.contains(&bare_t) {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare_t.into(),
                    source: op.source.clone(),
                });
            }
            return Ok(vec![("object-fit".into(), bare_t.into())]);
        }
    }
    // transform full value (token none or bind → var).
    if spec.name == "transform" {
        match arg {
            PaintArg::Token(t) => {
                let bare_t = strip_neg(t).1;
                if bare_t == "none" {
                    return Ok(vec![("transform".into(), "none".into())]);
                }
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare_t.into(),
                    source: op.source.clone(),
                });
            }
            PaintArg::Bind(name) => {
                let css = css_name_for_bind(name);
                return Ok(vec![("transform".into(), format!("var(--{css})"))]);
            }
            PaintArg::MemberPath { .. } | PaintArg::Slots(_) => {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: arg.display(),
                    source: op.source.clone(),
                });
            }
        }
    }
    // italic / not-italic.
    if spec.name == "italic" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css = match bare_t {
                "true" | "on" => "italic",
                "false" | "off" => "normal",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            return Ok(vec![("font-style".into(), css.into())]);
        }
    }
    // text-transform via textCase(uppercase|…)
    if spec.name == "textCase" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css = match bare_t {
                "uppercase" => "uppercase",
                "lowercase" => "lowercase",
                "capitalize" => "capitalize",
                "normal" => "none",
                // Casing channel: label-style text reads --label-transform so
                // themes govern casing (a literal silently beats the theme).
                "label-transform" => "var(--label-transform)",
                _ => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare_t.into(),
                        source: op.source.clone(),
                    });
                }
            };
            return Ok(vec![("text-transform".into(), css.into())]);
        }
    }
    // font-variant-numeric via nums(tabular|…)
    if spec.name == "nums" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css_val = match bare_t {
                "tabular" => "tabular-nums",
                "lining" => "lining-nums",
                "oldstyle" => "oldstyle-nums",
                "proportional" => "proportional-nums",
                "ordinal" => "ordinal",
                "slashed-zero" => "slashed-zero",
                "normal" => "normal",
                _ => bare_t,
            };
            return Ok(vec![("font-variant-numeric".into(), css_val.into())]);
        }
    }
    // transform-origin via origin(bottom|…)
    if spec.name == "origin" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            let css_val = bare_t.replace('-', " ");
            return Ok(vec![("transform-origin".into(), css_val)]);
        }
    }
    // text-gradient-* composite
    if spec.name == "textGradient" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if matches!(bare_t, "primary" | "accent") {
                let bg = match bare_t {
                    "primary" => "var(--gradient-primary)",
                    "accent" => "var(--gradient-accent)",
                    _ => unreachable!(),
                };
                return Ok(vec![
                    ("background".into(), bg.into()),
                    ("-webkit-background-clip".into(), "text".into()),
                    ("background-clip".into(), "text".into()),
                    ("color".into(), "transparent".into()),
                ]);
            }
        }
    }
    // bg-glow-* composite
    if spec.name == "bgGlow" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if matches!(bare_t, "primary" | "accent") {
                let bg = match bare_t {
                    "primary" => "var(--glow-primary)",
                    "accent" => "var(--glow-accent)",
                    _ => unreachable!(),
                };
                return Ok(vec![
                    ("position".into(), "absolute".into()),
                    ("inset".into(), "-4rem 0 0".into()),
                    ("background".into(), bg.into()),
                    ("pointer-events".into(), "none".into()),
                ]);
            }
        }
    }
    // Host-paint gap props for flex/grid containers outside layout keywords.
    if matches!(spec.name, "gap" | "gapX" | "gapY") {
        let value = match arg {
            PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
            PaintArg::Token(t) => {
                let bare = strip_neg(t).1;
                if bare == "0" {
                    "0".into()
                } else {
                    scales::lower("space", bare).ok_or_else(|| StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: t.clone(),
                        source: op.source.clone(),
                    })?
                }
            }
            PaintArg::MemberPath { .. } | PaintArg::Slots(_) => {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: arg.display(),
                    source: op.source.clone(),
                });
            }
        };
        let prop = match spec.name {
            "gap" => "gap",
            "gapX" => "column-gap",
            "gapY" => "row-gap",
            _ => unreachable!(),
        };
        return Ok(vec![(prop.into(), value)]);
    }
    // Tier-2 control chrome / press marker channels.
    if spec.name == "controlFrame" {
        return Ok(vec![("box-shadow".into(), "var(--control-frame)".into())]);
    }
    if spec.name == "controlLift" {
        return Ok(vec![("translate".into(), "var(--control-lift)".into())]);
    }
    if spec.name == "press" {
        return Ok(vec![
            ("translate".into(), "var(--press-translate)".into()),
            ("scale".into(), "var(--press-scale)".into()),
            ("box-shadow".into(), "var(--press-shadow)".into()),
        ]);
    }
    let (token, bare, negative) = match arg {
        PaintArg::Token(t) => {
            let (neg, bare) = strip_neg(t);
            (Some(bare.to_string()), bare.is_empty(), neg)
        }
        PaintArg::Bind(name) => {
            let css = css_name_for_bind(name);
            return Ok(bind_css_props(spec, &css));
        }
        // Bound group paths (`tone`, `tone.foreground`) → bridge vars.
        // Host reflects `--tone: var(--primary)` from the color-group typed prop.
        PaintArg::MemberPath { root, path } => {
            let css = crate::token_groups::bridge_var_name(root, path);
            return Ok(bind_css_props(spec, &css));
        }
        PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: arg.display(),
                source: op.source.clone(),
            });
        }
    };
    // Axis overflow / overscroll / text-align / tab-size / underline / list keywords
    if matches!(
        spec.name,
        "overflowX"
            | "overflowY"
            | "overscroll"
            | "textAlign"
            | "tab"
            | "underline"
            | "list"
            | "mixBlend"
            | "resize"
            | "contain"
            | "willChange"
    ) {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if !spec.keywords.contains(&bare_t) {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare_t.into(),
                    source: op.source.clone(),
                });
            }
            let css_prop = match spec.name {
                "overflowX" => "overflow-x",
                "overflowY" => "overflow-y",
                "overscroll" => "overscroll-behavior",
                "textAlign" => "text-align",
                "tab" => "tab-size",
                "underline" => "text-decoration-line",
                "list" => "list-style-type",
                "mixBlend" => "mix-blend-mode",
                "resize" => "resize",
                "contain" => "contain",
                "willChange" => "will-change",
                _ => spec.vocab,
            };
            let css_val = match (spec.name, bare_t) {
                ("underline", "true") => "underline",
                ("underline", "none") => "none",
                ("underline", "line-through") => "line-through",
                _ => bare_t,
            };
            return Ok(vec![(css_prop.into(), css_val.into())]);
        }
    }
    // Special-case display
    if spec.name == "display" {
        let tok = match arg {
            PaintArg::Token(t) => strip_neg(t).1,
            _ => "none",
        };
        let css = match tok {
            "none" => vec![("display".into(), "none".into())],
            "block" => vec![("display".into(), "block".into())],
            "inline" => vec![("display".into(), "inline".into())],
            "inline-block" => vec![("display".into(), "inline-block".into())],
            "flex" => vec![("display".into(), "flex".into())],
            "inline-flex" => vec![("display".into(), "inline-flex".into())],
            "contents" => vec![("display".into(), "contents".into())],
            _ => {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.into(),
                    source: op.source.clone(),
                });
            }
        };
        return Ok(css);
    }
    // Special-case position bare keywords
    if spec.name == "position" {
        let tok = match arg {
            PaintArg::Token(t) => strip_neg(t).1,
            _ => "relative",
        };
        let ok = matches!(tok, "relative" | "absolute" | "sticky" | "fixed" | "static");
        if !ok {
            return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.into(),
                source: op.source.clone(),
            });
        }
        return Ok(vec![("position".into(), tok.into())]);
    }
    // bg(none) → background-image:none (checkbox restyle clear)
    if spec.name == "bg" {
        if let PaintArg::Token(t) = arg {
            if strip_neg(t).1 == "none" {
                return Ok(vec![("background-image".into(), "none".into())]);
            }
        }
    }
    // outlineOffset special tokens
    if spec.name == "outlineOffset" {
        if let PaintArg::Token(t) = arg {
            match strip_neg(t).1 {
                "ring-offset" => {
                    return Ok(vec![("outline-offset".into(), "var(--ring-offset)".into())]);
                }
                "ring-inset" => {
                    return Ok(vec![(
                        "outline-offset".into(),
                        "calc(-1 * var(--ring-width))".into(),
                    )]);
                }
                _ => {}
            }
        }
    }
    // outline color tokens (primary, tone, ring, tone-ring, …) vs width/style keywords
    if spec.name == "outline" {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if bare_t == "ring-width" {
                return Ok(vec![("outline-width".into(), "var(--ring-width)".into())]);
            }
            if !matches!(
                bare_t,
                "none" | "solid" | "dashed" | "1" | "2" | "4" | "0" | "3" | "8"
            ) {
                for cand in token_candidates(bare_t) {
                    if let Some(v) = scales::lower("color", &cand) {
                        return Ok(vec![("outline-color".into(), v)]);
                    }
                }
            }
        }
    }
    // Dimension / composite special cases
    if let PaintArg::Token(t) = arg {
        let bare_t = strip_neg(t).1;
        match (spec.name, bare_t) {
            ("borderWidth", "control-stroke") => {
                return Ok(vec![(
                    "border-width".into(),
                    "var(--control-stroke)".into(),
                )]);
            }
            // The hairline channel: control hairlines spend --border-width,
            // never a literal px (channel-discipline ratchet, baseline 0).
            ("borderWidth", "border-width") => {
                return Ok(vec![("border-width".into(), "var(--border-width)".into())]);
            }
            ("w" | "h", "border-width") => {
                let prop = if spec.name == "w" { "width" } else { "height" };
                return Ok(vec![(prop.into(), "var(--border-width)".into())]);
            }
            ("w", "panel-toast-region-width") => {
                return Ok(vec![(
                    "width".into(),
                    "var(--panel-toast-region-width)".into(),
                )]);
            }
            // Closed product keyframe catalog (packages/sail/runtime
            // keyframes-catalog.ts). Style.animate(name) → CSS host Style var;
            // ambient @keyframes live in the generated contracts sheet,
            // not free CSS and not fx:loop. Keywords on the animate
            // animate keywords must stay in lockstep with styleAnimate:true rows.
            ("animate", name) if matches!(name, "spin") => {
                return Ok(vec![("animation".into(), format!("var(--animate-{name})"))]);
            }
            ("bgImage", "none") => {
                return Ok(vec![("background-image".into(), "none".into())]);
            }
            ("borderImage", "none") => {
                return Ok(vec![("border-image".into(), "none".into())]);
            }
            ("borderImage", "corner-image") => {
                return Ok(vec![("border-image".into(), "var(--corner-image)".into())]);
            }
            ("backdrop", "none") => {
                return Ok(vec![
                    ("backdrop-filter".into(), "none".into()),
                    ("-webkit-backdrop-filter".into(), "none".into()),
                ]);
            }
            ("backdrop", "card-backdrop-filter") => {
                return Ok(vec![
                    (
                        "backdrop-filter".into(),
                        "var(--card-backdrop-filter)".into(),
                    ),
                    (
                        "-webkit-backdrop-filter".into(),
                        "var(--card-backdrop-filter)".into(),
                    ),
                ]);
            }
            ("bgImage", "primary-accent") => {
                return Ok(vec![(
                    "background-image".into(),
                    "linear-gradient(135deg, var(--primary), var(--accent))".into(),
                )]);
            }
            ("w", "max") => {
                return Ok(vec![("width".into(), "max-content".into())]);
            }
            ("w", "panel-max-w-md") => {
                return Ok(vec![("width".into(), "var(--panel-max-w-md)".into())]);
            }
            ("w", "panel-max-w-sm") => {
                return Ok(vec![("width".into(), "var(--panel-max-w-sm)".into())]);
            }
            ("w", "panel-max-w-lg") => {
                return Ok(vec![("width".into(), "var(--panel-max-w-lg)".into())]);
            }
            ("w", "panel-max-w-xs") => {
                return Ok(vec![("width".into(), "var(--panel-max-w-xs)".into())]);
            }
            ("maxW", "panel-max-w-md") => {
                return Ok(vec![("max-width".into(), "var(--panel-max-w-md)".into())]);
            }
            ("maxW", "panel-max-w-sm") => {
                return Ok(vec![("max-width".into(), "var(--panel-max-w-sm)".into())]);
            }
            ("maxW", "panel-max-w-lg") => {
                return Ok(vec![("max-width".into(), "var(--panel-max-w-lg)".into())]);
            }
            ("maxW", "panel-max-w-xs") => {
                return Ok(vec![("max-width".into(), "var(--panel-max-w-xs)".into())]);
            }
            ("maxW", "panel-ch-w-md") => {
                return Ok(vec![("max-width".into(), "var(--panel-ch-w-md)".into())]);
            }
            ("maxW", "panel-ch-w-lg") => {
                return Ok(vec![("max-width".into(), "var(--panel-ch-w-lg)".into())]);
            }
            ("dir", dir @ ("row" | "column" | "row-reverse" | "column-reverse")) => {
                return Ok(vec![("flex-direction".into(), dir.into())]);
            }
            ("minW", "panel-menu-min-w") => {
                return Ok(vec![("min-width".into(), "var(--panel-menu-min-w)".into())]);
            }
            ("minW", "panel-rail-w") => {
                return Ok(vec![("min-width".into(), "var(--panel-rail-w)".into())]);
            }
            ("textAlign", align @ ("left" | "center" | "right" | "justify" | "start" | "end")) => {
                return Ok(vec![("text-align".into(), align.into())]);
            }
            ("break", "words") => {
                return Ok(vec![
                    ("overflow-wrap".into(), "anywhere".into()),
                    ("word-break".into(), "normal".into()),
                ]);
            }
            ("break", "all") => {
                return Ok(vec![
                    ("overflow-wrap".into(), "normal".into()),
                    ("word-break".into(), "break-all".into()),
                ]);
            }
            ("break", "keep") => {
                return Ok(vec![
                    ("overflow-wrap".into(), "normal".into()),
                    ("word-break".into(), "keep-all".into()),
                ]);
            }
            ("break", "normal") => {
                return Ok(vec![
                    ("overflow-wrap".into(), "normal".into()),
                    ("word-break".into(), "normal".into()),
                ]);
            }
            ("stroke", "current" | "currentColor") => {
                return Ok(vec![("stroke".into(), "currentColor".into())]);
            }
            ("stroke", "none") => {
                return Ok(vec![("stroke".into(), "none".into())]);
            }
            ("fill", "current" | "currentColor") => {
                return Ok(vec![("fill".into(), "currentColor".into())]);
            }
            ("fill", "none") => {
                return Ok(vec![("fill".into(), "none".into())]);
            }
            _ => {}
        }
    }
    if matches!(
        spec.name,
        "borderWidth"
            | "borderBWidth"
            | "borderTWidth"
            | "borderEWidth"
            | "borderSWidth"
            | "borderRWidth"
            | "borderLWidth"
    ) {
        if let PaintArg::Token(t) = arg {
            let bare_t = strip_neg(t).1;
            if bare_t == "1" || bare_t.is_empty() {
                let props = border_props(spec.name, "width");
                return Ok(props.into_iter().map(|p| (p, "1px".into())).collect());
            }
        }
    }
    if let Some(ref t) = token {
        for cand in token_candidates(t) {
            if let Some(pairs) = crate::plains::lower_plain_token(spec, &cand, negative) {
                return Ok(pairs);
            }
        }
    } else if bare {
        if let Some(pairs) = crate::plains::lower_plain_token(spec, "", negative) {
            return Ok(pairs);
        }
    }
    Err(StyleDiagnostic::BadToken {
        setter: op.setter.clone(),
        value: arg.display(),
        source: op.source.clone(),
    })
}

/// Clip / filter / 3D / container-query product setters → real CSS.
fn lower_gap_key_css(
    name: &str,
    op: &PaintOp,
) -> Option<Result<Vec<(String, String)>, StyleDiagnostic>> {
    let arg = op.first_arg();
    let (neg, bare) = match arg {
        PaintArg::Token(t) => strip_neg(t),
        PaintArg::Bind(n) => {
            let var = format!("var(--{})", n.replace('_', "-"));
            return Some(Ok(match name {
                "clipPath" => vec![("clip-path".into(), var)],
                "mask" | "maskImage" => vec![("mask-image".into(), var)],
                "blur" | "brightness" | "contrast" | "grayscale" | "hueRotate" | "invert"
                | "sepia" | "saturate" => {
                    let fn_name = match name {
                        "hueRotate" => "hue-rotate",
                        other => other,
                    };
                    vec![("filter".into(), format!("{fn_name}({var})"))]
                }
                "perspective" => vec![("perspective".into(), var)],
                "rotateX" => vec![("transform".into(), format!("rotateX({var})"))],
                "rotateY" => vec![("transform".into(), format!("rotateY({var})"))],
                "rotateZ" => vec![("transform".into(), format!("rotateZ({var})"))],
                "containerType" | "container" => vec![("container-type".into(), var)],
                "containerName" => vec![("container-name".into(), var)],
                _ => return None,
            }));
        }
        PaintArg::MemberPath { root, path } => {
            // Only gap-key setters consume this arm; color setters fall through
            // to bridge-var lowering in lower_op_css.
            let gap_keys = matches!(
                name,
                "clipPath"
                    | "mask"
                    | "maskImage"
                    | "blur"
                    | "brightness"
                    | "contrast"
                    | "grayscale"
                    | "hueRotate"
                    | "invert"
                    | "sepia"
                    | "saturate"
                    | "perspective"
                    | "rotateX"
                    | "rotateY"
                    | "rotateZ"
                    | "containerType"
                    | "container"
                    | "containerName"
            );
            if !gap_keys {
                return None;
            }
            let var = crate::token_groups::bridge_var_ref(root, path);
            return Some(Ok(match name {
                "clipPath" => vec![("clip-path".into(), var)],
                "mask" | "maskImage" => vec![("mask-image".into(), var)],
                "blur" | "brightness" | "contrast" | "grayscale" | "hueRotate" | "invert"
                | "sepia" | "saturate" => {
                    let fn_name = match name {
                        "hueRotate" => "hue-rotate",
                        other => other,
                    };
                    vec![("filter".into(), format!("{fn_name}({var})"))]
                }
                "perspective" => vec![("perspective".into(), var)],
                "rotateX" => vec![("transform".into(), format!("rotateX({var})"))],
                "rotateY" => vec![("transform".into(), format!("rotateY({var})"))],
                "rotateZ" => vec![("transform".into(), format!("rotateZ({var})"))],
                "containerType" | "container" => vec![("container-type".into(), var)],
                "containerName" => vec![("container-name".into(), var)],
                _ => return None,
            }));
        }
        PaintArg::Slots(_) => return None,
    };
    let sign = if neg { "-" } else { "" };
    Some(Ok(match name {
        "clipPath" => {
            let css = match bare {
                "none" => "none".into(),
                "circle" => "circle()".into(),
                "ellipse" => "ellipse()".into(),
                "inset-0" => "inset(0)".into(),
                "border-box" | "padding-box" | "content-box" | "fill-box" | "stroke-box"
                | "view-box" | "margin-box" => bare.into(),
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("clip-path".into(), css)]
        }
        "mask" | "maskImage" => {
            let css = match bare {
                "none" => "none".into(),
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("mask-image".into(), css)]
        }
        "blur" => {
            let px = match bare {
                "0" | "1" | "2" | "4" | "8" | "12" | "16" | "24" => bare,
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("filter".into(), format!("blur({px}px)"))]
        }
        "brightness" | "contrast" | "saturate" => {
            let pct = match bare {
                "0" | "50" | "60" | "65" | "75" | "90" | "95" | "100" | "105" | "110" | "125"
                | "150" | "200" => bare,
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("filter".into(), format!("{name}({pct}%)"))]
        }
        "grayscale" | "invert" | "sepia" => {
            let pct = match bare {
                "0" | "50" | "100" => bare,
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("filter".into(), format!("{name}({pct}%)"))]
        }
        "hueRotate" => {
            let deg = match bare {
                "0" | "15" | "30" | "60" | "90" | "180" => bare,
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("filter".into(), format!("hue-rotate({sign}{deg}deg)"))]
        }
        "perspective" => {
            let css = match bare {
                "none" => "none".into(),
                "0" => "0px".into(),
                "250" | "500" | "750" | "1000" | "1500" | "2000" => format!("{bare}px"),
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("perspective".into(), css)]
        }
        "rotateX" | "rotateY" | "rotateZ" => {
            let axis = match name {
                "rotateX" => "X",
                "rotateY" => "Y",
                _ => "Z",
            };
            // Angle scale tokens (e.g. 45 → 45deg) or 0.
            let deg = if bare == "0" {
                "0deg".into()
            } else if bare.chars().all(|c| c.is_ascii_digit()) {
                format!("{sign}{bare}deg")
            } else if let Some(v) = scales::lower("angle", bare) {
                if neg { format!("calc(-1 * {v})") } else { v }
            } else {
                return Some(Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: bare.into(),
                    source: op.source.clone(),
                }));
            };
            vec![("transform".into(), format!("rotate{axis}({deg})"))]
        }
        "containerType" | "container" => {
            let css = match bare {
                "normal" | "size" | "inline-size" => bare.into(),
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("container-type".into(), css)]
        }
        "containerName" => {
            let css = match bare {
                "none" => "none".into(),
                // Ident-like custom names (closed enough: [a-z0-9-]+).
                t if t
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                    && !t.is_empty() =>
                {
                    t.into()
                }
                _ => {
                    return Some(Err(StyleDiagnostic::BadToken {
                        setter: op.setter.clone(),
                        value: bare.into(),
                        source: op.source.clone(),
                    }));
                }
            };
            vec![("container-name".into(), css)]
        }
        _ => return None,
    }))
}

/// Setters that take two positional args: (width-token?, color-token).
/// One arg stays the current color/width/style-by-value behavior.
fn is_border_composite(name: &str) -> bool {
    matches!(
        name,
        "border"
            | "borderX"
            | "borderY"
            | "borderT"
            | "borderB"
            | "borderL"
            | "borderR"
            | "borderS"
            | "borderE"
    )
}

/// Validate a composite (two-arg) paint op positionally.
fn validate_composite_op(spec: &PlainSpec, op: &PaintOp) -> Result<(), StyleDiagnostic> {
    if op.args.len() != 2 {
        return Err(StyleDiagnostic::BadArity {
            setter: op.setter.clone(),
            got: op.args.len(),
            source: op.source.clone(),
        });
    }
    if is_border_composite(spec.name) {
        validate_border_width_arg(spec, op, &op.args[0])?;
        validate_border_color_arg(spec, op, &op.args[1])?;
        return Ok(());
    }
    if spec.name == "colorDeepen" || spec.name == "colorMix" {
        // Same arity/token rules: color token + closed percent ramp.
        return validate_color_deepen_args(spec, op);
    }
    Err(StyleDiagnostic::BadArity {
        setter: op.setter.clone(),
        got: op.args.len(),
        source: op.source.clone(),
    })
}

/// Position 0 of a composite border call: a width token on the closed
/// border-width ramp (or `control-stroke` on plain `border`, mirroring
/// `borderWidth`).
fn validate_border_width_arg(
    spec: &PlainSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<(), StyleDiagnostic> {
    match arg {
        PaintArg::Bind(_) => Ok(()),
        PaintArg::MemberPath { .. } | PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if !neg
                && (scales::contains("border-width", bare)
                    || (spec.name == "border" && bare == "control-stroke"))
            {
                return Ok(());
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Position 1 of a composite border call: a color token / keyword. The
/// resolved declarations must all be `*-color` (a width or style token in
/// this position is a positional error, fail closed).
fn validate_border_color_arg(
    _spec: &PlainSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<(), StyleDiagnostic> {
    match arg {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => Ok(()),
        PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if !neg
                && (matches!(bare, "transparent" | "current" | "currentColor")
                    || scales::contains("color", bare))
            {
                return Ok(());
            }
            if !neg {
                for cand in token_candidates(bare) {
                    if scales::contains("color", &cand) {
                        return Ok(());
                    }
                }
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// CSS longhands a border-family setter writes for one part (`width` /
/// `color` / `style`).
fn border_props(name: &str, part: &str) -> Vec<String> {
    let bases: &[&str] = match name {
        "borderX" => &["border-left", "border-right"],
        "borderY" => &["border-top", "border-bottom"],
        "borderT" => &["border-top"],
        "borderB" => &["border-bottom"],
        "borderL" => &["border-left"],
        "borderR" => &["border-right"],
        "borderS" => &["border-inline-start"],
        "borderE" => &["border-inline-end"],
        _ => &["border"],
    };
    bases.iter().map(|b| format!("{b}-{part}")).collect()
}

/// Lower composite position 0 (width) to declarations.
fn lower_border_width_arg(
    spec: &PlainSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    match arg {
        PaintArg::Bind(name) => {
            let var = format!("var(--{})", css_name_for_bind(name));
            Ok(border_props(spec.name, "width")
                .into_iter()
                .map(|p| (p, var.clone()))
                .collect())
        }
        PaintArg::MemberPath { .. } | PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let bare = strip_neg(tok).1;
            if spec.name == "border" && bare == "control-stroke" {
                return Ok(vec![(
                    "border-width".into(),
                    "var(--control-stroke)".into(),
                )]);
            }
            let css = if bare == "1" || bare.is_empty() {
                "1px".to_string()
            } else if let Some(v) = scales::lower("border-width", bare) {
                v
            } else if let Some(v) = scales::lower("space", bare) {
                v
            } else {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            };
            Ok(border_props(spec.name, "width")
                .into_iter()
                .map(|p| (p, css.clone()))
                .collect())
        }
    }
}

/// Lower composite position 1 (color) to declarations.
fn lower_border_color_arg(
    spec: &PlainSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    match arg {
        PaintArg::Bind(name) => {
            let var = format!("var(--{})", css_name_for_bind(name));
            Ok(border_props(spec.name, "color")
                .into_iter()
                .map(|p| (p, var.clone()))
                .collect())
        }
        PaintArg::MemberPath { root, path } => {
            let var = crate::token_groups::bridge_var_ref(root, path);
            Ok(border_props(spec.name, "color")
                .into_iter()
                .map(|p| (p, var.clone()))
                .collect())
        }
        PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let bare = strip_neg(tok).1;
            let css = if matches!(bare, "transparent" | "currentColor") {
                Some(bare.to_string())
            } else if bare == "current" {
                Some("currentColor".into())
            } else {
                token_candidates(bare)
                    .into_iter()
                    .find_map(|cand| scales::lower("color", &cand))
            };
            if let Some(css) = css {
                return Ok(border_props(spec.name, "color")
                    .into_iter()
                    .map(|p| (p, css.clone()))
                    .collect());
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Validate `colorDeepen(token, pct)`: token on color scale, pct in 5..95 step 5.
fn validate_color_deepen_args(_spec: &PlainSpec, op: &PaintOp) -> Result<(), StyleDiagnostic> {
    // Position 0: color token or bind.
    match &op.args[0] {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => {}
        PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: op.args[0].display(),
                source: op.source.clone(),
            });
        }
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg
                || (!scales::contains("color", bare)
                    && !matches!(bare, "transparent" | "current" | "currentColor"))
            {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
        }
    }
    // Position 1: closed percent ramp 5..95 step 5, or bind.
    match &op.args[1] {
        PaintArg::Bind(_) => {}
        PaintArg::MemberPath { .. } | PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: op.args[1].display(),
                source: op.source.clone(),
            });
        }
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
            let ok = bare
                .parse::<i32>()
                .is_ok_and(|n| (5..=95).contains(&n) && n % 5 == 0);
            if !ok {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Lower `colorDeepen(token, pct)` to a color-mix background-color.
fn lower_color_deepen_arg(
    _spec: &PlainSpec,
    _op: &PaintOp,
    token_arg: &PaintArg,
    pct_arg: &PaintArg,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let token = match token_arg {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::Token(t) => {
            let bare = strip_neg(t).1;
            match bare {
                "transparent" | "current" | "currentColor" => bare.into(),
                _ => format!("var(--{bare})"),
            }
        }
        PaintArg::MemberPath { root, path } => crate::token_groups::bridge_var_ref(root, path),
        PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: "colorDeepen".into(),
                value: token_arg.display(),
                source: _op.source.clone(),
            });
        }
    };
    let pct = match pct_arg {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::Token(t) => t.clone(),
        PaintArg::MemberPath { .. } | PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: "colorDeepen".into(),
                value: pct_arg.display(),
                source: _op.source.clone(),
            });
        }
    };
    Ok(vec![(
        "background-color".into(),
        format!("color-mix(in oklch, {token}, var(--foreground) {pct}%)"),
    )])
}

/// Lower `colorMix(token, pct)` to a transparent hue wash background-color.
/// Emits `color-mix(in oklch, {token} {pct}%, transparent)` — the former
/// tone-*-bg shape, without reintroducing tone-* design tokens.
fn lower_color_mix_wash_arg(
    _spec: &PlainSpec,
    _op: &PaintOp,
    token_arg: &PaintArg,
    pct_arg: &PaintArg,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let token = match token_arg {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::Token(t) => {
            let bare = strip_neg(t).1;
            match bare {
                "transparent" | "current" | "currentColor" => bare.into(),
                _ => format!("var(--{bare})"),
            }
        }
        PaintArg::MemberPath { root, path } => crate::token_groups::bridge_var_ref(root, path),
        PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: "colorMix".into(),
                value: token_arg.display(),
                source: _op.source.clone(),
            });
        }
    };
    let pct = match pct_arg {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::Token(t) => t.clone(),
        PaintArg::MemberPath { .. } | PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: "colorMix".into(),
                value: pct_arg.display(),
                source: _op.source.clone(),
            });
        }
    };
    Ok(vec![(
        "background-color".into(),
        format!("color-mix(in oklch, {token} {pct}%, transparent)"),
    )])
}

fn strip_neg(tok: &str) -> (bool, &str) {
    if let Some(rest) = tok.strip_prefix('-') {
        if !rest.is_empty() {
            return (true, rest);
        }
    }
    (false, tok)
}

fn token_candidates(tok: &str) -> Vec<String> {
    let mut out = vec![tok.to_string()];
    // camelCase → kebab
    let mut kebab = String::new();
    for (i, c) in tok.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                kebab.push('-');
            }
            kebab.extend(c.to_lowercase());
        } else {
            kebab.push(c);
        }
    }
    if kebab != tok {
        out.push(kebab.clone());
    }
    out.push(tok.replace('_', "-"));
    out
}

fn css_name_for_bind(name: &str) -> String {
    let mut kebab = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                kebab.push('-');
            }
            kebab.extend(c.to_lowercase());
        } else {
            kebab.push(c);
        }
    }
    kebab
}

fn bind_css_props(spec: &PlainSpec, css_name: &str) -> Vec<(String, String)> {
    let var = format!("var(--{css_name})");
    match spec.name {
        "top" => return vec![("top".into(), var)],
        "right" => return vec![("right".into(), var)],
        "bottom" => return vec![("bottom".into(), var)],
        "left" => return vec![("left".into(), var)],
        "inset" => return vec![("inset".into(), var)],
        "ms" => return vec![("margin-inline-start".into(), var)],
        "me" => return vec![("margin-inline-end".into(), var)],
        "ml" => return vec![("margin-left".into(), var)],
        "mr" => return vec![("margin-right".into(), var)],
        "mt" => return vec![("margin-top".into(), var)],
        "mb" => return vec![("margin-bottom".into(), var)],
        "pad" => return vec![("padding".into(), var)],
        "padX" => return vec![("padding-inline".into(), var)],
        "padY" => return vec![("padding-block".into(), var)],
        "padT" => return vec![("padding-top".into(), var)],
        "padB" => return vec![("padding-bottom".into(), var)],
        "padL" => return vec![("padding-left".into(), var)],
        "padR" => return vec![("padding-right".into(), var)],
        "padS" => return vec![("padding-inline-start".into(), var)],
        "padE" => return vec![("padding-inline-end".into(), var)],
        "w" => return vec![("width".into(), var)],
        "h" => return vec![("height".into(), var)],
        "minW" => return vec![("min-width".into(), var)],
        "minH" => return vec![("min-height".into(), var)],
        "maxW" => return vec![("max-width".into(), var)],
        "maxH" => return vec![("max-height".into(), var)],
        "z" => return vec![("z-index".into(), var)],
        "opacity" => return vec![("opacity".into(), var)],
        "mixBlend" => return vec![("mix-blend-mode".into(), var)],
        "fontSize" => return vec![("font-size".into(), var)],
        "fontWeight" => return vec![("font-weight".into(), var)],
        "fontFamily" => return vec![("font-family".into(), var)],
        "tracking" => return vec![("letter-spacing".into(), var)],
        "bg" => return vec![("background-color".into(), var)],
        "fg" | "color" => return vec![("color".into(), var)],
        "transform" => return vec![("transform".into(), var)],
        "translateY" => {
            return vec![(
                "translate".into(),
                format!("var(--sx-translate-x, 0) {var}"),
            )];
        }
        "translateX" => {
            return vec![(
                "translate".into(),
                format!("{var} var(--sx-translate-y, 0)"),
            )];
        }
        "rotate" => return vec![("rotate".into(), var)],
        "scale" => return vec![("scale".into(), var)],
        "basis" => return vec![("flex-basis".into(), var)],
        "decoration" => return vec![("text-decoration-color".into(), var)],
        "caret" => return vec![("caret-color".into(), var)],
        "accent" => return vec![("accent-color".into(), var)],
        "indent" => return vec![("text-indent".into(), var)],
        "underlineOffset" => return vec![("text-underline-offset".into(), var)],
        "antialiased" => return vec![("-webkit-font-smoothing".into(), var)],
        "objectPosition" => return vec![("object-position".into(), var)],
        "order" => return vec![("order".into(), var)],
        "justifySelf" => return vec![("justify-self".into(), var)],
        "bgClip" => return vec![("background-clip".into(), var)],
        "touch" => return vec![("touch-action".into(), var)],
        "resize" => return vec![("resize".into(), var)],
        "scroll" => return vec![("scroll-behavior".into(), var)],
        "willChange" => return vec![("will-change".into(), var)],
        "contain" => return vec![("contain".into(), var)],
        "gap" => return vec![("gap".into(), var)],
        "gapX" => return vec![("column-gap".into(), var)],
        "gapY" => return vec![("row-gap".into(), var)],
        "borderImage" => return vec![("border-image".into(), var)],
        "backdrop" => {
            return vec![
                ("backdrop-filter".into(), var.clone()),
                ("-webkit-backdrop-filter".into(), var),
            ];
        }
        _ => {}
    }
    if !spec.slots.is_empty() {
        return spec
            .slots
            .iter()
            .map(|p| ((*p).to_string(), var.clone()))
            .collect();
    }
    let prop = match spec.name {
        "bg" => "background-color",
        "fg" | "color" => "color",
        "padX" => "padding-inline",
        "padY" => "padding-block",
        "radius" => "border-radius",
        "fontWeight" => "font-weight",
        "fontFamily" => "font-family",
        other => other,
    };
    vec![(prop.into(), var)]
}

#[cfg(test)]
mod tests {
    use super::super::ir::{Style, StyleDiagnostic};
    use super::*;

    /// Product parse (channel-only).
    fn parse(src: &str) -> (Style, Vec<StyleDiagnostic>) {
        super::super::parse::parse(src)
    }

    fn only_op(src: &str) -> PaintOp {
        let (style, diags) = parse(src);
        assert!(
            diags.is_empty(),
            "{src}: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        match style {
            Style::Set(op) => op,
            other => panic!("{src}: expected single Set, got {other:?}"),
        }
    }

    #[test]
    fn absorbed_border_forms_are_retired() {
        for src in ["style({ borderX(2, primary) })"] {
            let (_, diags) = parse(src);
            assert!(
                diags
                    .iter()
                    .any(|d| matches!(d, StyleDiagnostic::RetiredSetter { .. })),
                "{src}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn markers_and_concerns_lower() {
        let cases: &[(&str, &str, &str)] = &[
            ("style({ objectPosition(top) })", "object-position", "top"),
            ("style({ touch(pan-x) })", "touch-action", "pan-x"),
            (
                "style({ fg({ caret: primary }) })",
                "caret-color",
                "var(--primary)",
            ),
            ("style({ fg({ accent: auto }) })", "accent-color", "auto"),
            ("style({ position({ order: first }) })", "order", "-9999"),
            ("style({ bg({ clip: text }) })", "background-clip", "text"),
            (
                "style({ overflow({ scroll: smooth }) })",
                "scroll-behavior",
                "smooth",
            ),
        ];
        for (src, prop, val) in cases {
            let op = only_op(src);
            let css = lower_op_css(&op).unwrap();
            assert!(
                css.contains(&(prop.to_string(), val.to_string())),
                "{src}: expected {prop}: {val} in {css:?}"
            );
        }
    }

    #[test]
    fn concern_clash_keys_are_kind_aware() {
        let op = only_op("style({ border(subtle) })");
        let slots = op_slots(&op);
        assert!(
            slots.contains(&"border:stroke:all")
                || (slots.contains(&"border:stroke:t") && slots.contains(&"border:stroke:b")),
            "{slots:?}"
        );
        let op = only_op("style({ border({ kind: ring, color: primary }) })");
        assert_eq!(op_slots(&op), vec!["border:ring"]);
        let op = only_op("style({ bg(primary) })");
        assert_eq!(op_slots(&op), vec!["bg"]);
    }

    #[test]
    fn outline_and_ring_setters_are_retired() {
        for src in [
            "style({ outline(solid) })",
            "style({ ring(primary) })",
            "style({ truncate() })",
            "style({ borderImage(none) })",
        ] {
            let (_, diags) = parse(src);
            assert!(
                diags.iter().any(|d| matches!(
                    d,
                    StyleDiagnostic::RetiredSetter { .. } | StyleDiagnostic::UnknownSetter { .. }
                )),
                "{src}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }
    }
}
