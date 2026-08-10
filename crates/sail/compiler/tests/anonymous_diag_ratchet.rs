//! SPEC-LINTS-1 Phase 0 — anonymous diagnostic ratchet gate.
//!
//! **Test-writer owns this file. Implementer MUST NOT modify these tests.**
//!
//! # Expected product API (`sail_compiler::diag_ratchet`)
//!
//! - [`is_stable_rule_id`](sail_compiler::diag_ratchet::is_stable_rule_id) —
//!   `true` iff the string matches `^sail-[a-z0-9-]+$`.
//! - [`classify_snippet`](sail_compiler::diag_ratchet::classify_snippet) /
//!   [`SiteClass`](sail_compiler::diag_ratchet::SiteClass) — classify a
//!   throw-site source window as id'd vs anonymous (fixture snippets, not
//!   message goldens).
//! - [`count_anonymous_sites`](sail_compiler::diag_ratchet::count_anonymous_sites) —
//!   live scan of `crates/sail/compiler/src/**/*.rs` for
//!   `CompileError::new(` / message-bearing `Diagnostic {` sites.
//! - [`anonymous_baseline`](sail_compiler::diag_ratchet::anonymous_baseline) —
//!   deliberate pin from a real tip remeasure (`<=` allows free debt drop).
//!
//! # How to fix a failure
//!
//! 1. **New anonymous site** — attach a stable id at the throw site:
//!    - `.with_category("sail-<area>-<symptom>")`, or
//!    - `rule: Some("sail-…".into())` / `rule: Some(RULE_ID)` with
//!      `const RULE_ID: &str = "sail-…"` in the same file.
//! 2. **Growth only** fails CI. Reducing anonymous debt is free under
//!    `count <= baseline`; no pin bump required for drops.
//! 3. **Baseline bump** only with operator approval after a real remeasure —
//!    never raise the pin just to silence growth.
//!
//! # Verify the ratchet bites (manual)
//!
//! 1. Temporarily insert `CompileError::new("ratchet-probe")` in `src/check/mod.rs`
//!    without a `sail-*` category.
//! 2. `cargo test -p sail-compiler --test anonymous_diag_ratchet`
//! 3. Expect `anonymous_count_does_not_grow` to fail.
//! 4. Revert the probe.

use sail_compiler::diag_ratchet::{
    SiteClass, anonymous_baseline, classify_snippet, count_anonymous_sites, is_stable_rule_id,
};

#[test]
fn stable_rule_id_accepts_sail_ids() {
    assert!(
        is_stable_rule_id("sail-foo"),
        "sail-foo must be a stable rule id"
    );
    assert!(
        is_stable_rule_id("sail-light-shadow-root"),
        "sail-light-shadow-root must be a stable rule id"
    );
}

#[test]
fn stable_rule_id_rejects_freeform() {
    for id in ["expression", "for-iterable", "on:click", "", "manifest"] {
        assert!(
            !is_stable_rule_id(id),
            "free-form category {id:?} must not count as a stable rule id"
        );
    }
}

#[test]
fn classifies_with_category_literal() {
    // Behavioral fixture: literal sail-* category on CompileError builder.
    let src = r#"
        CompileError::new("bad query key shape")
            .with_span(span)
            .with_category("sail-query-key")
    "#;
    assert_eq!(
        classify_snippet(src),
        SiteClass::Idd,
        "with_category(\"sail-query-key\") must classify as id'd"
    );
}

#[test]
fn classifies_missing_category_anonymous() {
    // Behavioral fixture: bare constructor, no category/rule attachment.
    let src = r#"CompileError::new("something went wrong")"#;
    assert_eq!(
        classify_snippet(src),
        SiteClass::Anonymous,
        "CompileError::new alone must classify as anonymous"
    );
}

#[test]
fn classifies_rule_some_literal() {
    // Behavioral fixture: Diagnostic with rule: Some("sail-…").
    let src = r#"
        out.push(Diagnostic {
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 1,
            severity: DiagSeverity::Error,
            message: "layout engine unavailable".into(),
            rule: Some("sail-layout".into()),
        });
    "#;
    assert_eq!(
        classify_snippet(src),
        SiteClass::Idd,
        "rule: Some(\"sail-layout\") must classify as id'd"
    );
}

#[test]
fn anonymous_count_does_not_grow() {
    let count = count_anonymous_sites();
    let baseline = anonymous_baseline();
    assert!(
        count <= baseline,
        "anonymous diagnostic sites grew: {count} > {baseline}\n\
         Attach a sail-* category/rule to new throw sites \
         (with_category(\"sail-…\") or rule: Some(\"sail-…\")).\n\
         Debt reduction (count drop) is free under <= baseline; \
         only growth fails. Bump baseline only with operator approval \
         after a real remeasure."
    );
}
