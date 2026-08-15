//! Hug-content clip must not be silenced with truncate.
//!
//! A hug control whose glyphs measure 1px past the box is a box bug.
//! `type({ wrap: truncate })` / overflow-hidden on that shape is a mask.

use std::path::PathBuf;

use sail_compiler::check::layout;
use sail_compiler::{DiagSeverity, check_all};

const RULE_CLIPPED_TEXT: &str = "sail-clipped-text";

/// Five size-sm `select` badges in a 24-step column — one or more measure 1px short.
const HUG_BADGE_SELECT: &str = r#"
<column style({ size({ max-w: 24 }) })>
  <row gap="1">
    <badge size="sm">select</badge>
    <badge size="sm">select</badge>
    <badge size="sm">select</badge>
    <badge size="sm">select</badge>
    <badge size="sm">select</badge>
  </row>
</column>
"#;

const HUG_BADGE_SELECT_TRUNCATE: &str = r#"
<column style({ size({ max-w: 24 }) })>
  <row gap="1">
    <badge size="sm" style({ type({ wrap: truncate }) })>select</badge>
    <badge size="sm" style({ type({ wrap: truncate }) })>select</badge>
    <badge size="sm" style({ type({ wrap: truncate }) })>select</badge>
    <badge size="sm" style({ type({ wrap: truncate }) })>select</badge>
    <badge size="sm" style({ type({ wrap: truncate }) })>select</badge>
  </row>
</column>
"#;

const HUG_BADGE_SELECT_OVERFLOW: &str = r#"
<column style({ size({ max-w: 24 }) })>
  <row gap="1">
    <badge size="sm" style({ overflow({ x: hidden }) })>select</badge>
    <badge size="sm" style({ overflow({ x: hidden }) })>select</badge>
    <badge size="sm" style({ overflow({ x: hidden }) })>select</badge>
    <badge size="sm" style({ overflow({ x: hidden }) })>select</badge>
    <badge size="sm" style({ overflow({ x: hidden }) })>select</badge>
  </row>
</column>
"#;

const MAX_W_TRUNCATE: &str = r#"
<p style({ size({ max-w: 24 }), type({ wrap: truncate }) })>Supercalifragilisticexpialidocious</p>
"#;

const CONSTRAINED_NO_TRUNCATE: &str = r#"
<p style({ size(16).overflow({ x: hidden }) })>Supercalifragilisticexpialidocious</p>
"#;

const CONSTRAINED_TRUNCATE: &str = r#"
<p style({ size(16).type({ wrap: truncate }) })>Supercalifragilisticexpialidocious</p>
"#;

fn clipped(src: &str) -> Vec<sail_compiler::Diagnostic> {
    layout::collect(src)
        .into_iter()
        .filter(|d| d.rule.as_deref() == Some(RULE_CLIPPED_TEXT))
        .collect()
}

fn suggests_truncate(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("add type({ wrap: truncate })")
        || lower.contains("add `type({ wrap: truncate })`")
        || lower.contains("add `text: truncate()`")
        || lower.contains("text: truncate()")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn hug_badge_select_one_px_short_errors_without_truncate_suggest() {
    let hits = clipped(HUG_BADGE_SELECT);
    assert!(
        !hits.is_empty(),
        "hug <badge>select</badge> 1px short must emit `{RULE_CLIPPED_TEXT}`: {hits:?}"
    );
    assert!(
        hits.iter().all(|d| d.severity == DiagSeverity::Error),
        "hug clip is Error: {hits:?}"
    );
    for d in &hits {
        assert!(
            !suggests_truncate(&d.message),
            "hug clip must not suggest truncate: {}",
            d.message
        );
    }
}

#[test]
fn hug_badge_truncate_mask_still_errors() {
    let hits = clipped(HUG_BADGE_SELECT_TRUNCATE);
    assert!(
        !hits.is_empty(),
        "hug + type({{ wrap: truncate }}) must still emit `{RULE_CLIPPED_TEXT}`: {hits:?}"
    );
    for d in &hits {
        assert!(
            !suggests_truncate(&d.message),
            "hug truncate-mask must not suggest truncate: {}",
            d.message
        );
    }
}

#[test]
fn hug_badge_overflow_hidden_mask_still_errors() {
    let hits = clipped(HUG_BADGE_SELECT_OVERFLOW);
    assert!(
        !hits.is_empty(),
        "hug + overflow hidden must still emit `{RULE_CLIPPED_TEXT}`: {hits:?}"
    );
    for d in &hits {
        assert!(
            !suggests_truncate(&d.message),
            "hug overflow-mask must not suggest truncate: {}",
            d.message
        );
    }
}

#[test]
fn max_w_truncate_is_legal() {
    let hits = clipped(MAX_W_TRUNCATE);
    assert!(
        hits.is_empty(),
        "max-w + type({{ wrap: truncate }}) is legal: {hits:?}"
    );
}

#[test]
fn isolated_hug_badge_that_fits_is_silent() {
    let hits = clipped("<badge>select</badge>");
    assert!(
        hits.is_empty(),
        "unconstrained hug badge must not fire: {hits:?}"
    );
}

#[test]
fn constrained_clip_still_suggests_truncate() {
    let hits = clipped(CONSTRAINED_NO_TRUNCATE);
    assert!(
        !hits.is_empty(),
        "explicit w + overflow hidden without truncate must stay red: {hits:?}"
    );
    assert!(
        hits.iter().any(|d| suggests_truncate(&d.message)),
        "constrained clip may suggest truncate: {hits:?}"
    );
}

#[test]
fn constrained_truncate_stays_silent() {
    let hits = clipped(CONSTRAINED_TRUNCATE);
    assert!(
        hits.is_empty(),
        "explicit w + truncate is the legal opt-in: {hits:?}"
    );
}

#[test]
fn pure_region_and_type_erase_still_fire() {
    let src = "\n\
<controller>\n\
export default controller((ctx) => {\n\
  const path = typeof location !== 'undefined' ? location.pathname : '/'\n\
  const hist = window.history\n\
  const n = null as unknown as number\n\
  return { path, hist, n }\n\
})\n\
</controller>\n\
<div />\n";
    let path = workspace_root().join("apps/fake/src/widget/clip-mask.sail");
    let diags = check_all(src, Some(&path), &[], None);
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-pure-region")),
        "existing DOM refusal must still fire: {diags:?}"
    );
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-no-type-erase")),
        "existing as unknown as refusal must still fire: {diags:?}"
    );
}
