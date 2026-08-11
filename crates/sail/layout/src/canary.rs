//! SPEC-LAYOUT-DIAG-1 / S4.2 — layout kinds corpus canary harness surface.
//!
//! Rollout protocol §3: run experimental kinds (`unintended-overflow`,
//! `clipped-text`) against a fixture corpus, record counts + sample findings
//! + FP/TP judgments, and **do not promote** to default severity.
//!
//! This module is a **minimal stub** so the red gate compiles. Implementer
//! replaces the body of [`run`] / [`report_to_json`] with a real corpus walk
//! under experimental options. **Do not weaken the gate tests.**

use std::path::Path;

use serde::Serialize;

/// Aggregate counts for the machine-readable RESULT section.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CanaryCounts {
    /// Number of fixture/corpus files actually scanned.
    pub files_scanned: usize,
    /// Findings with kind `unintended-overflow`.
    pub unintended_overflow: usize,
    /// Findings with kind `clipped-text`.
    pub clipped_text: usize,
}

impl CanaryCounts {
    /// Total experimental findings (both kinds).
    pub fn experimental_findings(&self) -> usize {
        self.unintended_overflow + self.clipped_text
    }
}

/// One sample finding for the RESULT table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanarySample {
    /// Corpus path or fixture label.
    pub path: String,
    /// Diagnostic kind (`unintended-overflow` | `clipped-text`).
    pub kind: String,
    /// Geometry / message fact (not a text-golden pin).
    pub message: String,
}

/// True/false positive judgment for a sample finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FpJudgment {
    TruePositive,
    FalsePositive,
    Unclear,
}

/// FP/TP verdict row keyed to a sample.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FpVerdict {
    pub path: String,
    pub kind: String,
    pub judgment: FpJudgment,
    /// Short note: why TP/FP (required substance for promotion law).
    pub note: String,
}

/// Promotion status — S4.2 records criteria but must leave default off.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromotionStatus {
    /// `true` when experimental kinds are still gated (not default Error).
    pub experimental_only: bool,
    /// Documented criteria for a future promote (non-empty prose).
    pub criteria: String,
}

/// Full canary RESULT structure (counts + samples + verdicts + promotion).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanaryReport {
    pub counts: CanaryCounts,
    pub samples: Vec<CanarySample>,
    pub fp_verdicts: Vec<FpVerdict>,
    pub promotion: PromotionStatus,
}

/// Run experimental layout kinds against labeled preprocessed fixtures and
/// optional on-disk corpus roots.
///
/// # Contract
///
/// - Every entry in `fixture_sources` is scanned under
///   [`crate::CheckOptions::experimental`].
/// - `corpus_roots` may be empty; when non-empty, implementer walks supported
///   files (preprocessed HTML or via an external preprocess step) as feasible.
/// - Report must populate [`CanaryCounts`], samples, per-sample FP verdicts,
///   and promotion criteria with `experimental_only: true`.
///
/// Stub returns an empty, non-conforming report so the gate stays RED until
/// the real harness lands.
pub fn run(
    fixture_sources: &[(&str, &str)],
    corpus_roots: &[&Path],
) -> anyhow::Result<CanaryReport> {
    let _ = (fixture_sources, corpus_roots);
    // S4.2 stub — implementer: experimental scan + RESULT shape + FP table.
    Ok(CanaryReport {
        counts: CanaryCounts::default(),
        samples: Vec::new(),
        fp_verdicts: Vec::new(),
        promotion: PromotionStatus {
            experimental_only: false,
            criteria: String::new(),
        },
    })
}

/// Serialize a canary report to machine-readable JSON for RESULT-SPIKE-S4-2.
pub fn report_to_json(report: &CanaryReport) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}
