//! Pixel goldens + paint/leak/dynamics assertions for static render.

use std::path::PathBuf;

use sail_render::{RenderOptions, assert_pixel_match, png_bytes, render_file, write_png};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/")).join(name)
}

fn fonts() -> PathBuf {
    fixture("fonts")
}

fn opts(viewport: (u32, u32)) -> RenderOptions {
    RenderOptions {
        viewport,
        fonts_dir: Some(fonts()),
        app_fonts_dir: None,
        theme: Some("ship".into()),
        mode: Some("light".into()),
        workspace: workspace_root(),
        file: None,
        drop_bg_token: None,
    }
}

fn workspace_root() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..8 {
        if dir.join("Cargo.toml").is_file() && dir.join("packages/sail/tokens").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn golden_path(name: &str) -> PathBuf {
    fixture(&format!("goldens/{name}.png"))
}

fn in_ci() -> bool {
    matches!(
        std::env::var("CI").ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES")
    )
}

fn ensure_or_compare(name: &str, result: &sail_render::RenderOutput) {
    let path = golden_path(name);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let update = std::env::var("SAIL_RENDER_UPDATE_GOLDEN").ok().as_deref() == Some("1");
    if !path.is_file() {
        if in_ci() && !update {
            panic!(
                "golden {name} missing at {} (CI refuses to write; run with \
                 SAIL_RENDER_UPDATE_GOLDEN=1 locally to mint)",
                path.display()
            );
        }
        write_png(&result.image, &path).expect("write golden");
        eprintln!("wrote golden {}", path.display());
        return;
    }
    if update {
        write_png(&result.image, &path).expect("write golden");
        eprintln!("updated golden {}", path.display());
        return;
    }
    let expected = std::fs::read(&path).expect("read golden");
    let diff = path.with_extension("diff.png");
    assert_pixel_match(&result.image, &expected, 3, Some(&diff)).unwrap_or_else(|e| {
        panic!("golden {name} failed: {e} (diff at {})", diff.display());
    });
}

#[test]
fn paint_fixture_golden_light() {
    let result = render_file(&fixture("paint.sail"), opts((400, 300))).unwrap();
    assert!(
        !result.html.contains("{ticks()}",),
        "expression leak in html: {}",
        result.html
    );
    assert!(
        result
            .dynamics
            .iter()
            .any(|d| d.reason.contains("dynamic-expr") || d.reason.contains("ticks")),
        "expected ticks dynamic: {:?}",
        result.dynamics
    );
    assert!(
        result
            .dynamics
            .iter()
            .any(|d| d.reason.contains("my-widget") || d.tag.as_deref() == Some("my-widget")),
        "expected unknown my-widget: {:?}",
        result.dynamics
    );
    assert!(!result.regions.is_empty());
    ensure_or_compare("paint-light-400x300", &result);
}

#[test]
fn paint_fixture_golden_dark() {
    let mut o = opts((400, 300));
    o.mode = Some("dark".into());
    let result = render_file(&fixture("paint.sail"), o).unwrap();
    ensure_or_compare("paint-dark-400x300", &result);
}

#[test]
fn static_only_golden() {
    let result = render_file(&fixture("static-only.sail"), opts((320, 200))).unwrap();
    assert!(
        result.dynamics.is_empty() || result.dynamics.iter().all(|d| !d.reason.contains("expr"))
    );
    ensure_or_compare("static-only-320x200", &result);
}

#[test]
fn drop_bg_mapping_fails_golden_with_diff() {
    // Establish baseline with full paint.
    let baseline = render_file(&fixture("paint.sail"), opts((400, 300))).unwrap();
    let bytes = png_bytes(&baseline.image).unwrap();

    // Drop primary mapping → button no longer paints primary.
    let mut o = opts((400, 300));
    o.drop_bg_token = Some("primary".into());
    let broken = render_file(&fixture("paint.sail"), o).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let diff = dir.path().join("drop-bg.diff.png");
    let err = assert_pixel_match(&broken.image, &bytes, 3, Some(&diff));
    assert!(
        err.is_err(),
        "expected golden fail after dropping bg mapping"
    );
    assert!(diff.is_file(), "diff artifact must be written on failure");
}

#[test]
fn no_expression_leaks_in_output_html() {
    let result = render_file(&fixture("paint.sail"), opts((400, 300))).unwrap();
    assert!(
        !result.html.contains('{'),
        "leaked brace in html: {}",
        result.html
    );
}

#[test]
fn mid_size_route_under_500ms_debug() {
    // Warm fonts/registry once — Studio repaints reuse loaded fonts (O8).
    // Spec budget is for repaint; cold first frame may exceed 500ms.
    let _ = render_file(&fixture("mid-size.sail"), opts((800, 600))).unwrap();
    let result = render_file(&fixture("mid-size.sail"), opts((800, 600))).unwrap();
    assert!(
        result.ms < 500,
        "warm render took {}ms (budget 500ms debug)",
        result.ms
    );
}

#[test]
fn golden_legacy_fixture_still_renders() {
    let result = render_file(&fixture("golden.sail"), opts((800, 600))).unwrap();
    assert_eq!(result.viewport, (800, 600));
    let rgba = result.image.as_raw();
    let non_white = rgba
        .chunks_exact(4)
        .filter(|c| c != &[255, 255, 255, 255])
        .count();
    assert!(non_white > 100, "expected visible content");
}

#[test]
fn sidecar_lists_dynamics() {
    let result = render_file(&fixture("paint.sail"), opts((400, 300))).unwrap();
    let dyn_json = serde_json::to_value(&result.dynamics).unwrap();
    assert!(dyn_json.as_array().unwrap().len() >= 2, "{dyn_json}");
}

#[test]
fn region_and_dynamic_ids_are_inspect_tree_ids() {
    // Library/CLI regions must use SailInspect content-hash ids (true join),
    // not a post-hoc layout-box substitution only on the ops path.
    let mut o = opts((400, 300));
    o.file = Some("crates/sail/render/tests/fixtures/paint.sail".into());
    let result = render_file(&fixture("paint.sail"), o).unwrap();
    let tree: std::collections::BTreeSet<_> = result.tree_ids.iter().cloned().collect();
    assert!(!tree.is_empty(), "author tree must produce node ids");
    assert!(!result.regions.is_empty());
    for r in &result.regions {
        assert!(
            tree.contains(&r.node_id),
            "region {} not in inspect tree {:?}",
            r.node_id,
            tree
        );
    }
    for d in &result.dynamics {
        assert!(
            tree.contains(&d.node_id),
            "dynamic {} not in inspect tree",
            d.node_id
        );
        assert!(
            !d.node_id.contains("dyn.expr"),
            "dynamics must not keep provisional ids"
        );
    }
}

#[test]
fn mid_size_golden_painted_tokens() {
    // Step-6 fixture coverage: multi-card painted route (page.sail 1440×900 is
    // intentionally not golden — cold full landing is heavy; this pins tokens).
    let result = render_file(&fixture("mid-size.sail"), opts((800, 600))).unwrap();
    assert!(!result.html.contains('{'));
    ensure_or_compare("mid-size-800x600", &result);
}

/// Assert a successful raster: valid size, PNG magic, and non-bg paint.
///
/// Uses top-left pixel as the page background (theme surface is near-white
/// oklch, not pure `#fff`), so a solid themed fill without chrome cannot
/// falsely satisfy the threshold.
fn assert_visible_png(result: &sail_render::RenderOutput, min_non_bg: usize) {
    assert!(
        result.image.width() > 0 && result.image.height() > 0,
        "viewport must be non-zero"
    );
    let png = png_bytes(&result.image).expect("encode png");
    assert!(png.starts_with(b"\x89PNG"), "must produce a PNG");
    let rgba = result.image.as_raw();
    let bg = [rgba[0], rgba[1], rgba[2], rgba[3]];
    let non_bg = rgba.chunks_exact(4).filter(|c| *c != bg.as_slice()).count();
    assert!(
        non_bg >= min_non_bg,
        "expected painted chrome/text (non-bg px {non_bg} < {min_non_bg}; bg={bg:?})"
    );
}

/// RENDER-TEXT-1: fontSize / weight / tracking / textCase paint, size cascade,
/// and inline accent span inside h1 (not block-hoisted).
#[test]
fn text_channel_ladder_golden_and_behavior() {
    let result = render_file(&fixture("text-channel.sail"), opts((480, 520))).unwrap();
    assert!(
        !result.html.contains('{'),
        "expression leak in html: {}",
        result.html
    );

    // Layout heights must cascade with s-fontSize (xs < 4xl).
    let spans: Vec<_> = result
        .regions
        .iter()
        .filter(|r| r.tag.as_deref() == Some("span"))
        .collect();
    assert!(
        spans.len() >= 8,
        "expected size ladder + styled + accent spans, got {}",
        spans.len()
    );
    let xs_h = spans[0].h;
    let four_xl_h = spans[6].h;
    assert!(
        four_xl_h > xs_h + 10.0,
        "4xl height ({four_xl_h}) must exceed xs ({xs_h}) — text channel ignored?"
    );

    // Inline accent: primary-colored glyphs exist and are localized (not a
    // full-width block strip). Layout regions still report taffy boxes; assert
    // on raster ink.
    let rgba = result.image.as_raw();
    let w = result.image.width() as usize;
    let mut primary_xs: Vec<usize> = Vec::new();
    for (i, c) in rgba.chunks_exact(4).enumerate() {
        // ship primary ≈ rgb(220, 119, 2)
        if c[0] > 180 && c[1] > 80 && c[1] < 160 && c[2] < 60 {
            primary_xs.push(i % w);
        }
    }
    assert!(
        primary_xs.len() > 40,
        "expected primary-colored accent glyphs, got {}",
        primary_xs.len()
    );
    let pmin = *primary_xs.iter().min().unwrap();
    let pmax = *primary_xs.iter().max().unwrap();
    assert!(
        pmax - pmin < w * 2 / 3,
        "accent paint must be localized, not full-bleed (span {}..{} of {w})",
        pmin,
        pmax
    );

    ensure_or_compare("text-channel-480x520", &result);
}

/// RENDER-TEXT-1: empty decorative divs with size()/w()/h() + paint must keep
/// non-zero boxes inside a flex row (no silent drop).
#[test]
fn empty_decorative_div_golden_and_behavior() {
    let result = render_file(&fixture("empty-div.sail"), opts((400, 200))).unwrap();

    let boxes: Vec<_> = result
        .regions
        .iter()
        .filter(|r| r.tag.as_deref() == Some("div") && r.w > 0.0 && r.h > 0.0)
        .collect();
    assert!(
        boxes.len() >= 6,
        "expected ≥6 non-zero empty decorative divs, got {} (regions={:?})",
        boxes.len(),
        result
            .regions
            .iter()
            .map(|r| (r.tag.clone(), r.w, r.h))
            .collect::<Vec<_>>()
    );
    // size(2) → 8×8, size(4) → 16×16
    assert!(
        boxes.iter().any(|r| (r.w - 8.0).abs() < 0.5 && (r.h - 8.0).abs() < 0.5),
        "size(2) must resolve to 8×8: {:?}",
        boxes.iter().map(|r| (r.w, r.h)).collect::<Vec<_>>()
    );
    assert!(
        boxes
            .iter()
            .any(|r| (r.w - 16.0).abs() < 0.5 && (r.h - 16.0).abs() < 0.5),
        "size(4) must resolve to 16×16"
    );
    // At least one painted background on an empty div.
    assert!(
        boxes.iter().any(|r| {
            r.style
                .as_ref()
                .and_then(|s| s.background_color.as_ref())
                .is_some()
        }),
        "empty decorative divs must resolve paint backgrounds"
    );

    ensure_or_compare("empty-div-400x200", &result);
}

/// RENDER-0: zero-width padded text used to panic parley (line_break assert).
#[test]
fn parley_zero_width_padded_text_does_not_panic() {
    let result = render_file(&fixture("parley-zero-width.sail"), opts((320, 200)))
        .expect("zero-width padded text must render without parley panic");
    // Root paints theme surface; require non-bg chrome/text so a blank raster fails.
    assert_visible_png(&result, 10);
}

/// RENDER-0: center + field + select (grid min-content of padded option text).
#[test]
fn parley_center_field_select_does_not_panic() {
    let result = render_file(
        &fixture("parley-center-field-select.sail"),
        opts((640, 900)),
    )
    .expect("center/field/select must render without parley panic");
    assert_visible_png(&result, 100);
    assert!(
        !result.regions.is_empty(),
        "form chrome must produce layout regions"
    );
    let html = result.html.to_lowercase();
    assert!(
        html.contains("shipping") && (html.contains("country") || html.contains("select")),
        "expected form chrome in static html"
    );
}

/// RENDER-0: agent-produced dependent-form page that crashed the baseline batch.
#[test]
fn parley_dependent_form_agent_repro_does_not_panic() {
    let result = render_file(
        &fixture("parley-dependent-form-repro.sail"),
        opts((1440, 900)),
    )
    .expect("dependent-form agent page must render without parley panic");
    assert_visible_png(&result, 100);
    assert!(
        !result.regions.is_empty(),
        "agent form must produce layout regions"
    );
    let html = result.html.to_lowercase();
    assert!(
        html.contains("shipping") && (html.contains("country") || html.contains("select")),
        "expected form chrome in static html, got a prefix: {:?}",
        html.chars().take(200).collect::<String>()
    );
}

/// RENDER-0 residual: unclamped multi-line text measures without panic after
/// dropping viewport-as-max_height. Pins non-zero text-host height (not pixel
/// parity with upstream’s viewport-capped measure).
#[test]
fn parley_unclamped_tall_text_measures_without_panic() {
    let result = render_file(
        &fixture("parley-unclamped-tall-text.sail"),
        opts((400, 200)),
    )
    .expect("unclamped tall text must render without parley panic");
    assert_visible_png(&result, 50);
    let textish: Vec<_> = result
        .regions
        .iter()
        .filter(|r| r.tag.as_deref() == Some("span") || r.h > 40.0)
        .collect();
    assert!(
        textish.iter().any(|r| r.h > 30.0),
        "expected multi-line text height > 30px, regions={:?}",
        result
            .regions
            .iter()
            .map(|r| (r.tag.clone(), r.w, r.h))
            .collect::<Vec<_>>()
    );
}

/// RENDER-0: float-aware BreakLines path (`clamp_lines` set) at zero content width.
///
/// The common panic path no longer hits this (viewport-as-max_height removed), but
/// `update_breaker_line` clamps still protect clamp-lines / ellipsis UI. Sail sx
/// does not lower `line-clamp`, so this builds a raw takumi tree.
///
/// Note: when clamp_lines is set, measure still uses viewport height in
/// HeightAndLines (see SAIL_PATCH.md residual) — paint uses content-box height.
/// Align those when host line-clamp lands.
#[test]
fn parley_clamp_lines_zero_width_does_not_panic() {
    use takumi::prelude::*;

    let fonts = sail_render::fonts::load_fonts(&fonts()).expect("load fixture fonts");
    let text =
        Node::text("hello world this text is long enough to wrap under a line clamp at zero width")
            .with_style(
                Style::default()
                    .with(StyleDeclaration::color(ColorInput::Value(Color::black())))
                    .with(StyleDeclaration::font_size(FontSize::Length(Length::Px(
                        16.0,
                    ))))
                    .with(StyleDeclaration::font_family(FontFamily::from_names([
                        "DM Sans".to_string(),
                    ]))),
            );
    // max-lines + continue:collapse → clamp_lines() = Some(2) → float-aware path.
    let clamped = Node::container(vec![text]).with_style(
        Style::default()
            .with(StyleDeclaration::display(Display::Block))
            .with(StyleDeclaration::width(Length::Px(0.0)))
            .with(StyleDeclaration::padding_left(Length::Px(12.0)))
            .with(StyleDeclaration::padding_right(Length::Px(12.0)))
            .with(StyleDeclaration::padding_top(Length::Px(12.0)))
            .with(StyleDeclaration::padding_bottom(Length::Px(12.0)))
            .with(StyleDeclaration::max_lines(Some(2)))
            .with(StyleDeclaration::r#continue(Continue::Collapse)),
    );
    // Also cover nowrap+ellipsis (forces single-line clamp via clamp_lines()).
    let ellipsis = Node::container(vec![
        Node::text("ellipsis clamp at zero width").with_style(
            Style::default()
                .with(StyleDeclaration::color(ColorInput::Value(Color::black())))
                .with(StyleDeclaration::font_size(FontSize::Length(Length::Px(
                    16.0,
                ))))
                .with(StyleDeclaration::font_family(FontFamily::from_names([
                    "DM Sans".to_string(),
                ]))),
        ),
    ])
    .with_style(
        Style::default()
            .with(StyleDeclaration::display(Display::Block))
            .with(StyleDeclaration::width(Length::Px(0.0)))
            .with(StyleDeclaration::padding_left(Length::Px(12.0)))
            .with(StyleDeclaration::padding_right(Length::Px(12.0)))
            .with(StyleDeclaration::text_wrap_mode(TextWrapMode::NoWrap))
            .with(StyleDeclaration::text_overflow(TextOverflow::Ellipsis)),
    );
    let root = Node::container(vec![clamped, ellipsis]).with_style(
        Style::default()
            .with(StyleDeclaration::display(Display::Flex))
            .with(StyleDeclaration::flex_direction(FlexDirection::Column))
            .with(StyleDeclaration::width(Length::Percentage(100.0)))
            .with(StyleDeclaration::height(Length::Percentage(100.0)))
            .with(StyleDeclaration::background_color(ColorInput::Value(
                Color::white(),
            ))),
    );
    let options = RenderOptions::builder()
        .viewport(Viewport::new((320u32, 200u32)))
        .node(root)
        .fonts(&fonts)
        .build();
    let image = takumi::render(options).expect("clamp-lines at zero width must not panic");
    assert!(image.width() > 0 && image.height() > 0);
    let png = png_bytes(&image).expect("encode png");
    assert!(png.starts_with(b"\x89PNG"), "must produce a PNG");
}
