//! Compiler Style composition — owned fixtures, not live apps.
//!
//! `apps/sail-site` and stdlib skins are product. This suite compiles
//! `.sail` the compiler test tree owns and checks the AOT path still
//! spends theme tokens (preprocess injects classes + CSS).

use sail_compiler::parse::preprocess::preprocess as preprocess_src;

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/style")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn pp_fixture(name: &str) -> String {
    preprocess_src(&fixture(name)).unwrap_or_else(|e| panic!("preprocess {name}: {e}"))
}

#[test]
fn panel_spends_pad_bg_radius_border_tokens() {
    let out = pp_fixture("panel.sail");
    assert!(out.contains("s-"), "must stamp classes:\n{out}");
    assert!(
        out.contains("var(--space-4)"),
        "pad(4) → space token:\n{out}"
    );
    assert!(
        out.contains("var(--bg-surface-color)") || out.contains("var(--surface)"),
        "bg(surface) → theme fill:\n{out}"
    );
    assert!(
        out.contains("var(--radius-md)"),
        "radius(md) → radius token:\n{out}"
    );
    assert!(
        out.contains("var(--border-subtle-color)") || out.contains("border-color"),
        "border(subtle) → border identity:\n{out}"
    );
}

#[test]
fn when_hover_emits_pseudo_and_muted_token() {
    let out = pp_fixture("when-hover.sail");
    assert!(out.contains(":hover"), "hover pseudo:\n{out}");
    assert!(
        out.contains("var(--bg-muted-color)") || out.contains("var(--muted)"),
        "hover bg(muted) token:\n{out}"
    );
}

#[test]
fn type_and_elevation_spend_type_and_shadow_tokens() {
    let out = pp_fixture("type-and-elevation.sail");
    assert!(out.contains("var(--text-sm)"), "type size sm:\n{out}");
    assert!(
        out.contains("var(--font-weight-medium)"),
        "type weight medium:\n{out}"
    );
    assert!(
        out.contains("var(--shadow-inset-ring)"),
        "elevation inset-ring:\n{out}"
    );
}

#[test]
fn raw_css_values_do_not_preprocess() {
    let err = preprocess_src(r#"<div style({ pad(16px) })>x</div>"#)
        .expect_err("raw px must fail closed");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("16px") || msg.to_ascii_lowercase().contains("token"),
        "raw px diagnostic: {msg}"
    );
}

#[test]
fn file_accept_mime_and_named_style_compile() {
    let src = fixture("file-accept.sail");
    sail_compiler::compile(&src, sail_compiler::CompileOpts::default())
        .unwrap_or_else(|e| panic!("file-accept compile: {e:?}"));
}

#[test]
fn input_multiline_named_style_compile() {
    let src = fixture("input-multiline.sail");
    sail_compiler::compile(&src, sail_compiler::CompileOpts::default())
        .unwrap_or_else(|e| panic!("input-multiline compile: {e:?}"));
}

#[test]
fn corpus_dir_has_no_app_paths() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/style");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let src = std::fs::read_to_string(&path).unwrap();
        assert!(
            !src.contains("apps/sail-site") && !src.contains("apps/studio"),
            "{} must not reach into live apps",
            path.display()
        );
    }
}
