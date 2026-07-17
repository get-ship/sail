//! Tests for the handler-complexity check (tier-1/tier-2 threshold rule, condition 1).
//!
//! Four cases:
//!   single-expr arrow — OK without .ts
//!   single-stmt block — OK without .ts
//!   multi-stmt block  — fails without .ts
//!   multi-stmt block  — OK with sibling .ts

use std::path::PathBuf;

use sail_compiler::{compile, CompileMode, CompileOpts};

fn view_opts(source_path: Option<PathBuf>) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn single_expr_arrow_passes_without_ts() {
    compile(
        r#"<button on:click={() => count.set(count() + 1)}>+</button>"#,
        view_opts(None),
    )
    .expect("single-expression handler must pass without paired .ts");
}

#[test]
fn single_stmt_block_passes_without_ts() {
    compile(
        r#"<button on:click={() => { count.set(count() + 1) }}>+</button>"#,
        view_opts(None),
    )
    .expect("single-statement block handler must pass without paired .ts");
}

#[test]
fn multi_stmt_fails_without_ts() {
    let err = compile(
        r#"<button on:click={() => { x.set(1); y.set(2) }}>+</button>"#,
        view_opts(None),
    )
    .expect_err("multi-statement handler without paired .ts must fail");

    assert!(
        err.message.contains("multi-statement"),
        "message should mention 'multi-statement': {}",
        err.message
    );
    assert!(
        err.message.contains("controller"),
        "message should mention 'controller': {}",
        err.message
    );
    assert!(err.span.is_some(), "error must carry a span");
    assert!(
        err.category.as_deref().map(|c| c.contains("on:")).unwrap_or(false),
        "category should name the on: event, got: {:?}",
        err.category
    );
}

#[test]
fn multi_stmt_passes_with_inline_controller() {
    // The controller is the inline `<controller>` block — the ONE source (no
    // paired `.controller.ts` fallback). It provides x/y so the multi-statement
    // handler is allowed and its scope refs bind.
    let src = "<controller>\n\
        import { controller, signal } from '@ship/sail-runtime'\n\
        export default controller((ctx) => ({ x: signal(0), y: signal(0) }))\n\
        </controller>\n\
        <button on:click={() => { x.set(1); y.set(2) }}>+</button>";

    compile(src, view_opts(None))
        .expect("multi-statement handler with inline <controller> must pass");
}
