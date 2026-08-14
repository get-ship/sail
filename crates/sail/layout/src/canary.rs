//! Layout kinds canary harness.
//!
//! `run` scans labeled fixtures under experimental. [`scan_preprocessed`]
//! accepts already-lowered HTML at caller viewports.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{CheckOptions, check_preprocessed_with};

/// Default viewport for canary scans (matches gate vehicles).
const DEFAULT_VIEWPORT: (f32, f32) = (400.0, 300.0);

const KIND_UNINTENDED_OVERFLOW: &str = "unintended-overflow";
const KIND_CLIPPED_TEXT: &str = "clipped-text";

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

/// Promotion criteria prose for RESULT / future promote decision.
///
/// Kinds stay experimental-only until a corpus canary shows ≈0 false
/// positives on known vehicles and a broader sample review. Do not flip
/// default severity from this spike.
const PROMOTION_CRITERIA: &str = "\
Promote unintended-overflow / clipped-text to default Error only when a \
corpus canary under experimental shows FP rate ≈ 0 on intentional true-positive \
vehicles and a reviewed sample of real corpus findings has no material false \
positives. Until then keep experimental_only = true (strict / layoutExperimental \
gate). S4.2 records counts + FP verdicts; it does not promote.";

/// Run experimental layout kinds against labeled preprocessed fixtures and
/// optional on-disk corpus roots.
///
/// # Contract
///
/// - Every entry in `fixture_sources` is scanned under
///   [`crate::CheckOptions::experimental`].
/// - `corpus_roots` may be empty; when non-empty, walks `*.html` files
///   (treated as preprocessed HTML) as a best-effort extension of the canary.
/// - Report populates [`CanaryCounts`], samples, per-sample FP verdicts,
///   and promotion criteria with `experimental_only: true`.
pub fn run(
    fixture_sources: &[(&str, &str)],
    corpus_roots: &[&Path],
) -> anyhow::Result<CanaryReport> {
    let opts = CheckOptions::experimental();
    let mut counts = CanaryCounts::default();
    let mut samples = Vec::new();
    let mut fp_verdicts = Vec::new();

    for (path, source) in fixture_sources {
        scan_source(
            path,
            source,
            opts,
            &mut counts,
            &mut samples,
            &mut fp_verdicts,
        )?;
    }

    for root in corpus_roots {
        for path in walk_html_files(root)? {
            let label = path.display().to_string();
            let source = match fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            // Best-effort: skip unreadable or layout-failing corpus files.
            if let Err(err) = scan_source(
                &label,
                &source,
                opts,
                &mut counts,
                &mut samples,
                &mut fp_verdicts,
            ) {
                let _ = err;
            }
        }
    }

    Ok(CanaryReport {
        counts,
        samples,
        fp_verdicts,
        promotion: PromotionStatus {
            experimental_only: true,
            criteria: PROMOTION_CRITERIA.to_string(),
        },
    })
}

fn scan_source(
    path: &str,
    source: &str,
    opts: CheckOptions,
    counts: &mut CanaryCounts,
    samples: &mut Vec<CanarySample>,
    fp_verdicts: &mut Vec<FpVerdict>,
) -> anyhow::Result<()> {
    let result = check_preprocessed_with(source, DEFAULT_VIEWPORT, opts)?;
    counts.files_scanned += 1;

    for d in &result.diagnostics {
        let kind = d.kind;
        if kind != KIND_UNINTENDED_OVERFLOW && kind != KIND_CLIPPED_TEXT {
            continue;
        }
        match kind {
            KIND_UNINTENDED_OVERFLOW => counts.unintended_overflow += 1,
            KIND_CLIPPED_TEXT => counts.clipped_text += 1,
            _ => {}
        }

        let sample = CanarySample {
            path: path.to_string(),
            kind: kind.to_string(),
            message: d.message.clone(),
        };
        let (judgment, note) = judge_finding(path, kind);
        fp_verdicts.push(FpVerdict {
            path: path.to_string(),
            kind: kind.to_string(),
            judgment,
            note,
        });
        samples.push(sample);
    }

    Ok(())
}

/// Coarse FP/TP judgment for promotion-law coverage.
///
/// Known intentional vehicles from the gate are marked true-positive.
/// Other experimental emissions stay `Unclear` until human corpus review —
/// canary structure is required; full FP quality review is out of scope.
fn judge_finding(path: &str, kind: &str) -> (FpJudgment, String) {
    let lower = path.to_ascii_lowercase();
    match kind {
        KIND_UNINTENDED_OVERFLOW
            if lower.contains("overflow")
                && (lower.contains("no-optin")
                    || lower.contains("no_optin")
                    || lower.contains("no-opt-in")) =>
        {
            (
                FpJudgment::TruePositive,
                "known vehicle: fixed-height column, child taller + shrink-0, \
                 no overflow opt-in — intentional TP under experimental"
                    .into(),
            )
        }
        KIND_CLIPPED_TEXT
            if lower.contains("clip")
                && (lower.contains("no-truncate") || lower.contains("no_truncate")) =>
        {
            (
                FpJudgment::TruePositive,
                "known vehicle: narrow box + long unbreakable text with \
                 overflow-x hidden and no truncate — intentional TP under experimental"
                    .into(),
            )
        }
        KIND_UNINTENDED_OVERFLOW => (
            FpJudgment::Unclear,
            "unintended-overflow on non-vehicle path; needs human FP review \
             before promote (experimental_only remains true)"
                .into(),
        ),
        KIND_CLIPPED_TEXT => (
            FpJudgment::Unclear,
            "clipped-text on non-vehicle path; needs human FP review before \
             promote (experimental_only remains true)"
                .into(),
        ),
        _ => (
            FpJudgment::Unclear,
            "unexpected experimental kind; not promotion-ready".into(),
        ),
    }
}

fn walk_html_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    if root.is_file() {
        if is_html(root) {
            out.push(root.to_path_buf());
        }
        return Ok(out);
    }
    walk_dir(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip common build/vendor noise.
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if matches!(name, "target" | "node_modules" | ".git" | "dist" | "build") {
                    continue;
                }
            }
            walk_dir(&path, out)?;
        } else if is_html(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_html(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("html"))
}

/// Serialize a canary report to machine-readable JSON for RESULT-SPIKE-S4-2.
pub fn report_to_json(report: &CanaryReport) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

/// One experimental finding from an already-preprocessed source, with
/// structured geometry (not a message-text pin).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KindFinding {
    pub kind: String,
    pub span: (usize, usize),
    pub viewport: [u32; 2],
    pub element_tag: String,
    pub element_class: String,
    pub element_index: usize,
    pub geometry: Option<crate::OverflowGeometry>,
}

/// Multi-viewport scan of one preprocessed source, filtered to a single kind.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PreprocessedScan {
    pub findings: Vec<KindFinding>,
    pub viewport_errors: Vec<String>,
    pub layout_ms: u64,
    pub viewports_ok: usize,
}

/// Scan already-preprocessed HTML at `viewports`, keeping only `kind`.
pub fn scan_preprocessed(
    preprocessed: &str,
    viewports: &[(f32, f32)],
    kind: &str,
    opts: CheckOptions,
) -> PreprocessedScan {
    let started = std::time::Instant::now();
    let mut findings = Vec::new();
    let mut viewport_errors = Vec::new();
    let mut viewports_ok = 0usize;

    for &viewport in viewports {
        match check_preprocessed_with(preprocessed, viewport, opts) {
            Ok(result) => {
                viewports_ok += 1;
                for d in result.diagnostics {
                    if d.kind != kind {
                        continue;
                    }
                    findings.push(KindFinding {
                        kind: d.kind.to_string(),
                        span: d.span,
                        viewport: [viewport.0.round() as u32, viewport.1.round() as u32],
                        element_tag: d.element_tag,
                        element_class: d.element_class,
                        element_index: d.element_index,
                        geometry: d.geometry.map(snap_geometry),
                    });
                }
            }
            Err(err) => {
                viewport_errors.push(format!(
                    "{}x{}: {err}",
                    viewport.0.round() as u32,
                    viewport.1.round() as u32
                ));
            }
        }
    }

    findings.sort_by(|a, b| {
        (
            a.span,
            a.viewport,
            a.element_index,
            a.element_tag.as_str(),
            a.element_class.as_str(),
        )
            .cmp(&(
                b.span,
                b.viewport,
                b.element_index,
                b.element_tag.as_str(),
                b.element_class.as_str(),
            ))
    });

    PreprocessedScan {
        findings,
        viewport_errors,
        layout_ms: started.elapsed().as_millis() as u64,
        viewports_ok,
    }
}

fn snap_geometry(g: crate::OverflowGeometry) -> crate::OverflowGeometry {
    crate::OverflowGeometry {
        axis: g.axis,
        child_main: snap_px(g.child_main),
        parent_main: snap_px(g.parent_main),
        child_index: g.child_index,
        parent_index: g.parent_index,
        child: crate::BoxExtents {
            x: snap_px(g.child.x),
            y: snap_px(g.child.y),
            width: snap_px(g.child.width),
            height: snap_px(g.child.height),
        },
        parent: crate::BoxExtents {
            x: snap_px(g.parent.x),
            y: snap_px(g.parent.y),
            width: snap_px(g.parent.width),
            height: snap_px(g.parent.height),
        },
    }
}

fn snap_px(v: f32) -> f32 {
    if !v.is_finite() {
        return 0.0;
    }
    let snapped = (v * 100.0).round() / 100.0;
    if snapped.abs() < 0.005 { 0.0 } else { snapped }
}

#[cfg(test)]
mod scan_preprocessed_tests {
    use super::*;
    use crate::CheckOptions;

    const OVERFLOW_NO_OPTIN: &str = r#"
<div class="sail-col s-h20 s-wfull">
  <div class="s-h40 s-shrink0">x</div>
</div>
"#;

    const CLIPPED_TEXT_NO_TRUNCATE: &str = r#"
<div class="s-w16 s-overflowxhidden">Supercalifragilisticexpialidocious</div>
"#;

    #[test]
    fn scan_preprocessed_filters_to_requested_kind_and_carries_geometry() {
        let scan = scan_preprocessed(
            OVERFLOW_NO_OPTIN,
            &[(400.0, 300.0), (1440.0, 900.0)],
            KIND_UNINTENDED_OVERFLOW,
            CheckOptions::experimental(),
        );
        assert!(
            scan.viewport_errors.is_empty(),
            "vehicle must layout: {:?}",
            scan.viewport_errors
        );
        assert!(
            !scan.findings.is_empty(),
            "overflow vehicle must emit `{KIND_UNINTENDED_OVERFLOW}`"
        );
        for f in &scan.findings {
            assert_eq!(f.kind, KIND_UNINTENDED_OVERFLOW);
            let g = f.geometry.as_ref().expect("geometry required");
            assert!(g.child_main > g.parent_main);
        }
    }

    #[test]
    fn scan_preprocessed_kind_filter_drops_clipped_text() {
        let scan = scan_preprocessed(
            CLIPPED_TEXT_NO_TRUNCATE,
            &[(400.0, 300.0)],
            KIND_UNINTENDED_OVERFLOW,
            CheckOptions::experimental(),
        );
        assert!(
            scan.findings.is_empty(),
            "clipped-text vehicle must not appear under overflow filter: {:?}",
            scan.findings
        );
    }

    #[test]
    fn scan_preprocessed_is_deterministic() {
        let a = scan_preprocessed(
            OVERFLOW_NO_OPTIN,
            &[(390.0, 844.0), (1440.0, 900.0)],
            KIND_UNINTENDED_OVERFLOW,
            CheckOptions::experimental(),
        );
        let b = scan_preprocessed(
            OVERFLOW_NO_OPTIN,
            &[(390.0, 844.0), (1440.0, 900.0)],
            KIND_UNINTENDED_OVERFLOW,
            CheckOptions::experimental(),
        );
        assert_eq!(a.findings, b.findings);
    }
}
