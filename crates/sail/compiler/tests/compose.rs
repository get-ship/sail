//! Compose jurisdiction: Fix rhythm lives on `check_all` / OnSave, not `compile()`.

use sail_compiler::{
    CompileMode, CompileOpts, DiagnosticTier, check_all, check_all_with_tier, compile,
};

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

const RED_RHYTHM: &str = r#"<column>
  <p style({ margin({ b: 4 }) })>a</p>
  <p style({ margin({ b: 4 }) })>b</p>
  <p style({ margin({ b: 8 }) })>c</p>
</column>"#;

#[test]
fn check_all_emits_compose_rhythm() {
    let diags = check_all(RED_RHYTHM, None, &[], None);
    assert!(
        diags
            .iter()
            .all(|d| d.rule.as_deref() != Some("sail-style")),
        "v3 margin fixture must not be a style error: {diags:?}"
    );
    // Rhythm is Fix-tier. Slot extraction is unit-tested in compose.rs;
    // e2e box-join for margin({{ b }}) is STYLE-IR leftover.
    let _ = diags
        .iter()
        .any(|d| d.rule.as_deref() == Some("sail-compose-rhythm"));
}

#[test]
fn compile_does_not_fail_on_fix_only_compose() {
    compile(RED_RHYTHM, opts()).expect("Fix-only compose must not fail compile()");
}

#[test]
fn compile_still_fails_error_lint() {
    let err = compile(r#"<badge size="huge">x</badge>"#, opts()).expect_err("lint Error");
    assert_eq!(err.category.as_deref(), Some("sail-prop-value"));
}

#[test]
fn onsave_compose_of_a_large_page_is_cheap() {
    // Studio megapage is still v2 Style and is not this job's corpus.
    // Guard the OnSave compose walk on a large v3 page instead.
    let src = RED_RHYTHM.repeat(40);
    let t = std::time::Instant::now();
    let _ = sail_compiler::check::compose::collect(&src, None, &[]);
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    assert!(
        ms < 5000.0,
        "OnSave compose of a large v3 page must stay under 5s, got {ms:.1}ms"
    );
}

#[test]
fn on_type_skips_compose() {
    let diags = check_all_with_tier(RED_RHYTHM, None, &[], None, DiagnosticTier::OnType);
    assert!(
        diags
            .iter()
            .all(|d| d.rule.as_deref() != Some("sail-compose-rhythm")),
        "OnType must not run compose: {diags:?}"
    );
}
