//! Integration tests for `check_all` — the multi-diagnostic entry point
//! behind the LSP + agent tooling.
//!
//! The defining contract vs `compile()`: `compile()` short-circuits on the
//! first error (a failed build needs one reason); `check_all` collects every
//! issue so an author/agent fixes them in one pass.

use sui_compiler::{check_all, compile, CompileMode, CompileOpts, DiagSeverity};

fn opts() -> CompileOpts {
    CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn collects_multiple_errors_where_compile_stops_at_one() {
    // Two distinct renderable-subset violations on two lines: `new` and `this`.
    // Content tags (<p>/<em>) — not raw <div>/<span> — so the only diagnostics
    // are the two expr-subset Errors, keeping this test about multi-collection.
    let src = "<p>{new Date()}</p>\n<em>{this.value}</em>";

    // compile() stops at the first.
    compile(src, opts()).expect_err("compile should fail");

    // check_all surfaces both.
    let diags = check_all(src, None, &[]);
    assert!(
        diags.len() >= 2,
        "expected >=2 diagnostics, got {}: {diags:?}",
        diags.len()
    );
    assert!(diags.iter().any(|d| d.line == 1), "missing line-1 diag: {diags:?}");
    assert!(diags.iter().any(|d| d.line == 2), "missing line-2 diag: {diags:?}");
    assert!(diags.iter().all(|d| matches!(d.severity, DiagSeverity::Error)));
}

#[test]
fn clean_source_yields_no_diagnostics() {
    // Bare interpolation is renderable; with no controller path, scope-bound
    // checks are skipped, so this is clean.
    let diags = check_all("<p>{name}</p>", None, &[]);
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");
}

#[test]
fn parse_error_is_a_single_diagnostic() {
    // Nothing can run without an AST, so a parse failure is exactly one diag.
    let diags = check_all("<p>{", None, &[]);
    assert_eq!(diags.len(), 1, "parse error should yield one diag: {diags:?}");
}

#[test]
fn diagnostics_are_sorted_by_position() {
    let src = "<em>{this.value}</em>\n<p>{new Date()}</p>";
    let diags = check_all(src, None, &[]);
    let positions: Vec<(usize, usize)> = diags.iter().map(|d| (d.line, d.col)).collect();
    let mut sorted = positions.clone();
    sorted.sort();
    assert_eq!(positions, sorted, "diagnostics must be position-sorted: {diags:?}");
}
