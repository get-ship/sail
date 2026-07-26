//! `transition:` is RETIRED — there is one animation system (`fx:*`).
//! The directive must fail compile with a message that names the replacement,
//! so an author (or agent) migrating old sources gets steered, not stranded.

use sail_compiler::{compile, CompileMode, CompileOpts};

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

#[test]
fn transition_directive_is_a_compile_error_pointing_at_motion() {
    let err = compile(r#"<div transition:fade>x</div>"#, opts())
        .expect_err("retired directive must not compile");
    assert!(
        err.message.contains("retired")
            && (err.message.contains("style.enter")
                || err.message.contains(".enter")
                || err.message.contains("style.exit")),
        "error must name the Style motion replacement: {}",
        err.message
    );
}

#[test]
fn custom_transition_names_are_equally_retired() {
    let err = compile(r#"<div transition:myfx={{ dur: 1 }}>x</div>"#, opts())
        .expect_err("retired directive must not compile");
    assert!(err.message.contains("retired"), "{}", err.message);
}
