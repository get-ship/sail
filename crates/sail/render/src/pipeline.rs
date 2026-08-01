//! End-to-end render pipeline: static projection → layout → painted takumi → PNG.

use std::time::Instant;

use anyhow::{Context, Result};
use sail_layout::html::Element;
use takumi::prelude::*;

use crate::convert;
use crate::fonts;
use crate::identity::{
    self, DynamicRegion, InspectTreeIndex, Region, build_inspect_tree, join_expr_dynamics,
    join_layout_regions,
};
use crate::paint::{Paint, StyleSheet};
use crate::placeholder;
use crate::theme::{ThemePalette, ThemeSelect};

/// Full render result (image + sidecar metadata).
#[derive(Debug)]
pub struct PipelineOutput {
    pub image: Bitmap,
    pub regions: Vec<Region>,
    pub dynamics: Vec<DynamicRegion>,
    pub viewport: (u32, u32),
    pub theme: ThemeSelect,
    pub ms: u64,
    /// Preprocessed HTML after expression stripping (tests / leak checks).
    pub html: String,
    /// Author tree used for identity join (tests).
    pub tree: InspectTreeIndex,
}

/// Pipeline options.
#[derive(Debug, Clone)]
pub struct PipelineOptions {
    pub viewport: (u32, u32),
    pub fonts_dir: std::path::PathBuf,
    /// Optional `<project>/fonts` from SailTheme/FontAdd (same path Studio uses).
    pub app_fonts_dir: Option<std::path::PathBuf>,
    pub theme: ThemeSelect,
    pub workspace: Option<std::path::PathBuf>,
    /// Optional file label for node id generation (workspace-relative).
    pub file: Option<String>,
    /// Test hook: drop a bg token mapping so goldens fail.
    pub drop_bg_token: Option<String>,
}

/// Render `.sail` source text through the static no-JS projection.
pub fn render(source: &str, opts: &PipelineOptions) -> Result<PipelineOutput> {
    let started = Instant::now();
    let file = opts.file.as_deref().unwrap_or("_");

    // 0. Author tree (SailInspect identity) before projection.
    let tree = build_inspect_tree(source, file);

    // 1. Preprocess → lowered static HTML+CSS (same artifact SSR ships without JS).
    let preprocessed = sail_compiler::parse::preprocess::preprocess(source)
        .context("preprocessing sail source")?;
    let (html_raw, css) = split_html_css(&preprocessed);

    // 2. Strip template expression leaks; collect dynamics (joined after).
    let (html, expr_dynamics_raw) = strip_expression_leaks(html_raw, file);

    // 3. Theme palette (DTCG registry + theme mode).
    let mut palette = ThemePalette::resolve(opts.theme.clone(), opts.workspace.as_deref())
        .context("resolving theme palette")?;
    if let Some(tok) = &opts.drop_bg_token {
        palette.remove_mapping(tok);
    }

    // 4. Parse paint stylesheet + element tree.
    // L1 skins live in the framework sheet (not inlined by preprocess). Merge
    // them with the page's atomic CSS so host-stamped sx classes paint.
    let skins = sail_compiler::stdlib::skins_css();
    let mut css_all = String::with_capacity(skins.len() + css.len() + 2);
    css_all.push_str(&skins);
    css_all.push('\n');
    css_all.push_str(css);
    let sheet = StyleSheet::parse(&css_all);
    let elements = sail_layout::html::parse_source(&html);
    if elements.len() <= 1 {
        anyhow::bail!("preprocessed source produced no renderable elements");
    }

    // 5. Layout.
    let viewport_f = (opts.viewport.0 as f32, opts.viewport.1 as f32);
    let layout = sail_layout::taffy_tree::build(elements.clone(), viewport_f)
        .context("computing layout with sail_layout")?;

    // 6. Regions + unknown dynamics joined to SailInspect node ids (T-I3).
    let (mut regions, mut dynamics) = join_layout_regions(&elements, &layout, &tree);
    dynamics.extend(join_expr_dynamics(&expr_dynamics_raw, &tree));
    // Attach resolved paint styles (same sheet+palette as the takumi tree).
    attach_region_styles(&mut regions, &elements, &layout, &sheet, &palette);

    // 7. Takumi tree with paint.
    let root = build_takumi_tree(&elements, &layout, &sheet, &palette)?;

    // 8. Fonts + rasterize (framework defaults + optional app FontAdd vendors).
    let mut fonts = fonts::load_fonts(&opts.fonts_dir).context("loading token fonts")?;
    if let Some(app_dir) = &opts.app_fonts_dir {
        fonts::load_app_fonts(&mut fonts, app_dir).context("loading app-vendored fonts")?;
    }
    let options = RenderOptions::builder()
        .viewport(Viewport::new((opts.viewport.0, opts.viewport.1)))
        .node(root)
        .fonts(&fonts)
        .build();
    let image = takumi::render(options).context("takumi render")?;

    let ms = started.elapsed().as_millis() as u64;
    Ok(PipelineOutput {
        image,
        regions,
        dynamics,
        viewport: opts.viewport,
        theme: palette.theme,
        ms,
        html,
        tree,
    })
}

/// Split the preprocessed string into HTML and CSS portions.
pub fn split_html_css(preprocessed: &str) -> (&str, &str) {
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

/// Replace `{expr}` text interpolations with empty SSR fallback and record them.
fn strip_expression_leaks(html: &str, file: &str) -> (String, Vec<DynamicRegion>) {
    // Byte-walked but UTF-8 safe: every split point is ASCII (`<`, `>`,
    // `{`, `}`, quotes), and multi-byte chars are copied as whole `char`s —
    // byte-at-a-time `b as char` pushes Latin-1-mangled text (&#34;·&#34; → &#34;Â·&#34;).
    fn push_char_at(out: &mut String, src: &str, i: usize) -> usize {
        let b = src.as_bytes()[i];
        let len = match b {
            0x00..=0x7F => 1,
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            _ => 4,
        };
        out.push_str(&src[i..i + len]);
        len
    }
    let mut out = String::with_capacity(html.len());
    let mut dynamics = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0;
    let mut expr_i = 0usize;
    let mut in_tag = false;
    let mut in_quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if in_tag {
            i += push_char_at(&mut out, html, i);
            if let Some(q) = in_quote {
                if b == q {
                    in_quote = None;
                }
            } else if b == b'"' || b == b'\'' {
                in_quote = Some(b);
            } else if b == b'>' {
                in_tag = false;
            }
            continue;
        }
        if b == b'<' {
            in_tag = true;
            out.push('<');
            i += 1;
            continue;
        }
        if b == b'{' {
            let mut j = i + 1;
            let mut depth = 1i32;
            while j < bytes.len() && depth > 0 {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    b'<' => break,
                    _ => {}
                }
                j += 1;
            }
            if depth == 0 && j > i + 1 {
                let expr = &html[i + 1..j - 1];
                if looks_like_expr(expr) {
                    // Provisional id; remapped via join_expr_dynamics to parent node.
                    let id = identity::node_id(file, &format!("dyn.expr.{expr_i}"), expr);
                    dynamics.push(DynamicRegion {
                        node_id: id,
                        reason: format!("dynamic-expr:{{{expr}}}"),
                        tag: None,
                    });
                    expr_i += 1;
                    i = j;
                    continue;
                }
            }
        }
        i += push_char_at(&mut out, html, i);
    }
    (out, dynamics)
}

fn looks_like_expr(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    let first = t.chars().next().unwrap();
    first.is_ascii_alphabetic() || first == '_' || first == '(' || first == '['
}

fn build_takumi_tree(
    elements: &[Element],
    layout: &sail_layout::taffy_tree::Layout,
    sheet: &StyleSheet,
    palette: &ThemePalette,
) -> Result<Node> {
    let mut children = Vec::new();
    for &child_idx in &elements[0].children {
        if let Some(node) = build_node(elements, layout, sheet, palette, child_idx)? {
            children.push(node);
        }
    }

    let root_bg = crate::paint::parse_color(&palette.resolve_css_value(palette.root_background()))
        .unwrap_or_else(|| {
            crate::paint::parse_color(palette.get("surface-0").unwrap_or("oklch(1 0 0)"))
                .unwrap_or(Color::white())
        });

    Ok(Node::container(children).with_style(
        Style::default()
            .with(StyleDeclaration::width(Length::Percentage(100.0)))
            .with(StyleDeclaration::height(Length::Percentage(100.0)))
            .with(StyleDeclaration::display(Display::Block))
            .with(StyleDeclaration::background_color(ColorInput::Value(
                root_bg,
            ))),
    ))
}

/// Fill `Region.style` from the paint sheet in the same element order as join.
fn attach_region_styles(
    regions: &mut [Region],
    elements: &[Element],
    layout: &sail_layout::taffy_tree::Layout,
    sheet: &StyleSheet,
    palette: &ThemePalette,
) {
    let mut ri = 0usize;
    for (idx, elem) in elements.iter().enumerate() {
        if idx == 0 || elem.tag.is_empty() {
            continue;
        }
        if layout.computed(idx).is_none() {
            continue;
        }
        if ri >= regions.len() {
            break;
        }
        // Match `:empty` collapse: regions still exist from layout, but paint
        // is skipped — leave style None so parity doesn't false-positive chrome.
        let paint = if class_has(&elem.class, "sx-empty-displaynone")
            && elem.children.is_empty()
            && elem.text.trim().is_empty()
        {
            Paint::default()
        } else {
            Paint::from_decls(&sheet.decls_for_element(elem), palette)
        };
        regions[ri].style = paint.to_region_style();
        ri += 1;
    }
}

fn build_node(
    elements: &[Element],
    layout: &sail_layout::taffy_tree::Layout,
    sheet: &StyleSheet,
    palette: &ThemePalette,
    idx: usize,
) -> Result<Option<Node>> {
    let elem = &elements[idx];

    if elem.is_unknown() {
        let computed = layout.computed(idx);
        let (w, h) = computed
            .map(|l| (l.size.width, l.size.height))
            .unwrap_or((0.0, 0.0));
        return Ok(Some(placeholder::node(
            &elem.tag,
            w.max(0.0),
            h.max(0.0),
            palette,
        )));
    }

    let resolved = sail_layout::style::resolve(elem, elements, layout.viewport.0);
    if resolved.visibility == sail_layout::style::Visibility::Ignored {
        return Ok(None);
    }

    let mut child_nodes = Vec::new();
    for &child_idx in &elem.children {
        if let Some(child) = build_node(elements, layout, sheet, palette, child_idx)? {
            child_nodes.push(child);
        }
    }

    // Collapse `:empty { display:none }` chrome (card header/footer slots).
    // Layout taffy does not model :empty; skip paint so empty slots don't
    // reserve padded chrome. Regions still come from the shared layout pass.
    if class_has(&elem.class, "sx-empty-displaynone")
        && elem.children.is_empty()
        && elem.text.trim().is_empty()
    {
        return Ok(None);
    }

    let paint = Paint::from_decls(&sheet.decls_for_element(elem), palette);

    if !elem.text.is_empty() {
        child_nodes.push(text_node(elem, &paint, palette));
    }

    let node = Node::container(child_nodes);
    let mut style = convert::style(&resolved.style);
    style = paint.apply(style);
    Ok(Some(node.with_style(style)))
}

fn class_has(class_attr: &str, needle: &str) -> bool {
    class_attr.split_whitespace().any(|c| c == needle)
}

fn text_node(elem: &Element, paint: &Paint, palette: &ThemePalette) -> Node {
    let (family, size) = resolve_family_size(elem);
    let font_size_px = size.px();
    let color = paint
        .color
        .or_else(|| {
            crate::paint::parse_color(&palette.resolve_css_value(palette.root_foreground()))
        })
        .unwrap_or_else(|| {
            crate::paint::parse_color(palette.get("foreground").unwrap_or("oklch(0.15 0 0)"))
                .unwrap_or(Color::black())
        });

    Node::text(elem.text.clone()).with_style(
        Style::default()
            .with(StyleDeclaration::color(ColorInput::Value(color)))
            .with(StyleDeclaration::font_size(FontSize::Length(Length::Px(
                font_size_px,
            ))))
            .with(StyleDeclaration::font_family(FontFamily::from_names([
                family_name(family),
            ]))),
    )
}

fn resolve_family_size(
    elem: &Element,
) -> (sail_layout::metrics::Family, sail_layout::metrics::Size) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: byte-at-a-time copying Latin-1-mangled multi-byte text
    /// ("·" → "Â·") through the expr-leak stripper every render runs.
    #[test]
    fn strip_expression_leaks_preserves_utf8_text() {
        let html = r#"<div class="x" title="a · b">Save ⌘S — setup · rails</div>"#;
        let (out, dynamics) = strip_expression_leaks(html, "t.sail");
        assert_eq!(out, html, "utf-8 text must survive byte-walked copying");
        assert!(dynamics.is_empty());
    }

    #[test]
    fn strip_expression_leaks_strips_expr_keeps_utf8() {
        let html = "<p>café · {status()} — naïve</p>";
        let (out, dynamics) = strip_expression_leaks(html, "t.sail");
        assert_eq!(out, "<p>café ·  — naïve</p>");
        assert_eq!(dynamics.len(), 1);
    }
}
