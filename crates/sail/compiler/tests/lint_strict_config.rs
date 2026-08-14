//! SPEC-LINTS-1 §3 — lint severity floor + top-level `strict: true` (S3.2).
//!
//! Operator surface locked here (implementer must land parsing + gate application;
//! **do not** weaken these assertions to green-wash a partial implementation):
//!
//! ```jsonc
//! "lint": { "severity": "error",                 // global floor: off|warn|error
//!           "rules": { "sail-no-free-aria": "off" } }
//! "strict": true                                // every lint finding → Error at gate
//! ```
//!
//! Behavioral contract (job-spec-spike-s3-2-strict):
//! 1. Default (no strict / no floor): a known natural-Warn lint stays non-blocking.
//! 2. `strict: true` (or `lint.severity: "error"` floor): that finding becomes Error
//!    and fails `compile` / surfaces as Error in `check_all`.
//! 3. Per-rule `off` remains a kill switch under strict (silence still works).
//! 4. Compiler errors are never downgradable via lint config.
//! 5. Unknown top-level keys stay loud (config-loud-strict).
//!
//! Fixture lint: `sail-no-free-aria` — natural `Warn`, not a floor/capability rule,
//! so capability preset `full` does not mask the severity floor under test.

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::check::policy::{Policy, ResolutionKind};
use sail_compiler::policy::{LintSeverityValue, apply_config_to_resolved};
use sail_compiler::{
    CompileMode, CompileOpts, DiagSeverity, LintOverride, OverrideSeverity, check_all, compile,
};
use sail_lint::config::{RECOGNIZED_KEYS, SailConfig};

/// Natural-Warn lint used as the promotion probe (SPEC path-gated apps).
const WARN_RULE: &str = "sail-no-free-aria";

/// Source that fires `sail-no-free-aria` and nothing else hard.
const WARN_SRC: &str = r#"<button type="button" aria-label="Go" style({ size({ min-h: control-h-md, min-w: control-h-md }) })>Go</button>"#;

/// Source that is a pure compiler error (parse) — never a severity-configurable lint.
const PARSE_ERR_SRC: &str = r#"<button type="button">unterminated"#;

fn write_fixture(config: &str, sail_body: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    // Ambient app path: `…/src/*.sail` so path-gated lints (no-free-aria) fire.
    fs::write(tmp.path().join("sail.config.json"), config).unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let sail = src_dir.join("w.sail");
    fs::write(&sail, sail_body).unwrap();
    (tmp, sail)
}

fn opts_for(path: &Path) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Auto,
        source_path: Some(path.to_path_buf()),
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Policy::overrides_for(path),
    }
}

fn diags_for(path: &Path, src: &str) -> Vec<sail_compiler::Diagnostic> {
    let overrides = Policy::overrides_for(path);
    check_all(src, Some(path), &overrides, None)
}

fn warn_rule_diags(path: &Path, src: &str) -> Vec<sail_compiler::Diagnostic> {
    diags_for(path, src)
        .into_iter()
        .filter(|d| d.rule.as_deref() == Some(WARN_RULE))
        .collect()
}

// ── 1. Config recognition (SPEC keys) ───────────────────────────────────────

#[test]
fn top_level_strict_is_a_recognized_config_key() {
    // SPEC-LINTS-1: `"strict": true` is an operator key, not a typo.
    assert!(
        RECOGNIZED_KEYS.contains(&"strict"),
        "RECOGNIZED_KEYS must include `strict` (got {RECOGNIZED_KEYS:?})"
    );
    let cfg = SailConfig::from_json(r#"{ "strict": true }"#);
    assert!(cfg.parse_ok());
    let unknown = cfg.unknown_keys();
    assert!(
        !unknown.iter().any(|k| k == "strict"),
        "`strict` must not be unknown: {unknown:?}"
    );
}

#[test]
fn top_level_lint_object_is_a_recognized_config_key() {
    // SPEC-LINTS-1: top-level `"lint": { severity, rules }` — not only nested
    // `policy.lint.severity.<rule>`.
    assert!(
        RECOGNIZED_KEYS.contains(&"lint"),
        "RECOGNIZED_KEYS must include `lint` (got {RECOGNIZED_KEYS:?})"
    );
    let cfg = SailConfig::from_json(r#"{ "lint": { "severity": "error" } }"#);
    assert!(cfg.parse_ok());
    let unknown = cfg.unknown_keys();
    assert!(
        !unknown.iter().any(|k| k == "lint"),
        "`lint` must not be unknown: {unknown:?}"
    );
}

#[test]
fn strict_and_lint_config_do_not_fail_closed_as_unknown_keys() {
    // Unlike a typo (`polcy`), recognized SPEC keys must resolve cleanly —
    // not Rejected(unknown key).
    let (_tmp, sail) = write_fixture(
        r#"{ "preset": "full", "strict": true, "lint": { "severity": "error" } }"#,
        WARN_SRC,
    );
    let (_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert!(
        !matches!(resolution.kind, ResolutionKind::Rejected(ref r) if r.contains("strict") || r.contains("\"lint\"") || r.contains("unrecognized key")),
        "strict/lint must not be rejected as unknown keys: {:?}",
        resolution.kind
    );
}

// ── 2. Product config/policy APIs (parsing surface) ─────────────────────────
//
// Prefer missing API = compile-fail red. Implementer lands:
//   SailConfig::strict() -> bool
//   SailConfig::lint_severity_floor() -> Option<&str>  // "error"|"warn"|"off"
//   ResolvedPolicy { strict, lint_severity_floor, lint_severities, … }
//     via apply_config_to_resolved (merges top-level lint.rules into the tree)

#[test]
fn sail_config_strict_accessor_reads_top_level_flag() {
    let on = SailConfig::from_json(r#"{ "strict": true }"#);
    assert!(
        on.strict(),
        "strict: true must parse via SailConfig::strict()"
    );
    let off = SailConfig::from_json(r#"{ "preset": "full" }"#);
    assert!(!off.strict(), "absent strict defaults false");
    let explicit_false = SailConfig::from_json(r#"{ "strict": false }"#);
    assert!(!explicit_false.strict());
}

#[test]
fn sail_config_lint_severity_floor_reads_global_floor() {
    // `"lint": { "severity": "error"|"warn"|"off" }` — global floor token (string).
    // Lives on SailConfig (lint crate) — no dependency on compiler enums.
    let err = SailConfig::from_json(r#"{ "lint": { "severity": "error" } }"#);
    assert_eq!(
        err.lint_severity_floor(),
        Some("error"),
        "lint.severity: error → \"error\" floor token"
    );
    let warn = SailConfig::from_json(r#"{ "lint": { "severity": "warn" } }"#);
    assert_eq!(warn.lint_severity_floor(), Some("warn"));
    let off = SailConfig::from_json(r#"{ "lint": { "severity": "off" } }"#);
    assert_eq!(off.lint_severity_floor(), Some("off"));
    let absent = SailConfig::from_json(r#"{ "preset": "full" }"#);
    assert_eq!(absent.lint_severity_floor(), None);
}

#[test]
fn apply_config_resolves_strict_and_floor_and_lint_rules() {
    let cfg = SailConfig::from_json(
        r#"{
            "preset": "full",
            "strict": true,
            "lint": {
                "severity": "error",
                "rules": { "sail-no-free-aria": "off" }
            }
        }"#,
    );
    let resolved = apply_config_to_resolved(&cfg).expect("resolve");
    assert!(
        resolved.strict,
        "ResolvedPolicy.strict must reflect top-level strict: true"
    );
    assert_eq!(
        resolved.lint_severity_floor,
        Some(LintSeverityValue::Error),
        "global lint.severity floor must land on ResolvedPolicy"
    );
    assert_eq!(
        resolved.lint_severities.get(WARN_RULE),
        Some(&LintSeverityValue::Off),
        "lint.rules.* must merge into the severity tree (dual-read with policy.lint.severity.*)"
    );
}

// ── 3. Default: natural Warn stays non-blocking ─────────────────────────────

#[test]
fn default_without_strict_keeps_warn_lint_non_blocking() {
    // Document fixture: full preset (capabilities granted) + no strict/floor.
    // sail-no-free-aria remains natural Warn → compile succeeds.
    let (_tmp, sail) = write_fixture(r#"{ "preset": "full" }"#, WARN_SRC);
    let diags = warn_rule_diags(&sail, WARN_SRC);
    assert_eq!(
        diags.len(),
        1,
        "sanity: {WARN_RULE} must fire on fixture: {diags:?}"
    );
    assert_eq!(
        diags[0].severity,
        DiagSeverity::Warning,
        "natural severity is Warning without strict/floor: {diags:?}"
    );
    compile(&fs::read_to_string(&sail).unwrap(), opts_for(&sail))
        .expect("without strict/floor, natural-Warn lint must not fail the build gate");
}

// ── 4. strict: true promotes Warn → Error at the gate ───────────────────────

#[test]
fn strict_true_promotes_warn_lint_to_error_at_gate() {
    let (_tmp, sail) = write_fixture(r#"{ "preset": "full", "strict": true }"#, WARN_SRC);
    let diags = warn_rule_diags(&sail, WARN_SRC);
    assert_eq!(
        diags.len(),
        1,
        "{WARN_RULE} must still fire under strict: {diags:?}"
    );
    assert_eq!(
        diags[0].severity,
        DiagSeverity::Error,
        "strict: true promotes every lint finding to Error: {diags:?}"
    );
    let err = compile(&fs::read_to_string(&sail).unwrap(), opts_for(&sail))
        .expect_err("strict: true must fail the build gate on a promoted lint");
    assert_eq!(
        err.category.as_deref(),
        Some(WARN_RULE),
        "gate failure must name the promoted lint rule: {err:?}"
    );
}

#[test]
fn lint_severity_error_floor_promotes_warn_lint_to_error() {
    // Global floor alone (no strict flag) is enough when set to error.
    let (_tmp, sail) = write_fixture(
        r#"{ "preset": "full", "lint": { "severity": "error" } }"#,
        WARN_SRC,
    );
    let diags = warn_rule_diags(&sail, WARN_SRC);
    assert_eq!(diags.len(), 1, "{WARN_RULE} must fire: {diags:?}");
    assert_eq!(
        diags[0].severity,
        DiagSeverity::Error,
        "lint.severity: error is a global floor: {diags:?}"
    );
    compile(&fs::read_to_string(&sail).unwrap(), opts_for(&sail))
        .expect_err("lint severity floor=error must fail the gate");
}

// ── 5. Per-rule off still silences under strict (kill switch) ────────────────

#[test]
fn per_rule_off_still_silences_under_strict() {
    // SPEC-faithful kill switch: strict promotes warn→error, but explicit
    // `off` still drops the finding (operator escape hatch).
    let (_tmp, sail) = write_fixture(
        r#"{
            "preset": "full",
            "strict": true,
            "lint": {
                "rules": { "sail-no-free-aria": "off" }
            }
        }"#,
        WARN_SRC,
    );
    let diags = warn_rule_diags(&sail, WARN_SRC);
    assert!(
        diags.is_empty(),
        "per-rule off under strict must silence {WARN_RULE}: {diags:?}"
    );
    compile(&fs::read_to_string(&sail).unwrap(), opts_for(&sail)).expect(
        "kill switch: lint.rules.<id>=off under strict must not fail the gate for that rule",
    );
}

#[test]
fn policy_lint_severity_off_still_silences_under_strict() {
    // Dual-read: existing `policy.lint.severity.*` remains a valid kill switch
    // under top-level strict (no forced cutover break).
    let (_tmp, sail) = write_fixture(
        r#"{
            "preset": "full",
            "strict": true,
            "policy": {
                "lint": { "severity": { "sail-no-free-aria": "off" } }
            }
        }"#,
        WARN_SRC,
    );
    let diags = warn_rule_diags(&sail, WARN_SRC);
    assert!(
        diags.is_empty(),
        "policy.lint.severity.* off under strict must silence: {diags:?}"
    );
    compile(&fs::read_to_string(&sail).unwrap(), opts_for(&sail))
        .expect("dual-read kill switch via policy.lint.severity must work under strict");
}

// ── 6. Compiler errors never downgraded ─────────────────────────────────────

#[test]
fn compiler_errors_never_downgraded_by_lint_config() {
    // Parse / language errors are non-configurable (jurisdiction law).
    // lint severity off / strict:false must not open a downgrade path.
    let (_tmp, sail) = write_fixture(
        r#"{
            "preset": "full",
            "strict": false,
            "lint": { "severity": "off" }
        }"#,
        PARSE_ERR_SRC,
    );
    let err = compile(&fs::read_to_string(&sail).unwrap(), opts_for(&sail))
        .expect_err("compiler/parse errors must still hard-fail under lint.severity: off");
    // Not a lint rule id — any hard failure is enough; category may be free-form.
    assert!(
        err.category.as_deref() != Some(WARN_RULE),
        "must be a compiler/parse failure, not a lint: {err:?}"
    );

    // Explicit Off override on a non-lint category (if ever named) cannot
    // silence a pure parse failure either — compile still errs with empty overrides.
    let bare = CompileOpts {
        source_path: Some(sail.clone()),
        lint_overrides: vec![LintOverride {
            rule: "expression".into(),
            severity: OverrideSeverity::Off,
        }],
        ..CompileOpts::default()
    };
    compile(PARSE_ERR_SRC, bare)
        .expect_err("lint_overrides cannot downgrade a compiler parse error");
}

// ── 7. Unknown keys stay loud ───────────────────────────────────────────────

#[test]
fn unknown_config_key_still_refused_loudly() {
    // config-loud-strict culture survives the new keys: typos still Reject.
    let (_tmp, sail) = write_fixture(r#"{ "polcy": "strict" }"#, "<p>hi</p>");
    let (config_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert_eq!(resolution.tier, Policy::STRICT);
    let ResolutionKind::Rejected(reason) = &resolution.kind else {
        panic!("expected Rejected for typo key, got {:?}", resolution.kind);
    };
    assert!(
        reason.contains("polcy"),
        "must name the offending key: {reason}"
    );
    assert!(
        reason.contains("allowed keys:"),
        "must name allowed keys: {reason}"
    );
    assert!(
        config_path.ends_with("sail.config.json"),
        "must name the file: {config_path:?}"
    );
}

#[test]
fn unknown_nested_lint_rule_shape_does_not_silently_accept_garbage_top_level() {
    // A made-up top-level key beside valid lint/strict still fails closed loud.
    let cfg = SailConfig::from_json(
        r#"{ "strict": true, "lint": { "severity": "error" }, "lintt": true }"#,
    );
    assert!(cfg.parse_ok());
    let unknown = cfg.unknown_keys();
    assert!(
        unknown.iter().any(|k| k == "lintt"),
        "typo lintt must remain unknown even when strict/lint are valid: {unknown:?}"
    );
}
