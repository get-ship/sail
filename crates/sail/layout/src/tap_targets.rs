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
}
