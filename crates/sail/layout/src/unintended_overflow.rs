//! `unintended-overflow` emit predicates (promoted, LAYOUT-5).

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
        viewport,
    ) {
        return false;
    }
    // Unbound `{expr}` width is a measurement lie — the source text is not
    // the runtime string. Recorded as this class; do not emit.
    if !main_axis_is_height && child_width_is_unbound_expr(layout, child_index) {
        return false;
    }
    // Stdlib splice: L1 host min-size vs leftover (or inner splice vs host
    // token) is cascade geometry, not author overflow.
    if host_splice_owns_geometry(layout, parent_index, child_index) {
        return false;
    }
    true
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
    if class_has_overflow_or_truncate_opt_in(&elem.class) {
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
    if class_has_truncate(&elem.class) && !height_axis {
        return true;
    }
    if class_scrolls_axis(&elem.class, height_axis) {
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

fn class_scrolls_axis(class: &str, height_axis: bool) -> bool {
    class.split_whitespace().any(|c| {
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        let (x, y) = overflow_token_scroll_axes(&s);
        if height_axis { y } else { x }
    })
}

/// `(scrolls_x, scrolls_y)` for one class token after `s-` fold.
fn overflow_token_scroll_axes(s: &str) -> (bool, bool) {
    let Some(idx) = s.find("overflow") else {
        return (false, false);
    };
    let rest = &s[idx + "overflow".len()..];
    if !(rest.contains("auto") || rest.contains("scroll")) {
        return (false, false);
    }
    let before = &s[..idx];
    if rest.starts_with('y') {
        return (false, true);
    }
    if rest.starts_with('x') {
        return (true, false);
    }
    if before.contains("vertical") && !before.contains("both") {
        return (false, true);
    }
    if before.contains("horizontal") && !before.contains("both") {
        return (true, false);
    }
    (true, true)
}

fn class_has_overflow_or_truncate_opt_in(class: &str) -> bool {
    class.split_whitespace().any(|c| {
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        s == "truncate"
            || s.ends_with("-truncate")
            || s.starts_with("overflow")
            || s.contains("-overflow")
    })
}

fn class_has_truncate(class: &str) -> bool {
    class.split_whitespace().any(|c| {
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        s == "truncate" || s.ends_with("-truncate")
    })
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
    if class_is_ungated(&elem.class) {
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
    style.style.position == Position::Absolute
        || class_is_out_of_flow(&elem.class)
        || is_overlay_host(elem)
        || has_popover_attr(&elem.attrs)
}

fn is_overlay_host(elem: &Element) -> bool {
    let tag = elem.tag.as_str();
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
    ) || elem.class.split_whitespace().any(|c| {
        matches!(
            c,
            "sail-dropdown"
                | "sail-dialog"
                | "sail-popover"
                | "sail-tooltip"
                | "sail-hover-card"
                | "sail-context-menu"
        )
    }) || elem
        .attrs
        .get("role")
        .is_some_and(|r| matches!(r.as_str(), "dialog" | "menu" | "listbox" | "tooltip"))
}

fn has_popover_attr(attrs: &std::collections::HashMap<String, String>) -> bool {
    attrs.contains_key("popover")
}

/// `{ident}` / `{ident.method()}` source text is not a solved width.
fn text_is_unbound_expr(text: &str) -> bool {
    let t = text.trim();
    let Some(inner) = t.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        return false;
    };
    let inner = inner.trim();
    !inner.is_empty() && !inner.contains('{') && !inner.contains('}')
}

fn child_width_is_unbound_expr(layout: &Layout, index: usize) -> bool {
    let elem = &layout.elements[index];
    if text_is_unbound_expr(&elem.text) {
        return true;
    }
    if elem.children.is_empty() {
        return false;
    }
    // Mixed static + expr still has a known static contribution.
    elem.children
        .iter()
        .all(|&i| child_width_is_unbound_expr(layout, i))
        && elem.text.trim().is_empty()
}

fn is_spliced_l1_host(elem: &Element) -> bool {
    elem.class.split_whitespace().any(|c| {
        matches!(
            c,
            "sail-button"
                | "sail-input"
                | "sail-select"
                | "sail-textarea"
                | "sail-switch"
                | "sail-checkbox"
                | "sail-radio"
                | "sail-slider"
        )
    })
}

fn host_splice_owns_geometry(layout: &Layout, parent_index: usize, child_index: usize) -> bool {
    is_spliced_l1_host(&layout.elements[parent_index])
        || is_spliced_l1_host(&layout.elements[child_index])
}

fn class_is_out_of_flow(class: &str) -> bool {
    class.split_whitespace().any(|c| {
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        matches!(
            s.as_str(),
            "absolute" | "fixed" | "positionabsolute" | "positionfixed"
        ) || s.contains("positionabsolute")
            || s.contains("positionfixed")
    })
}

fn class_is_ungated(class: &str) -> bool {
    class.split_whitespace().any(|c| {
        let s = c.strip_prefix("s-").unwrap_or(c).to_ascii_lowercase();
        matches!(
            s.as_str(),
            "hidden" | "displaynone" | "displaycontents" | "contents"
        ) || s.contains("displaynone")
            || s.contains("displaycontents")
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
    viewport: (f32, f32),
) -> bool {
    if !subtree_has_out_of_flow(layout, child_index)
        && !subtree_has_never_shown(layout, child_index, viewport)
    {
        return false;
    }
    in_flow_main(layout, child_index, height_axis, viewport) <= parent_main + KIND_EPSILON
}

fn subtree_has_never_shown(layout: &Layout, index: usize, viewport: (f32, f32)) -> bool {
    let mut stack = layout.elements[index].children.clone();
    while let Some(i) = stack.pop() {
        if is_never_shown(&layout.elements[i], &layout.styles[i], viewport) {
            return true;
        }
        stack.extend(layout.elements[i].children.iter().copied());
    }
    false
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

fn in_flow_main(layout: &Layout, index: usize, height_axis: bool, viewport: (f32, f32)) -> f32 {
    if is_out_of_flow(&layout.elements[index], &layout.styles[index]) {
        return 0.0;
    }
    if is_never_shown(&layout.elements[index], &layout.styles[index], viewport) {
        return 0.0;
    }
    let children = &layout.elements[index].children;
    let must_decompose =
        subtree_has_out_of_flow(layout, index) || subtree_has_never_shown(layout, index, viewport);
    if children.is_empty() || !must_decompose {
        return element_main(layout, index, height_axis);
    }
    children
        .iter()
        .map(|&i| in_flow_main(layout, i, height_axis, viewport))
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
    fn overflow_opt_in_matches_orientation_prefixed_class() {
        assert!(class_has_overflow_or_truncate_opt_in(
            "s-orientationvertical-overflowYauto"
        ));
        assert!(class_has_overflow_or_truncate_opt_in(
            "s-orientationhorizontal-overflowXauto"
        ));
        assert!(class_has_overflow_or_truncate_opt_in("s-overflowYauto"));
        assert!(!class_has_overflow_or_truncate_opt_in(
            "s-overscrollcontain s-scrollbarThin"
        ));
        assert!(!class_has_overflow_or_truncate_opt_in("s-h20 s-wfull"));
        assert!(class_scrolls_axis(
            "s-orientationvertical-overflowYauto",
            true
        ));
        assert!(!class_scrolls_axis(
            "s-orientationvertical-overflowYauto",
            false
        ));
        assert!(class_scrolls_axis(
            "s-orientationhorizontal-overflowXauto",
            false
        ));
        assert!(!class_scrolls_axis(
            "s-orientationhorizontal-overflowXauto",
            true
        ));
        assert!(!class_scrolls_axis("s-overflowhidden", true));
        assert!(!class_scrolls_axis(
            "s-orientationvertical-overflowXhidden",
            false
        ));
        assert!(is_ticker_host("ticker", ""));
        assert!(is_ticker_host("div", "sail-ticker s-overflowhidden"));
        assert!(!is_ticker_host("div", "sail-col s-overflowhidden"));
    }

    #[test]
    fn overflow_token_scroll_axes_are_axis_specific() {
        assert_eq!(overflow_token_scroll_axes("overflowyauto"), (false, true));
        assert_eq!(overflow_token_scroll_axes("overflowxauto"), (true, false));
        assert_eq!(overflow_token_scroll_axes("overflowauto"), (true, true));
        assert_eq!(
            overflow_token_scroll_axes("orientationvertical-overflowyauto"),
            (false, true)
        );
        assert_eq!(
            overflow_token_scroll_axes("orientationhorizontal-overflowxauto"),
            (true, false)
        );
        assert_eq!(overflow_token_scroll_axes("overflowhidden"), (false, false));
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
    fn unbound_expr_text_is_a_single_interpolation() {
        assert!(text_is_unbound_expr("{prop.type}"));
        assert!(text_is_unbound_expr("{session.paintMode()}"));
        assert!(text_is_unbound_expr("  {id}  "));
        assert!(!text_is_unbound_expr("{row.from} → {row.to}"));
        assert!(!text_is_unbound_expr("light"));
        assert!(!text_is_unbound_expr(""));
        assert!(!text_is_unbound_expr("{unclosed"));
    }

    #[test]
    fn popover_and_dialog_role_are_overlay_hosts() {
        let mut elem = Element {
            tag: "div".into(),
            ..Default::default()
        };
        elem.attrs.insert("popover".into(), String::new());
        assert!(has_popover_attr(&elem.attrs));
        assert!(is_overlay_host(&Element {
            tag: "dropdown".into(),
            ..Default::default()
        }));
        let mut dialog = Element {
            tag: "div".into(),
            ..Default::default()
        };
        dialog.attrs.insert("role".into(), "dialog".into());
        assert!(is_overlay_host(&dialog));
        assert!(!is_overlay_host(&Element {
            tag: "button".into(),
            ..Default::default()
        }));
    }

    #[test]
    fn spliced_l1_host_matches_sail_button_class() {
        assert!(is_spliced_l1_host(&Element {
            tag: "button".into(),
            class: "sail-button s-sizemd-minHcontrolhmd".into(),
            ..Default::default()
        }));
        assert!(!is_spliced_l1_host(&Element {
            tag: "button".into(),
            class: String::new(),
            ..Default::default()
        }));
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
