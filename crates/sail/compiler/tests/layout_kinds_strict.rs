//! SPEC-LAYOUT-DIAG-1 / S4.1 — compiler bridge: experimental layout kinds + strict gate.
//!
//! Product contract:
//! - Layout kinds `unintended-overflow` / `clipped-text` map to stable rule ids
//!   `sail-unintended-overflow` / `sail-clipped-text` (register for diagnostic ratchet).
//! - Enabled only under top-level `strict: true` **or** explicit layout experimental
//!   config (align with S3.2 rollout: experimental → canary → promote).
//! - Strict-off / experimental-off → kinds **not emitted** (not merely soft).
//!
//! Config surface locked here (implementer lands recognition + bridge wiring):
//! ```jsonc
//! "strict": true
//! // or without full lint strict:
//! "layoutExperimental": true
//! "layoutTextMetrics": true   // optional; clipped-text path
//! ```
//!
//! Geometry fixtures use preprocessed-equivalent product Style classes that
//! taffy already solves (see sail-layout layout_kinds tests).

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::check::layout;
use sail_compiler::diagnostics::DiagSeverity;
use sail_lint::config::{SailConfig, RECOGNIZED_KEYS};

const RULE_UNINTENDED_OVERFLOW: &str = "sail-unintended-overflow";
const RULE_CLIPPED_TEXT: &str = "sail-clipped-text";

/// Child taller than fixed column, no overflow opt-in.
const OVERFLOW_SRC: &str = r#"
<column style({ box: h(20) })>
  <div style({ box: h(40).shrink(0) })>x</div>
</column>
"#;

/// Overflow opted in via overflowY(auto).
const OVERFLOW_OPTIN_SRC: &str = r#"
<column style({ box: h(20).overflowY(auto) })>
  <div style({ box: h(40).shrink(0) })>x</div>
</column>
"#;

/// Narrow + long text without truncate — clipped-text probe.
const CLIPPED_TEXT_SRC: &str = r#"
<p style({ box: w(16).overflowX(hidden) })>Supercalifragilisticexpialidocious</p>
"#;

/// Truncate opt-in — must not fire clipped-text.
const TRUNCATE_SRC: &str = r#"
<p style({ box: w(16), text: truncate() })>Supercalifragilisticexpialidocious</p>
"#;

fn write_pack(config: &str, body: &str) -> (tempfile::TempDir, PathBuf) {
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

fn has_rule(diags: &[sail_compiler::Diagnostic], rule: &str) -> bool {
    diags.iter().any(|d| d.rule.as_deref() == Some(rule))
}

// ── Config recognition (experimental layout keys) ───────────────────────────

#[test]
fn layout_experimental_keys_are_recognized_config_keys() {
    // Explicit experimental gate without forcing full lint `strict`.
    for key in ["layoutExperimental", "layoutTextMetrics"] {
        assert!(
            RECOGNIZED_KEYS.contains(&key),
            "RECOGNIZED_KEYS must include `{key}` for S4.1 experimental layout kinds \
             (got {RECOGNIZED_KEYS:?})"
        );
        let cfg = SailConfig::from_json(&format!(r#"{{ "{key}": true }}"#));
        assert!(cfg.parse_ok(), "{key} config must parse");
        assert!(
            !cfg.unknown_keys().iter().any(|k| k == key),
            "`{key}` must not be unknown: {:?}",
            cfg.unknown_keys()
        );
    }
}

#[test]
fn sail_config_layout_experimental_accessors() {
    // Prefer missing API = compile-fail red. Implementer lands:
    //   SailConfig::layout_experimental() -> bool
    //   SailConfig::layout_text_metrics() -> bool
    let on = SailConfig::from_json(
        r#"{ "layoutExperimental": true, "layoutTextMetrics": true }"#,
    );
    assert!(
        on.layout_experimental(),
        "layoutExperimental: true via SailConfig::layout_experimental()"
    );
    assert!(
        on.layout_text_metrics(),
        "layoutTextMetrics: true via SailConfig::layout_text_metrics()"
    );

    let off = SailConfig::from_json(r#"{ "preset": "full" }"#);
    assert!(!off.layout_experimental());
    assert!(!off.layout_text_metrics());
}

// ── 5. Strict/experimental off → not emitted ────────────────────────────────

#[test]
fn default_collect_does_not_emit_experimental_layout_rules() {
    let diags = layout_diags(OVERFLOW_SRC, None);
    assert!(
        !has_rule(&diags, RULE_UNINTENDED_OVERFLOW),
        "default (no strict / no layoutExperimental) must not emit \
         `{RULE_UNINTENDED_OVERFLOW}`: {diags:?}"
    );
    let diags = layout_diags(CLIPPED_TEXT_SRC, None);
    assert!(
        !has_rule(&diags, RULE_CLIPPED_TEXT),
        "default must not emit `{RULE_CLIPPED_TEXT}`: {diags:?}"
    );
}

#[test]
fn package_without_strict_does_not_emit_experimental_layout_rules() {
    let (_tmp, sail) = write_pack(r#"{ "preset": "full" }"#, OVERFLOW_SRC);
    let diags = layout_diags(OVERFLOW_SRC, Some(&sail));
    assert!(
        !has_rule(&diags, RULE_UNINTENDED_OVERFLOW),
        "strict-off package must not emit `{RULE_UNINTENDED_OVERFLOW}`: {diags:?}"
    );
}

// ── 1. Overflow without opt-in under strict → finding + rule id ─────────────

#[test]
fn strict_true_emits_unintended_overflow_with_stable_rule_id() {
    let (_tmp, sail) = write_pack(r#"{ "preset": "full", "strict": true }"#, OVERFLOW_SRC);
    let diags = layout_diags(OVERFLOW_SRC, Some(&sail));
    let hit = diags
        .iter()
        .find(|d| d.rule.as_deref() == Some(RULE_UNINTENDED_OVERFLOW))
        .unwrap_or_else(|| {
            panic!(
                "strict: true must enable `{RULE_UNINTENDED_OVERFLOW}` on \
                 overflow-without-opt-in: {diags:?}"
            )
        });
    // Under strict, gate is Error (S3.2: every finding → Error at gate).
    // Experimental kinds may land as Warning first; strict promotes them.
    assert_eq!(
        hit.severity,
        DiagSeverity::Error,
        "under strict, `{RULE_UNINTENDED_OVERFLOW}` must be Error at the gate: {hit:?}"
    );
    assert!(
        hit.line >= 1,
        "must map to a source span/line: {hit:?}"
    );
}

#[test]
fn layout_experimental_config_emits_unintended_overflow_without_full_strict() {
    let (_tmp, sail) = write_pack(
        r#"{ "preset": "full", "layoutExperimental": true }"#,
        OVERFLOW_SRC,
    );
    let diags = layout_diags(OVERFLOW_SRC, Some(&sail));
    assert!(
        has_rule(&diags, RULE_UNINTENDED_OVERFLOW),
        "layoutExperimental: true must enable `{RULE_UNINTENDED_OVERFLOW}` \
         without top-level strict: {diags:?}"
    );
}

// ── 2. Opt-in → no unintended-overflow rule ─────────────────────────────────

#[test]
fn strict_does_not_fire_unintended_overflow_when_overflow_y_auto() {
    let (_tmp, sail) = write_pack(r#"{ "preset": "full", "strict": true }"#, OVERFLOW_OPTIN_SRC);
    let diags = layout_diags(OVERFLOW_OPTIN_SRC, Some(&sail));
    assert!(
        !has_rule(&diags, RULE_UNINTENDED_OVERFLOW),
        "overflowY(auto) opt-in must suppress `{RULE_UNINTENDED_OVERFLOW}`: {diags:?}"
    );
}

// ── 3–4. Clipped text under experimental text metrics ───────────────────────

#[test]
fn strict_emits_clipped_text_without_truncate() {
    let (_tmp, sail) = write_pack(
        r#"{ "preset": "full", "strict": true, "layoutTextMetrics": true }"#,
        CLIPPED_TEXT_SRC,
    );
    let diags = layout_diags(CLIPPED_TEXT_SRC, Some(&sail));
    assert!(
        has_rule(&diags, RULE_CLIPPED_TEXT),
        "strict + text metrics must emit `{RULE_CLIPPED_TEXT}` without truncate: {diags:?}"
    );
}

#[test]
fn layout_experimental_and_text_metrics_emit_clipped_text() {
    let (_tmp, sail) = write_pack(
        r#"{
            "preset": "full",
            "layoutExperimental": true,
            "layoutTextMetrics": true
        }"#,
        CLIPPED_TEXT_SRC,
    );
    let diags = layout_diags(CLIPPED_TEXT_SRC, Some(&sail));
    assert!(
        has_rule(&diags, RULE_CLIPPED_TEXT),
        "layoutExperimental + layoutTextMetrics must emit `{RULE_CLIPPED_TEXT}`: {diags:?}"
    );
}

#[test]
fn truncate_suppresses_clipped_text_under_strict() {
    let (_tmp, sail) = write_pack(
        r#"{ "preset": "full", "strict": true, "layoutTextMetrics": true }"#,
        TRUNCATE_SRC,
    );
    let diags = layout_diags(TRUNCATE_SRC, Some(&sail));
    assert!(
        !has_rule(&diags, RULE_CLIPPED_TEXT),
        "truncate() opt-in must suppress `{RULE_CLIPPED_TEXT}`: {diags:?}"
    );
}

// ── Rule id ratchet registration (implementer) ──────────────────────────────

#[test]
fn experimental_layout_rule_ids_are_registered_for_ratchet() {
    // IDs are forever (diagnostic_id_drift pin). Adding the kinds requires
    // registering these in sail_lint::all_rule_ids + PINNED_RULE_IDS.
    let live = sail_lint::all_rule_ids();
    for id in [RULE_UNINTENDED_OVERFLOW, RULE_CLIPPED_TEXT] {
        assert!(
            live.iter().any(|r| r == &id),
            "`{id}` must be registered in sail_lint::all_rule_ids() for the \
             diagnostic ratchet (got {live:?})"
        );
    }
}
