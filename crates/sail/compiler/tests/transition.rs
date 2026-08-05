//! `transition:` / `in:` / `out:` are not sail vocabulary — unknown directive.
//! Authors use Style motion (`style.enter` / `style.exit`).

use sail_compiler::{CompileMode, CompileOpts, compile};

fn opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn assert_unknown_directive(src: &str) {
    let err = compile(src, opts()).expect_err("must not compile");
    assert!(
        err.message.contains("unknown directive") || err.message.contains("style."),
        "expected unknown-directive or Style pointer: {}",
        err.message
    );
    assert!(
        !err.message.to_ascii_lowercase().contains("retired"),
        "must not embalm: {}",
        err.message
    );
}

#[test]
fn transition_directive_is_unknown_vocabulary() {
    assert_unknown_directive(r#"<div transition:fade>x</div>"#);
}

#[test]
fn custom_transition_names_are_equally_unknown() {
    assert_unknown_directive(r#"<div transition:myfx={{ dur: 1 }}>x</div>"#);
}

#[test]
fn in_and_out_are_unknown_vocabulary() {
    assert_unknown_directive(r#"<div in:slide>x</div>"#);
    assert_unknown_directive(r#"<div out:fade>x</div>"#);
}
