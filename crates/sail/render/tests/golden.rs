//! Golden render test for `sail-render`.
//!
//! The fixture is rendered with the committed token fonts and a fixed viewport.
//! We avoid raw PNG byte hashes and instead assert geometry, non-blank output,
//! and the presence of the expected text/placeholder pixels.

use std::path::PathBuf;

use sail_render::{RenderOptions, render_file, write_png};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/")).join(name)
}

fn source() -> String {
    std::fs::read_to_string(fixture("golden.sail")).unwrap()
}

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

fn preprocessed_layout() -> (Vec<sail_layout::html::Element>, sail_layout::Layout) {
    let preprocessed = sail_compiler::parse::preprocess::preprocess(&source()).expect("preprocess");
    let (html, _css) = split_html_css(&preprocessed);
    let elements = sail_layout::html::parse_source(html);
    let layout =
        sail_layout::taffy_tree::build(elements.clone(), (800.0, 600.0)).unwrap();
    (elements, layout)
}

fn nonwhite_per_row(rgba: &[u8], width: u32) -> Vec<usize> {
    rgba.chunks_exact(4 * width as usize)
        .map(|row| {
            row.chunks_exact(4)
                .filter(|c| c != &[255, 255, 255, 255])
                .count()
        })
        .collect()
}

#[test]
fn golden_renders_at_viewport_size() {
    let out = tempfile::NamedTempFile::with_suffix(".png").unwrap();
    let opts = RenderOptions {
        viewport: (800, 600),
        fonts_dir: Some(fixture("fonts")),
    };
    let result = render_file(&fixture("golden.sail"), opts).unwrap();
    assert_eq!(result.viewport, (800, 600));
    write_png(&result.image, out.path()).unwrap();

    let meta = std::fs::metadata(out.path()).unwrap();
    assert!(meta.len() > 0, "PNG output must be non-empty");
}

#[test]
fn golden_is_not_blank() {
    let opts = RenderOptions {
        viewport: (800, 600),
        fonts_dir: Some(fixture("fonts")),
    };
    let result = render_file(&fixture("golden.sail"), opts).unwrap();

    let rgba = result.image.as_raw();
    assert!(!rgba.is_empty());

    let non_white = rgba
        .chunks_exact(4)
        .filter(|c| c != &[255, 255, 255, 255])
        .count();
    assert!(
        non_white > 1000,
        "rendered image has only {non_white} non-white pixels; expected visible content"
    );
}

#[test]
fn placeholder_is_only_my_widget_and_text_renders_above_it() {
    let (elements, layout) = preprocessed_layout();

    // Exactly one element in the fixture is unknown, and it is <my-widget>.
    let unknown_idx = elements
        .iter()
        .position(|e| e.is_unknown())
        .expect("fixture should contain an unknown element");
    assert_eq!(
        elements[unknown_idx].tag, "my-widget",
        "the only unknown element should be <my-widget>"
    );

    // Render the fixture.
    let opts = RenderOptions {
        viewport: (800, 600),
        fonts_dir: Some(fixture("fonts")),
    };
    let result = render_file(&fixture("golden.sail"), opts).unwrap();
    let width = result.image.width();
    let height = result.image.height();
    let nonwhite = nonwhite_per_row(result.image.as_raw(), width);

    // Locate the placeholder's vertical band from the computed layout.
    let computed = layout
        .computed(unknown_idx)
        .expect("layout should contain the unknown element");
    let top_y = computed.location.y as usize;
    let placeholder_height = computed.size.height.max(100.0) as usize;
    let bottom_y = (top_y + placeholder_height).min(height as usize);

    // Text must render above the placeholder.
    let content_above: usize = nonwhite.iter().take(top_y).sum();
    assert!(
        content_above > 0,
        "expected rendered text above the <my-widget> placeholder (placeholder top at y={top_y})"
    );

    // The placeholder itself must be visible.
    let placeholder_pixels: usize = nonwhite[top_y..bottom_y].iter().sum();
    assert!(
        placeholder_pixels > 1000,
        "expected visible placeholder for <my-widget> at y={top_y}..{bottom_y}, got {placeholder_pixels} non-white pixels"
    );

    // Nothing below the placeholder: the column is content-height, so the rest
    // of the viewport stays blank white.
    let below: usize = nonwhite.iter().skip(bottom_y).sum();
    assert_eq!(below, 0, "expected no rendered content below the placeholder");
}
