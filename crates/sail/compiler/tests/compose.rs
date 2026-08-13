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
  <p style({ box: mb(4) })>a</p>
  <p style({ box: mb(4) })>b</p>
  <p style({ box: mb(8) })>c</p>
</column>"#;

#[test]
fn check_all_emits_compose_rhythm() {
    let diags = check_all(RED_RHYTHM, None, &[], None);
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-compose-rhythm")),
        "red rhythm fixture must produce sail-compose-rhythm: {diags:?}"
    );
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
fn onsave_compose_of_studio_megapage_is_cheap() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../apps/studio/src/theme-components-surface/theme-components-surface.sail"
    );
    let src = std::fs::read_to_string(path).expect("megapage fixture");
    let t = std::time::Instant::now();
    let _ = sail_compiler::check::compose::collect(&src, Some(std::path::Path::new(path)), &[]);
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    // compile() no longer runs compose. This guards the remaining OnSave walk.
    assert!(
        ms < 5000.0,
        "OnSave compose of theme-components-surface must stay under 5s, got {ms:.1}ms"
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
