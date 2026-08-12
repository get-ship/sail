//! SPEC-LINTS-1 §4 Phase 1 — DiagSpec registry skeleton gate (S3.3).
//!
//! **Test-writer owns this file. Implementer MUST NOT modify these tests.**
//!
//! # Expected product API (`sail_compiler::diagnostics`)
//!
//! - [`DiagSpec`](sail_compiler::diagnostics::DiagSpec) — static row:
//!   `id`, `jurisdiction`, `default_severity`, `message_template`,
//!   `allowed_list_source`, `fix_text`, `docs_anchor`.
//! - [`Jurisdiction`](sail_compiler::diagnostics::Jurisdiction) —
//!   `CompilerError` | `Lint` (SPEC-LINTS-1 §1).
//! - [`registry`](sail_compiler::diagnostics::registry) → `&'static [DiagSpec]`
//! - [`lookup`](sail_compiler::diagnostics::lookup) → `Option<&'static DiagSpec>`
//!
//! # Gate contract (job-spec-spike-s3-3-registry)
//!
//! 1. DiagSpec type + `registry()` / `lookup(id)` API exists (compile + call).
//! 2. Known id'd rules (`sail-layout`, `sail-unintended-overflow`) lookup ok.
//! 3. At least one live emission path stamps a rule id present in the registry
//!    (behavioral: emitted `Diagnostic.rule` is `lookup`-able).
//! 4. Anonymous growth still blocked (S3.1 ratchet still green).
//!
//! # Out of scope (do not weaken gates into these)
//!
//! - Mass migration of anonymous sites
//! - Text golden of full message catalogs
//! - Completeness of every lint / compiler throw site

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::check::layout;
use sail_compiler::diag_ratchet::{anonymous_baseline, count_anonymous_sites};
use sail_compiler::diagnostics::{DiagSeverity, DiagSpec, Jurisdiction, lookup, registry};

/// Known product rule ids that Phase 1 must register as sample / proof rows.
const KNOWN_IDS: &[&str] = &["sail-layout", "sail-unintended-overflow"];

/// Child taller than fixed column — fires experimental layout under strict.
const OVERFLOW_SRC: &str = r#"
<column style({ box: h(20) })>
  <div style({ box: h(40).shrink(0) })>x</div>
</column>
"#;

fn write_fixture(config: &str, body: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("sail.config.json"), config).unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let sail = src_dir.join("page.sail");
    fs::write(&sail, body).unwrap();
    (tmp, sail)
}

fn layout_diags(src: &str, path: Option<&Path>) -> Vec<sail_compiler::Diagnostic> {
    match path {
        Some(p) => layout::collect_for(src, Some(p)),
        None => layout::collect(src),
    }
}

// ── 1. API surface ──────────────────────────────────────────────────────────

#[test]
fn diag_spec_type_exposes_phase1_fields() {
    // Structural proof: DiagSpec fields are the Phase 1 skeleton (constructible).
    // Implementer uses this shape for table rows; values here are fixture-only.
    let sample = DiagSpec {
        id: "sail-fixture-only",
        jurisdiction: Jurisdiction::Lint,
        default_severity: DiagSeverity::Warning,
        message_template: "fixture {0}",
        allowed_list_source: Some("fixture::vocab"),
        fix_text: Some("fix it"),
        docs_anchor: Some("docs#fixture"),
    };
    assert_eq!(sample.id, "sail-fixture-only");
    assert_eq!(sample.jurisdiction, Jurisdiction::Lint);
    assert_eq!(sample.default_severity, DiagSeverity::Warning);
    assert_eq!(sample.message_template, "fixture {0}");
    assert_eq!(sample.allowed_list_source, Some("fixture::vocab"));
    assert_eq!(sample.fix_text, Some("fix it"));
    assert_eq!(sample.docs_anchor, Some("docs#fixture"));

    let err_jurisdiction = DiagSpec {
        id: "sail-fixture-error",
        jurisdiction: Jurisdiction::CompilerError,
        default_severity: DiagSeverity::Error,
        message_template: "broken",
        allowed_list_source: None,
        fix_text: None,
        docs_anchor: None,
    };
    assert_eq!(err_jurisdiction.jurisdiction, Jurisdiction::CompilerError);
}

#[test]
fn registry_and_lookup_api_are_callable() {
    // API existence: registry() returns a slice; lookup is total over ids.
    let table = registry();
    // Phase 1 requires a non-empty skeleton (at least sample / known rows).
    assert!(
        !table.is_empty(),
        "DiagSpec registry() must be non-empty in Phase 1 \
         (register at least sail-layout + sail-unintended-overflow samples)"
    );
    // lookup of a nonsense id is None (stable total function).
    assert!(
        lookup("not-a-real-sail-rule-zzzz").is_none(),
        "lookup of unknown id must return None"
    );
}

// ── 2. Known id'd rules lookup ──────────────────────────────────────────────

#[test]
fn known_rules_lookup_ok() {
    for id in KNOWN_IDS {
        let spec = lookup(id).unwrap_or_else(|| {
            panic!(
                "lookup({id:?}) must succeed — Phase 1 sample/known rules must be \
                 registered in DiagSpec registry (got registry ids: {:?})",
                registry().iter().map(|s| s.id).collect::<Vec<_>>()
            )
        });
        assert_eq!(spec.id, *id, "DiagSpec.id must equal lookup key");
        assert!(
            !spec.message_template.is_empty(),
            "{id}: message_template must be non-empty (not a text-golden; just present)"
        );
        // Severity is always a real enum value if the type typechecks; pin
        // that sample rows are not silently Error-for-everything unless
        // jurisdiction says so — just ensure the field is readable.
        let _ = spec.default_severity;
        let _ = spec.jurisdiction;
        let _ = spec.allowed_list_source;
        let _ = spec.fix_text;
        let _ = spec.docs_anchor;
    }
}

#[test]
fn registry_contains_known_rule_ids() {
    let ids: Vec<&str> = registry().iter().map(|s| s.id).collect();
    for id in KNOWN_IDS {
        assert!(
            ids.contains(id),
            "registry() must include {id:?}; present={ids:?}"
        );
    }
}

// ── 3. Emission path tied to registry ───────────────────────────────────────

#[test]
fn at_least_one_emission_path_rule_is_registered() {
    // Behavioral (not text-golden): layout bridge under strict emits
    // sail-unintended-overflow (or another layout rule id). That emitted
    // rule id must be present in the DiagSpec registry — proof that the
    // product path and the table meet for at least one rule.
    let (_tmp, sail) = write_fixture(r#"{ "preset": "full", "strict": true }"#, OVERFLOW_SRC);
    let diags = layout_diags(OVERFLOW_SRC, Some(&sail));
    let emitted_rules: Vec<&str> = diags.iter().filter_map(|d| d.rule.as_deref()).collect();
    assert!(
        !emitted_rules.is_empty(),
        "expected layout emission with a rule id under strict (got diags={diags:?})"
    );

    let registered = emitted_rules
        .iter()
        .copied()
        .find(|id| lookup(id).is_some());
    assert!(
        registered.is_some(),
        "at least one emitted rule id must be in DiagSpec registry.\n\
         emitted={emitted_rules:?}\n\
         registry={:?}\n\
         Wire emission to a registered id (e.g. sail-unintended-overflow / sail-layout).",
        registry().iter().map(|s| s.id).collect::<Vec<_>>()
    );
}

// ── 4. Anonymous ratchet still holds ────────────────────────────────────────

#[test]
fn anonymous_diag_ratchet_still_holds() {
    // S3.1 Phase 0 gate must remain green while the registry skeleton lands.
    // Mass anonymous migration is out of scope; growth is still forbidden.
    let count = count_anonymous_sites();
    let baseline = anonymous_baseline();
    assert!(
        count <= baseline,
        "anonymous diagnostic sites grew while landing DiagSpec registry: \
         {count} > {baseline}. Attach sail-* ids to new sites; do not mass-migrate."
    );
}
