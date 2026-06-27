//! `morph` directive parse + emit fixture.
//!
//! The directive opts the element into in-place DOM patching when a
//! HTML-producing binding (`bindRawHtml`) writes to its contents. The
//! compiler's job is minimal: accept the bare attribute, emit it as a
//! setAttribute call, leave runtime to detect the marker.

use std::fs;
use std::path::PathBuf;

use sui_compiler::{compile, CompileMode, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn compile_sui(name: &str) -> String {
    let path = fixtures_dir().join(format!("{name}.sui"));
    let source = fs::read_to_string(&path).expect("read fixture");
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        styles: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
    };
    let out = compile(&source, opts).expect("compile morph fixture");
    out.js
}

#[test]
fn morph_directive_compiles_without_error() {
    // Smoke-test: the compile() call inside compile_sui panics on
    // diagnostic; reaching this assertion means the directive parsed.
    let _ = compile_sui("morph-directive");
}

#[test]
fn morph_directive_emits_setattribute() {
    let js = compile_sui("morph-directive");
    // The directive lowers to a plain Static attribute — the marker
    // ships to the DOM as `morph=""` for `bindRawHtml` (and other
    // HTML-producing bindings) to detect at runtime.
    assert!(
        js.contains("setAttribute(\"morph\", \"\")"),
        "expected setAttribute(\"morph\", \"\") in emit; got:\n{js}"
    );
}

#[test]
fn morph_directive_emits_no_reactive_wiring() {
    let js = compile_sui("morph-directive");
    // The directive is a static marker — no bindAttr, no effect, no
    // hydration call should be generated for it.
    assert!(
        !js.contains("bindAttr") || js.matches("bindAttr").count() == 0,
        "morph directive should not emit bindAttr; got:\n{js}"
    );
}
