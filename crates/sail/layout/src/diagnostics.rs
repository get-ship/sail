//! Walk a computed layout and emit the three unambiguous box-tree diagnostics
//! the spike targets.
//!
//! 1. **collapsed-into-overflow-hidden-parent** — the parent collapsed to 0px
//!    on its main axis while a child wants space AND the parent has
//!    `overflow:hidden` (silent-clip). This is the 1779px-in-900px regression.
//! 2. **child-overflows-parent** — the child's main-axis extent exceeds the
//!    parent's measured main-axis extent and the parent is NOT
//!    `overflow:hidden`.
//! 3. **zero-main-axis** — the parent has 0 main axis + non-zero child sum
//!    regardless of overflow.
//!
//! Each span is into the **preprocessed** source.

use crate::taffy_tree::Layout;
use serde::Serialize;
use taffy::style::Overflow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub kind: &'static str,
    pub severity: Severity,
    pub element_index: usize,
    pub element_tag: String,
    pub element_class: String,
    pub message: String,
    pub span: (usize, usize),
}

#[derive(Debug, Default, Serialize)]
pub struct ScanResult {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ScanInput {
    pub viewport: (f32, f32),
}

const EPSILON: f32 = 0.5;

pub fn scan(layout: &Layout, _viewport: (f32, f32)) -> anyhow::Result<ScanResult> {
    let mut out = Vec::new();

    for (i, elem) in layout.elements.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let Some(node) = layout.nodes[i] else { continue };
        let computed = layout.taffy.layout(node).cloned()?;
        let children = &elem.children;
        if children.is_empty() {
            continue;
        }

        let is_row = elem.class.split_whitespace().any(|c| c == "sail-row");
        let main_axis_is_height = !is_row;
        let main_size = if main_axis_is_height {
            computed.size.height
        } else {
            computed.size.width
        };

        let mut child_main_total: f32 = 0.0;
        let mut biggest_child_main: (usize, f32) = (0, 0.0);
        for &ci in children {
            let child_node = layout.nodes[ci].expect("child has a node");
            let child_layout = layout.taffy.layout(child_node).cloned()?;
            let c_main = if main_axis_is_height {
                child_layout.size.height
            } else {
                child_layout.size.width
            };
            child_main_total += c_main;
            if c_main > biggest_child_main.1 {
                biggest_child_main = (ci, c_main);
            }
        }

        let subtree_unknown = layout.unknown_subtree.get(i).copied().unwrap_or(false);
        let biggest_child_unknown = layout
            .unknown_subtree
            .get(biggest_child_main.0)
            .copied()
            .unwrap_or(false);

        let parent_overflow = layout.styles[i].style.overflow;
        let parent_silently_absorbs = matches!(
            parent_overflow,
            taffy::geometry::Point {
                x: Overflow::Hidden | Overflow::Clip,
                y: Overflow::Hidden | Overflow::Clip
            }
        );

        // Case 1: the COLLAPSED-PARENT 1779px regression. Parent has 0 main
        // axis size + non-zero children + `overflow:hidden` (or `clip`) →
        // the silent clip is exactly what the spec example describes.
        // Suppressed when the parent's subtree contains an unknown element
        // — we can't tell whether the child wants space at compile time.
        if !subtree_unknown
            && !biggest_child_unknown
            && main_size <= EPSILON
            && biggest_child_main.1 > EPSILON
            && parent_silently_absorbs
        {
            let (cidx, child_main) = biggest_child_main;
            out.push(Diagnostic {
                kind: "collapsed-into-overflow-hidden-parent",
                severity: Severity::Error,
                element_index: cidx,
                element_tag: layout.elements[cidx].tag.clone(),
                element_class: layout.elements[cidx].class.clone(),
                message: format!(
                    "<{}> ({:.0}px content) bounds against a 0px parent with `overflow:hidden` \
                     — the parent collapsed while its content wanted more. The excess is \
                     silently clipped (the 1779px-in-900px regression). The parent likely \
                     lacks a height constraint that propagates from a stretching ancestor — \
                     check that `<row align=\"stretch\" h-fill>` is itself inside a height-fixed \
                     shell, or switch `overflow-hidden` → `overflow-auto` to surface the bug.",
                    layout.elements[cidx].tag, child_main,
                ),
                span: layout.elements[cidx].span,
            });
            continue;
        }

        // Case 2: child is bigger than parent's main axis (parent not silent).
        if biggest_child_main.1 > main_size + EPSILON && main_size > EPSILON {
            let (cidx, child_main) = biggest_child_main;
            let kind = if parent_silently_absorbs {
                "unbounded-growth-into-overflow-hidden-parent"
            } else {
                "child-overflows-parent"
            };
            let severity = if parent_silently_absorbs {
                Severity::Error
            } else {
                Severity::Warning
            };
            let msg = if parent_silently_absorbs {
                format!(
                    "<{}> ({:.0}px main) overflows its parent <{}> ({:.0}px main) which has \
                     `overflow:hidden` — the excess is clipped silent (the 1779px-in-900px \
                     regression). Bound the child (`max-h-*`) or the parent (`max-h-*` / \
                     `h-fill + overflow-auto`).",
                    layout.elements[cidx].tag,
                    child_main,
                    elem.tag,
                    main_size,
                )
            } else {
                format!(
                    "<{}> ({:.0}px main) overflows its parent <{}> ({:.0}px main) on the main axis.",
                    layout.elements[cidx].tag,
                    child_main,
                    elem.tag,
                    main_size,
                )
            };
            out.push(Diagnostic {
                kind,
                severity,
                element_index: cidx,
                element_tag: layout.elements[cidx].tag.clone(),
                element_class: layout.elements[cidx].class.clone(),
                message: msg,
                span: layout.elements[cidx].span,
            });
            continue;
        }

        // Case 3: zero main-axis without silent absorbing — the parent has
        // truly zero size; report it as a layout collapse (no regression
        // framing, just a hint that the developer probably wanted more).
        // Suppressed when the parent's subtree contains an unknown
        // element — a zero measurement on a column wrapping
        // `<sail-code-editor>` is not a layout bug; it's a foreign web
        // component whose body only exists at runtime.
        if !subtree_unknown && main_size <= EPSILON && !children.is_empty() {
            out.push(Diagnostic {
                kind: "zero-main-axis",
                severity: Severity::Warning,
                element_index: i,
                element_tag: elem.tag.clone(),
                element_class: elem.class.clone(),
                message: format!(
                    "<{}> measures 0px on its main axis while having {} child(ren) \
                     totaling {:.0}px. The box is collapsed — likely a missing \
                     width/height on a container whose children measure Auto, \
                     or a flex column whose parent doesn't stretch.",
                    elem.tag,
                    children.len(),
                    child_main_total
                ),
                span: elem.span,
            });
        }
    }

    Ok(ScanResult { diagnostics: out })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Element;
    use crate::taffy_tree::build;

    #[test]
    fn zero_main_axis_on_known_container_still_fires() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col sx-grow".to_string(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "div".to_string(),
                class: String::new(),
                children: vec![],
                parent: Some(1),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        let result = scan(&tree, (1440.0, 900.0)).unwrap();
        assert!(
            result.diagnostics.iter().any(|d| d.kind == "zero-main-axis"),
            "known empty child should still trigger zero-main-axis"
        );
    }

    #[test]
    fn zero_main_axis_suppressed_for_unknown_child() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col sx-grow".to_string(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "iframe".to_string(),
                class: String::new(),
                children: vec![],
                parent: Some(1),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        let result = scan(&tree, (1440.0, 900.0)).unwrap();
        assert!(
            !result.diagnostics.iter().any(|d| d.kind == "zero-main-axis"),
            "unknown child should suppress zero-main-axis: {:?}",
            result.diagnostics
        );
    }
}

#[allow(dead_code)]
fn _viewport_unused(v: (f32, f32)) -> (f32, f32) {
    v
}
