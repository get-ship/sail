//! The floor is a LANGUAGE PROPERTY enforced by the compiler — not opt-in lint
//! wiring a host can skip.
//!
//! These tests pin the four halves of the T2 ruling:
//!   1. Authored `<style>` / inline `style=` / `class=` REFUSE with
//!      `lint_overrides: Vec::new()` — the exact wasm/playground host shape.
//!   2. No `lint_override` (Off or Warn) can turn a floor rule off.
//!   3. A broken/unknown-key `sui.config.json` fails CLOSED (STRICT), not FULL.
//!      (Unit-tested next to the resolver in `check::policy`; the whole-config
//!      path is exercised here via the undeclared-stylesheet floor rule.)
//!   4. The config key the docs advertise matches the key the parser reads
//!      (`preset`), verified by a source scan.
//!
//! The floor bites in BOTH the build gate (`compile`) and the LSP/agent surface
//! (`check_all`) because both flow through the single `check::collect` chokepoint
//! where `enforce_floor` runs unconditionally.

use std::fs;
use std::path::{Path, PathBuf};

use sui_compiler::{
    check_all, compile, CompileOpts, DiagSeverity, LintOverride, OverrideSeverity,
};

/// The empty-override host shape (what `wasm.rs` and a bare napi call pass).
fn empty_host() -> CompileOpts {
    CompileOpts::default() // lint_overrides: Vec::new()
}

/// True iff `check_all` (with the given overrides) reports `rule` as an Error.
fn reports_floor_error(src: &str, rule: &str, overrides: &[LintOverride]) -> bool {
    check_all(src, None, overrides, None).iter().any(|d| {
        d.severity == DiagSeverity::Error && d.rule.as_deref() == Some(rule)
    })
}

// ---------------------------------------------------------------------------
// 1. The floor bites with ZERO host wiring (`lint_overrides: Vec::new()`).
// ---------------------------------------------------------------------------

#[test]
fn style_block_refuses_with_empty_overrides() {
    let src = "<p>hi</p><style>x</style>";
    // The build gate refuses.
    assert!(
        compile(src, empty_host()).is_err(),
        "authored <style> must refuse the build with no host overrides"
    );
    // The analysis surface (LSP/agent) flags it as an Error too.
    assert!(
        reports_floor_error(src, "sui-no-style-block", &[]),
        "sui-no-style-block must be an Error with empty overrides"
    );
}

#[test]
fn inline_style_refuses_with_empty_overrides() {
    let src = r#"<p style="color:red">hi</p>"#;
    assert!(
        compile(src, empty_host()).is_err(),
        "inline style= must refuse the build with no host overrides"
    );
    assert!(
        reports_floor_error(src, "sui-no-inline-style", &[]),
        "sui-no-inline-style must be an Error with empty overrides"
    );
}

#[test]
fn class_refuses_with_empty_overrides() {
    let src = r#"<p class="x">hi</p>"#;
    assert!(
        compile(src, empty_host()).is_err(),
        "class= must refuse the build with no host overrides"
    );
    assert!(
        reports_floor_error(src, "sui-no-class", &[]),
        "sui-no-class must be an Error with empty overrides"
    );
}

/// The natural-`Warn` floor rules (`sui-no-style-block`, `sui-no-inline-style`)
/// are the ones the old opt-in floor let slip on the wasm host: their Warn never
/// reached the Error gate unless a policy promoted them. Pin that they now gate.
#[test]
fn natural_warn_floor_rules_gate_without_promotion() {
    assert!(compile("<p>hi</p><style>x</style>", empty_host()).is_err());
    assert!(compile(r#"<p style="x">y</p>"#, empty_host()).is_err());
}

// ---------------------------------------------------------------------------
// 2. No `lint_override` can turn a floor rule off (Off or Warn are ignored).
// ---------------------------------------------------------------------------

/// Every (floor rule, offending source) pair, for the template-only rules that
/// don't need an on-disk pack. `sui-no-undeclared-stylesheet` is covered
/// separately (it needs a `sui.config.json` + a companion `.css`).
const FLOOR_CASES: &[(&str, &str)] = &[
    ("sui-no-style-block", "<p>hi</p><style>x</style>"),
    ("sui-no-inline-style", r#"<p style="color:red">hi</p>"#),
    ("sui-no-class", r#"<p class="x">hi</p>"#),
];

#[test]
fn no_override_can_soften_a_floor_rule() {
    for (rule, src) in FLOOR_CASES {
        for sev in [OverrideSeverity::Off, OverrideSeverity::Warn] {
            let opts = CompileOpts::default().with_lint_override(*rule, sev);
            assert!(
                compile(src, opts).is_err(),
                "override {sev:?} on floor rule {rule} must NOT let `{src}` compile"
            );
            // ...and the analysis surface still reports it as an Error.
            let overrides = [LintOverride { rule: rule.to_string(), severity: sev }];
            assert!(
                reports_floor_error(src, rule, &overrides),
                "override {sev:?} on floor rule {rule} must not drop/downgrade its Error"
            );
        }
    }
}

/// A NON-floor rule stays softenable — the override channel still works for
/// everything above the floor. (Guards against over-broadening the floor guard.)
#[test]
fn non_floor_rules_remain_softenable() {
    // `<badge size="huge">` trips the natural-Error `sui-prop-value` rule (the
    // same source the gate tests use). It is NOT a floor rule, so an `Off`
    // override must drop it — proving the override channel is still live for
    // everything above the floor.
    let token_src = r#"<badge size="huge">x</badge>"#;
    // With no override, it errors.
    assert!(
        reports_floor_error(token_src, "sui-prop-value", &[]),
        "sanity: sui-prop-value fires with no override"
    );
    // With an Off override, the finding is dropped.
    let off = [LintOverride {
        rule: "sui-prop-value".to_string(),
        severity: OverrideSeverity::Off,
    }];
    let with_off = check_all(token_src, None, &off, None);
    assert!(
        !with_off.iter().any(|d| d.rule.as_deref() == Some("sui-prop-value")),
        "a NON-floor rule (sui-prop-value) must be softenable via override: {with_off:?}"
    );
    // But turning off a FLOOR rule the same way changes nothing (proven above).
    let floor_off = CompileOpts::default()
        .with_lint_override("sui-no-class", OverrideSeverity::Off);
    assert!(compile(r#"<p class="x">hi</p>"#, floor_off).is_err());
}

// ---------------------------------------------------------------------------
// The 4th floor rule: `sui-no-undeclared-stylesheet` needs a real pack layout
// (a `sui.config.json` + a companion `.css`). Prove it, too, is non-softenable.
// ---------------------------------------------------------------------------

fn write_pack(config: &str, css: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("sui.config.json"), config).unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let sui = src_dir.join("w.sui");
    fs::write(&sui, "<p>{name}</p>").unwrap();
    for (rel, body) in css {
        fs::write(src_dir.join(rel), body).unwrap();
    }
    (tmp, sui)
}

#[test]
fn undeclared_stylesheet_cannot_be_softened() {
    let (_tmp, sui) = write_pack(
        r#"{"stylesheets": []}"#,
        &[("orphan.css", ".x{color:red}")],
    );
    let src = fs::read_to_string(&sui).unwrap();
    let off = [LintOverride {
        rule: "sui-no-undeclared-stylesheet".to_string(),
        severity: OverrideSeverity::Off,
    }];
    let diags = check_all(&src, Some(&sui), &off, None);
    assert!(
        diags.iter().any(|d| d.rule.as_deref() == Some("sui-no-undeclared-stylesheet")
            && d.severity == DiagSeverity::Error),
        "an Off override must not soften the undeclared-stylesheet floor rule: {diags:?}"
    );
}

// ---------------------------------------------------------------------------
// 4. Doc/parser key match: no `.rs` DOC/prose in compiler/src or lint/src may
//    advertise the drifted `{"policy":"floor|stripped|strictest"}` schema. The
//    parser reads `"preset"` (proven in `check::policy` unit tests); the docs
//    must name the same key. Test bodies are excluded (they assert on the
//    drifted form on purpose).
// ---------------------------------------------------------------------------

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            if !matches!(p.file_name().and_then(|n| n.to_str()), Some("target")) {
                rs_files(&p, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

#[test]
fn no_source_advertises_the_drifted_policy_key() {
    // crates/sui/compiler -> crates/sui
    let sui_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut offenders = Vec::new();
    for sub in ["compiler/src", "lint/src"] {
        let mut files = Vec::new();
        rs_files(&sui_root.join(sub), &mut files);
        for f in files {
            let text = fs::read_to_string(&f).unwrap_or_default();
            // Exclude test modules: they intentionally reference the drifted
            // form to assert the parser rejects it.
            let real = text.split("#[cfg(test)]").next().unwrap_or(&text);
            if real.contains("\"policy\":") {
                offenders.push(f.strip_prefix(sui_root).unwrap().to_path_buf());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "docs must name the parser's config key `preset`, not the drifted \
         `{{\"policy\":\"floor|stripped|strictest\"}}`. Offending files: {offenders:#?}"
    );
}
