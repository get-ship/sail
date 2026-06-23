//! Tests for the handler-complexity check (tier-1/tier-2 threshold rule, condition 1).
//!
//! Four cases:
//!   single-expr arrow — OK without .ts
//!   single-stmt block — OK without .ts
//!   multi-stmt block  — fails without .ts
//!   multi-stmt block  — OK with sibling .ts

use std::path::PathBuf;

use sui_compiler::{compile, CompileMode, CompileOpts};

fn view_opts(source_path: Option<PathBuf>) -> CompileOpts {
    CompileOpts {
        setup_url: None,
        mode: CompileMode::View,
        source_path,
        styles: None,
        component_meta: None,
        emit_dts: false,
        emit_ssr: false,
        emit_component_imports: false,
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
fn multi_stmt_passes_with_sibling_controller_ts() {
    let dir = tempfile::tempdir().expect("create tempdir");
    let controller_src = "import { controller, signal } from '@ship/sui-runtime'\n\
        export default controller({\n\
          scope: () => ({ x: signal(0), y: signal(0) }),\n\
        })\n";
    std::fs::write(dir.path().join("view.controller.ts"), controller_src).unwrap();
    let sui_path = dir.path().join("view.sui");
    let src = r#"<button on:click={() => { x.set(1); y.set(2) }}>+</button>"#;
    std::fs::write(&sui_path, src).unwrap();

    compile(src, view_opts(Some(sui_path)))
        .expect("multi-statement handler with paired .controller.ts must pass");
}
