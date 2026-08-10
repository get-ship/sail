//! SPEC-LAYOUT-DIAG-1 / S4.1 — red gate for experimental layout kinds.
//!
//! Kinds (Diagnostic.kind):
//! - `unintended-overflow` — child extents exceed container with no author
//!   opt-in (no scroll-area, no overflow setter, no truncate)
//! - `clipped-text` — measured text run cut without a truncate marker
//!
//! Both are **strict/experimental only**. Product entry:
//! [`sail_layout::check_preprocessed_with`] + [`sail_layout::CheckOptions`].
//!
//! Gate cases (job-spec-spike-s4-1-layout-kinds):
//! 1. Overflow without opt-in → finding with kind id
//! 2. Overflow with scroll-area / overflow setter → no finding
//! 3. Clipped text without truncate → finding (experimental/text_metrics path)
//! 4. Truncate present → no clipped-text
//! 5. Experimental off → kinds not emitted
//!
//! Implementer: do not weaken these assertions to green-wash a partial land.
//! Adversarial "must not fire" cases are part of the promotion law.

use sail_layout::{CheckOptions, Diagnostic, check_preprocessed, check_preprocessed_with};

const VP: (f32, f32) = (400.0, 300.0);

const KIND_UNINTENDED_OVERFLOW: &str = "unintended-overflow";
const KIND_CLIPPED_TEXT: &str = "clipped-text";

/// Fixed-height column, child taller + shrink-0, **no** overflow setter.
/// Solved geometry overflows on the main axis without author opt-in.
const OVERFLOW_NO_OPTIN: &str = r#"
<div class="sail-col s-h20 s-wfull">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// Same overflow geometry, author opted in via overflow-y:auto.
const OVERFLOW_WITH_OVERFLOW_Y_AUTO: &str = r#"
<div class="sail-col s-h20 s-wfull s-overflowyauto">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// Same geometry, author opted in via overflow:scroll.
const OVERFLOW_WITH_OVERFLOW_SCROLL: &str = r#"
<div class="sail-col s-h20 s-wfull s-overflowscroll">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// Scroll-area vocabulary opt-in (stdlib host tag + overflow axis).
const OVERFLOW_WITH_SCROLL_AREA: &str = r#"
<scroll-area class="s-h20 s-wfull s-overflowyauto">
  <div class="s-h40 s-shrink0">x</div>
</scroll-area>
"#;

/// Narrow box + long unbreakable text, overflow-x hidden, **no** truncate.
/// Text metrics path must report clipped-text under experimental+text_metrics.
const CLIPPED_TEXT_NO_TRUNCATE: &str = r#"
<div class="s-w16 s-overflowxhidden">Supercalifragilisticexpialidocious</div>
"#;

/// Same shape with truncate opt-in — must not fire clipped-text.
const CLIPPED_TEXT_WITH_TRUNCATE: &str = r#"
<div class="s-w16 s-truncate">Supercalifragilisticexpialidocious</div>
"#;

fn experimental() -> CheckOptions {
    CheckOptions::experimental()
}

fn kinds(diags: &[Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.kind).collect()
}

fn has_kind(diags: &[Diagnostic], kind: &str) -> bool {
    diags.iter().any(|d| d.kind == kind)
}

// ── 1. Overflow without opt-in → finding ────────────────────────────────────

#[test]
fn unintended_overflow_fires_without_opt_in_under_experimental() {
    let result = check_preprocessed_with(OVERFLOW_NO_OPTIN, VP, experimental())
        .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "overflow without scroll-area / overflow setter / truncate must emit \
         `{KIND_UNINTENDED_OVERFLOW}` under experimental: kinds={:?} diags={:?}",
        kinds(&result.diagnostics),
        result.diagnostics
    );
    let d = result
        .diagnostics
        .iter()
        .find(|d| d.kind == KIND_UNINTENDED_OVERFLOW)
        .unwrap();
    assert!(
        d.span.0 < d.span.1 || d.span.1 > 0,
        "finding must carry a source span via layout mapping: {d:?}"
    );
}

// ── 2. Opt-in vocabulary → no unintended-overflow ───────────────────────────

#[test]
fn unintended_overflow_suppressed_when_overflow_y_auto() {
    let result = check_preprocessed_with(OVERFLOW_WITH_OVERFLOW_Y_AUTO, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "overflowY(auto) is an opt-in overflow setter — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_when_overflow_scroll() {
    let result = check_preprocessed_with(OVERFLOW_WITH_OVERFLOW_SCROLL, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "overflow(scroll) is an opt-in overflow setter — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

#[test]
fn unintended_overflow_suppressed_for_scroll_area() {
    let result = check_preprocessed_with(OVERFLOW_WITH_SCROLL_AREA, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
        "scroll-area is explicit scroll vocabulary — must NOT emit \
         `{KIND_UNINTENDED_OVERFLOW}`: {:?}",
        result.diagnostics
    );
}

// ── 3–4. Clipped text ───────────────────────────────────────────────────────

#[test]
fn clipped_text_fires_without_truncate_under_experimental_text_metrics() {
    let result = check_preprocessed_with(CLIPPED_TEXT_NO_TRUNCATE, VP, experimental())
        .expect("layout check");
    assert!(
        has_kind(&result.diagnostics, KIND_CLIPPED_TEXT),
        "measured text cut without truncate must emit `{KIND_CLIPPED_TEXT}` \
         under experimental+text_metrics: kinds={:?} diags={:?}",
        kinds(&result.diagnostics),
        result.diagnostics
    );
}

#[test]
fn clipped_text_suppressed_when_truncate_present() {
    let result = check_preprocessed_with(CLIPPED_TEXT_WITH_TRUNCATE, VP, experimental())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_CLIPPED_TEXT),
        "truncate() is the opt-in for ellipsis clipping — must NOT emit \
         `{KIND_CLIPPED_TEXT}`: {:?}",
        result.diagnostics
    );
}

// ── 5. Experimental off → kinds not emitted ─────────────────────────────────

#[test]
fn experimental_off_does_not_emit_new_kinds_on_overflow_fixture() {
    // Default entry and explicit experimental:false share the same contract.
    let defaulted = check_preprocessed(OVERFLOW_NO_OPTIN, VP).expect("default check");
    let explicit = check_preprocessed_with(OVERFLOW_NO_OPTIN, VP, CheckOptions::default())
        .expect("explicit off");
    for (label, result) in [("default", &defaulted), ("explicit-off", &explicit)] {
        assert!(
            !has_kind(&result.diagnostics, KIND_UNINTENDED_OVERFLOW),
            "{label}: `{KIND_UNINTENDED_OVERFLOW}` must not emit when experimental is off \
             (rollout protocol): {:?}",
            result.diagnostics
        );
        assert!(
            !has_kind(&result.diagnostics, KIND_CLIPPED_TEXT),
            "{label}: `{KIND_CLIPPED_TEXT}` must not emit when experimental is off: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn experimental_off_does_not_emit_clipped_text() {
    let result = check_preprocessed_with(CLIPPED_TEXT_NO_TRUNCATE, VP, CheckOptions::default())
        .expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_CLIPPED_TEXT),
        "`{KIND_CLIPPED_TEXT}` is experimental/text_metrics only: {:?}",
        result.diagnostics
    );
}

#[test]
fn text_metrics_false_suppresses_clipped_text_even_when_experimental() {
    // SPEC: if text measurement is noisy, gate clipped-text behind textMetrics.
    let opts = CheckOptions {
        experimental: true,
        text_metrics: Some(false),
    };
    let result = check_preprocessed_with(CLIPPED_TEXT_NO_TRUNCATE, VP, opts).expect("layout check");
    assert!(
        !has_kind(&result.diagnostics, KIND_CLIPPED_TEXT),
        "text_metrics: false must suppress `{KIND_CLIPPED_TEXT}` even under experimental: {:?}",
        result.diagnostics
    );
}
