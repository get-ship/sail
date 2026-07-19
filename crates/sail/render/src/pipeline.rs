//! End-to-end render pipeline: preprocess → parse → resolve → takumi → PNG.

use std::path::Path;

use anyhow::{Context, Result};
use sail_layout::html::Element;
use takumi::prelude::*;

use crate::convert;
use crate::fonts;
use crate::placeholder;

/// Render `.sail` source text to a `takumi::Bitmap`.
pub fn render(source: &str, viewport: (u32, u32), fonts_dir: &Path) -> Result<Bitmap> {
    // 1. Preprocess: sx + layout lowering, emits HTML with embedded <style>.
    let preprocessed = sail_compiler::parse::preprocess::preprocess(source);

    // 2. Split the emitted CSS from the lowered HTML.
    let (html, _css) = split_html_css(&preprocessed);

    // 3. Parse into a flat element tree.
    let elements = sail_layout::html::parse_source(html);
    if elements.len() <= 1 {
        anyhow::bail!("preprocessed source produced no renderable elements");
    }

    // 4. Compute layout with sail_layout to obtain resolved styles, computed
    //    boxes, and unknown-subtree flags.
    let viewport_f = (viewport.0 as f32, viewport.1 as f32);
    let layout = sail_layout::taffy_tree::build(elements.clone(), viewport_f)
        .context("computing layout with sail_layout")?;

    // 5. Convert to a takumi node tree.
    let root = build_takumi_tree(&elements, &layout, viewport.0 as f32)?;

    // 6. Load fonts.
    let fonts = fonts::load_fonts(fonts_dir)
        .context("loading token fonts")?;

    // 7. Render.
    let options = RenderOptions::builder()
        .viewport(Viewport::new((viewport.0, viewport.1)))
        .node(root)
        .fonts(&fonts)
        .build();

    takumi::render(options).context("takumi render")
}

/// Split the preprocessed string into HTML and CSS portions.
/// Mirrors the helper in `crates/sail/compiler/tests/layout_conformance.rs`.
fn split_html_css(preprocessed: &str) -> (&str, &str) {
    let Some(start) = preprocessed.find("<style>") else {
        return (preprocessed, "");
    };
    let css_start = start + "<style>".len();
    let Some(end) = preprocessed[css_start..].find("</style>") else {
        return (preprocessed, "");
    };
    let html = &preprocessed[end + css_start + "</style>".len()..];
    (html, &preprocessed[css_start..css_start + end])
}

/// Build the takumi node tree, collapsing unknown subtrees into placeholders.
fn build_takumi_tree(
    elements: &[Element],
    layout: &sail_layout::taffy_tree::Layout,
    _viewport_width: f32,
) -> Result<Node> {
    // The synthetic document root is index 0; its children are the top-level
    // elements produced by the parser.
    let mut children = Vec::new();
    for &child_idx in &elements[0].children {
        if let Some(node) = build_node(elements, layout, child_idx)? {
            children.push(node);
        }
    }

    Ok(Node::container(children).with_style(
        Style::default()
            .with(StyleDeclaration::width(Length::Percentage(100.0)))
            .with(StyleDeclaration::height(Length::Percentage(100.0)))
            .with(StyleDeclaration::display(Display::Block))
            .with(StyleDeclaration::background_color(ColorInput::Value(Color::from_rgb(0xffffff)))),
    ))
}

/// Recursively build a takumi node for element `idx`.
fn build_node(
    elements: &[Element],
    layout: &sail_layout::taffy_tree::Layout,
    idx: usize,
) -> Result<Option<Node>> {
    let elem = &elements[idx];

    // Unknown element: render a placeholder and do not recurse.
    if elem.is_unknown() {
        let computed = layout.computed(idx);
        let (w, h) = computed
            .map(|l| (l.size.width, l.size.height))
            .unwrap_or((0.0, 0.0));
        return Ok(Some(placeholder::node(
            &elem.tag,
            w.max(0.0),
            h.max(0.0),
        )));
    }

    // Resolve style using the same path as diagnostics.
    let resolved = sail_layout::style::resolve(elem, elements, layout.viewport.0);
    if resolved.visibility == sail_layout::style::Visibility::Ignored {
        return Ok(None);
    }

    // Children first.
    let mut child_nodes = Vec::new();
    for &child_idx in &elem.children {
        if let Some(child) = build_node(elements, layout, child_idx)? {
            child_nodes.push(child);
        }
    }

    // Direct text becomes an inline-ish text child.
    if !elem.text.is_empty() {
        child_nodes.push(text_node(elem, &resolved));
    }

    let node = Node::container(child_nodes);
    let style = convert::style(&resolved.style);
    Ok(Some(node.with_style(style)))
}

fn text_node(elem: &Element, _resolved: &sail_layout::style::ResolvedStyle) -> Node {
    let (family, size) = resolve_family_size(elem);
    let font_size_px = size.px();

    Node::text(elem.text.clone()).with_style(
        Style::default()
            .with(StyleDeclaration::color(ColorInput::Value(Color::from_rgb(0x0f172a))))
            .with(StyleDeclaration::font_size(FontSize::Length(Length::Px(font_size_px))))
            .with(StyleDeclaration::font_family(FontFamily::from_names([family_name(family)]))),
    )
}

fn resolve_family_size(elem: &Element) -> (sail_layout::metrics::Family, sail_layout::metrics::Size) {
    use sail_layout::metrics::{Family, Size};

    let mut family = None;
    let mut size = None;

    for c in elem.class.split_whitespace() {
        if family.is_none() {
            family = match c {
                "font-mono" => Some(Family::Mono),
                "font-display" => Some(Family::Display),
                "font-press-start" => Some(Family::PressStart),
                _ => None,
            };
        }
        if size.is_none() {
            if let Some(tok) = c.strip_prefix("text-") {
                size = Some(Size::from_class_token(tok));
            }
        }
    }

    if size.is_none() {
        size = tag_default_size(&elem.tag);
    }

    (family.unwrap_or(Family::Body), size.unwrap_or(Size::Base))
}

fn tag_default_size(tag: &str) -> Option<sail_layout::metrics::Size> {
    use sail_layout::metrics::Size;
    match tag {
        "h1" => Some(Size::Xxl),
        "h2" => Some(Size::Xl),
        "h3" => Some(Size::Lg),
        "small" => Some(Size::Sm),
        _ => None,
    }
}

fn family_name(f: sail_layout::metrics::Family) -> String {
    match f {
        sail_layout::metrics::Family::Body => "DM Sans",
        sail_layout::metrics::Family::Mono => "JetBrains Mono",
        sail_layout::metrics::Family::Display => "Syne",
        sail_layout::metrics::Family::PressStart => "Press Start 2P",
    }
    .to_string()
}
