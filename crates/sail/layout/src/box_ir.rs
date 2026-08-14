//! Resolve box (display / grow / shrink / overflow / position / pad / size)
//! from Style IR.
//!
//! Layout consumes this resolution. Class spelling is output — measure does
//! not parse `s-displayflex` / `s-grow1` to learn what Style already decided.

use sail_lint::style::{PaintArg, PaintOp, Style, StyleEnv};

/// Tokens Style spent on the box channel (resting layer, not when/hover).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedBox {
    pub display: Option<String>,
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub overflow_x: Option<String>,
    pub overflow_y: Option<String>,
    /// `truncate()` / `type({ wrap: truncate })` — ellipsis opt-in, not a class.
    pub truncate: bool,
    pub position: Option<String>,
    pub pad: PadTokens,
    pub width: Option<String>,
    pub height: Option<String>,
    pub min_width: Option<String>,
    pub min_height: Option<String>,
    pub max_width: Option<String>,
    pub max_height: Option<String>,
}

/// Space-scale steps on each padding side (`4`, `1_5`, `0`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PadTokens {
    pub all: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub t: Option<String>,
    pub r: Option<String>,
    pub b: Option<String>,
    pub l: Option<String>,
}

impl PadTokens {
    pub fn is_empty(&self) -> bool {
        self.all.is_none()
            && self.x.is_none()
            && self.y.is_none()
            && self.t.is_none()
            && self.r.is_none()
            && self.b.is_none()
            && self.l.is_none()
    }
}

impl ResolvedBox {
    pub fn is_empty(&self) -> bool {
        self.display.is_none()
            && self.grow.is_none()
            && self.shrink.is_none()
            && self.overflow_x.is_none()
            && self.overflow_y.is_none()
            && !self.truncate
            && self.position.is_none()
            && self.pad.is_empty()
            && self.width.is_none()
            && self.height.is_none()
            && self.min_width.is_none()
            && self.min_height.is_none()
            && self.max_width.is_none()
            && self.max_height.is_none()
    }

    /// Overlay `other` onto self — set fields win.
    pub fn overlay(&mut self, other: &Self) {
        if other.display.is_some() {
            self.display.clone_from(&other.display);
        }
        if other.grow.is_some() {
            self.grow = other.grow;
        }
        if other.shrink.is_some() {
            self.shrink = other.shrink;
        }
        if other.overflow_x.is_some() {
            self.overflow_x.clone_from(&other.overflow_x);
        }
        if other.overflow_y.is_some() {
            self.overflow_y.clone_from(&other.overflow_y);
        }
        if other.truncate {
            self.truncate = true;
        }
        if other.position.is_some() {
            self.position.clone_from(&other.position);
        }
        overlay_pad(&mut self.pad, &other.pad);
        if other.width.is_some() {
            self.width.clone_from(&other.width);
        }
        if other.height.is_some() {
            self.height.clone_from(&other.height);
        }
        if other.min_width.is_some() {
            self.min_width.clone_from(&other.min_width);
        }
        if other.min_height.is_some() {
            self.min_height.clone_from(&other.min_height);
        }
        if other.max_width.is_some() {
            self.max_width.clone_from(&other.max_width);
        }
        if other.max_height.is_some() {
            self.max_height.clone_from(&other.max_height);
        }
    }
}

fn overlay_pad(dst: &mut PadTokens, src: &PadTokens) {
    if src.all.is_some() {
        dst.all.clone_from(&src.all);
    }
    if src.x.is_some() {
        dst.x.clone_from(&src.x);
    }
    if src.y.is_some() {
        dst.y.clone_from(&src.y);
    }
    if src.t.is_some() {
        dst.t.clone_from(&src.t);
    }
    if src.r.is_some() {
        dst.r.clone_from(&src.r);
    }
    if src.b.is_some() {
        dst.b.clone_from(&src.b);
    }
    if src.l.is_some() {
        dst.l.clone_from(&src.l);
    }
}

/// Style-resolved box tokens joined onto a parsed host (class list is
/// identity only — measure never parses `s-display*` / `s-grow*`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoxHost {
    pub classes: Vec<String>,
    pub resolved: ResolvedBox,
}

impl BoxHost {
    pub fn from_resolved(classes: &str, resolved: &ResolvedBox) -> Option<Self> {
        if resolved.is_empty() {
            return None;
        }
        let classes: Vec<String> = classes
            .split_whitespace()
            .filter(|c| !c.is_empty())
            .map(|c| c.to_string())
            .collect();
        if classes.is_empty() {
            return None;
        }
        Some(Self {
            classes,
            resolved: resolved.clone(),
        })
    }
}

/// Overlay Style-resolved box onto hosts whose emitted class identity matches.
pub fn apply_resolved_boxes(elements: &mut [crate::html::Element], hosts: &[BoxHost]) {
    if hosts.is_empty() {
        return;
    }
    for elem in elements.iter_mut().skip(1) {
        let class_set: std::collections::HashSet<&str> = elem.class.split_whitespace().collect();
        let mut best: Option<&BoxHost> = None;
        let mut best_n = 0usize;
        for host in hosts {
            if host.classes.is_empty() {
                continue;
            }
            if host.classes.iter().all(|c| class_set.contains(c.as_str()))
                && host.classes.len() > best_n
            {
                best_n = host.classes.len();
                best = Some(host);
            }
        }
        if let Some(host) = best {
            elem.box_style.overlay(&host.resolved);
        }
    }
}

/// Walk Style IR and collect the resting box tokens (base layer, not when/hover).
pub fn resolve_box(style: &Style, env: &StyleEnv) -> ResolvedBox {
    resolve_box_inner(style, env, &mut Vec::new())
}

fn resolve_box_inner(style: &Style, env: &StyleEnv, resolving: &mut Vec<String>) -> ResolvedBox {
    let mut out = ResolvedBox::default();
    match style {
        Style::Empty | Style::MotionAttach { .. } => {}
        Style::Set(op) => out.overlay(&from_op(op)),
        Style::Chain(items) | Style::List(items) => {
            for item in items {
                out.overlay(&resolve_box_inner(item, env, resolving));
            }
        }
        Style::Compound { body, .. } => {
            out.overlay(&resolve_box_inner(body, env, resolving));
        }
        Style::When { .. } | Style::Match { .. } => {}
        Style::Ref(name) => {
            if resolving.iter().any(|n| n == name) {
                return out;
            }
            resolving.push(name.clone());
            if let Some(inner) = env.get(name) {
                out.overlay(&resolve_box_inner(inner, env, resolving));
            }
            resolving.pop();
        }
    }
    out
}

fn from_op(op: &PaintOp) -> ResolvedBox {
    match op.setter.as_str() {
        "display" => ResolvedBox {
            display: token_arg(op.first_arg()),
            ..Default::default()
        },
        "grow" => ResolvedBox {
            grow: number_arg(op.first_arg()),
            ..Default::default()
        },
        "shrink" => ResolvedBox {
            shrink: number_arg(op.first_arg()),
            ..Default::default()
        },
        "overflow" => overflow_from_arg(op.first_arg()),
        "truncate" => truncate_box(),
        "wrap" => wrap_from_arg(op.first_arg()),
        "type" => type_wrap_from_arg(op.first_arg()),
        "overflowX" => ResolvedBox {
            overflow_x: token_arg(op.first_arg()),
            ..Default::default()
        },
        "overflowY" => ResolvedBox {
            overflow_y: token_arg(op.first_arg()),
            ..Default::default()
        },
        "position" => ResolvedBox {
            position: position_from_arg(op.first_arg()),
            ..Default::default()
        },
        "pad" => pad_from_arg(op.first_arg(), "all"),
        "padX" => pad_from_arg(op.first_arg(), "x"),
        "padY" => pad_from_arg(op.first_arg(), "y"),
        "padT" => pad_from_arg(op.first_arg(), "t"),
        "padR" => pad_from_arg(op.first_arg(), "r"),
        "padB" => pad_from_arg(op.first_arg(), "b"),
        "padL" => pad_from_arg(op.first_arg(), "l"),
        "padS" => pad_from_arg(op.first_arg(), "l"),
        "padE" => pad_from_arg(op.first_arg(), "r"),
        "size" => size_from_arg(op.first_arg()),
        "w" => ResolvedBox {
            width: token_arg(op.first_arg()),
            ..Default::default()
        },
        "h" => ResolvedBox {
            height: token_arg(op.first_arg()),
            ..Default::default()
        },
        "minW" => ResolvedBox {
            min_width: token_arg(op.first_arg()),
            ..Default::default()
        },
        "minH" => ResolvedBox {
            min_height: token_arg(op.first_arg()),
            ..Default::default()
        },
        "maxW" => ResolvedBox {
            max_width: token_arg(op.first_arg()),
            ..Default::default()
        },
        "maxH" => ResolvedBox {
            max_height: token_arg(op.first_arg()),
            ..Default::default()
        },
        _ => ResolvedBox::default(),
    }
}

fn truncate_box() -> ResolvedBox {
    ResolvedBox {
        overflow_x: Some("hidden".into()),
        overflow_y: Some("hidden".into()),
        truncate: true,
        ..Default::default()
    }
}

fn wrap_from_arg(arg: &PaintArg) -> ResolvedBox {
    match arg {
        PaintArg::Token(name) if name.eq_ignore_ascii_case("truncate") => truncate_box(),
        _ => ResolvedBox::default(),
    }
}

fn type_wrap_from_arg(arg: &PaintArg) -> ResolvedBox {
    match arg {
        PaintArg::Slots(slots) => slots.get("wrap").map(wrap_from_arg).unwrap_or_default(),
        _ => ResolvedBox::default(),
    }
}

fn overflow_from_arg(arg: &PaintArg) -> ResolvedBox {
    match arg {
        PaintArg::Token(name) if !name.is_empty() => ResolvedBox {
            overflow_x: Some(name.clone()),
            overflow_y: Some(name.clone()),
            ..Default::default()
        },
        PaintArg::Slots(slots) => {
            let mut out = ResolvedBox::default();
            if let Some(v) = slots.get("all").and_then(token_arg_ref) {
                out.overflow_x = Some(v.clone());
                out.overflow_y = Some(v);
            }
            if let Some(v) = slots.get("x").and_then(token_arg_ref) {
                out.overflow_x = Some(v);
            }
            if let Some(v) = slots.get("y").and_then(token_arg_ref) {
                out.overflow_y = Some(v);
            }
            out
        }
        _ => ResolvedBox::default(),
    }
}

fn position_from_arg(arg: &PaintArg) -> Option<String> {
    match arg {
        PaintArg::Token(name) if !name.is_empty() => Some(name.clone()),
        PaintArg::Slots(slots) => slots.get("scheme").and_then(token_arg_ref),
        _ => None,
    }
}

fn pad_from_arg(arg: &PaintArg, side: &str) -> ResolvedBox {
    let mut pad = PadTokens::default();
    match arg {
        PaintArg::Token(name) if !name.is_empty() => match side {
            "x" => pad.x = Some(name.clone()),
            "y" => pad.y = Some(name.clone()),
            "t" => pad.t = Some(name.clone()),
            "r" => pad.r = Some(name.clone()),
            "b" => pad.b = Some(name.clone()),
            "l" => pad.l = Some(name.clone()),
            _ => pad.all = Some(name.clone()),
        },
        PaintArg::Slots(slots) => {
            if let Some(v) = slots.get("all").and_then(token_arg_ref) {
                pad.all = Some(v);
            }
            if let Some(v) = slots.get("x").and_then(token_arg_ref) {
                pad.x = Some(v);
            }
            if let Some(v) = slots.get("y").and_then(token_arg_ref) {
                pad.y = Some(v);
            }
            if let Some(v) = slots.get("t").and_then(token_arg_ref) {
                pad.t = Some(v);
            }
            if let Some(v) = slots.get("r").and_then(token_arg_ref) {
                pad.r = Some(v);
            }
            if let Some(v) = slots.get("b").and_then(token_arg_ref) {
                pad.b = Some(v);
            }
            if let Some(v) = slots.get("l").and_then(token_arg_ref) {
                pad.l = Some(v);
            }
            if let Some(v) = slots.get("s").and_then(token_arg_ref) {
                pad.l = Some(v);
            }
            if let Some(v) = slots.get("e").and_then(token_arg_ref) {
                pad.r = Some(v);
            }
        }
        _ => {}
    }
    ResolvedBox {
        pad,
        ..Default::default()
    }
}

fn size_from_arg(arg: &PaintArg) -> ResolvedBox {
    match arg {
        PaintArg::Token(name) if !name.is_empty() => ResolvedBox {
            width: Some(name.clone()),
            height: Some(name.clone()),
            ..Default::default()
        },
        PaintArg::Slots(slots) => {
            let mut out = ResolvedBox::default();
            if let Some(v) = slots.get("w").and_then(token_arg_ref) {
                out.width = Some(v);
            }
            if let Some(v) = slots.get("h").and_then(token_arg_ref) {
                out.height = Some(v);
            }
            if let Some(v) = slots
                .get("min-w")
                .or_else(|| slots.get("minW"))
                .and_then(token_arg_ref)
            {
                out.min_width = Some(v);
            }
            if let Some(v) = slots
                .get("min-h")
                .or_else(|| slots.get("minH"))
                .and_then(token_arg_ref)
            {
                out.min_height = Some(v);
            }
            if let Some(v) = slots
                .get("max-w")
                .or_else(|| slots.get("maxW"))
                .and_then(token_arg_ref)
            {
                out.max_width = Some(v);
            }
            if let Some(v) = slots
                .get("max-h")
                .or_else(|| slots.get("maxH"))
                .and_then(token_arg_ref)
            {
                out.max_height = Some(v);
            }
            out
        }
        _ => ResolvedBox::default(),
    }
}

fn token_arg(arg: &PaintArg) -> Option<String> {
    token_arg_ref(arg)
}

fn token_arg_ref(arg: &PaintArg) -> Option<String> {
    match arg {
        PaintArg::Token(t) if !t.is_empty() => Some(t.clone()),
        _ => None,
    }
}

fn number_arg(arg: &PaintArg) -> Option<f32> {
    token_arg_ref(arg)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Element;
    use crate::style::resolve;
    use std::collections::HashMap;
    use taffy::prelude::*;
    use taffy::style::Overflow;

    fn resolved(src: &str) -> ResolvedBox {
        let (style, diags) = sail_lint::style::parse(src);
        assert!(
            diags.is_empty(),
            "{src}: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        resolve_box(&style, &HashMap::new())
    }

    fn element_with_box(resolved: ResolvedBox) -> Element {
        Element {
            tag: "div".into(),
            class: "host-id".into(),
            box_style: resolved,
            ..Default::default()
        }
    }

    #[test]
    fn display_flex_from_style_not_class() {
        let r = resolved("style({ display(flex) })");
        assert_eq!(r.display.as_deref(), Some("flex"));
        let elem = element_with_box(r);
        let got = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(got.style.display, Display::Flex);
    }

    #[test]
    fn grow_one_from_style_not_class() {
        let r = resolved("style({ grow(1) })");
        assert_eq!(r.grow, Some(1.0));
        let elem = element_with_box(r);
        let got = resolve(&elem, &[Element::default()], 1440.0);
        assert!((got.style.flex_grow - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn truncate_from_style_not_class() {
        let r = resolved("style({ type({ wrap: truncate }) })");
        assert!(
            r.truncate,
            "type({{ wrap: truncate }}) is Style IR, not s-truncate"
        );
        assert_eq!(r.overflow_x.as_deref(), Some("hidden"));
    }

    #[test]
    fn overflow_hidden_from_style_slots() {
        let r = resolved("style({ overflow({ all: hidden }) })");
        assert_eq!(r.overflow_x.as_deref(), Some("hidden"));
        assert_eq!(r.overflow_y.as_deref(), Some("hidden"));
        let elem = element_with_box(r);
        let got = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(got.style.overflow.x, Overflow::Hidden);
        assert_eq!(got.style.overflow.y, Overflow::Hidden);
    }

    #[test]
    fn position_absolute_from_style_scheme() {
        let r = resolved("style({ position({ scheme: absolute }) })");
        assert_eq!(r.position.as_deref(), Some("absolute"));
        let elem = element_with_box(r);
        let got = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(got.style.position, Position::Absolute);
    }

    #[test]
    fn style_box_does_not_need_emitted_class_spelling() {
        let r = resolved(
            "style({ display(flex).grow(1).overflow({ all: hidden }).position({ scheme: absolute }) })",
        );
        let elem = Element {
            tag: "div".into(),
            class: "not-an-atom".into(),
            box_style: r,
            ..Default::default()
        };
        let got = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(got.style.display, Display::Flex);
        assert!((got.style.flex_grow - 1.0).abs() < f32::EPSILON);
        assert_eq!(got.style.overflow.x, Overflow::Hidden);
        assert_eq!(got.style.overflow.y, Overflow::Hidden);
        assert_eq!(got.style.position, Position::Absolute);
    }

    #[test]
    fn style_box_wins_over_class_spelling() {
        let r = resolved("style({ display(flex).grow(1) })");
        let elem = Element {
            tag: "div".into(),
            class: "s-displaynone s-grow0".into(),
            box_style: r,
            ..Default::default()
        };
        let got = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(got.style.display, Display::Flex);
        assert!((got.style.flex_grow - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pad_token_from_style_not_class() {
        let r = resolved("style({ pad(4) })");
        assert_eq!(r.pad.all.as_deref(), Some("4"));
        let elem = Element {
            tag: "div".into(),
            class: "s-pad8".into(),
            box_style: r,
            ..Default::default()
        };
        let got = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(got.style.padding.top, LengthPercentage::length(16.0));
        assert_eq!(got.style.padding.left, LengthPercentage::length(16.0));
    }

    #[test]
    fn when_hover_does_not_replace_resting_box() {
        let r = resolved("style({ display(flex), when: { hover: { display(none) } } })");
        assert_eq!(r.display.as_deref(), Some("flex"));
    }

    #[test]
    fn apply_resolved_boxes_matches_class_identity() {
        let r = resolved("style({ grow(1) })");
        let host = BoxHost::from_resolved("s-grow1 host-id", &r).expect("host");
        let mut elements = vec![
            Element::default(),
            Element {
                tag: "div".into(),
                class: "host-id s-grow1".into(),
                ..Default::default()
            },
        ];
        apply_resolved_boxes(&mut elements, &[host]);
        assert_eq!(elements[1].box_style.grow, Some(1.0));
    }
}
