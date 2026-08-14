//! Experimental `unintended-overflow` emit predicates.

use crate::html::Element;
use crate::style::ResolvedStyle;
use crate::taffy_tree::Layout;
use taffy::style::{Overflow, Position};

const EPSILON: f32 = 0.5;
const KIND_EPSILON: f32 = 2.0;

const WHEN_TABLET_MIN_WIDTH: f32 = 768.0;
const WHEN_DESKTOP_MIN_WIDTH: f32 = 1280.0;
const WHEN_MOBILE_MIN_WIDTH: f32 = 390.0;

pub(crate) fn should_emit(
    layout: &Layout,
    parent_index: usize,
    child_index: usize,
    main_axis_is_height: bool,
    viewport: (f32, f32),
    child_main: f32,
    parent_main: f32,
) -> bool {
    if child_main <= parent_main + KIND_EPSILON {
        return false;
    }
    if has_overflow_opt_in(&layout.elements[parent_index], &layout.styles[parent_index]) {
        return false;
    }
    if ancestor_scrolls_axis(layout, parent_index, main_axis_is_height) {
        return false;
    }
    if is_control_flow_tag(layout.elements[parent_index].tag.as_str())
        || is_control_flow_tag(layout.elements[child_index].tag.as_str())
    {
        return false;
    }
    if main_axis_is_height && !parent_has_definite_clipped_or_fixed_height(layout, parent_index) {
        return false;
    }
    if never_shown_self_or_ancestor(layout, parent_index, viewport)
        || never_shown_self_or_ancestor(layout, child_index, viewport)
    {
        return false;
    }
    if is_out_of_flow(&layout.elements[parent_index], &layout.styles[parent_index])
        || is_out_of_flow(&layout.elements[child_index], &layout.styles[child_index])
    {
        return false;
    }
    if out_of_flow_descendants_account_for_overflow(
        layout,
        child_index,
        main_axis_is_height,
        parent_main,
    ) {
        return false;
    }
    // Host splice / replaced subtree: taffy cannot see the runtime box.
    if layout
        .unknown_subtree
        .get(parent_index)
        .copied()
        .unwrap_or(false)
        || layout
            .unknown_subtree
            .get(child_index)
            .copied()
            .unwrap_or(false)
    {
        return false;
    }
    // Unbound `{expr}` text is measured as source, not runtime — unknown width.
    if measure_unknown(layout, parent_index) || subtree_measure_unknown(layout, child_index) {
        return false;
    }
    if is_display_contents(&layout.styles[parent_index]) {
        return false;
    }
    true
}

fn subtree_measure_unknown(layout: &Layout, index: usize) -> bool {
    if measure_unknown(layout, index) {
        return true;
    }
    layout.elements[index]
        .children
        .iter()
        .any(|&c| subtree_measure_unknown(layout, c))
}

/// Source interpolation / bind size: the solved box is a measurement lie.
pub(crate) fn measure_unknown(layout: &Layout, index: usize) -> bool {
    let elem = &layout.elements[index];
    if text_is_unbound_expr(&elem.text) {
        return true;
    }
    if attr_is_unbound(elem, "size")
        || attr_is_unbound(elem, "width")
        || attr_is_unbound(elem, "height")
    {
        return true;
    }
    for key in [
        "w", "h", "width", "height", "min-w", "min-h", "max-w", "max-h",
    ] {
        if elem.style.get(key).is_some_and(|v| value_is_unbound(v)) {
            return true;
        }
    }
    false
}

pub(crate) fn text_is_unbound_expr(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return false;
    }
    t.contains('{') && t.contains('}')
}

fn attr_is_unbound(elem: &Element, name: &str) -> bool {
    elem.attrs.get(name).is_some_and(|v| value_is_unbound(v))
}

fn value_is_unbound(v: &str) -> bool {
    let v = v.trim();
    v.contains('{') && v.contains('}')
}

fn is_display_contents(style: &ResolvedStyle) -> bool {
    // taffy 0.12 Display is Block/Flex/Grid/None — no Contents variant.
    let _ = style;
    false
}

fn is_control_flow_tag(tag: &str) -> bool {
    matches!(
        tag,
        "if" | "else" | "else-if" | "for" | "match" | "when" | "case" | "default"
    )
}

fn has_overflow_opt_in(elem: &Element, style: &ResolvedStyle) -> bool {
    if is_scroll_area_tag(&elem.tag)
        || class_has_scroll_area(&elem.class)
        || is_ticker_host(&elem.tag, &elem.class)
    {
        return true;
    }
    if elem.box_style.truncate {
        return true;
    }
    let o = style.style.overflow;
    !matches!(o.x, Overflow::Visible) || !matches!(o.y, Overflow::Visible)
}

fn ancestor_scrolls_axis(layout: &Layout, mut index: usize, height_axis: bool) -> bool {
    loop {
        match layout.elements[index].parent {
            Some(parent) if parent != 0 => index = parent,
            _ => return false,
        }
        if scrolls_axis(&layout.elements[index], &layout.styles[index], height_axis) {
            return true;
        }
    }
}

fn scrolls_axis(elem: &Element, style: &ResolvedStyle, height_axis: bool) -> bool {
    if is_scroll_area_tag(&elem.tag) || class_has_scroll_area(&elem.class) {
        return scroll_host_covers_axis(elem, height_axis);
    }
    if is_ticker_host(&elem.tag, &elem.class) {
        return !height_axis;
    }
    if elem.box_style.truncate && !height_axis {
        return true;
    }
    let o = if height_axis {
        style.style.overflow.y
    } else {
        style.style.overflow.x
    };
    matches!(o, Overflow::Scroll)
}

fn scroll_host_covers_axis(elem: &Element, height_axis: bool) -> bool {
    let raw = elem
        .attrs
        .get("orientation")
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let orientation = if raw.is_empty() {
        "vertical"
    } else {
        raw.as_str()
    };
    match orientation {
        "horizontal" => !height_axis,
        "both" => true,
        _ => height_axis,
    }
}

fn is_scroll_area_tag(tag: &str) -> bool {
    tag == "scroll-area" || tag.ends_with("-scroll-area") || tag == "scrollarea"
}

fn class_has_scroll_area(class: &str) -> bool {
    class
        .split_whitespace()
        .any(|c| c == "sail-scroll-area" || c.ends_with("-scroll-area"))
}

fn is_ticker_host(tag: &str, class: &str) -> bool {
    tag == "ticker"
        || tag.ends_with("-ticker")
        || class
            .split_whitespace()
            .any(|c| c == "sail-ticker" || c.ends_with("-ticker"))
}

fn parent_has_definite_clipped_or_fixed_height(layout: &Layout, parent_index: usize) -> bool {
    let style = &layout.styles[parent_index].style;
    let clips = matches!(
        style.overflow.y,
        Overflow::Hidden | Overflow::Clip | Overflow::Scroll
    );
    clips || !style.size.height.is_auto() || !style.max_size.height.is_auto()
}

fn never_shown_self_or_ancestor(layout: &Layout, mut index: usize, viewport: (f32, f32)) -> bool {
    loop {
        if is_never_shown(&layout.elements[index], &layout.styles[index], viewport) {
            return true;
        }
        match layout.elements[index].parent {
            Some(parent) if parent != 0 => index = parent,
            _ => return false,
        }
    }
}

fn is_never_shown(elem: &Element, style: &ResolvedStyle, viewport: (f32, f32)) -> bool {
    if style.style.display == taffy::style::Display::None {
        return true;
    }
    if attr_is_hidden(&elem.attrs) {
        return true;
    }
    if elem.tag == "when" && when_inactive_at_viewport(elem, viewport.0) {
        return true;
    }
    false
}

fn is_out_of_flow(elem: &Element, style: &ResolvedStyle) -> bool {
    style.style.position == Position::Absolute || is_overlay_host(&elem.tag, &elem.class)
}

fn is_overlay_host(tag: &str, class: &str) -> bool {
    matches!(
        tag,
        "dropdown"
            | "dialog"
            | "popover"
            | "tooltip"
            | "hover-card"
            | "hovercard"
            | "context-menu"
            | "contextmenu"
    ) || class.split_whitespace().any(|c| {
        matches!(
            c,
            "sail-dropdown"
                | "sail-dialog"
                | "sail-popover"
                | "sail-tooltip"
                | "sail-hover-card"
                | "sail-context-menu"
        )
    })
}

fn attr_is_hidden(attrs: &std::collections::HashMap<String, String>) -> bool {
    match attrs.get("hidden") {
        None => false,
        Some(v) => {
            let v = v.trim();
            v.is_empty() || v == "true" || v == "hidden" || v == "1"
        }
    }
}

fn when_inactive_at_viewport(elem: &Element, viewport_w: f32) -> bool {
    let token = when_breakpoint_token(elem);
    match token.as_deref() {
        Some("tablet") => viewport_w + EPSILON < WHEN_TABLET_MIN_WIDTH,
        Some("desktop") => viewport_w + EPSILON < WHEN_DESKTOP_MIN_WIDTH,
        Some("mobile") => viewport_w + EPSILON < WHEN_MOBILE_MIN_WIDTH,
        _ => false,
    }
}

fn when_breakpoint_token(elem: &Element) -> Option<String> {
    for key in elem.attrs.keys() {
        let raw = key
            .trim()
            .trim_matches('{')
            .trim_matches('}')
            .trim()
            .to_ascii_lowercase();
        if matches!(raw.as_str(), "tablet" | "desktop" | "mobile") {
            return Some(raw);
        }
    }
    None
}

fn out_of_flow_descendants_account_for_overflow(
    layout: &Layout,
    child_index: usize,
    height_axis: bool,
    parent_main: f32,
) -> bool {
    if !subtree_has_out_of_flow(layout, child_index) {
        return false;
    }
    in_flow_main(layout, child_index, height_axis) <= parent_main + KIND_EPSILON
}

fn subtree_has_out_of_flow(layout: &Layout, index: usize) -> bool {
    let mut stack = layout.elements[index].children.clone();
    while let Some(i) = stack.pop() {
        if is_out_of_flow(&layout.elements[i], &layout.styles[i]) {
            return true;
        }
        stack.extend(layout.elements[i].children.iter().copied());
    }
    false
}

fn in_flow_main(layout: &Layout, index: usize, height_axis: bool) -> f32 {
    if is_out_of_flow(&layout.elements[index], &layout.styles[index]) {
        return 0.0;
    }
    let children = &layout.elements[index].children;
    if children.is_empty() || !subtree_has_out_of_flow(layout, index) {
        return element_main(layout, index, height_axis);
    }
    children
        .iter()
        .map(|&i| in_flow_main(layout, i, height_axis))
        .sum()
}

fn element_main(layout: &Layout, index: usize, height_axis: bool) -> f32 {
    let Some(node) = layout.nodes[index] else {
        return 0.0;
    };
    let Ok(computed) = layout.taffy.layout(node) else {
        return 0.0;
    };
    if height_axis {
        computed.size.height
    } else {
        computed.size.width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Element;
    use crate::taffy_tree::build;

    #[test]
    fn overflow_opt_in_reads_taffy_style() {
        use crate::style::resolve;
        let hidden = Element {
            tag: "div".into(),
            box_style: crate::box_ir::ResolvedBox {
                overflow_x: Some("hidden".into()),
                overflow_y: Some("hidden".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let auto_y = Element {
            tag: "div".into(),
            box_style: crate::box_ir::ResolvedBox {
                overflow_y: Some("auto".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let plain = Element {
            tag: "div".into(),
            class: "s-h20 s-wfull".into(),
            ..Default::default()
        };
        let all = vec![Element::default()];
        assert!(has_overflow_opt_in(&hidden, &resolve(&hidden, &all, 400.0)));
        assert!(has_overflow_opt_in(&auto_y, &resolve(&auto_y, &all, 400.0)));
        assert!(!has_overflow_opt_in(&plain, &resolve(&plain, &all, 400.0)));
        assert!(is_ticker_host("ticker", ""));
        assert!(is_ticker_host("div", "sail-ticker"));
        assert!(!is_ticker_host("div", "sail-col"));
        let trunc = Element {
            tag: "div".into(),
            box_style: crate::box_ir::ResolvedBox {
                overflow_x: Some("hidden".into()),
                truncate: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(has_overflow_opt_in(&trunc, &resolve(&trunc, &all, 400.0)));
    }

    #[test]
    fn default_scroll_area_covers_height_only() {
        let elem = Element {
            tag: "scroll-area".into(),
            ..Default::default()
        };
        assert!(scroll_host_covers_axis(&elem, true));
        assert!(!scroll_host_covers_axis(&elem, false));
        let mut horizontal = Element {
            tag: "scroll-area".into(),
            ..Default::default()
        };
        horizontal
            .attrs
            .insert("orientation".into(), "horizontal".into());
        assert!(!scroll_host_covers_axis(&horizontal, true));
        assert!(scroll_host_covers_axis(&horizontal, false));
    }

    #[test]
    fn auto_height_parent_is_not_a_definite_clip() {
        let elements = vec![
            Element::default(),
            Element {
                tag: "div".to_string(),
                class: "sail-col s-wfull".to_string(),
                children: vec![2],
                parent: Some(0),
                ..Default::default()
            },
            Element {
                tag: "span".to_string(),
                class: "s-fontSizexs".to_string(),
                children: vec![],
                parent: Some(1),
                text: "No results".to_string(),
                ..Default::default()
            },
        ];
        let tree = build(elements, (400.0, 300.0)).unwrap();
        assert!(!parent_has_definite_clipped_or_fixed_height(&tree, 1));
    }

    #[test]
    fn fixed_height_parent_is_a_definite_clip() {
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
        assert!(parent_has_definite_clipped_or_fixed_height(&tree, 1));
    }

    #[test]
    fn hidden_false_is_not_unshown() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("hidden".into(), "false".into());
        assert!(!attr_is_hidden(&attrs));
        attrs.insert("hidden".into(), "true".into());
        assert!(attr_is_hidden(&attrs));
        attrs.insert("hidden".into(), String::new());
        assert!(attr_is_hidden(&attrs));
    }

    #[test]
    fn when_tablet_inactive_only_below_breakpoint() {
        let mut elem = Element {
            tag: "when".into(),
            ..Default::default()
        };
        elem.attrs.insert("{tablet}".into(), String::new());
        assert!(when_inactive_at_viewport(&elem, 390.0));
        assert!(!when_inactive_at_viewport(&elem, 1440.0));
        elem.attrs.clear();
        elem.attrs.insert("{desktop}".into(), String::new());
        assert!(when_inactive_at_viewport(&elem, 800.0));
        assert!(!when_inactive_at_viewport(&elem, 1440.0));
    }
}
