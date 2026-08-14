//! Real-corpus canary for experimental `clipped-text`.
//!
//! Findings are information. Green when the driver scans, is deterministic,
//! and writes JSON — not when the corpus is empty of clipped text.

use std::path::PathBuf;

use sail_compiler::check::layout_corpus::{self, TREE_NAMES, default_clipped_out_path};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn corpus_canary_clipped_text_writes_json_and_is_deterministic() {
    let root = workspace_root();
    let first = layout_corpus::scan_kind(&root, "clipped-text").expect("clipped-text scan 1");
    let second = layout_corpus::scan_kind(&root, "clipped-text").expect("clipped-text scan 2");
    assert_eq!(
        first.findings, second.findings,
        "two consecutive clipped-text scans must be identical"
    );

    assert!(
        first.counts.files_discovered >= 80,
        "must discover the real framework corpus: {:?}",
        first.counts
    );
    assert!(
        first.counts.files_scanned >= 80,
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
        assert_eq!(f.kind, "clipped-text");
        assert!(!f.path.is_empty());
    }

    let out = default_clipped_out_path(&root);
    layout_corpus::write_report(&out, &first).expect("write clipped-text canary json");
    assert!(out.is_file(), "missing {}", out.display());
}
