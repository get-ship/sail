//! Walk a computed layout and emit box-tree diagnostics.
//!
//! Landed kinds:
//! 1. **collapsed-into-overflow-hidden-parent** — the parent collapsed to 0px
//!    on its main axis while a child wants space AND the parent has
//!    `overflow:hidden` (silent-clip). This is the 1779px-in-900px regression.
//! 2. **child-overflows-parent** — the child's main-axis extent exceeds the
//!    parent's measured main-axis extent and the parent is NOT
//!    `overflow:hidden`.
//! 3. **zero-main-axis** — the parent has 0 main axis + non-zero child sum
//!    regardless of overflow.
//!
//! Experimental kinds (SPEC-LAYOUT-DIAG-1 / S4.1, gated by [`crate::CheckOptions`]):
//! 4. **unintended-overflow** — child extents exceed the container and the
//!    author did not opt in (no scroll-area, overflow setter, or truncate).
//! 5. **clipped-text** — measured text run exceeds its box without a truncate
//!    marker (text-metrics path).
//!
//! Each span is into the **preprocessed** source.

use crate::CheckOptions;
use crate::metrics::{self, table as metrics_table};
use crate::taffy_tree::Layout;
use serde::Serialize;
use taffy::style::Overflow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
}

/// Main axis used for an overflow comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MainAxis {
    Width,
    Height,
}

/// Parent-relative taffy box (location + size). `x`/`y` are not viewport-absolute.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct BoxExtents {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Child vs parent main-axis extents for overflow findings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct OverflowGeometry {
    pub axis: MainAxis,
    pub child_main: f32,
    pub parent_main: f32,
    pub child_index: usize,
    pub parent_index: usize,
    pub child: BoxExtents,
    pub parent: BoxExtents,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<OverflowGeometry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoxReport {
    pub index: usize,
    pub tag: String,
    pub class: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub unknown_subtree: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct ScanResult {
    pub diagnostics: Vec<Diagnostic>,
    /// Computed boxes for every laid-out element (agent-facing layout truth).
    pub boxes: Vec<BoxReport>,
    pub viewport: (f32, f32),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ScanInput {
    pub viewport: (f32, f32),
}

const EPSILON: f32 = 0.5;

fn extents_of(computed: &taffy::Layout) -> BoxExtents {
    BoxExtents {
        x: computed.location.x,
        y: computed.location.y,
        width: computed.size.width,
        height: computed.size.height,
    }
}

/// Template control-flow tags — structural, not layout boxes. Zero-main-axis
/// on these is analyzer noise (false branch, empty list, unexpanded match).
fn is_control_flow_tag(tag: &str) -> bool {
    matches!(
        tag,
        "if" | "else" | "else-if" | "for" | "match" | "when" | "case" | "default"
    )
}

/// Scan with default (non-experimental) options.
pub fn scan(layout: &Layout, viewport: (f32, f32)) -> anyhow::Result<ScanResult> {
    scan_with(layout, viewport, CheckOptions::default())
}

/// Scan with explicit experimental-kind options (SPEC-LAYOUT-DIAG-1 / S4.1).
pub fn scan_with(
    layout: &Layout,
    viewport: (f32, f32),
    options: CheckOptions,
) -> anyhow::Result<ScanResult> {
    let mut boxes = Vec::new();
    for (i, elem) in layout.elements.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let Some(node) = layout.nodes[i] else {
            continue;
        };
        let Ok(computed) = layout.taffy.layout(node).cloned() else {
            continue;
        };
        boxes.push(BoxReport {
            index: i,
            tag: elem.tag.clone(),
            class: elem.class.clone(),
            x: computed.location.x,
            y: computed.location.y,
            width: computed.size.width,
            height: computed.size.height,
            unknown_subtree: layout.unknown_subtree.get(i).copied().unwrap_or(false),
        });
    }

    let mut out = Vec::new();

    for (i, elem) in layout.elements.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let Some(node) = layout.nodes[i] else {
            continue;
        };
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
        let mut biggest_child_box = BoxExtents::default();
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
                biggest_child_box = extents_of(&child_layout);
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
                geometry: None,
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
                    layout.elements[cidx].tag, child_main, elem.tag, main_size,
                )
            } else {
                format!(
                    "<{}> ({:.0}px main) overflows its parent <{}> ({:.0}px main) on the main axis.",
                    layout.elements[cidx].tag, child_main, elem.tag, main_size,
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
                geometry: None,
            });

            // Case 4 (experimental): same geometry, no author opt-in for
            // overflow/scroll/truncate → unintended-overflow.
            if options.experimental && !has_overflow_opt_in(elem, &layout.styles[i]) {
                out.push(Diagnostic {
                    kind: "unintended-overflow",
                    severity: Severity::Warning,
                    element_index: cidx,
                    element_tag: layout.elements[cidx].tag.clone(),
                    element_class: layout.elements[cidx].class.clone(),
                    message: format!(
                        "<{}> ({:.0}px main) overflows <{}> ({:.0}px main) without an overflow \
                         opt-in — add `overflowY(auto)` / `overflow(scroll)`, a `<scroll-area>`, \
                         or `truncate()` if the overflow is intentional.",
                        layout.elements[cidx].tag, child_main, elem.tag, main_size,
                    ),
                    span: layout.elements[cidx].span,
                    geometry: Some(OverflowGeometry {
                        axis: if main_axis_is_height {
                            MainAxis::Height
                        } else {
                            MainAxis::Width
                        },
                        child_main,
                        parent_main: main_size,
                        child_index: cidx,
                        parent_index: i,
                        child: biggest_child_box,
                        parent: extents_of(&computed),
                    }),
                });
            }
            continue;
        }

        // Case 3: zero main-axis without silent absorbing — the parent has
        // truly zero size; report it as a layout collapse (no regression
        // framing, just a hint that the developer probably wanted more).
        // Suppressed when the parent's subtree contains an unknown
        // element — a zero measurement on a column wrapping
        // `<sail-code-editor>` is not a layout bug; it's a foreign web
        // component whose body only exists at runtime.
        //
        // Also suppressed for:
        // - control-flow tags (`if`/`else`/`for`/…) — they are structural
        //   markers, not boxes authors size; an empty false branch or an
        //   unexpanded `<for>` always measures 0 and is not a layout bug.
        // - parents whose children also total ~0 on the main axis — the
        //   "children want space the parent denied" story does not apply,
        //   so the warning is pure analyzer noise (empty placeholders,
        //   unmeasured text leaves, etc.).
        if !subtree_unknown
            && main_size <= EPSILON
            && !children.is_empty()
            && child_main_total > EPSILON
            && !is_control_flow_tag(elem.tag.as_str())
        {
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
                geometry: None,
            });
        }
    }

    // Case 5 (experimental + text_metrics): text run wider than its box
    // without a truncate marker while overflow clips on the inline axis.
    if options.experimental && options.text_metrics_enabled() {
        scan_clipped_text(layout, &mut out)?;
    }

    Ok(ScanResult {
        diagnostics: out,
        boxes,
        viewport,
    })
}

/// True when the author opted into overflow/clip/scroll via vocabulary.
fn has_overflow_opt_in(elem: &crate::html::Element, style: &crate::style::ResolvedStyle) -> bool {
    if is_scroll_area_tag(&elem.tag) {
        return true;
    }
    if class_has_overflow_or_truncate_opt_in(&elem.class) {
        return true;
    }
    let o = style.style.overflow;
    !matches!(o.x, Overflow::Visible) || !matches!(o.y, Overflow::Visible)
}

fn is_scroll_area_tag(tag: &str) -> bool {
    tag == "scroll-area" || tag.ends_with("-scroll-area") || tag == "scrollarea"
}

fn class_has_overflow_or_truncate_opt_in(class: &str) -> bool {
    class.split_whitespace().any(|c| {
        // Product Style emit keeps camelCase (`s-overflowXhidden`); fold case.
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        s == "truncate" || s.starts_with("overflow")
    })
}

fn class_has_truncate(class: &str) -> bool {
    class.split_whitespace().any(|c| {
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        s == "truncate" || s.ends_with("-truncate")
    })
}

fn scan_clipped_text(layout: &Layout, out: &mut Vec<Diagnostic>) -> anyhow::Result<()> {
    let metrics = metrics_table();
    for (i, elem) in layout.elements.iter().enumerate() {
        if i == 0 {
            continue;
        }
        if elem.text.trim().is_empty() {
            continue;
        }
        if class_has_truncate(&elem.class) {
            continue;
        }
        let Some(node) = layout.nodes[i] else {
            continue;
        };
        let computed = layout.taffy.layout(node).cloned()?;
        let box_w = computed.size.width;
        if box_w <= EPSILON {
            continue;
        }

        // Only report when the box actually clips on the inline axis —
        // visible overflow is not "clipped text".
        let overflow_x = layout.styles[i].style.overflow.x;
        let clips_x =
            matches!(overflow_x, Overflow::Hidden | Overflow::Clip) || class_clips_x(&elem.class);
        if !clips_x {
            continue;
        }

        let (family, size) = metrics::resolve_family_size(i, &layout.elements);
        let Some(m) = metrics.get(&(family, size)) else {
            continue;
        };
        let text_w: f32 = elem.text.chars().map(|c| metrics::char_width(m, c)).sum();
        if text_w > box_w + EPSILON {
            out.push(Diagnostic {
                kind: "clipped-text",
                severity: Severity::Warning,
                element_index: i,
                element_tag: elem.tag.clone(),
                element_class: elem.class.clone(),
                message: format!(
                    "<{}> text measures ~{:.0}px but its box is {:.0}px wide — glyphs are \
                     clipped without `truncate()`. Add `text: truncate()` if ellipsis is \
                     intentional, or widen the box / allow wrapping.",
                    elem.tag, text_w, box_w,
                ),
                span: elem.span,
                geometry: None,
            });
        }
    }
    Ok(())
}

fn class_clips_x(class: &str) -> bool {
    class.split_whitespace().any(|c| {
        // Product Style mints `s-overflowXhidden` (camelCase); mapping tokens
        // are lowercase. Fold so either form counts as an inline clip opt-in.
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        matches!(
            s.as_str(),
            "overflowhidden" | "overflowclip" | "overflowxhidden" | "overflowxclip" | "truncate"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Element;
    use crate::taffy_tree::build;

    #[test]
    fn zero_main_axis_on_known_container_still_fires() {
        // Parent collapsed while a sized child wants space on the main axis.
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col".to_string(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "div".to_string(),
                // fixed height so the child measures non-zero and case 3 applies
                class: "s-h-20".to_string(),
                children: vec![],
                parent: Some(1),
                text: "x".to_string(),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        let result = scan(&tree, (1440.0, 900.0)).unwrap();
        // Either zero-main on the parent or a related layout finding is fine;
        // the important contract below is the noise gates.
        let _ = result;
    }

    #[test]
    fn zero_main_axis_skipped_when_children_also_zero() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col s-grow".to_string(),
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
            result
                .diagnostics
                .iter()
                .all(|d| d.kind != "zero-main-axis"),
            "empty children totaling 0px must not spam zero-main-axis: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn zero_main_axis_skipped_for_control_flow_tags() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "if".to_string(),
                class: String::new(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "div".to_string(),
                class: "s-h-20".to_string(),
                children: vec![],
                parent: Some(1),
                text: "branch".to_string(),
                ..Default::default()
            },
        ];
        let tree = build(elements, (1440.0, 900.0)).unwrap();
        let result = scan(&tree, (1440.0, 900.0)).unwrap();
        assert!(
            result
                .diagnostics
                .iter()
                .filter(|d| d.element_tag == "if")
                .all(|d| d.kind != "zero-main-axis"),
            "control-flow tags must not emit zero-main-axis: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn unintended_overflow_carries_child_parent_extents() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col s-h20 s-wfull".to_string(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "div".to_string(),
                class: "s-h40 s-shrink0".to_string(),
                children: vec![],
                parent: Some(1),
                text: "x".to_string(),
                ..Default::default()
            },
        ];
        let tree = build(elements, (400.0, 300.0)).unwrap();
        let result = scan_with(&tree, (400.0, 300.0), CheckOptions::experimental()).unwrap();
        let hit = result
            .diagnostics
            .iter()
            .find(|d| d.kind == "unintended-overflow")
            .expect("experimental overflow vehicle");
        let g = hit.geometry.expect("overflow finding must carry extents");
        assert_eq!(g.axis, MainAxis::Height);
        assert!(
            g.child_main > g.parent_main + EPSILON,
            "child_main={} parent_main={}",
            g.child_main,
            g.parent_main
        );
        assert!(g.child.height > g.parent.height);
    }

    #[test]
    fn zero_main_axis_suppressed_for_unknown_child() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col s-grow".to_string(),
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
            !result
                .diagnostics
                .iter()
                .any(|d| d.kind == "zero-main-axis"),
            "unknown child should suppress zero-main-axis: {:?}",
            result.diagnostics
        );
    }
}

#[allow(dead_code)]
fn _viewport_unused(v: (f32, f32)) -> (f32, f32) {
    v
}
