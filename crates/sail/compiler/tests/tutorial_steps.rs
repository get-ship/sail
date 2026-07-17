//! Compile-check the SUI tutorial's starter + solved templates under the
//! **playground-shaped** path (`source_path: None`, no `setup_url`).
//!
//! The docs site's `TutorialApp` compiles user edits in the browser via
//! the WASM build of `analyze(source, None)` — which skips scope-bound /
//! controller-aware passes. This test mirrors that path against the
//! on-disk tutorial templates, so a template that fails here is one a
//! user will see a compile error for in the playground.
//!
//! Pair with the audit at `.artifacts/tutorial-audit.json` — every
//! `compile_suspicion` finding should be re-verified against the actual
//! compiler output.

use std::fs;
use std::path::Path;

use sail_compiler::analyze;

fn load_template(step: &str, variant: &str) -> Option<String> {
    // Tests run from the compiler crate's CWD (crates/sail/compiler/);
    // walk up to the workspace root and into apps/site.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("workspace root");
    let p = root.join(format!(
        "apps/site/tutorial-examples/{step}/{variant}/template.sui"
    ));
    // The tutorial fixtures are docs-SITE content (the site's interactive
    // playground), which lives OUTSIDE the framework-boundary repo. When
    // `apps/site` isn't checked out (the framework repo), skip — this is a
    // site-app conformance concern, tracked for relocation to the site app.
    fs::read_to_string(&p).ok()
}

fn assert_clean(step: &str, variant: &str) {
    let Some(src) = load_template(step, variant) else {
        eprintln!(
            "SKIP tutorial_steps {step}/{variant}: apps/site/tutorial-examples absent \
             (docs-site content, outside the framework boundary)."
        );
        return;
    };
    let analysis = analyze(&src, None, &[]);
    let errors: Vec<_> = analysis
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, sail_compiler::DiagSeverity::Error))
        .collect();
    assert!(
        errors.is_empty(),
        "step {step}/{variant} has {} Error diagnostic(s) under playground shape — \
         the user will see this compile error in the docs site.\n\
         First error: line {} col {} — {}\n\nSource:\n{}",
        errors.len(),
        errors.first().map(|d| d.line).unwrap_or(0),
        errors.first().map(|d| d.col).unwrap_or(0),
        errors.first().map(|d| d.message.as_str()).unwrap_or(""),
        src,
    );
}

// ---- Smoke tests — known-clean steps that should keep passing. ----

#[test]
fn step_01_signals_clean() {
    assert_clean("01-hello-signal", "solved");
}

#[test]
fn step_02_derived_clean() {
    assert_clean("02-derived", "solved");
}

#[test]
fn step_04_effect_clean() {
    assert_clean("04-effect", "solved");
}

#[test]
fn step_07_event_modifiers_clean() {
    assert_clean("07-event-modifiers", "solved");
}

#[test]
fn step_22_data_live_clean() {
    assert_clean("22-data-live", "solved");
}

#[test]
fn step_24_morphs_clean() {
    assert_clean("24-morphs", "solved");
}

// ---- Fix targets — these were broken in the audit and must now be clean. ----

#[test]
fn step_17_selector_clean_after_fix() {
    assert_clean("17-selector", "starter");
    assert_clean("17-selector", "solved");
}

#[test]
fn step_19_template_hooks_clean_after_fix() {
    assert_clean("19-template-hooks", "starter");
    assert_clean("19-template-hooks", "solved");
}

#[test]
fn step_20_controller_hooks_clean_after_fix() {
    assert_clean("20-controller-hooks", "starter");
    assert_clean("20-controller-hooks", "solved");
}
