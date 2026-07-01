//! Coverage for the `--styles` / `CompileOpts.styles` CSS-merge path.
//!
//! Components keep their CSS in sibling `.css` files; the build
//! pipeline passes the file's contents to the compiler so the emitted
//! `component()` call carries the styles. These tests exercise both
//! the library entry point (the merged CSS reaches the emitted JS) and
//! the CLI flag (the binary reads the file and produces the same
//! output as an in-line `<style>` block would).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn binary_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !p.join("target").exists() {
        p = p.parent().expect("workspace root").to_path_buf();
    }
    let exe = if cfg!(windows) { "sui-compiler.exe" } else { "sui-compiler" };
    p.join("target").join("debug").join(exe)
}

#[test]
fn styles_opt_emits_styles_field_in_component_export() {
    let path = fixtures_dir().join("ship-styled.sui");
    let src = fs::read_to_string(&path).unwrap();
    let css = fs::read_to_string(fixtures_dir().join("ship-styled.css")).unwrap();
    let out = compile(
        &src,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::Component,
            source_path: Some(path),
            styles: Some(css.clone()),
            component_meta: Some(ComponentMeta { name: "ship-styled-test".into(), shadow: true }),
            emit_dts: false,
            emit_component_imports: false,
        lint_overrides: Vec::new(),
        },
    )
    .expect("compile with styles");
    assert!(
        out.js.contains("styles:"),
        "emitted module should include a styles field: {}",
        out.js
    );
    assert!(
        out.js.contains(":host { display: inline-flex; }"),
        "emitted module should carry the merged CSS: {}",
        out.js
    );
    assert!(
        out.js.contains("shadow: true"),
        "shadow attribute should round-trip through emit: {}",
        out.js
    );
}

#[test]
fn styles_opt_rejects_non_component_source() {
    let src = "<p>plain view</p>";
    let err = compile(
        src,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::Auto,
            source_path: None,
            styles: Some(":host { color: red }".into()),
            component_meta: None,
            emit_dts: false,
            emit_component_imports: false,
        
            lint_overrides: Vec::new(),},
    )
    .expect_err("must reject styles on non-component source");
    assert!(
        err.message.contains("--styles is only valid for component sources"),
        "unexpected error: {}",
        err.message
    );
}

#[test]
fn styles_opt_merges_with_existing_inline_style_block() {
    // Authors are free to keep an inline `<style>` block too — the
    // sibling .css just appends. Both should end up concatenated in
    // source order.
    let src = r#"<prop name="x" type="string" default="" />
<style>
  .inline { color: var(--foreground); }
</style>
<span class="inline">hi</span>"#;
    let out = compile(
        src,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::Component,
            source_path: None,
            styles: Some(".external { color: var(--muted-foreground); }".into()),
            component_meta: Some(ComponentMeta { name: "ship-mixed-test".into(), shadow: false }),
            emit_dts: false,
            emit_component_imports: false,
        
            lint_overrides: Vec::new(),},
    )
    .expect("compile mixed inline + external styles");
    assert!(out.js.contains(".inline { color: var(--foreground); }"), "inline style lost: {}", out.js);
    assert!(out.js.contains(".external { color: var(--muted-foreground); }"), "external style lost: {}", out.js);
}

#[test]
fn view_inline_style_is_installed_at_mount_not_dropped() {
    // A route `page.sui` (View mode) with an inline `<style>` must NOT be
    // dropped. Before this, only component mode emitted `program.styles`; the
    // view emit ignored them, so a route's inline `<style>` produced no styling
    // and no error. The view emit now installs the lifted CSS as a `<style>`
    // into the view's `container` at mount — first-class inline style for
    // views, the same runtime install components and the synthetic path use.
    let src = r#"<style>
  .hero { color: var(--foreground); }
</style>
<div class="hero">hi</div>"#;
    let out = compile(
        src,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::Component,
            source_path: Some(PathBuf::from("routes/landing/page.sui")),
            styles: None,
            component_meta: None,
            emit_dts: false,
            emit_component_imports: false,
        
            lint_overrides: Vec::new(),},
    )
    .expect("view with inline <style> compiles");
    assert!(
        out.js.contains(".hero { color: var(--foreground); }"),
        "inline <style> CSS must be bundled into the view module, not dropped: {}",
        out.js
    );
    assert!(
        out.js.contains("data-sui-style"),
        "the runtime style install (data-sui-style dedupe key) must be emitted: {}",
        out.js
    );
    assert!(
        out.js.contains("createElement(\"style\")"),
        "the install must create a <style> element at mount: {}",
        out.js
    );
}

#[test]
fn view_without_inline_style_emits_no_style_install() {
    // No `<style>` → no install block. The injection is gated on a non-blank
    // `program.styles`, so an ordinary view stays byte-for-byte what it was.
    let src = r#"<div class="plain">hi</div>"#;
    let out = compile(
        src,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::Component,
            source_path: Some(PathBuf::from("routes/plain/page.sui")),
            styles: None,
            component_meta: None,
            emit_dts: false,
            emit_component_imports: false,
        
            lint_overrides: Vec::new(),},
    )
    .expect("plain view compiles");
    assert!(
        !out.js.contains("data-sui-style"),
        "a view with no inline <style> must emit no style install: {}",
        out.js
    );
}

#[test]
fn cli_styles_flag_threads_css_into_emitted_module() {
    let bin = binary_path();
    if !bin.exists() {
        eprintln!("binary not built; run `cargo build -p sui-compiler`");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let input = fixtures_dir().join("ship-styled.sui");
    let css = fixtures_dir().join("ship-styled.css");
    let out = tmp.path().to_path_buf();
    let status = Command::new(&bin)
        .args(["build"])
        .arg(&input)
        .args(["--out"])
        .arg(&out)
        .args(["--component-name", "ship-styled-test"])
        .args(["--styles"])
        .arg(&css)
        .status()
        .expect("invoke sui-compiler");
    assert!(status.success(), "CLI failed");
    let js = fs::read_to_string(out.join("ship-styled.js")).unwrap();
    assert!(js.contains("styles:"), "emitted JS missing styles field: {js}");
    assert!(
        js.contains(":host { display: inline-flex; }"),
        "emitted JS missing CSS body: {js}"
    );
}

#[test]
fn cli_styles_flag_rejects_directory_input() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let input = fixtures_dir(); // directory, not file
    let css = fixtures_dir().join("ship-styled.css");
    let out = tmp.path().to_path_buf();
    let result = Command::new(&bin)
        .args(["build"])
        .arg(&input)
        .args(["--out"])
        .arg(&out)
        .args(["--styles"])
        .arg(&css)
        .output()
        .expect("invoke");
    assert!(!result.status.success(), "expected directory + --styles to fail");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("--styles requires a single .sui input"),
        "unexpected stderr: {stderr}"
    );
}
