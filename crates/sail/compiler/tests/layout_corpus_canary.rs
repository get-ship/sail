//! Real-corpus canary for experimental `unintended-overflow`.
//!
//! Findings are information: this test stays green when overflow exists.
//! It fails only if the driver does not scan the corpus, drops geometry,
//! writes nothing, or is non-deterministic.

use std::path::PathBuf;

use sail_compiler::check::layout_corpus::{
    self, CorpusReport, Determinism, TREE_NAMES, default_out_path,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn corpus_canary_unintended_overflow_writes_json_and_is_deterministic() {
    let root = workspace_root();
    for rel in [
        "apps/sail-site/routes",
        "apps/sail-site/src",
        "packages/components",
    ] {
        assert!(
            root.join(rel).is_dir(),
            "corpus root missing: {} ({})",
            rel,
            root.display()
        );
    }

    let first = layout_corpus::scan(&root).expect("corpus scan 1");
    let second = layout_corpus::scan(&root).expect("corpus scan 2");
    assert_eq!(
        first.findings, second.findings,
        "two consecutive corpus scans must emit identical findings \
         (files/spans/kinds/geometry)"
    );

    assert!(
        first.counts.files_discovered >= 150,
        "must discover the real corpus, not a toy subset: {:?}",
        first.counts
    );
    assert!(
        first.counts.files_scanned >= 150,
        "must scan the real corpus (errors are recorded, not a reason to scan nothing): \
         {:?}",
        first.counts
    );
    for name in TREE_NAMES {
        let tree = first
            .trees
            .get(*name)
            .unwrap_or_else(|| panic!("missing tree {name}: {:?}", first.trees.keys()));
        assert!(
            tree.files_scanned > 0,
            "tree `{name}` must scan at least one file: {tree:?}"
        );
    }

    for f in &first.findings {
        assert_eq!(f.kind, "unintended-overflow");
        assert!(!f.path.is_empty());
        assert!(
            f.geometry.is_some(),
            "every overflow finding needs child/parent extents: {f:?}"
        );
        if f.span_mapped {
            assert!(
                f.span.end > f.span.start,
                "mapped span must cover an opening tag, not a point/EOF: {:?}",
                f.span
            );
            assert!(f.span.line >= 1, "mapped line is 1-based: {:?}", f.span);
        } else {
            assert_eq!(
                f.span.line, 0,
                "unmapped span must not invent an author line: {f:?}"
            );
            assert_eq!(f.span.start, 0);
            assert_eq!(f.span.end, 0);
        }
    }

    let mut report: CorpusReport = first;
    report.determinism = Some(Determinism { matched: true });

    let out = default_out_path(&root);
    layout_corpus::write_report(&out, &report).expect("write findings JSON");
    assert!(
        out.is_file(),
        "corpus canary must persist machine-readable findings: {}",
        out.display()
    );

    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).expect("read findings"))
            .expect("findings JSON");
    assert_eq!(
        written["stdlib_expand"].as_bool(),
        Some(report.stdlib_expand),
        "written stdlib_expand must match the scan"
    );
    if !report.stdlib_expand {
        let err = written["stdlib_error"].as_str().unwrap_or("");
        assert!(
            !err.is_empty(),
            "stdlib_expand=false must record stdlib_error: {written}"
        );
    }
}
