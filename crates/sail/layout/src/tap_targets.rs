//! Experimental `sub-min-tap-target` (SPEC-LAYOUT-DIAG-1 §2).
//!
//! Interactive hosts whose solved shorter axis is below the kind floor
//! (`FLOOR_PX` = 24). Token `--control-h-2xs` is 22px; this kind does not
//! follow that token.

use crate::diagnostics::{Diagnostic, Severity};
use crate::taffy_tree::Layout;

pub const KIND: &str = "sub-min-tap-target";
pub const FLOOR_PX: f32 = 24.0;
const EPSILON: f32 = 0.5;

pub fn scan(layout: &Layout, out: &mut Vec<Diagnostic>) -> anyhow::Result<()> {
    for (i, elem) in layout.elements.iter().enumerate() {
        if i == 0 {
            continue;
        }
        if !is_interactive(elem) {
            continue;
        }
        // Phrasing / control internals are not standalone targets. A label
        // caption is line-height, not chrome; the label owns the hit area
        // of a wrapped face.
        if is_phrasing_or_internal(elem, layout, i) {
            continue;
        }
        let Some(node) = layout.nodes[i] else {
            continue;
        };
        let computed = layout.taffy.layout(node).cloned()?;
        let w = computed.size.width;
        let h = computed.size.height;
        if w <= EPSILON || h <= EPSILON {
            continue;
        }
        // Only the unknown host itself is a measurement lie. A known control
        // with an unknown descendant still has an honest solved box.
        if elem.is_unknown() {
            continue;
        }
        let short = w.min(h);
        if short + EPSILON >= FLOOR_PX {
            continue;
        }
        out.push(Diagnostic {
            kind: KIND,
            severity: Severity::Warning,
            element_index: i,
            element_tag: elem.tag.clone(),
            element_class: elem.class.clone(),
            message: format!(
                "<{}> tap target is {short:.0}×{long:.0}px — shorter axis is below \
                 control-h-2xs ({FLOOR_PX:.0}px). Grow the control or use a \
                 `control-h-*` size token.",
                elem.tag,
                long = w.max(h),
            ),
            span: elem.span,
            geometry: Some(crate::OverflowGeometry {
                axis: if w <= h {
                    crate::MainAxis::Width
                } else {
                    crate::MainAxis::Height
                },
                child_main: short,
                parent_main: FLOOR_PX,
                child_index: i,
                parent_index: i,
                child: crate::BoxExtents {
                    x: computed.location.x,
                    y: computed.location.y,
                    width: w,
                    height: h,
                },
                parent: crate::BoxExtents {
                    x: 0.0,
                    y: 0.0,
                    width: FLOOR_PX,
                    height: FLOOR_PX,
                },
            }),
        });
    }
    Ok(())
}

fn is_phrasing_or_internal(elem: &crate::html::Element, layout: &Layout, index: usize) -> bool {
    if is_control_internal(elem) {
        return true;
    }
    if is_caption_label(elem) {
        return true;
    }
    if is_phrasing_anchor(elem) {
        return true;
    }
    if elem.tag == "summary" && !has_author_box(elem) {
        return true;
    }
    if elem.tag == "text" {
        return true;
    }
    // Face sitting inside a label — the label is the target.
    if is_control_face_tag(elem) && ancestor_is_label(layout, index) {
        return true;
    }
    false
}

fn is_control_internal(elem: &crate::html::Element) -> bool {
    if elem.tag == "input" {
        let ty = elem.attrs.get("type").map(|s| s.as_str()).unwrap_or("text");
        if matches!(ty, "checkbox" | "radio" | "range") {
            return true;
        }
    }
    elem.class.split_whitespace().any(|c| {
        matches!(
            c,
            "sail-switch" | "sail-checkbox" | "sail-radio" | "sail-slider"
        )
    })
}

fn is_control_face_tag(elem: &crate::html::Element) -> bool {
    matches!(elem.tag.as_str(), "input" | "span")
}

fn is_caption_label(elem: &crate::html::Element) -> bool {
    elem.tag == "label" && !has_author_box(elem)
}

fn is_phrasing_anchor(elem: &crate::html::Element) -> bool {
    if elem.tag != "a" || !elem.attrs.contains_key("href") {
        return false;
    }
    if has_author_box(elem) || is_flex_control(elem) {
        return false;
    }
    true
}

fn is_flex_control(elem: &crate::html::Element) -> bool {
    elem.class.split_whitespace().any(|c| {
        matches!(
            c,
            "sail-row" | "sail-col" | "sail-row-i" | "sail-col-i" | "sail-center"
        ) || {
            let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
            matches!(
                s.as_str(),
                "displayflex" | "displayinlineflex" | "inlineflex" | "inline-flex"
            )
        }
    })
}

fn has_author_box(elem: &crate::html::Element) -> bool {
    const STYLE_KEYS: &[&str] = &[
        "w",
        "h",
        "width",
        "height",
        "min-width",
        "min-height",
        "minw",
        "minh",
    ];
    if elem.style.keys().any(|k| {
        let k = k.trim_start_matches('-').to_ascii_lowercase();
        STYLE_KEYS.iter().any(|s| k == *s)
    }) {
        return true;
    }
    elem.class.split_whitespace().any(|c| {
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        s.contains("minw")
            || s.contains("minh")
            || s.contains("controlh")
            || s.contains("iconsiz")
            || is_explicit_wh_token(&s)
    })
}

fn is_explicit_wh_token(s: &str) -> bool {
    let rest = if let Some(r) = s.strip_prefix("minw") {
        r
    } else if let Some(r) = s.strip_prefix("minh") {
        r
    } else if let Some(r) = s.strip_prefix('w') {
        r
    } else if let Some(r) = s.strip_prefix('h') {
        r
    } else {
        return false;
    };
    rest.chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit() || c == '(')
}

fn ancestor_is_label(layout: &Layout, mut index: usize) -> bool {
    loop {
        match layout.elements[index].parent {
            Some(parent) if parent != 0 => index = parent,
            _ => return false,
        }
        if layout.elements[index].tag == "label" {
            return true;
        }
    }
}

fn is_interactive(elem: &crate::html::Element) -> bool {
    let tag = elem.tag.as_str();
    if matches!(tag, "button" | "select" | "textarea" | "summary" | "label") {
        return true;
    }
    if tag == "a" && elem.attrs.contains_key("href") {
        return true;
    }
    if tag == "input" {
        let ty = elem.attrs.get("type").map(|s| s.as_str()).unwrap_or("text");
        return ty != "hidden";
    }
    if let Some(role) = elem.attrs.get("role") {
        if matches!(
            role.as_str(),
            "button" | "link" | "tab" | "switch" | "checkbox" | "radio" | "menuitem"
        ) {
            return true;
        }
    }
    elem.attrs.keys().any(|k| {
        let k = k.to_ascii_lowercase();
        k.starts_with("on:") || k.starts_with("onclick") || k == "on"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Element;
    use std::collections::HashMap;

    fn el(tag: &str, attrs: &[(&str, &str)]) -> Element {
        let mut map = HashMap::new();
        for (k, v) in attrs {
            map.insert((*k).into(), (*v).into());
        }
        Element {
            tag: tag.into(),
            attrs: map,
            ..Default::default()
        }
    }

    #[test]
    fn button_is_interactive() {
        assert!(is_interactive(&el("button", &[])));
    }

    #[test]
    fn anchor_needs_href() {
        assert!(!is_interactive(&el("a", &[])));
        assert!(is_interactive(&el("a", &[("href", "#")])));
    }

    #[test]
    fn on_handler_is_interactive() {
        assert!(is_interactive(&el("div", &[("on:click", "{go}")])));
        assert!(!is_interactive(&el("div", &[])));
    }

    #[test]
    fn hidden_input_is_not() {
        assert!(!is_interactive(&el("input", &[("type", "hidden")])));
        assert!(is_interactive(&el("input", &[("type", "text")])));
    }

    #[test]
    fn checkbox_radio_range_are_control_internals() {
        assert!(is_control_internal(&el("input", &[("type", "checkbox")])));
        assert!(is_control_internal(&el("input", &[("type", "radio")])));
        assert!(is_control_internal(&el("input", &[("type", "range")])));
        assert!(!is_control_internal(&el("input", &[("type", "text")])));
        assert!(!is_control_internal(&el("button", &[])));
    }

    #[test]
    fn caption_label_without_author_box_is_not_a_standalone_target() {
        assert!(is_caption_label(&el("label", &[])));
        let mut boxed = el("label", &[]);
        boxed.style.insert("w".into(), "23px".into());
        boxed.style.insert("h".into(), "40px".into());
        assert!(!is_caption_label(&boxed));
    }

    #[test]
    fn phrasing_anchor_needs_href_and_no_author_box() {
        assert!(!is_phrasing_anchor(&el("a", &[])));
        assert!(is_phrasing_anchor(&el("a", &[("href", "#")])));
        let mut boxed = el("a", &[("href", "#")]);
        boxed.style.insert("w".into(), "23px".into());
        boxed.style.insert("h".into(), "40px".into());
        assert!(!is_phrasing_anchor(&boxed));
        let mut flex = el("a", &[("href", "#")]);
        flex.class = "sail-row s-displayinlineflex".into();
        assert!(!is_phrasing_anchor(&flex));
    }
}
