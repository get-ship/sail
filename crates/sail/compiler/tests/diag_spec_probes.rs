//! SPEC-LINTS-1 Phase 2 / SPEC-TESTING-1 — DiagSpec structural probes.
//!
//! Every live `sail-*` product id must be in the registry. Allowed-list
//! sources, when present, must be non-empty (live path/symbol name).
//! Not a message-text golden.

use std::fs;
use std::path::PathBuf;

use sail_compiler::check_all;
use sail_compiler::diag_ratchet::{idd_rule_ids, is_stable_rule_id};
use sail_compiler::diagnostics::{DiagSeverity, lookup, registry};

fn compiler_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn lint_rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../lint/src/rules")
}

fn collect_lint_rule_ids() -> Vec<String> {
    let mut ids = Vec::new();
    let dir = lint_rules_dir();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for line in text.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("pub const RULE_ID") {
                if let Some(q0) = rest.find('"') {
                    let rest = &rest[q0 + 1..];
                    if let Some(q1) = rest.find('"') {
                        ids.push(rest[..q1].to_string());
                    }
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

#[test]
fn every_lint_rule_id_is_registered() {
    let ids = collect_lint_rule_ids();
    assert!(
        ids.len() >= 40,
        "expected a real lint RULE_ID scan, got {ids:?}"
    );
    let mut missing = Vec::new();
    for id in &ids {
        assert!(
            is_stable_rule_id(id),
            "lint RULE_ID is not a sail-* id: {id}"
        );
        if lookup(id).is_none() {
            missing.push(id.as_str());
        }
    }
    assert!(
        missing.is_empty(),
        "lint RULE_IDs missing from DiagSpec registry: {missing:?}"
    );
}

#[test]
fn every_registry_row_has_jurisdiction_and_template() {
    for spec in registry() {
        let _ = spec.jurisdiction;
        assert!(!spec.message_template.is_empty(), "{}", spec.id);
        if let Some(src) = spec.allowed_list_source {
            assert!(!src.is_empty(), "{} empty allowed_list_source", spec.id);
            // Live-ish: names a rust path (`::`) or a known crate prefix.
            assert!(
                src.contains("::") || src.starts_with("sail_") || src.contains('/'),
                "{} allowed_list_source {src:?} does not look like a live path/symbol",
                spec.id
            );
        }
    }
}

#[test]
fn every_live_compiler_id_resolves_in_registry() {
    let ids = idd_rule_ids();
    assert!(
        !ids.is_empty(),
        "expected live Idd throw-site ids from compiler src"
    );
    let mut missing = Vec::new();
    let mut incomplete = Vec::new();
    for id in &ids {
        assert!(is_stable_rule_id(id), "live throw id is not sail-*: {id}");
        let Some(spec) = lookup(id) else {
            missing.push(id.as_str());
            continue;
        };
        let _ = spec.jurisdiction;
        let _ = spec.default_severity;
        if spec.message_template.is_empty() {
            incomplete.push(id.as_str());
        }
        assert_eq!(spec.id, id.as_str());
        let _ = matches!(
            spec.default_severity,
            DiagSeverity::Error | DiagSeverity::Warning | DiagSeverity::Fix
        );
    }
    assert!(
        missing.is_empty(),
        "live throw-site ids missing from DiagSpec registry: {missing:?}"
    );
    assert!(
        incomplete.is_empty(),
        "registry rows missing message_template: {incomplete:?}"
    );
}

#[test]
fn check_all_emitted_ids_are_in_registry() {
    // Same stream sail_check / LSP OnSave consume (`check_all`).
    let snippets = [
        "<foobarbaz>hi</foobarbaz>",
        "<p>{}</p>",
        "<p>{a; b}</p>",
        "<for {123 as n} key={0}><p>{n}</p></for>",
        "<if {true}><p>x</p>",
        "<button><icon name=\"x\" /></button>",
        r#"<div mutation="notavalid"></div>"#,
        r#"<button on:click={count()}></button>"#,
    ];
    let mut seen = Vec::new();
    let mut missing = Vec::new();
    for src in snippets {
        for d in check_all(src, None, &[], None) {
            let Some(id) = d.rule.as_deref() else {
                continue;
            };
            seen.push(id.to_string());
            if lookup(id).is_none() {
                missing.push(id.to_string());
            }
        }
    }
    assert!(
        !seen.is_empty(),
        "expected check_all to emit at least one rule id across snippets"
    );
    assert!(
        missing.is_empty(),
        "check_all/sail_check rule ids not in DiagSpec registry: {missing:?} (emitted={seen:?})"
    );
}

#[test]
fn known_compiler_ids_are_registered() {
    let known = [
        "sail-block-order",
        "sail-no-type-erase",
        "sail-pure-region",
        "sail-query-key",
        "sail-type-gate",
        "sail-unintended-overflow",
        "sail-clipped-text",
        "sail-sub-min-tap-target",
    ];
    for id in known {
        assert!(lookup(id).is_some(), "missing {id}");
    }
    let _ = compiler_src();
}
