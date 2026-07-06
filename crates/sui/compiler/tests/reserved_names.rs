//! The language owns the bare-name namespace: unknown bare tags are errors,
//! valid HTML/SVG and hyphenated components pass.

use sui_compiler::{compile, CompileMode, CompileOpts};

fn opts() -> CompileOpts {
    CompileOpts {
        setup_url: None,
        mode: CompileMode::Auto,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn unknown_bare_tag_is_rejected() {
    let err = compile("<card>hi</card>", opts()).unwrap_err();
    let m = format!("{err:?}");
    assert!(m.contains("unknown element") && m.contains("card"), "{m}");
    assert!(m.contains("reserved for the language"), "{m}");
}

#[test]
fn a_typo_is_caught() {
    assert!(compile("<buton>x</buton>", opts()).is_err());
}

#[test]
fn valid_html_passes() {
    assert!(compile("<div><span>hi</span><button>go</button></div>", opts()).is_ok());
}

#[test]
fn hyphenated_component_passes() {
    // A user's own component — the language does not police the hyphen space.
    assert!(compile("<my-card>hi</my-card>", opts()).is_ok());
}

#[test]
fn svg_including_camelcase_passes() {
    assert!(compile(
        "<svg viewBox=\"0 0 1 1\"><linearGradient><stop/></linearGradient><path d=\"M0 0\"/></svg>",
        opts()
    )
    .is_ok());
}
