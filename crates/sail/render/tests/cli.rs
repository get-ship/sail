//! CLI coverage moved to deck-cli `sail render`. Keep a library smoke here.

use sail_render::{RenderOptions, render_file, write_png};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/")).join(name)
}

#[test]
fn library_renders_png() {
    let out = tempfile::NamedTempFile::with_suffix(".png").unwrap();
    let opts = RenderOptions {
        viewport: (400, 300),
        fonts_dir: Some(fixture("fonts")),
        app_fonts_dir: None,
        theme: Some("sail".into()),
        mode: Some("light".into()),
        workspace: None,
        file: Some("paint.sail".into()),
        drop_bg_token: None,
    };
    let result = render_file(&fixture("paint.sail"), opts).unwrap();
    write_png(&result.image, out.path()).unwrap();
    assert!(std::fs::metadata(out.path()).unwrap().len() > 0);
    assert!(!result.html.contains("{ticks()}"));
}
