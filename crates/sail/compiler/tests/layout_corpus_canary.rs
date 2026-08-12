//! Real-corpus canary for experimental `unintended-overflow`.
//!
//! Findings are information: this test stays green when overflow exists.
//! It fails only if the driver does not scan the corpus, drops geometry,
//! writes nothing, or is non-deterministic.

use std::path::PathBuf;

use sail_compiler::check::layout_corpus::{
    self, CorpusReport, Determinism, canonical_findings_json, default_out_path, sha256_hex,
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
    assert!(
        root.join("apps/sail-site/routes").is_dir(),
        "workspace root must contain sail-site routes: {}",
        root.display()
    );

    let first = layout_corpus::scan(&root).expect("corpus scan 1");
    let second = layout_corpus::scan(&root).expect("corpus scan 2");

    let canon1 = canonical_findings_json(&first.findings);
    let canon2 = canonical_findings_json(&second.findings);
    let hash1 = sha256_hex(canon1.as_bytes());
    let hash2 = sha256_hex(canon2.as_bytes());
    assert_eq!(
        hash1, hash2,
        "two consecutive corpus scans must emit identical findings \
         (files/spans/kinds/geometry): run1={hash1} run2={hash2}"
    );
    assert_eq!(first.findings, second.findings);

    assert!(
        first.counts.files_discovered >= 200,
        "must discover the real corpus, not a toy subset: {:?}",
        first.counts
    );
    assert!(
        first.counts.files_scanned >= 150,
        "must scan the real corpus (errors are recorded, not a reason to scan nothing): \
         {:?}",
        first.counts
    );

    for f in &first.findings {
        assert_eq!(f.kind, "unintended-overflow");
        assert!(!f.path.is_empty());
        assert!(
            f.geometry.is_some(),
            "every overflow finding needs child/parent extents: {f:?}"
        );
        assert!(
            f.span.end >= f.span.start,
            "span must be ordered: {:?}",
            f.span
        );
    }

    let mut report: CorpusReport = first;
    report.determinism = Some(Determinism {
        run1_sha256: hash1,
        run2_sha256: hash2,
        matched: true,
    });

    let out = default_out_path(&root);
    layout_corpus::write_report(&out, &report).expect("write findings JSON");
    assert!(
        out.is_file(),
        "corpus canary must persist machine-readable findings: {}",
        out.display()
    );
}
