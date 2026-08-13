//! JOB-E1 type gate success gates (integration).
//!
//! - RED: synthetic Pure controller with `window`/`location` → sail-pure-region.
//! - GREEN (E2a): live sail-docs-rail is Pure — no DOM globals after corpus scrub.
//! - OnType does not run the type gate.
//! - Clean controller compiles under check_all OnSave.

use std::path::PathBuf;

use sail_compiler::{DiagSeverity, DiagnosticTier, check_all, check_all_with_tier};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

/// Synthetic RED fixture (was live docs-rail before E2a scrub).
/// Line numbers are fixed: L3 = location, L4 = window.history.
#[test]
fn pure_region_red_window_location() {
    // Line 1 blank so controller body lines match comments below.
    let src = "\n\
<controller>\n\
export default controller((ctx) => {\n\
  const path = typeof location !== 'undefined' ? location.pathname : '/'\n\
  const hist = window.history\n\
  return { path, hist }\n\
})\n\
</controller>\n\
<div />\n";
    let path = workspace_root().join("apps/fake/src/widget/impure.sail");
    let diags = check_all(src, Some(&path), &[], None);
    let pure: Vec<_> = diags
        .iter()
        .filter(|d| d.rule.as_deref() == Some("sail-pure-region"))
        .collect();
    assert!(
        !pure.is_empty(),
        "expected sail-pure-region on window/location fixture, got: {:?}",
        diags
            .iter()
            .filter(|d| d.severity == DiagSeverity::Error)
            .map(|d| (&d.rule, d.line, &d.message))
            .collect::<Vec<_>>()
    );
    let window_hits: Vec<_> = pure
        .iter()
        .filter(|d| d.message.contains("`window`"))
        .collect();
    assert!(
        !window_hits.is_empty(),
        "expected sail-pure-region on `window`, got: {pure:?}"
    );
    assert!(
        pure.iter().any(|d| d.message.contains("`location`")),
        "expected location pure-region coverage, got: {pure:?}"
    );
}

/// E2a: docs rail reads `route.pathname` only — green under the live gate.
#[test]
fn docs_rail_green_after_e2a_scrub() {
    let path = workspace_root().join("apps/sail-site/src/sail-docs-rail/sail-docs-rail.sail");
    let src = std::fs::read_to_string(&path).expect("read sail-docs-rail.sail");
    let diags = check_all(&src, Some(&path), &[], None);
    let type_errs: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && matches!(
                    d.rule.as_deref(),
                    Some(
                        "sail-pure-region"
                            | "sail-action-surface"
                            | "sail-type-gate"
                            | "sail-no-ts-ignore"
                    )
                )
        })
        .collect();
    assert!(
        type_errs.is_empty(),
        "E2a scrubbed docs-rail must type-gate green: {type_errs:?}"
    );
}

fn type_gate_errors(rel: &str) -> Vec<sail_compiler::Diagnostic> {
    let path = workspace_root().join(rel);
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    check_all(&src, Some(&path), &[], None)
        .into_iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && matches!(
                    d.rule.as_deref(),
                    Some("sail-pure-region" | "sail-action-surface" | "sail-type-gate")
                )
        })
        .collect()
}

/// HostBag leftovers: canvas assignability, host callbacks, DomEv-as-value.
/// `fetch` under Pure on annotation-layer stays red on purpose.
#[test]
fn hostbag_region_leftovers() {
    let media =
        type_gate_errors("packages/components/sail/media/src/sail-media-av/sail-media-av.sail");
    assert!(
        !media.iter().any(|d| d.message.contains("getContext")
            || d.message.contains("TS2739")
            || d.line == 103),
        "media-av drawWaveform/HostBag must be green: {media:?}"
    );

    let list = type_gate_errors(
        "packages/components/sail/canvas/src/sail-annotation/sail-annotation-list.sail",
    );
    assert!(
        !list
            .iter()
            .any(|d| d.message.contains("onRemove") || d.line == 51),
        "annotation-list onRemove must be green: {list:?}"
    );

    let editor = type_gate_errors(
        "packages/components/sail/code-editor/src/sail-code-editor/sail-code-editor.sail",
    );
    assert!(
        !editor
            .iter()
            .any(|d| d.message.contains("DomEv") || d.message.contains("TS2693") || d.line == 225),
        "code-editor hostDispatch must be green: {editor:?}"
    );

    let layer = type_gate_errors(
        "packages/components/sail/canvas/src/sail-annotation/sail-annotation-layer.sail",
    );
    assert!(
        layer.iter().any(|d| d.message.contains("fetch")),
        "annotation-layer fetch-in-Pure must stay red: {layer:?}"
    );
}

#[test]
fn on_type_skips_type_gate() {
    let src = r#"
<controller>
export default controller((ctx) => {
  const hist = window.history
  return { hist }
})
</controller>
<div />
"#;
    let path = workspace_root().join("apps/fake/src/widget/widget.sail");
    let on_type = check_all_with_tier(src, Some(&path), &[], None, DiagnosticTier::OnType);
    assert!(
        !on_type.iter().any(|d| matches!(
            d.rule.as_deref(),
            Some("sail-pure-region" | "sail-type-gate" | "sail-action-surface")
        )),
        "OnType must not run type gate: {on_type:?}"
    );
    let on_save = check_all_with_tier(src, Some(&path), &[], None, DiagnosticTier::OnSave);
    assert!(
        on_save
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-pure-region")),
        "OnSave must run type gate: {on_save:?}"
    );
}

#[test]
fn clean_controller_check_all_no_type_gate_error() {
    let src = r#"
<controller>
export default controller((ctx) => {
  const n = signal(0)
  const inc = () => n.set(n() + 1)
  return { n, inc }
})
</controller>
<button on:click={inc}>{n()}</button>
"#;
    let path = workspace_root().join("apps/fake/src/widget/clean.sail");
    let diags = check_all(src, Some(&path), &[], None);
    let type_errs: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && matches!(
                    d.rule.as_deref(),
                    Some(
                        "sail-pure-region"
                            | "sail-action-surface"
                            | "sail-type-gate"
                            | "sail-no-ts-ignore"
                    )
                )
        })
        .collect();
    assert!(
        type_errs.is_empty(),
        "clean Pure controller must not type-gate fail: {type_errs:?}"
    );
}
