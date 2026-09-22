//! The compiler must never stay silent (D1–D4).
//!
//! Four surfaces that failed by DOING NOTHING — or by blaming the wrong line.
//! Each test asserts the DIAGNOSTIC (message + location), not merely that the
//! compile failed: the whole defect class is that the compiler's *answer* was
//! useless, so a test that only checks `is_err()` would pass on a phantom
//! error at a consumer line.

use sail_compiler::{CompileMode, CompileOpts, compile};

fn opts() -> CompileOpts {
    CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
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

/// 1-based (line, col) of a compile error's span.
fn at(src: &str, err: &sail_compiler::CompileError) -> (usize, usize) {
    let span = err.span.expect("diagnostic carries a span");
    sail_compiler::diagnostics::line_col(src, span.start)
}

const CTRL: &str = "<controller>\n\
    import { controller, signal } from '@ship/sail-runtime'\n\
    export default controller((ctx) => ({ p: signal(0) }))\n\
    </controller>\n";

// ── D1 · a bare `{signal}` renders the FUNCTION SOURCE ──────────────────────

#[test]
fn d1_bare_signal_in_a_text_hole_is_refused() {
    let src = format!("{CTRL}<p>{{p}}</p>");
    let err = compile(&src, opts()).expect_err("a bare `{p}` text hole must not compile");
    assert_eq!(
        err.category.as_deref(),
        Some("sail-signal-not-called"),
        "{err:?}"
    );
    assert!(
        err.message.contains("`{p}`") && err.message.contains("`{p()}`"),
        "must name the mistake AND the exact fix, got: {}",
        err.message
    );
    assert_eq!(at(&src, &err).0, 5, "blame the line the hole is on");
}

#[test]
fn d1_bare_signal_in_an_attribute_value_is_refused() {
    let src = format!("{CTRL}<progress value={{p}} max=\"100\"></progress>");
    let err = compile(&src, opts()).expect_err("a bare `value={p}` must not compile");
    assert_eq!(
        err.category.as_deref(),
        Some("sail-signal-not-called"),
        "{err:?}"
    );
    assert!(err.message.contains("p()"), "{}", err.message);
}

#[test]
fn d1_called_signal_still_compiles() {
    let src = format!("{CTRL}<progress value={{p()}} max=\"100\">{{p()}}</progress>");
    compile(&src, opts()).expect("`{p()}` is the correct form and must keep compiling");
}

#[test]
fn d1_bind_value_still_takes_the_uncalled_signal() {
    // `bind:value` is a two-way binding: it needs the CELL, not its value.
    // The new check must not swallow the one place a bare signal is right.
    let src = format!("{CTRL}<input bind:value={{p}} />");
    compile(&src, opts()).expect("bind:value={p} must keep compiling");
}

// ── D4 · a regex literal in `<controller>` collapsed the whole scope ────────

const CTRL_REGEX: &str = "<controller>\n\
    import { controller, signal } from '@ship/sail-runtime'\n\
    export default controller((ctx) => {\n\
    \x20 const bez = (v) => v.match(/cubic-bezier\\(([^)]+)\\)/)\n\
    \x20 const rail = signal(0)\n\
    \x20 return { rail, bez }\n\
    })\n\
    </controller>\n";

#[test]
fn d4_regex_literal_in_a_controller_does_not_collapse_the_scope() {
    let src = format!("{CTRL_REGEX}<div>{{rail()}}</div>");
    compile(&src, opts())
        .expect("a regex literal is valid JS — the controller's returned scope must survive it");
}

#[test]
fn d4_regex_literal_keeps_signal_classification() {
    // Not just "declared": the scope member must still be classified a SIGNAL,
    // which is what `(--rail)` sx reflection and the on:-callable check read.
    let src = format!("{CTRL_REGEX}<button on:click={{rail}}>x</button>");
    let err = compile(&src, opts()).expect_err("a signal is not a callable handler");
    assert!(
        err.message.contains("signal"),
        "`rail` must still classify as a signal past the regex, got: {}",
        err.message
    );
}

// ── D2 · comma-declared signals: only the FIRST registered ─────────────────

const CTRL_COMMA: &str = "<controller>\n\
    import { controller, signal } from '@ship/sail-runtime'\n\
    export default controller((ctx) => {\n\
    \x20 const a = signal(0), b = signal(1)\n\
    \x20 return { a, b }\n\
    })\n\
    </controller>\n";

#[test]
fn d2_every_declarator_in_a_multi_declarator_const_registers() {
    // `(--b)` is the sx reflection of the SECOND declarator. Before the fix it
    // failed `spinnaker-bind` with a did-you-mean pointing at its sibling `a`.
    let src = format!(
        "{CTRL_COMMA}<div style({{ opacity(bind(a)) }})>\n  <span style({{ opacity(bind(b)) }})></span>\n</div>"
    );
    compile(&src, opts()).expect("both `a` and `b` are signals — both must reflect");
}

#[test]
fn d2_later_declarator_classifies_as_a_signal_not_a_value() {
    let src = format!("{CTRL_COMMA}<button on:click={{b}}>x</button>");
    let err = compile(&src, opts()).expect_err("a signal is not a callable handler");
    assert!(
        err.message.contains("signal"),
        "`b` must classify as a signal, not an unclassified value, got: {}",
        err.message
    );
}

// ── D3 · `analyze()` was stricter than `compile()` ─────────────────────────

#[test]
fn d3_analyze_agrees_with_compile_on_an_unconfigured_source() {
    // No `sail.config.json` in reach → no capability policy is declared, so the
    // capability-gated rules must stay quiet. Editor squiggles that name a
    // policy nobody opted into are squiggles that lie about what builds.
    let src = r#"<div style({ pad(4) })> hi</div>"#;
    compile(src, opts()).expect("sx compiles with no policy declared");
    let a = sail_compiler::analyze(src, None, &[]);
    let noise: Vec<_> = a
        .diagnostics
        .iter()
        .filter(|d| d.rule.as_deref() == Some("sail-no-sx"))
        .collect();
    assert!(
        noise.is_empty(),
        "analyze must not warn about a policy the compile never applied: {noise:?}"
    );
}
