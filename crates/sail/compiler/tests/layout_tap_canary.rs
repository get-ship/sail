//! Real-corpus canary for experimental `sub-min-tap-target`.
//!
//! Findings are information. Green when the driver scans, is deterministic,
//! and writes JSON — not when the corpus is empty of sub-min tap targets.

use std::path::PathBuf;

use sail_compiler::check::layout;
use sail_compiler::check::layout_corpus::{self, TREE_NAMES, default_tap_out_path};
use sail_compiler::parse::preprocess::preprocess_with_style;
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
        first.counts.files_discovered >= 150,
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

fn scan_file_kind(path: &std::path::Path, kind: &'static str) -> canary::PreprocessedScan {
    let label = path.display();
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{label}: {e}"));
    let (pre, hosts) =
        preprocess_with_style(&src).unwrap_or_else(|e| panic!("preprocess {label}: {e}"));
    let viewports = layout::layout_viewports(&pre);
    let mut scan = canary::PreprocessedScan::default();
    for &viewport in &viewports {
        match sail_layout::check_preprocessed_with_hosts(
            &pre,
            viewport,
            CheckOptions::experimental(),
            &hosts.types,
            &hosts.boxes,
        ) {
            Ok(result) => {
                scan.viewports_ok += 1;
                for d in result.diagnostics {
                    if d.kind != kind {
                        continue;
                    }
                    scan.findings.push(canary::KindFinding {
                        kind: d.kind.to_string(),
                        span: d.span,
                        viewport: [viewport.0.round() as u32, viewport.1.round() as u32],
                        element_tag: d.element_tag,
                        element_class: d.element_class,
                        element_index: d.element_index,
                        geometry: d.geometry,
                    });
                }
            }
            Err(err) => scan.viewport_errors.push(format!(
                "{}x{}: {err}",
                viewport.0.round() as u32,
                viewport.1.round() as u32
            )),
        }
    }
    scan
}

/// Owned quiet-clip page: short wrapping copy plus truncate opt-in.
/// Must stay silent for `clipped-text` at every collect viewport.
const QUIET_CLIP_SRC: &str = r#"
<column style({ pad(4).size({ w: full, min-w: 0 }) })>
  <h1>Quiet clip</h1>
  <p>Short copy that fits every viewport.</p>
  <p style({ size({ w: 16, min-w: 0 }).type({ wrap: truncate }) })>Supercalifragilisticexpialidocious</p>
</column>
"#;

#[test]
fn wi_layout_4_product_files_quiet_for_overflow_and_clipped() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("quiet-clip.sail");
    std::fs::write(&path, QUIET_CLIP_SRC).expect("write quiet-clip.sail");

    let clipped = scan_file_kind(&path, "clipped-text");
    assert!(
        clipped.viewport_errors.is_empty(),
        "fixture layout errors: {:?}",
        clipped.viewport_errors
    );
    assert!(
        clipped.findings.is_empty(),
        "WI-LAYOUT-2-1 must be quiet: {:?}",
        clipped.findings
    );
}
