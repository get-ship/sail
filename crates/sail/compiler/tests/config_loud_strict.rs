//! config-loud-strict: silent capability stripping is a bug, not a feature.
//!
//! A `sail.config.json` that fails closed to `strict` — whether from a typo'd
//! key, unparseable JSON, or a well-formed config that just doesn't happen to
//! carry a capability signal (`{"stylesheets": []}` alone, the incident that
//! bit the owner for weeks) — must never do so in silence. These tests pin
//! the three tier OUTCOME rules end to end, through the real `overrides_for`
//! log seam and the real `compile()` build gate, not just the pure resolver.
//!
//! `PolicyResolution` / `ResolutionKind` carry WHY a tier resolved the way it
//! did; `crate::check::policy::tests` (unit tests next to the resolver) pin
//! the exact rendered wording. These tests pin the OBSERVABLE BEHAVIOR: what
//! `compile()` does with the resolved overrides, and that every fail-closed
//! path names the file.

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::check::policy::{Policy, ResolutionKind};
use sail_compiler::{CompileError, CompileMode, CompileOpts, compile};

/// `style="…"` is refused by the floor (`spinnaker` / `sail-no-inline-style`).
/// Product Style is the tag call `style({…})` — ungated. `compile()` returns
/// the first Error. Do not accept a silent compile.
fn assert_style_eq_refused(err: &CompileError) {
    assert!(
        matches!(
            err.category.as_deref(),
            Some("spinnaker") | Some("sail-no-inline-style")
        ),
        "style= must be refused by the floor (spinnaker or sail-no-inline-style), got {:?}",
        err.category
    );
}

fn write_fixture(config: &str, sail_body: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
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

// ---------------------------------------------------------------------------
// Rule 1: unparseable config or unknown keys → fail closed (strict) AND
// refuse LOUDLY, naming the file, the offending key, and the resulting tier.
// ---------------------------------------------------------------------------

#[test]
fn unknown_key_fails_closed_and_the_refusal_names_file_key_and_tier() {
    let (_tmp, sail) = write_fixture(r#"{"polcy":"strict"}"#, "<p>hi</p>");
    let (config_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert_eq!(
        resolution.tier,
        Policy::STRICT,
        "unknown key fails closed to strict"
    );
    let ResolutionKind::Rejected(reason) = &resolution.kind else {
        panic!("expected Rejected, got {:?}", resolution.kind);
    };
    assert!(
        reason.contains("polcy"),
        "must name the offending key: {reason}"
    );
    assert!(
        reason.contains("allowed keys:"),
        "must name the allowed keys: {reason}"
    );
    assert!(
        reason.contains("preset"),
        "allowed keys list must include a known key: {reason}"
    );
    assert!(
        config_path.ends_with("sail.config.json"),
        "must name the file: {config_path:?}"
    );

    // Residual `style="p-4"` is refused by the floor (`spinnaker` /
    // `sail-no-inline-style`). Product attach is the tag call `style({…})`.
    let (_tmp2, sail_sx) = write_fixture(
        r#"{"polcy":"strict"}"#,
        r#"<sail-card style="p-4">x</sail-card>"#,
    );
    let err = compile(&fs::read_to_string(&sail_sx).unwrap(), opts_for(&sail_sx))
        .expect_err("style= must be refused by the floor under fail-closed strict");
    assert_style_eq_refused(&err);
}

#[test]
fn unparseable_config_fails_closed_and_refuses_loudly() {
    let (_tmp, sail) = write_fixture("{ not valid json", "<p>hi</p>");
    let (_config_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert_eq!(resolution.tier, Policy::STRICT);
    assert!(
        matches!(&resolution.kind, ResolutionKind::Rejected(r) if r.contains("not valid JSON")),
        "{:?}",
        resolution.kind
    );
}

#[test]
fn wrong_type_strict_fails_closed_and_refuses_loudly() {
    let (_tmp, sail) = write_fixture(r#"{"strict": 1}"#, "<p>hi</p>");
    let (_config_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert_eq!(resolution.tier, Policy::STRICT);
    let ResolutionKind::Rejected(reason) = &resolution.kind else {
        panic!(
            "expected Rejected for wrong-typed strict, got {:?}",
            resolution.kind
        );
    };
    assert!(
        reason.contains("strict") && reason.contains("boolean"),
        "must name the key and expected type: {reason}"
    );
}

#[test]
fn leftover_sx_key_is_rejected_as_unknown() {
    let (_tmp, sail) = write_fixture(r#"{"sx": true}"#, "<p>hi</p>");
    let (_config_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert_eq!(resolution.tier, Policy::STRICT);
    let ResolutionKind::Rejected(reason) = &resolution.kind else {
        panic!(
            "expected Rejected for leftover sx, got {:?}",
            resolution.kind
        );
    };
    assert!(reason.contains("sx"), "must name leftover key: {reason}");
    assert!(
        reason.contains("allowed keys:"),
        "must name allowed keys: {reason}"
    );
}

#[test]
fn unrecognized_preset_value_is_rejected_not_silently_none() {
    // `{"preset":"strictest"}` names a RECOGNIZED key (`preset`) with a value
    // the parser doesn't know — same fail-closed-and-loud bucket as a typo'd
    // key, not the softer "no signal" bucket.
    let (_tmp, sail) = write_fixture(r#"{"preset":"strictest"}"#, "<p>hi</p>");
    let (_config_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert_eq!(resolution.tier, Policy::STRICT);
    assert!(
        matches!(&resolution.kind, ResolutionKind::Rejected(r) if r.contains("strictest")),
        "{:?}",
        resolution.kind
    );
}

// ---------------------------------------------------------------------------
// Rule 2: recognized keys with no capability signal (stylesheets alone) →
// the resulting tier must be ANNOUNCED — the original incident.
// ---------------------------------------------------------------------------

#[test]
fn stylesheets_only_config_is_announced_not_silently_rejected() {
    // The exact incident: `{"stylesheets": []}` used to fail closed to strict
    // with ZERO signal. It must now resolve to strict via the `NoSignal`
    // bucket (an announcement, not a "your config is broken" refusal) and
    // name what recognized key WAS present.
    let (_tmp, sail) = write_fixture(r#"{"stylesheets": []}"#, "<p>hi</p>");
    let (config_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert_eq!(resolution.tier, Policy::STRICT);
    let ResolutionKind::NoSignal(hint) = &resolution.kind else {
        panic!("expected NoSignal (announced), got {:?}", resolution.kind);
    };
    assert!(
        hint.contains("stylesheets"),
        "must name which key produced no signal: {hint}"
    );
    assert!(config_path.ends_with("sail.config.json"));

    // Residual `style="p-4"` is still a floor refuse. The walk already named
    // the config file above. There is no grant key that turns `style=` on.
    let (_tmp2, sail_sx) = write_fixture(
        r#"{"stylesheets": []}"#,
        r#"<sail-card style="p-4">hi</sail-card>"#,
    );
    let err = compile(&fs::read_to_string(&sail_sx).unwrap(), opts_for(&sail_sx))
        .expect_err("style= must be refused — floor, not a grant");
    assert_style_eq_refused(&err);
}

// ---------------------------------------------------------------------------
// Rule 3 (Verify section): granted keys → full, no noise beyond the one
// announcement line — a clean, explicit config compiles quietly.
// ---------------------------------------------------------------------------

#[test]
fn granted_full_preset_compiles_clean_with_one_recognized_announcement() {
    let (_tmp, sail) = write_fixture(
        r#"{"preset":"full"}"#,
        r#"<sail-card style({ pad(4) })> hi</sail-card>"#,
    );
    let (_config_path, resolution) = Policy::resolve_outcome(&sail).expect("config resolves");
    assert_eq!(resolution.tier, Policy::FULL);
    assert!(matches!(&resolution.kind, ResolutionKind::Recognized(s) if s == "preset: full"));

    // Product Style is ungated — the build succeeds without a Style grant.
    compile(&fs::read_to_string(&sail).unwrap(), opts_for(&sail))
        .expect("full preset must compile Style tag calls");
}

// ---------------------------------------------------------------------------
// Rule 3: no config at all — default semantics unchanged (opt-in; natural
// lint severities apply, nothing is promoted to strict).
// ---------------------------------------------------------------------------

#[test]
fn no_config_leaves_default_semantics_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let sail = tmp.path().join("w.sail");
    fs::write(&sail, r#"<sail-card style({ pad(4) })> hi</sail-card>"#).unwrap();
    assert!(
        Policy::resolve_outcome(&sail).is_none(),
        "no config anywhere on the walk"
    );
    assert!(
        Policy::overrides_for(&sail).is_empty(),
        "no config → no capability overrides"
    );
    // Product Style is ungated — an unconfigured package compiles it.
    compile(&fs::read_to_string(&sail).unwrap(), opts_for(&sail))
        .expect("unconfigured package must compile unchanged");
}
