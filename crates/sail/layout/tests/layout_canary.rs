//! SPEC-LAYOUT-DIAG-1 / S4.2 — red gate for layout kinds corpus canary.
//!
//! **Test-writer owns this file. Implementer MUST NOT modify these tests.**
//!
//! # Product surface (`sail_layout::canary`)
//!
//! | Symbol | Contract |
//! |--------|----------|
//! | [`CanaryCounts`](sail_layout::canary::CanaryCounts) | `files_scanned`, `unintended_overflow`, `clipped_text` |
//! | [`CanarySample`](sail_layout::canary::CanarySample) | path + kind + message |
//! | [`FpVerdict`](sail_layout::canary::FpVerdict) | path + kind + judgment + note |
//! | [`PromotionStatus`](sail_layout::canary::PromotionStatus) | `experimental_only` + criteria prose |
//! | [`CanaryReport`](sail_layout::canary::CanaryReport) | counts + samples + fp_verdicts + promotion |
//! | [`run`](sail_layout::canary::run) | experimental scan over fixtures (+ optional corpus roots) |
//! | [`report_to_json`](sail_layout::canary::report_to_json) | machine-readable RESULT JSON |
//!
//! # Gate contract (job-spec-spike-s4-2-canary)
//!
//! 1. Canary harness exists and runs experimental kinds on a fixture corpus.
//! 2. RESULT machine-readable section: counts, sample findings, FP verdicts.
//! 3. Promotion criteria documented; default severity still off (no silent promote).
//!
//! # Out of scope
//!
//! - Promoting kinds to default Error
//! - Text goldens of diagnostic messages
//! - Full FP judgment quality review (structure + coverage only)
//!
//! Implementer: do not weaken these assertions to green-wash a partial land.

use std::path::Path;

use sail_layout::canary::{
    self, CanaryCounts, CanaryReport, CanarySample, FpJudgment, FpVerdict, PromotionStatus,
};
use sail_layout::{CheckOptions, check_preprocessed, check_preprocessed_with};

const VP: (f32, f32) = (400.0, 300.0);

const KIND_UNINTENDED_OVERFLOW: &str = "unintended-overflow";
const KIND_CLIPPED_TEXT: &str = "clipped-text";

/// Fixed-height column, child taller + shrink-0, **no** overflow setter.
/// Known TP vehicle for `unintended-overflow` under experimental (S4.1).
const OVERFLOW_NO_OPTIN: &str = r#"
<div class="sail-col s-h20 s-wfull">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

/// Narrow box + long unbreakable text, overflow-x hidden, **no** truncate.
const CLIPPED_TEXT_NO_TRUNCATE: &str = r#"
<div class="s-w16 s-overflowxhidden">Supercalifragilisticexpialidocious</div>
"#;

/// Clean control — must not invent experimental findings.
const CLEAN_BOX: &str = r#"
<div class="sail-col s-wfull">
  <div class="s-h10">ok</div>
</div>
"#;

fn fixture_corpus() -> Vec<(&'static str, &'static str)> {
    vec![
        ("fixtures/overflow-no-optin.html", OVERFLOW_NO_OPTIN),
        ("fixtures/clipped-text-no-truncate.html", CLIPPED_TEXT_NO_TRUNCATE),
        ("fixtures/clean-box.html", CLEAN_BOX),
    ]
}

fn run_canary() -> CanaryReport {
    canary::run(&fixture_corpus(), &[] as &[&Path]).expect("canary harness must run")
}

// ── 1. Harness runs experimental kinds on fixture corpus ────────────────────

#[test]
fn canary_harness_scans_fixture_corpus_files() {
    let report = run_canary();
    let n = fixture_corpus().len();
    assert!(
        report.counts.files_scanned >= n,
        "canary must scan every labeled fixture (files_scanned={}, expected ≥ {n}): {:?}",
        report.counts.files_scanned,
        report.counts
    );
}

#[test]
fn canary_detects_unintended_overflow_on_known_fixture() {
    // Sanity: product experimental path still fires on the vehicle (S4.1 landed).
    let direct = check_preprocessed_with(OVERFLOW_NO_OPTIN, VP, CheckOptions::experimental())
        .expect("direct experimental check");
    assert!(
        direct
            .diagnostics
            .iter()
            .any(|d| d.kind == KIND_UNINTENDED_OVERFLOW),
        "S4.1 vehicle must still emit `{KIND_UNINTENDED_OVERFLOW}` under experimental \
         (canary cannot be green-washed if kinds regressed): {:?}",
        direct.diagnostics
    );

    let report = run_canary();
    assert!(
        report.counts.unintended_overflow >= 1,
        "canary under experimental must count ≥1 `{KIND_UNINTENDED_OVERFLOW}` on the \
         overflow fixture: counts={:?}",
        report.counts
    );
    assert!(
        report
            .samples
            .iter()
            .any(|s| s.kind == KIND_UNINTENDED_OVERFLOW),
        "RESULT samples must include a `{KIND_UNINTENDED_OVERFLOW}` finding: {:?}",
        report.samples
    );
}

#[test]
fn canary_detects_clipped_text_on_known_fixture() {
    let direct = check_preprocessed_with(CLIPPED_TEXT_NO_TRUNCATE, VP, CheckOptions::experimental())
        .expect("direct experimental check");
    assert!(
        direct
            .diagnostics
            .iter()
            .any(|d| d.kind == KIND_CLIPPED_TEXT),
        "S4.1 vehicle must still emit `{KIND_CLIPPED_TEXT}` under experimental: {:?}",
        direct.diagnostics
    );

    let report = run_canary();
    assert!(
        report.counts.clipped_text >= 1,
        "canary under experimental must count ≥1 `{KIND_CLIPPED_TEXT}` on the clipped-text \
         fixture: counts={:?}",
        report.counts
    );
    assert!(
        report.samples.iter().any(|s| s.kind == KIND_CLIPPED_TEXT),
        "RESULT samples must include a `{KIND_CLIPPED_TEXT}` finding: {:?}",
        report.samples
    );
}

// ── 2. RESULT machine-readable: counts, samples, FP verdicts ────────────────

#[test]
fn canary_counts_structure_is_consistent() {
    let report = run_canary();
    let CanaryCounts {
        files_scanned,
        unintended_overflow,
        clipped_text,
    } = report.counts;
    assert!(
        files_scanned > 0,
        "counts.files_scanned must be > 0 when fixtures are provided"
    );
    let experimental = unintended_overflow + clipped_text;
    assert_eq!(
        report.counts.experimental_findings(),
        experimental,
        "experimental_findings helper must match kind sum"
    );
    // Samples are a RESULT table of findings — must not exceed total counts
    // and must not invent kinds outside the experimental set.
    assert!(
        report.samples.len() <= experimental || experimental == 0 && report.samples.is_empty(),
        "samples ({}) must not exceed experimental finding counts ({experimental}): {:?}",
        report.samples.len(),
        report.samples
    );
    for s in &report.samples {
        assert!(
            s.kind == KIND_UNINTENDED_OVERFLOW || s.kind == KIND_CLIPPED_TEXT,
            "sample kind must be an experimental layout kind: {s:?}"
        );
        assert!(!s.path.is_empty(), "sample path/label required: {s:?}");
        assert!(!s.message.is_empty(), "sample message required: {s:?}");
    }
}

#[test]
fn canary_fp_verdicts_cover_samples() {
    let report = run_canary();
    // When the canary reports samples, each needs an FP/TP judgment (promotion law).
    if report.samples.is_empty() {
        // Still require the verdicts *field* to exist as a vec (structure).
        let _ = &report.fp_verdicts;
        // Counts claimed findings without samples still fail the sample tests above.
        return;
    }
    assert_eq!(
        report.fp_verdicts.len(),
        report.samples.len(),
        "every sample finding needs a matching FP verdict row: samples={} verdicts={} \
         samples={:?} verdicts={:?}",
        report.samples.len(),
        report.fp_verdicts.len(),
        report.samples,
        report.fp_verdicts
    );
    for sample in &report.samples {
        let v = report.fp_verdicts.iter().find(|v| {
            v.path == sample.path && v.kind == sample.kind
        });
        assert!(
            v.is_some(),
            "missing FP verdict for sample path={} kind={}: verdicts={:?}",
            sample.path,
            sample.kind,
            report.fp_verdicts
        );
        let v = v.unwrap();
        assert!(
            !v.note.trim().is_empty(),
            "FP verdict note required (why TP/FP): {v:?}"
        );
        // Judgment must be one of the three enums (exhaustiveness via match).
        match v.judgment {
            FpJudgment::TruePositive | FpJudgment::FalsePositive | FpJudgment::Unclear => {}
        }
    }
}

#[test]
fn canary_report_json_is_machine_readable_result_shape() {
    let report = run_canary();
    let json = canary::report_to_json(&report).expect("report_to_json");
    let v: serde_json::Value = serde_json::from_str(&json).expect("RESULT JSON must parse");

    let counts = v
        .get("counts")
        .unwrap_or_else(|| panic!("RESULT JSON missing `counts`: {json}"));
    for key in ["files_scanned", "unintended_overflow", "clipped_text"] {
        assert!(
            counts.get(key).and_then(|x| x.as_u64()).is_some(),
            "counts.{key} must be a number in RESULT JSON: {json}"
        );
    }
    assert!(
        v.get("samples").and_then(|x| x.as_array()).is_some(),
        "RESULT JSON missing `samples` array: {json}"
    );
    assert!(
        v.get("fp_verdicts").and_then(|x| x.as_array()).is_some(),
        "RESULT JSON missing `fp_verdicts` array: {json}"
    );
    let promotion = v
        .get("promotion")
        .unwrap_or_else(|| panic!("RESULT JSON missing `promotion`: {json}"));
    assert!(
        promotion.get("experimental_only").and_then(|x| x.as_bool()).is_some(),
        "promotion.experimental_only must be bool: {json}"
    );
    assert!(
        promotion.get("criteria").and_then(|x| x.as_str()).is_some(),
        "promotion.criteria must be string: {json}"
    );
}

// ── 3. Promotion criteria documented; default severity still off ────────────

#[test]
fn canary_documents_promotion_criteria_and_keeps_experimental_only() {
    let report = run_canary();
    assert!(
        report.promotion.experimental_only,
        "S4.2 must NOT promote experimental kinds to default — \
         promotion.experimental_only must be true: {:?}",
        report.promotion
    );
    let criteria = report.promotion.criteria.trim();
    assert!(
        !criteria.is_empty(),
        "promotion criteria must be documented (non-empty) for RESULT-SPIKE-S4-2"
    );
    let lower = criteria.to_ascii_lowercase();
    let substantive = lower.contains("false")
        || lower.contains("fp")
        || lower.contains("promote")
        || lower.contains("canary")
        || lower.contains("experimental");
    assert!(
        substantive,
        "promotion.criteria should describe the promote law (FP≈0 / experimental / canary): {criteria:?}"
    );
}

#[test]
fn no_silent_promote_default_check_still_skips_experimental_kinds() {
    // Regression guard: landing the canary must not flip default severity on.
    let overflow = check_preprocessed(OVERFLOW_NO_OPTIN, VP).expect("default check");
    let clipped = check_preprocessed(CLIPPED_TEXT_NO_TRUNCATE, VP).expect("default check");
    for (label, result) in [("overflow", &overflow), ("clipped", &clipped)] {
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.kind == KIND_UNINTENDED_OVERFLOW || d.kind == KIND_CLIPPED_TEXT),
            "{label}: default check must not emit experimental kinds (no silent promote): {:?}",
            result.diagnostics
        );
    }
    let explicit = check_preprocessed_with(OVERFLOW_NO_OPTIN, VP, CheckOptions::default())
        .expect("explicit off");
    assert!(
        !explicit
            .diagnostics
            .iter()
            .any(|d| d.kind == KIND_UNINTENDED_OVERFLOW),
        "CheckOptions::default must keep experimental kinds off: {:?}",
        explicit.diagnostics
    );
}

// ── Type surface smoke (compile-time locks via use) ─────────────────────────

#[test]
fn canary_public_types_are_constructible_for_implementer_guidance() {
    // Documents the RESULT row shapes without being the canary itself.
    let _sample = CanarySample {
        path: "fixtures/example.html".into(),
        kind: KIND_UNINTENDED_OVERFLOW.into(),
        message: "child extents exceed container".into(),
    };
    let _verdict = FpVerdict {
        path: "fixtures/example.html".into(),
        kind: KIND_UNINTENDED_OVERFLOW.into(),
        judgment: FpJudgment::TruePositive,
        note: "no overflow opt-in on fixed-height column".into(),
    };
    let _promo = PromotionStatus {
        experimental_only: true,
        criteria: "promote only when corpus canary FP rate ≈ 0".into(),
    };
    let _ = CanaryReport {
        counts: CanaryCounts {
            files_scanned: 1,
            unintended_overflow: 1,
            clipped_text: 0,
        },
        samples: vec![_sample],
        fp_verdicts: vec![_verdict],
        promotion: _promo,
    };
}
