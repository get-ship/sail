//! Concern → CSS longhand map and concern lowering.
//!
//! Domain `ConcernSlot` carries scale/keywords only. This table is the
//! adapter's longhand map.

use spinnaker::concerns::{
    ConcernSlot, ConcernSpec, GRADIENT_MOLECULES, concern_teaching, find_concern, find_slot,
    is_named_scale_step, is_structured_value, reject_size_pad_members, resolve_concern_name,
    validate_structured,
};
use spinnaker::ir::{PaintArg, PaintOp, StyleDiagnostic};
use spinnaker::scales;
use spinnaker::tokens::CompositeToken;

/// CSS longhand this concern slot writes, if any.
/// Empty atlas slots (`edge`, `kind`, `gradient`) return `None`.
pub fn concern_css_prop(concern: &str, slot: &str) -> Option<&'static str> {
    match (concern, slot) {
        ("bg", "color") => Some("background-color"),
        ("bg", "mix") => Some("background-color"),
        ("bg", "image") => Some("background-image"),
        ("bg", "clip") => Some("background-clip"),
        ("bg", "glow") => Some("box-shadow"),
        ("fg", "color") => Some("color"),
        ("fg", "caret") => Some("caret-color"),
        ("fg", "accent") => Some("accent-color"),
        ("border", "color") => Some("border-color"),
        ("border", "width") => Some("border-width"),
        ("border", "style") => Some("border-style"),
        ("border", "offset") => Some("outline-offset"),
        ("radius", "all") => Some("border-radius"),
        ("radius", "t") => Some("border-top-left-radius"),
        ("radius", "b") => Some("border-bottom-left-radius"),
        ("radius", "l") => Some("border-top-left-radius"),
        ("radius", "r") => Some("border-top-right-radius"),
        ("elevation", "shadow") => Some("box-shadow"),
        ("filter", "blur") => Some("filter"),
        ("filter", "brightness") => Some("filter"),
        ("filter", "contrast") => Some("filter"),
        ("filter", "grayscale") => Some("filter"),
        ("filter", "hue") => Some("filter"),
        ("filter", "invert") => Some("filter"),
        ("filter", "sepia") => Some("filter"),
        ("filter", "saturate") => Some("filter"),
        ("filter", "backdrop") => Some("backdrop-filter"),
        ("filter", "blend") => Some("mix-blend-mode"),
        ("filter", "opacity") => Some("opacity"),
        ("size", "w") => Some("width"),
        ("size", "h") => Some("height"),
        ("size", "min-w") => Some("min-width"),
        ("size", "min-h") => Some("min-height"),
        ("size", "max-w") => Some("max-width"),
        ("size", "max-h") => Some("max-height"),
        ("size", "aspect") => Some("aspect-ratio"),
        ("size", "basis") => Some("flex-basis"),
        ("pad", "all") => Some("padding"),
        ("pad", "x") => Some("padding-inline"),
        ("pad", "y") => Some("padding-block"),
        ("pad", "t") => Some("padding-top"),
        ("pad", "b") => Some("padding-bottom"),
        ("pad", "l") => Some("padding-left"),
        ("pad", "r") => Some("padding-right"),
        ("pad", "s") => Some("padding-inline-start"),
        ("pad", "e") => Some("padding-inline-end"),
        ("margin", "all") => Some("margin"),
        ("margin", "x") => Some("margin-inline"),
        ("margin", "y") => Some("margin-block"),
        ("margin", "t") => Some("margin-top"),
        ("margin", "b") => Some("margin-bottom"),
        ("margin", "l") => Some("margin-left"),
        ("margin", "r") => Some("margin-right"),
        ("margin", "s") => Some("margin-inline-start"),
        ("margin", "e") => Some("margin-inline-end"),
        ("position", "scheme") => Some("position"),
        ("position", "inset") => Some("inset"),
        ("position", "t") => Some("top"),
        ("position", "b") => Some("bottom"),
        ("position", "l") => Some("left"),
        ("position", "r") => Some("right"),
        ("position", "z") => Some("z-index"),
        ("position", "order") => Some("order"),
        ("position", "anchor") => Some("position-anchor"),
        ("position", "area") => Some("position-area"),
        ("position", "name") => Some("anchor-name"),
        ("overflow", "all") => Some("overflow"),
        ("overflow", "x") => Some("overflow-x"),
        ("overflow", "y") => Some("overflow-y"),
        ("overflow", "overscroll") => Some("overscroll-behavior"),
        ("overflow", "scroll") => Some("scroll-behavior"),
        ("overflow", "snap") => Some("scroll-snap-type"),
        ("type", "family") => Some("font-family"),
        ("type", "size") => Some("font-size"),
        ("type", "weight") => Some("font-weight"),
        ("type", "leading") => Some("line-height"),
        ("type", "tracking") => Some("letter-spacing"),
        ("type", "align") => Some("text-align"),
        ("type", "case") => Some("text-transform"),
        ("type", "wrap") => Some("white-space"),
        ("type", "list") => Some("list-style-type"),
        ("type", "decoration") => Some("text-decoration-line"),
        ("type", "italic") => Some("font-style"),
        ("type", "nums") => Some("font-variant-numeric"),
        ("type", "break") => Some("overflow-wrap"),
        ("bg", "gradient")
        | ("fg", "gradient")
        | ("border", "gradient")
        | ("border", "edge")
        | ("border", "kind") => None,
        _ => None,
    }
}

fn slot_css(spec: &ConcernSpec, slot: &ConcernSlot) -> Option<&'static str> {
    concern_css_prop(spec.name, slot.name)
}

pub fn lower_concern_op(
    spec: &ConcernSpec,
    op: &PaintOp,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    if op.args.len() != 1 {
        return Err(StyleDiagnostic::RetiredSetter {
            setter: spec.name.to_string(),
            replacement: concern_teaching(spec.name),
            source: op.source.clone(),
        });
    }
    match op.first_arg() {
        PaintArg::Slots(slots) if is_structured_value(slots) => {
            validate_structured(slots, &op.source)?;
            let css = lower_structured(slots, &op.source)?;
            Ok(primary_decls(spec, css))
        }
        PaintArg::Slots(slots) => lower_slot_object(spec, op, slots),
        PaintArg::Token(name) => {
            if let Some(inner) = spinnaker::raw_inner(name) {
                if spec.name == "pad" || spec.name == "margin" {
                    if is_numeric_raw(inner) {
                        return Ok(primary_decls(spec, format!("{inner}px")));
                    }
                }
                return lower_concern_op(
                    spec,
                    &PaintOp {
                        setter: op.setter.clone(),
                        args: vec![PaintArg::Token(inner.to_string())],
                        source: op.source.clone(),
                    },
                );
            }
            if spec.name == "size" {
                if let Some(decls) = language_size_decls(name) {
                    return Ok(decls);
                }
            }
            if let Some(composite) = resolve_concern_name(spec, name) {
                if spec.name == "size" {
                    reject_size_pad_members(composite, op)?;
                }
                return Ok(lower_composite(spec, composite));
            }
            if is_named_scale_step(spec, name) && !spinnaker::is_language_size_preset(name) {
                return lower_named_scale_step(spec, name);
            }
            Err(StyleDiagnostic::RetiredSetter {
                setter: format!("{}({name})", spec.name),
                replacement: concern_teaching(spec.name),
                source: op.source.clone(),
            })
        }
        PaintArg::Bind(name) => {
            let css = format!("var(--{})", css_name_for_bind(name));
            Ok(primary_decls(spec, css))
        }
        PaintArg::MemberPath { root, path } => {
            let css = spinnaker::token_groups::bridge_var_ref(root, path);
            Ok(primary_decls(spec, css))
        }
    }
}

/// `size(hug|grow|max)` — participation, never a width token.
/// grow/max must not emit `width: 100%`.
fn language_size_decls(name: &str) -> Option<Vec<(String, String)>> {
    match name {
        "hug" => Some(vec![
            ("flex-grow".into(), "0".into()),
            ("flex-shrink".into(), "0".into()),
            ("flex-basis".into(), "auto".into()),
            ("min-width".into(), "auto".into()),
            ("min-height".into(), "auto".into()),
        ]),
        "grow" => Some(vec![
            ("flex-grow".into(), "1".into()),
            ("flex-shrink".into(), "1".into()),
            ("flex-basis".into(), "0".into()),
            ("min-width".into(), "0".into()),
            ("min-height".into(), "0".into()),
        ]),
        "max" => Some(vec![
            ("align-self".into(), "stretch".into()),
            ("min-width".into(), "0".into()),
            ("min-height".into(), "0".into()),
        ]),
        _ => None,
    }
}

fn primary_decls(spec: &ConcernSpec, css: String) -> Vec<(String, String)> {
    match spec.name {
        "bg" => vec![("background-color".into(), css)],
        "fg" => vec![("color".into(), css)],
        "border" => vec![("border-color".into(), css)],
        "radius" => vec![("border-radius".into(), css)],
        "elevation" => vec![("box-shadow".into(), css)],
        "size" => vec![("width".into(), css.clone()), ("height".into(), css)],
        _ => spec
            .slots
            .iter()
            .find(|s| slot_css(spec, s).is_some())
            .map(|s| vec![(slot_css(spec, s).unwrap_or("").to_string(), css)])
            .unwrap_or_default(),
    }
}

fn lower_named_scale_step(
    spec: &ConcernSpec,
    name: &str,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    match spec.name {
        "radius" => {
            let slot = find_slot(spec, "all").unwrap();
            Ok(vec![lower_slot_value(
                spec,
                slot,
                &PaintArg::Token(name.into()),
            )?])
        }
        "elevation" => {
            let slot = find_slot(spec, "shadow").unwrap();
            Ok(vec![lower_slot_value(
                spec,
                slot,
                &PaintArg::Token(name.into()),
            )?])
        }
        "size" => {
            let w = find_slot(spec, "w").unwrap();
            let h = find_slot(spec, "h").unwrap();
            let tok = PaintArg::Token(name.into());
            Ok(vec![
                lower_slot_value(spec, w, &tok)?,
                lower_slot_value(spec, h, &tok)?,
            ])
        }
        _ => Ok(Vec::new()),
    }
}

/// Lower a resolved composite to the concern's CSS longhands (member vars).
pub fn lower_composite(spec: &ConcernSpec, composite: &CompositeToken) -> Vec<(String, String)> {
    let edge = if spec.name == "border" {
        composite
            .members
            .iter()
            .find(|(member, _)| *member == "edge")
            .map(|(_, v)| v.clone())
    } else {
        None
    };
    let mut out = Vec::new();
    for slot in spec.slots {
        if slot_css(spec, slot).is_none() {
            continue;
        }
        if let Some((_, css)) = composite
            .members
            .iter()
            .find(|(member, _)| *member == slot.name)
        {
            if let Some(e) = &edge {
                if matches!(slot.name, "color" | "width" | "style") {
                    for prop in edge_props(e, slot.name) {
                        out.push((prop.to_string(), format!("var(--{css})")));
                    }
                    continue;
                }
            }
            if spec.name == "radius" && slot.name == "all" {
                out.push((
                    slot_css(spec, slot).unwrap_or("").to_string(),
                    format!("var(--{css})"),
                ));
            } else if spec.name == "radius" && matches!(slot.name, "t" | "b" | "l" | "r") {
                for prop in radius_side_props(slot.name) {
                    out.push((prop.to_string(), format!("var(--{css})")));
                }
            } else {
                out.push((
                    slot_css(spec, slot).unwrap_or("").to_string(),
                    format!("var(--{css})"),
                ));
            }
        }
    }
    out
}

fn lower_structured(
    slots: &std::collections::BTreeMap<String, PaintArg>,
    source: &str,
) -> Result<String, StyleDiagnostic> {
    if let Some(r) = slots.get("ref") {
        return lower_structured_operand(r, source);
    }
    if let Some(PaintArg::Token(op)) = slots.get("op") {
        if op == "mix" {
            let from = slots
                .get("from")
                .or_else(|| slots.get("color"))
                .ok_or_else(|| StyleDiagnostic::BadToken {
                    setter: "structured:mix".into(),
                    value: "from".into(),
                    source: source.to_string(),
                })?;
            let onto = slots
                .get("onto")
                .or_else(|| slots.get("with"))
                .ok_or_else(|| StyleDiagnostic::BadToken {
                    setter: "structured:mix".into(),
                    value: "onto".into(),
                    source: source.to_string(),
                })?;
            let from_css = lower_structured_operand(from, source)?;
            let onto_css = lower_structured_operand(onto, source)?;
            let amount = slots
                .get("amount")
                .or_else(|| slots.get("pct"))
                .and_then(|v| match v {
                    PaintArg::Token(t) => t.trim_end_matches('%').parse::<f64>().ok(),
                    _ => None,
                })
                .unwrap_or(0.5);
            let pct = if amount <= 1.0 {
                amount * 100.0
            } else {
                amount
            };
            return Ok(format!(
                "color-mix(in oklch, {from_css} {pct:.2}%, {onto_css})"
            ));
        }
        return Err(StyleDiagnostic::BadToken {
            setter: "structured:op".into(),
            value: op.clone(),
            source: source.to_string(),
        });
    }
    Err(StyleDiagnostic::BadToken {
        setter: "structured".into(),
        value: "ref / op".into(),
        source: source.to_string(),
    })
}

fn lower_structured_operand(value: &PaintArg, source: &str) -> Result<String, StyleDiagnostic> {
    match value {
        PaintArg::Bind(n) => Ok(format!("var(--{})", css_name_for_bind(n))),
        PaintArg::MemberPath { root, path } => {
            Ok(spinnaker::token_groups::bridge_var_ref(root, path))
        }
        PaintArg::Slots(inner) => lower_structured(inner, source),
        PaintArg::Token(t) => {
            if t == "current" || t == "currentColor" {
                return Ok("currentColor".into());
            }
            if scales::is_keyword_color(t) {
                return Ok(t.clone());
            }
            Ok(format!("var(--{t})"))
        }
    }
}

fn lower_gradient_name(setter: &str, value: &PaintArg) -> Result<Option<String>, StyleDiagnostic> {
    match value {
        PaintArg::Token(t) if GRADIENT_MOLECULES.contains(&t.as_str()) => {
            if t == "none" {
                Ok(None)
            } else {
                Ok(Some(format!("var(--gradient-{t})")))
            }
        }
        PaintArg::Bind(n) => Ok(Some(format!("var(--{})", css_name_for_bind(n)))),
        other => Err(StyleDiagnostic::BadToken {
            setter: setter.into(),
            value: other.display(),
            source: other.display(),
        }),
    }
}

fn lower_slot_object(
    spec: &ConcernSpec,
    op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    if spec.name == "border" {
        return lower_border_slots(spec, op, slots);
    }
    if spec.name == "bg" {
        return lower_fill_slots(spec, op, slots);
    }
    if spec.name == "fg" {
        return lower_ink_slots(spec, op, slots);
    }
    if spec.name == "filter" {
        return lower_filter_slots(spec, op, slots);
    }
    if spec.name == "type" {
        return lower_type_slots(spec, op, slots);
    }
    if spec.name == "motion" {
        return lower_motion_slots(spec, op, slots);
    }
    if spec.name == "radius" {
        return lower_radius_slots(spec, op, slots);
    }
    let mut out = Vec::new();
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            return Err(StyleDiagnostic::UnknownConcernSlot {
                setter: spec.name.to_string(),
                slot: name.clone(),
                source: op.source.clone(),
            });
        };
        if slot_css(spec, slot).is_none() {
            continue;
        }
        out.push(lower_slot_value(spec, slot, value)?);
    }
    Ok(out)
}

fn lower_radius_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut out = Vec::new();
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        let (_, css) = lower_slot_value(spec, slot, value)?;
        match name.as_str() {
            "all" => out.push(("border-radius".into(), css)),
            "t" => {
                out.push(("border-top-left-radius".into(), css.clone()));
                out.push(("border-top-right-radius".into(), css));
            }
            "b" => {
                out.push(("border-bottom-left-radius".into(), css.clone()));
                out.push(("border-bottom-right-radius".into(), css));
            }
            "l" => {
                out.push(("border-top-left-radius".into(), css.clone()));
                out.push(("border-bottom-left-radius".into(), css));
            }
            "r" => {
                out.push(("border-top-right-radius".into(), css.clone()));
                out.push(("border-bottom-right-radius".into(), css));
            }
            _ => {}
        }
    }
    Ok(out)
}

fn radius_side_props(side: &str) -> &'static [&'static str] {
    match side {
        "t" => &["border-top-left-radius", "border-top-right-radius"],
        "b" => &["border-bottom-left-radius", "border-bottom-right-radius"],
        "l" => &["border-top-left-radius", "border-bottom-left-radius"],
        "r" => &["border-top-right-radius", "border-bottom-right-radius"],
        _ => &["border-radius"],
    }
}

fn lower_border_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let kind = match slots.get("kind") {
        Some(PaintArg::Token(k)) => k.as_str(),
        _ => "stroke",
    };
    let edge = match slots.get("edge") {
        Some(PaintArg::Token(e)) => e.as_str(),
        _ => "all",
    };
    let color = slots.get("color");
    let width = slots.get("width");
    let style = slots.get("style");
    if let Some(g) = slots.get("gradient") {
        let mut out = Vec::new();
        match lower_gradient_name("border:gradient", g)? {
            Some(css) => {
                out.push(("border-image-source".into(), css));
                out.push(("border-image-slice".into(), "1".into()));
            }
            None => out.push(("border-image".into(), "none".into())),
        }
        if let Some(v) = width {
            for prop in edge_props(edge, "width") {
                out.push(lower_named_css(
                    spec,
                    prop,
                    find_slot(spec, "width").unwrap(),
                    v,
                )?);
            }
        }
        if let Some(v) = style {
            for prop in edge_props(edge, "style") {
                out.push(lower_named_css(
                    spec,
                    prop,
                    find_slot(spec, "style").unwrap(),
                    v,
                )?);
            }
        }
        if let Some(v) = color {
            for prop in edge_props(edge, "color") {
                out.push(lower_named_css(
                    spec,
                    prop,
                    find_slot(spec, "color").unwrap(),
                    v,
                )?);
            }
        }
        return Ok(out);
    }
    match kind {
        "outline" => {
            let mut out = Vec::new();
            if let Some(v) = color {
                out.push(lower_named_css(
                    spec,
                    "outline-color",
                    find_slot(spec, "color").unwrap(),
                    v,
                )?);
            }
            if let Some(v) = width {
                out.push(lower_named_css(
                    spec,
                    "outline-width",
                    find_slot(spec, "width").unwrap(),
                    v,
                )?);
            }
            if let Some(v) = style {
                out.push(lower_named_css(
                    spec,
                    "outline-style",
                    find_slot(spec, "style").unwrap(),
                    v,
                )?);
            }
            if let Some(v) = slots.get("offset") {
                out.push(lower_outline_offset(v)?);
            }
            Ok(out)
        }
        "ring" => lower_ring(spec, color, width),
        "image" => {
            let mut out = Vec::new();
            if let Some(v) = color {
                out.push(lower_named_css(
                    spec,
                    "border-image",
                    find_slot(spec, "color").unwrap(),
                    v,
                )?);
            }
            Ok(out)
        }
        _ => {
            let mut out = Vec::new();
            if let Some(v) = color {
                for prop in edge_props(edge, "color") {
                    out.push(lower_named_css(
                        spec,
                        prop,
                        find_slot(spec, "color").unwrap(),
                        v,
                    )?);
                }
            }
            if let Some(v) = width {
                for prop in edge_props(edge, "width") {
                    out.push(lower_named_css(
                        spec,
                        prop,
                        find_slot(spec, "width").unwrap(),
                        v,
                    )?);
                }
            }
            if let Some(v) = style {
                for prop in edge_props(edge, "style") {
                    out.push(lower_named_css(
                        spec,
                        prop,
                        find_slot(spec, "style").unwrap(),
                        v,
                    )?);
                }
            }
            Ok(out)
        }
    }
}

fn edge_props(edge: &str, part: &str) -> Vec<&'static str> {
    match (edge, part) {
        ("all", "color") => vec!["border-color"],
        ("all", "width") => vec!["border-width"],
        ("all", "style") => vec!["border-style"],
        ("x", "color") => vec!["border-left-color", "border-right-color"],
        ("x", "width") => vec!["border-left-width", "border-right-width"],
        ("x", "style") => vec!["border-left-style", "border-right-style"],
        ("y", "color") => vec!["border-top-color", "border-bottom-color"],
        ("y", "width") => vec!["border-top-width", "border-bottom-width"],
        ("y", "style") => vec!["border-top-style", "border-bottom-style"],
        ("t", "color") => vec!["border-top-color"],
        ("t", "width") => vec!["border-top-width"],
        ("t", "style") => vec!["border-top-style"],
        ("b", "color") => vec!["border-bottom-color"],
        ("b", "width") => vec!["border-bottom-width"],
        ("b", "style") => vec!["border-bottom-style"],
        ("l", "color") => vec!["border-left-color"],
        ("l", "width") => vec!["border-left-width"],
        ("l", "style") => vec!["border-left-style"],
        ("r", "color") => vec!["border-right-color"],
        ("r", "width") => vec!["border-right-width"],
        ("r", "style") => vec!["border-right-style"],
        ("s", "color") => vec!["border-inline-start-color"],
        ("s", "width") => vec!["border-inline-start-width"],
        ("s", "style") => vec!["border-inline-start-style"],
        ("e", "color") => vec!["border-inline-end-color"],
        ("e", "width") => vec!["border-inline-end-width"],
        ("e", "style") => vec!["border-inline-end-style"],
        _ => vec!["border-color"],
    }
}

fn lower_ring(
    spec: &ConcernSpec,
    color: Option<&PaintArg>,
    width: Option<&PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let color_css = if let Some(v) = color {
        lower_slot_value(spec, find_slot(spec, "color").unwrap(), v)?.1
    } else {
        "var(--ring)".into()
    };
    let width_css = if let Some(v) = width {
        lower_slot_value(spec, find_slot(spec, "width").unwrap(), v)?.1
    } else {
        "var(--ring-width)".into()
    };
    // Existing ring path: double box-shadow (offset hole + ring).
    let shadow = format!(
        "0 0 0 var(--ring-offset) var(--background), 0 0 0 calc(var(--ring-offset) + {width_css}) {color_css}"
    );
    Ok(vec![("box-shadow".into(), shadow)])
}

fn lower_named_css(
    spec: &ConcernSpec,
    prop: &str,
    slot: &ConcernSlot,
    value: &PaintArg,
) -> Result<(String, String), StyleDiagnostic> {
    let (_, css) = lower_slot_value(spec, slot, value)?;
    Ok((prop.to_string(), css))
}

fn lower_outline_offset(value: &PaintArg) -> Result<(String, String), StyleDiagnostic> {
    let spec = find_concern("border").unwrap();
    let css = match value {
        PaintArg::Token(t) => match t.as_str() {
            "ring-offset" => "var(--ring-offset)".into(),
            "ring-inset" => "calc(-1 * var(--ring-width))".into(),
            other => {
                let slot = find_slot(spec, "offset").unwrap();
                lower_slot_value(spec, slot, &PaintArg::Token(other.into()))?.1
            }
        },
        other => {
            let slot = find_slot(spec, "offset").unwrap();
            lower_slot_value(spec, slot, other)?.1
        }
    };
    Ok(("outline-offset".into(), css))
}

fn lower_ink_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut out = Vec::new();
    for (name, value) in slots {
        if name == "gradient" {
            let bare = match value {
                PaintArg::Token(t) => t.as_str(),
                PaintArg::Bind(n) => {
                    out.push((
                        "background".into(),
                        format!("var(--{})", css_name_for_bind(n)),
                    ));
                    out.push(("-webkit-background-clip".into(), "text".into()));
                    out.push(("background-clip".into(), "text".into()));
                    out.push(("color".into(), "transparent".into()));
                    continue;
                }
                other => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: "ink:gradient".into(),
                        value: other.display(),
                        source: other.display(),
                    });
                }
            };
            match bare {
                // surface is a page wash that defaults to none — clip-to-none
                // makes ink invisible. Keep authored color, same as `none`.
                "none" | "surface" => {
                    out.push(("background".into(), "none".into()));
                    out.push(("-webkit-background-clip".into(), "border-box".into()));
                    out.push(("background-clip".into(), "border-box".into()));
                }
                "primary" | "accent" => {
                    out.push(("background".into(), format!("var(--gradient-{bare})")));
                    out.push(("-webkit-background-clip".into(), "text".into()));
                    out.push(("background-clip".into(), "text".into()));
                    out.push(("color".into(), "transparent".into()));
                }
                other => {
                    return Err(StyleDiagnostic::BadToken {
                        setter: "ink:gradient".into(),
                        value: other.into(),
                        source: other.into(),
                    });
                }
            }
            continue;
        }
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        if slot_css(spec, slot).is_none() {
            continue;
        }
        out.push(lower_slot_value(spec, slot, value)?);
    }
    Ok(out)
}

fn lower_fill_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    if let (Some(color), Some(mix)) = (slots.get("color"), slots.get("mix")) {
        let token = lower_slot_value(spec, find_slot(spec, "color").unwrap(), color)?.1;
        let pct = match mix {
            PaintArg::Token(t) => t.clone(),
            PaintArg::Bind(n) => format!("var(--{})", css_name_for_bind(n)),
            other => other.display(),
        };
        let mut out = vec![(
            "background-color".into(),
            format!("color-mix(in oklch, {token} {pct}%, transparent)"),
        )];
        for (name, value) in slots {
            if name == "color" || name == "mix" {
                continue;
            }
            let Some(slot) = find_slot(spec, name) else {
                continue;
            };
            if slot_css(spec, slot).is_none() || slot_css(spec, slot) == Some("background-color") {
                continue;
            }
            out.push(lower_slot_value(spec, slot, value)?);
        }
        return Ok(out);
    }
    let mut out = Vec::new();
    for (name, value) in slots {
        if name == "mix" {
            continue;
        }
        if name == "gradient" {
            match lower_gradient_name("bg:gradient", value)? {
                Some(css) => out.push(("background-image".into(), css)),
                None => out.push(("background-image".into(), "none".into())),
            }
            continue;
        }
        if name == "image" {
            if let PaintArg::Token(t) = value {
                if t == "primary-accent" {
                    out.push((
                        "background-image".into(),
                        "linear-gradient(135deg, var(--primary), var(--accent))".into(),
                    ));
                    continue;
                }
                if t == "none" {
                    out.push(("background-image".into(), "none".into()));
                    continue;
                }
            }
        }
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        if slot_css(spec, slot).is_none() {
            continue;
        }
        out.push(lower_slot_value(spec, slot, value)?);
    }
    Ok(out)
}

fn lower_filter_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut filter_parts = Vec::new();
    let mut out = Vec::new();
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        let bare = match value {
            PaintArg::Token(t) => t
                .strip_prefix('-')
                .filter(|r| !r.is_empty())
                .unwrap_or(t)
                .to_string(),
            PaintArg::Bind(n) => format!("var(--{})", css_name_for_bind(n)),
            other => other.display(),
        };
        match name.as_str() {
            "blur" => filter_parts.push(format!("blur({bare}px)")),
            "brightness" => filter_parts.push(format!("brightness({}%)", bare)),
            "contrast" => filter_parts.push(format!("contrast({bare}%)")),
            "grayscale" => filter_parts.push(format!("grayscale({bare}%)")),
            "hue" => filter_parts.push(format!("hue-rotate({bare}deg)")),
            "invert" => filter_parts.push(format!("invert({bare}%)")),
            "sepia" => filter_parts.push(format!("sepia({bare}%)")),
            "saturate" => filter_parts.push(format!("saturate({bare}%)")),
            "backdrop" => {
                let css = if bare == "none" {
                    "none".into()
                } else {
                    format!("var(--{bare})")
                };
                out.push(("backdrop-filter".into(), css.clone()));
                out.push(("-webkit-backdrop-filter".into(), css));
            }
            "blend" => out.push(("mix-blend-mode".into(), bare)),
            "opacity" => {
                let css = if let Ok(n) = bare.parse::<u32>() {
                    format!("{}", n as f32 / 100.0)
                } else {
                    lower_slot_value(spec, slot, value)?.1
                };
                out.push(("opacity".into(), css));
            }
            _ => {}
        }
    }
    if !filter_parts.is_empty() {
        out.push(("filter".into(), filter_parts.join(" ")));
    }
    Ok(out)
}

fn lower_type_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut out = Vec::new();
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        if name == "wrap" {
            let bare = match value {
                PaintArg::Token(t) => t.as_str(),
                _ => {
                    out.push(lower_slot_value(spec, slot, value)?);
                    continue;
                }
            };
            match bare {
                "truncate" => {
                    out.push(("overflow".into(), "hidden".into()));
                    out.push(("text-overflow".into(), "ellipsis".into()));
                    out.push(("white-space".into(), "nowrap".into()));
                }
                "pretty" => {
                    out.push(("text-wrap".into(), "pretty".into()));
                    out.push(("white-space".into(), "normal".into()));
                }
                "nowrap" => out.push(("white-space".into(), "nowrap".into())),
                "normal" => out.push(("white-space".into(), "normal".into())),
                "pre-wrap" => out.push(("white-space".into(), "pre-wrap".into())),
                "words" => out.push(("overflow-wrap".into(), "break-word".into())),
                _ => out.push(lower_slot_value(spec, slot, value)?),
            }
            continue;
        }
        if name == "italic" {
            let css = match value {
                PaintArg::Token(t) => match t.as_str() {
                    "true" | "on" | "italic" => "italic",
                    "false" | "off" | "normal" => "normal",
                    other => {
                        return Err(StyleDiagnostic::BadToken {
                            setter: "type:italic".into(),
                            value: other.into(),
                            source: other.into(),
                        });
                    }
                },
                _ => {
                    out.push(lower_slot_value(spec, slot, value)?);
                    continue;
                }
            };
            out.push(("font-style".into(), css.into()));
            continue;
        }
        if name == "nums" {
            let css = match value {
                PaintArg::Token(t) => match t.as_str() {
                    "tabular" => "tabular-nums",
                    "lining" => "lining-nums",
                    "oldstyle" => "oldstyle-nums",
                    "proportional" => "proportional-nums",
                    "ordinal" => "ordinal",
                    "slashed-zero" => "slashed-zero",
                    "normal" => "normal",
                    other => {
                        return Err(StyleDiagnostic::BadToken {
                            setter: "type:nums".into(),
                            value: other.into(),
                            source: other.into(),
                        });
                    }
                },
                _ => {
                    out.push(lower_slot_value(spec, slot, value)?);
                    continue;
                }
            };
            out.push(("font-variant-numeric".into(), css.into()));
            continue;
        }
        if name == "break" {
            let css = match value {
                PaintArg::Token(t) => match t.as_str() {
                    "words" => "break-word",
                    "normal" => "normal",
                    "anywhere" => "anywhere",
                    other => {
                        return Err(StyleDiagnostic::BadToken {
                            setter: "type:break".into(),
                            value: other.into(),
                            source: other.into(),
                        });
                    }
                },
                _ => {
                    out.push(lower_slot_value(spec, slot, value)?);
                    continue;
                }
            };
            out.push(("overflow-wrap".into(), css.into()));
            continue;
        }
        if name == "case" {
            if let PaintArg::Token(t) = value {
                let css = match t.as_str() {
                    "normal" => "none",
                    "label-transform" => "var(--label-transform)",
                    other => other,
                };
                out.push(("text-transform".into(), css.into()));
                continue;
            }
        }
        out.push(lower_slot_value(spec, slot, value)?);
    }
    Ok(out)
}

fn lower_motion_slots(
    spec: &ConcernSpec,
    _op: &PaintOp,
    slots: &std::collections::BTreeMap<String, PaintArg>,
) -> Result<Vec<(String, String)>, StyleDiagnostic> {
    let mut out = Vec::new();
    let mut tx: Option<String> = None;
    let mut ty: Option<String> = None;
    for (name, value) in slots {
        let Some(slot) = find_slot(spec, name) else {
            continue;
        };
        match name.as_str() {
            "x" => tx = Some(lower_slot_value(spec, slot, value)?.1),
            "y" => ty = Some(lower_slot_value(spec, slot, value)?.1),
            "scale" => {
                let (_, css) = lower_slot_value(spec, slot, value)?;
                let scaled = if let PaintArg::Token(t) = value {
                    if let Ok(n) = t.parse::<f32>() {
                        format!("{}", n / 100.0)
                    } else {
                        css
                    }
                } else {
                    css
                };
                out.push(("scale".into(), scaled));
            }
            "property" => {
                let css = match value {
                    PaintArg::Token(t) => match t.as_str() {
                        "colors" => {
                            "color, background-color, border-color, outline-color, fill, stroke"
                                .into()
                        }
                        "shadow" => "box-shadow".into(),
                        "transform" => "transform, translate, scale, rotate".into(),
                        other => other.to_string(),
                    },
                    other => other.display(),
                };
                out.push(("transition-property".into(), css));
            }
            "animate" => {
                let css = match value {
                    PaintArg::Token(t) if t == "spin" => "var(--animate-spin)".into(),
                    PaintArg::Token(t) if t == "none" => "none".into(),
                    other => other.display(),
                };
                out.push(("animation".into(), css));
            }
            _ => out.push(lower_slot_value(spec, slot, value)?),
        }
    }
    match (tx, ty) {
        (Some(x), Some(y)) => out.push(("translate".into(), format!("{x} {y}"))),
        (Some(x), None) => out.push(("translate".into(), format!("{x} 0"))),
        (None, Some(y)) => out.push(("translate".into(), format!("0 {y}"))),
        (None, None) => {}
    }
    Ok(out)
}

pub fn lower_slot_value(
    spec: &ConcernSpec,
    slot: &ConcernSlot,
    value: &PaintArg,
) -> Result<(String, String), StyleDiagnostic> {
    let css = match value {
        PaintArg::Bind(name) => format!("var(--{})", css_name_for_bind(name)),
        PaintArg::MemberPath { root, path } => spinnaker::token_groups::bridge_var_ref(root, path),
        PaintArg::Slots(inner) if is_structured_value(inner) => {
            lower_structured(inner, &value.display())?
        }
        PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: slot.name.to_string(),
                value: value.display(),
                source: value.display(),
            });
        }
        PaintArg::Token(tok) => {
            if let Some(inner) = spinnaker::raw_inner(tok) {
                if slot.scale == Some("space") && is_numeric_raw(inner) {
                    return Ok((
                        slot_css(spec, slot).unwrap_or("").to_string(),
                        format!("{inner}px"),
                    ));
                }
                return lower_slot_value(spec, slot, &PaintArg::Token(inner.to_string()));
            }
            if let Some(inner) = spinnaker::inverse_inner(tok) {
                let (_, css) = lower_slot_value(spec, slot, &PaintArg::Token(inner.to_string()))?;
                return Ok((
                    slot_css(spec, slot).unwrap_or("").to_string(),
                    invert_css_var(&css),
                ));
            }
            let negative = tok.starts_with('-') && tok.len() > 1;
            let bare = tok
                .strip_prefix('-')
                .filter(|r| !r.is_empty())
                .unwrap_or(tok);
            let signed = |lowered: String| {
                if negative {
                    format!("calc(-1 * {lowered})")
                } else {
                    lowered
                }
            };
            if bare == "hairline" && slot.name == "width" {
                "var(--border-width)".into()
            } else if bare == "control-stroke" && slot.name == "width" {
                "var(--control-stroke)".into()
            } else if bare == "ring-width" && slot.name == "width" {
                "var(--ring-width)".into()
            } else if bare == "border-width" && matches!(slot.name, "w" | "h") {
                "var(--border-width)".into()
            } else if bare == "half" && matches!(slot.name, "w" | "h") {
                "50%".into()
            } else if slot.name == "aspect" {
                match bare {
                    "square" => "1 / 1".into(),
                    "video" => "16 / 9".into(),
                    "auto" => "auto".into(),
                    other => other.to_string(),
                }
            } else if spec.name == "position" && slot.name == "area" {
                match bare {
                    "top-end" => "top span-right".into(),
                    "bottom-end" => "bottom span-right".into(),
                    "left-end" => "left span-bottom".into(),
                    "right-end" => "right span-bottom".into(),
                    other => other.to_string(),
                }
            } else if spec.name == "position" && slot.name == "anchor" {
                if bare == "auto" {
                    "auto".into()
                } else {
                    format!("--{bare}")
                }
            } else if spec.name == "position" && slot.name == "name" {
                format!("--{bare}")
            } else if slot.name == "order" && bare == "first" {
                "-9999".into()
            } else if slot.name == "order" && bare == "last" {
                "9999".into()
            } else if slot.keywords.contains(&bare) && slot.scale.is_none() {
                match bare {
                    "current" => "currentColor".into(),
                    other => other.to_string(),
                }
            } else if spec.name == "size" && spinnaker::is_free_size_value(bare) {
                strip_size_quotes(bare)
            } else if let Some(scale) = slot.scale {
                if let Some(lowered) = scales::lower(scale, bare) {
                    signed(lowered)
                } else if matches!(bare, "transparent" | "current" | "currentColor") {
                    if bare == "current" {
                        "currentColor".into()
                    } else {
                        bare.into()
                    }
                } else if slot.keywords.contains(&bare) {
                    match bare {
                        "current" => "currentColor".into(),
                        other => other.to_string(),
                    }
                } else if slot.name == "width" && bare.parse::<u32>().is_ok() {
                    format!("{bare}px")
                } else {
                    format!("var(--{bare})")
                }
            } else if slot.name == "width" && bare.parse::<u32>().is_ok() {
                format!("{bare}px")
            } else {
                match bare {
                    "current" => "currentColor".into(),
                    other => other.to_string(),
                }
            }
        }
    };
    Ok((slot_css(spec, slot).unwrap_or("").to_string(), css))
}

/// `var(--surface)` → `var(--inverse-surface)`. Non-var colors stay as-is.
fn invert_css_var(css: &str) -> String {
    let t = css.trim();
    if let Some(name) = t.strip_prefix("var(--").and_then(|s| s.strip_suffix(')')) {
        if name.starts_with("inverse-") {
            return t.to_string();
        }
        return format!("var(--inverse-{name})");
    }
    t.to_string()
}

fn is_numeric_raw(inner: &str) -> bool {
    let t = inner.trim();
    !t.is_empty() && t.replace('_', ".").parse::<f64>().is_ok()
}

fn strip_size_quotes(raw: &str) -> String {
    let s = raw.trim();
    s.strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .or_else(|| s.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
        .unwrap_or(s)
        .to_string()
}

fn css_name_for_bind(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod measure_column {
    use super::*;
    use spinnaker::concerns::{find_concern, find_slot};
    use spinnaker::ir::PaintArg;

    #[test]
    fn margin_slot_negatives_emit_calc() {
        let spec = find_concern("margin").expect("margin concern");
        let slot = find_slot(spec, "t").expect("margin t");
        let (_, css) = lower_slot_value(spec, slot, &PaintArg::Token("-5".into())).expect("lower");
        assert_eq!(css, "calc(-1 * var(--space-5))", "{css}");
    }

    #[test]
    fn inverse_color_lowers_to_inverse_var() {
        let spec = find_concern("bg").expect("bg");
        let slot = find_slot(spec, "color").expect("color");
        let (_, css) = lower_slot_value(spec, slot, &PaintArg::Token("inverse(surface)".into()))
            .expect("lower");
        assert_eq!(css, "var(--inverse-surface)", "{css}");
    }

    #[test]
    fn size_free_percent_is_literal_not_a_var() {
        let spec = find_concern("size").expect("size concern");
        let slot = find_slot(spec, "w").expect("size w");
        let (_, css) =
            lower_slot_value(spec, slot, &PaintArg::Token("100%".into())).expect("lower");
        assert_eq!(css, "100%", "{css}");
    }
}
