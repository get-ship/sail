//! Static-subtree detection + HTML serialization for the cloneNode fast path.
//!
//! A subtree with no dynamic parts — all-static attributes, and only
//! static-text / comment / static-element children — compiles to a hoisted
//! `<template>` that is `cloneNode`'d at runtime, instead of an imperative
//! `createElement`/`setAttribute`/`appendChild` chain. This is what Solid and
//! Marko do for static subtrees; it cuts emitted-code size and construction
//! instructions on static-heavy markup, and the server serializes the same
//! string verbatim (no per-node building).
//!
//! Behavior-preserving and conservative: the cloned DOM is exactly what the
//! imperative path would have built, and anything not provably static falls
//! back to the imperative emit. The first cut excludes custom elements, SVG,
//! and the behavior-bearing special tags (`<data>`, `<slot>`, `<use-*>`), whose
//! imperative paths do more than build plain DOM.

use crate::ir::template::{AttrInstr, ElementInstr, Instr};

/// HTML void elements — serialized without children or a closing tag.
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

/// Tags whose imperative emit does more than build plain DOM, so they are never
/// cloned: custom elements (upgrade), SVG (namespaced), `<data>` (skipped),
/// `<slot>` (slotting), `<use-*>` (template hooks).
fn tag_blocks_clone(tag_lower: &str) -> bool {
    tag_lower.contains('-')
        || tag_lower == "svg"
        || tag_lower == "data"
        || tag_lower == "slot"
        || tag_lower.starts_with("use-")
}

/// True when `el` and its entire subtree can be emitted as a cloned template.
pub fn is_static_element(el: &ElementInstr) -> bool {
    if tag_blocks_clone(&el.tag.to_ascii_lowercase()) {
        return false;
    }
    el.attrs.iter().all(|a| matches!(a, AttrInstr::Static { .. }))
        && el.children.iter().all(is_static_child)
}

fn is_static_child(c: &Instr) -> bool {
    match c {
        Instr::StaticText { .. } | Instr::Comment { .. } => true,
        Instr::Element(el) => is_static_element(el),
        _ => false,
    }
}

/// Serialize a proven-static element subtree to an HTML string.
///
/// Precondition: `is_static_element(el)` returned true. Comments are dropped to
/// match the imperative emit (which discards them). Attribute values and text
/// are HTML-escaped so the parsed clone is byte-for-byte the intended tree.
pub fn serialize_element(el: &ElementInstr) -> String {
    let mut out = String::new();
    write_element(&mut out, el);
    out
}

fn write_element(out: &mut String, el: &ElementInstr) {
    out.push('<');
    out.push_str(&el.tag);
    for a in &el.attrs {
        if let AttrInstr::Static { name, value, .. } = a {
            out.push(' ');
            out.push_str(name);
            out.push_str("=\"");
            escape_attr(out, value);
            out.push('"');
        }
    }
    out.push('>');
    if VOID.contains(&el.tag.to_ascii_lowercase().as_str()) {
        return; // void element: no children, no closing tag
    }
    for c in &el.children {
        match c {
            Instr::StaticText { text, .. } => escape_text(out, text),
            Instr::Element(child) => write_element(out, child),
            // Comments are dropped (imperative emit does too); the static
            // precondition guarantees no other variants reach here.
            _ => {}
        }
    }
    out.push_str("</");
    out.push_str(&el.tag);
    out.push('>');
}

fn escape_attr(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

fn escape_text(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    fn sp() -> Span {
        Span::new(0, 0)
    }
    fn expr(src: &str) -> crate::ir::expr::Expr {
        crate::ir::expr::Expr { source: src.to_string(), span: sp(), allow_arrow: false }
    }
    fn el(tag: &str, attrs: Vec<AttrInstr>, children: Vec<Instr>) -> ElementInstr {
        ElementInstr { tag: tag.to_string(), attrs, children, span: sp() }
    }
    fn attr(name: &str, value: &str) -> AttrInstr {
        AttrInstr::Static { name: name.to_string(), value: value.to_string(), span: sp() }
    }
    fn text(t: &str) -> Instr {
        Instr::StaticText { text: t.to_string(), span: sp() }
    }

    #[test]
    fn plain_static_subtree_is_static_and_serializes() {
        let tree = el(
            "div",
            vec![attr("class", "card")],
            vec![Instr::Element(el("span", vec![], vec![text("hi")])), text(" there")],
        );
        assert!(is_static_element(&tree));
        assert_eq!(serialize_element(&tree), r#"<div class="card"><span>hi</span> there</div>"#);
    }

    #[test]
    fn escapes_attrs_and_text() {
        let tree = el(
            "p",
            vec![attr("title", r#"a "b" & c"#)],
            vec![text("x < y & z")],
        );
        assert_eq!(serialize_element(&tree), r#"<p title="a &quot;b&quot; &amp; c">x &lt; y &amp; z</p>"#);
    }

    #[test]
    fn void_elements_have_no_closing_tag() {
        let tree = el("div", vec![], vec![Instr::Element(el("br", vec![], vec![]))]);
        assert_eq!(serialize_element(&tree), "<div><br></div>");
    }

    #[test]
    fn dynamic_content_blocks_static() {
        // A dynamic-text child is not static.
        let dyn_tree = el("div", vec![], vec![Instr::DynamicText { expr: expr("x") }]);
        assert!(!is_static_element(&dyn_tree));
        // A bound attr is not static.
        let bound = el("div", vec![AttrInstr::Class { name: "on".into(), expr: expr("y"), span: sp() }], vec![]);
        assert!(!is_static_element(&bound));
    }

    #[test]
    fn custom_elements_svg_and_special_tags_block_clone() {
        assert!(!is_static_element(&el("sui-icon", vec![attr("name", "home")], vec![])));
        assert!(!is_static_element(&el("svg", vec![], vec![])));
        assert!(!is_static_element(&el("slot", vec![], vec![])));
        assert!(!is_static_element(&el("data", vec![], vec![])));
        assert!(!is_static_element(&el("use-tooltip", vec![], vec![])));
    }
}
