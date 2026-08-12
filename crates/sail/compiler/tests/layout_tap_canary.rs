//! Real-corpus canary for experimental `sub-min-tap-target`.
//!
//! Findings are information. Green when the driver scans, is deterministic,
//! and writes JSON — not when the corpus is empty of sub-min tap targets.

use std::path::PathBuf;

use sail_compiler::check::layout;
use sail_compiler::check::layout_corpus::{self, TREE_NAMES, default_tap_out_path};
use sail_compiler::parse::preprocess::preprocess;
use sail_layout::CheckOptions;
use sail_layout::canary;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn corpus_canary_sub_min_tap_target_writes_json_and_is_deterministic() {
    let root = workspace_root();
    let first = layout_corpus::scan_kind(&root, "sub-min-tap-target").expect("tap scan 1");
    let second = layout_corpus::scan_kind(&root, "sub-min-tap-target").expect("tap scan 2");
    assert_eq!(
        first.findings, second.findings,
        "two consecutive sub-min-tap-target scans must be identical"
    );

    assert!(
        first.counts.files_discovered >= 200,
        "must discover the real corpus: {:?}",
        first.counts
    );
    assert!(
        first.counts.files_scanned >= 150,
        "must scan the real corpus: {:?}",
        first.counts
    );
    for name in TREE_NAMES {
        let tree = first
            .trees
            .get(*name)
            .unwrap_or_else(|| panic!("missing tree {name}"));
        assert!(
            tree.files_scanned > 0,
            "tree `{name}` must scan at least one file"
        );
    }

    for f in &first.findings {
        assert_eq!(f.kind, "sub-min-tap-target");
        assert!(!f.path.is_empty());
    }

    assert_eq!(
        first.counts.sub_min_tap_target,
        first.findings.len(),
        "counter must match finding rows"
    );

    let out = default_tap_out_path(&root);
    layout_corpus::write_report(&out, &first).expect("write tap canary json");
    assert!(out.is_file(), "missing {}", out.display());

    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).expect("read tap findings"))
            .expect("tap findings JSON");
    let written_findings = written["findings"]
        .as_array()
        .expect("written findings array");
    assert_eq!(
        written_findings.len(),
        first.findings.len(),
        "written findings.len must match the in-memory scan"
    );
    assert_eq!(
        written["counts"]["sub_min_tap_target"].as_u64(),
        Some(first.counts.sub_min_tap_target as u64),
        "written counts.sub_min_tap_target must match the in-memory scan"
    );
    for (i, row) in written_findings.iter().enumerate() {
        assert!(
            row.get("geometry").map(|g| !g.is_null()).unwrap_or(false),
            "tap row {i} must carry geometry: {row}"
        );
    }
}

fn scan_file_kind(
    root: &std::path::Path,
    rel: &str,
    kind: &'static str,
) -> canary::PreprocessedScan {
    let src = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let pre = preprocess(&src).unwrap_or_else(|e| panic!("preprocess {rel}: {e}"));
    let viewports = layout::layout_viewports(&pre);
    canary::scan_preprocessed(&pre, &viewports, kind, CheckOptions::experimental())
}

#[test]
fn wi_layout_4_product_files_quiet_for_overflow_and_clipped() {
    let root = workspace_root();
    let studio = "apps/studio/src/components-surface/components-surface.sail";
    let landing = "apps/sail-site/routes/page.sail";

    let overflow = scan_file_kind(&root, studio, "unintended-overflow");
    assert!(
        overflow.viewport_errors.is_empty(),
        "studio layout errors: {:?}",
        overflow.viewport_errors
    );
    let action_overflow = overflow.findings.iter().any(|f| {
        f.element_class.contains("overflowxauto")
            || (f.element_class.contains("sail-row")
                && f.geometry
                    .as_ref()
                    .is_some_and(|g| g.child_main >= 300.0 && g.parent_main <= 150.0))
    });
    assert!(
        !action_overflow,
        "WI-LAYOUT-1-1 action cluster must no longer overflow: {:?}",
        overflow.findings
    );

    let studio_src = std::fs::read_to_string(root.join(studio)).unwrap();
    let studio_pre = preprocess(&studio_src).unwrap();
    let mobile = sail_layout::check_preprocessed_with(
        &studio_pre,
        (390.0, 844.0),
        CheckOptions::experimental(),
    )
    .expect("studio 390");
    let desktop = sail_layout::check_preprocessed_with(
        &studio_pre,
        (1440.0, 900.0),
        CheckOptions::experimental(),
    )
    .expect("studio 1440");
    let header_390 = mobile
        .boxes
        .iter()
        .find(|b| {
            b.class.contains("sail-row")
                && b.class.to_ascii_lowercase().contains("overflowxauto")
                && !b.class.contains("scroll-area")
        })
        .expect("header scrollport at 390");
    let actions_390 = mobile
        .boxes
        .iter()
        .find(|b| {
            let c = b.class.as_str();
            c.contains("sail-nowrap")
                && c.contains("s-shrink0")
                && c.contains("sail-g-2")
                && c.contains("sail-ai-center")
                && !c.to_ascii_lowercase().contains("overflowxauto")
        })
        .expect("action cluster at 390");
    assert!(
        header_390.width <= 110.5,
        "header scrollport must fit the 110px leftover: {header_390:?}"
    );
    assert!(
        actions_390.width >= 320.0,
        "action cluster must stay intact (~325px), not shrink-to-fit: {actions_390:?}"
    );
    assert!(
        actions_390.width > header_390.width,
        "actions stay wider than the 110px leftover: header={header_390:?} actions={actions_390:?}"
    );
    let header_1440 = desktop
        .boxes
        .iter()
        .find(|b| {
            b.class.contains("sail-row")
                && b.class.to_ascii_lowercase().contains("overflowxauto")
                && !b.class.contains("scroll-area")
        })
        .expect("header scrollport at 1440");
    assert!(
        header_1440.width >= 800.0,
        "desktop header must keep the wide leftover: {header_1440:?}"
    );

    let clipped = scan_file_kind(&root, landing, "clipped-text");
    assert!(
        clipped.viewport_errors.is_empty(),
        "landing layout errors: {:?}",
        clipped.viewport_errors
    );
    assert!(
        clipped.findings.is_empty(),
        "WI-LAYOUT-2-1 must be quiet: {:?}",
        clipped.findings
    );
}
