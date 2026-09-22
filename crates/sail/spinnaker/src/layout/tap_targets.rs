//! Experimental `sub-min-tap-target` (SPEC-LAYOUT-DIAG-1 §2).
//!
//! Interactive hosts whose solved shorter axis is below the kind floor
//! (`FLOOR_PX` = 24). Token `--control-h-2xs` is 22px; this kind does not
//! follow that token.
//!
//! LAYOUT-5 scopes (not standalone targets):
//! - phrasing / inline content (`a`, `em`, `span`, … without control chrome)
//! - control internals (checkbox/radio/switch face, descendants of a label/button)
//! - the wrapping `<label>` owns the hit area

use crate::layout::diagnostics::{Diagnostic, Severity};
use crate::layout::taffy_tree::Layout;
use crate::layout::unintended_overflow;

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
        if unintended_overflow::measure_unknown(layout, i) {
            continue;
        }
        if is_phrasing_not_control(layout, i) {
            continue;
        }
        if is_control_internal(layout, i) {
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
                 the 24px floor. Grow the control so both axes are at least 24px, \
                 or use a `control-h-*` size token.",
                elem.tag,
                long = w.max(h),
            ),
            span: elem.span,
            geometry: Some(crate::layout::OverflowGeometry {
                axis: if w <= h {
                    crate::layout::MainAxis::Width
                } else {
                    crate::layout::MainAxis::Height
                },
                child_main: short,
                parent_main: FLOOR_PX,
                child_index: i,
                parent_index: i,
                child: crate::layout::BoxExtents {
                    x: computed.location.x,
                    y: computed.location.y,
                    width: w,
                    height: h,
                },
                parent: crate::layout::BoxExtents {
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

fn is_interactive(elem: &crate::layout::html::Element) -> bool {
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

fn is_phrasing_not_control(layout: &Layout, index: usize) -> bool {
    let elem = &layout.elements[index];
    if !is_phrasing_tag(elem.tag.as_str()) {
        return false;
    }
    if has_control_chrome(elem) || has_explicit_box(elem) {
        return false;
    }
    let Some(parent) = elem.parent.filter(|&p| p != 0) else {
        return true;
    };
    // Flex/grid/nav clusters (docs rail, toolbars) are standalone targets.
    !parent_is_standalone_cluster(&layout.elements[parent])
}

fn is_phrasing_tag(tag: &str) -> bool {
    matches!(
        tag,
        "a" | "em"
            | "strong"
            | "span"
            | "code"
            | "small"
            | "i"
            | "b"
            | "u"
            | "cite"
            | "abbr"
            | "mark"
            | "kbd"
            | "time"
    )
}

fn parent_is_standalone_cluster(parent: &crate::layout::html::Element) -> bool {
    if matches!(
        parent.tag.as_str(),
        "nav" | "row" | "column" | "grid" | "menu" | "toolbar"
    ) {
        return true;
    }
    if parent.class.split_whitespace().any(|c| {
        matches!(
            c,
            "sail-row" | "sail-col" | "sail-grid" | "sail-center" | "sail-nav"
        )
    }) {
        return true;
    }
    matches!(
        parent.box_style.display.as_deref(),
        Some("flex") | Some("grid") | Some("inline-flex")
    )
}

fn has_explicit_box(elem: &crate::layout::html::Element) -> bool {
    let b = &elem.box_style;
    if b.width.is_some() || b.height.is_some() || b.min_width.is_some() || b.min_height.is_some() {
        return true;
    }
    for key in ["w", "h", "width", "height", "min-w", "min-h"] {
        if elem.style.contains_key(key) {
            return true;
        }
    }
    false
}

fn has_control_chrome(elem: &crate::layout::html::Element) -> bool {
    elem.class.split_whitespace().any(|c| {
        matches!(
            c,
            "sail-button"
                | "sail-chip"
                | "sail-toggle"
                | "sail-tab"
                | "sail-menu-item"
                | "sail-page"
        ) || c.starts_with("sail-button")
            || c.contains("controlh")
            || c.contains("control-h")
    })
}

fn is_control_internal(layout: &Layout, index: usize) -> bool {
    let elem = &layout.elements[index];
    if is_control_face(elem) && ancestor_is_hit_owner(layout, index) {
        return true;
    }
    ancestor_is_hit_owner(layout, index)
}

fn is_control_face(elem: &crate::layout::html::Element) -> bool {
    if elem.tag == "input" {
        let ty = elem.attrs.get("type").map(|s| s.as_str()).unwrap_or("text");
        if matches!(ty, "checkbox" | "radio") {
            return true;
        }
    }
    if elem
        .attrs
        .get("role")
        .is_some_and(|r| matches!(r.as_str(), "checkbox" | "radio" | "switch"))
    {
        return true;
    }
    elem.class.split_whitespace().any(|c| {
        matches!(
            c,
            "sail-checkbox" | "sail-radio" | "sail-switch" | "sail-icon"
        )
    })
}

/// Label (or a wrapping button/summary) owns the hit; internals are not targets.
fn ancestor_is_hit_owner(layout: &Layout, mut index: usize) -> bool {
    loop {
        match layout.elements[index].parent {
            Some(parent) if parent != 0 => index = parent,
            _ => return false,
        }
        let tag = layout.elements[index].tag.as_str();
        if matches!(tag, "label" | "button" | "summary") {
            return true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::html::Element;
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
    fn phrasing_tags_and_control_chrome() {
        assert!(is_phrasing_tag("a"));
        assert!(is_phrasing_tag("em"));
        assert!(!is_phrasing_tag("button"));
        let mut btn = el("a", &[("href", "#")]);
        btn.class = "sail-button".into();
        assert!(has_control_chrome(&btn));
    }
}
