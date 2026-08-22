//! Job 0 — same law everywhere. Catalog stamps are not trusted.
//!
//! Synthetic source + catalog-shaped paths only. These tests do **not**
//! require the on-disk `packages/sail/runtime/src/actions/*` tree to be
//! CSSOM-clean (later jobs delete that dirt).

use std::path::Path;

use crate::report::Finding;
use crate::rules::no_imperative_style;
use crate::{is_action_source_path, lint_action_source, lint_path};

const STYLE_FLOOR: &str = "sail-no-imperative-style";

const CATALOG_STAMP: &str = "/repo/packages/sail/runtime/src/actions/grid-tracks.ts";
const AUTHOR_ACTION: &str = "/repo/apps/demo/src/actions/autofocus.ts";
const SPOTLIGHT_STAMP: &str = "/repo/packages/sail/runtime/src/actions/spotlight.ts";
const THEME_WRITER: &str = "/repo/packages/sail/runtime/src/theme.ts";
const MOUNT_WRITER: &str = "/repo/packages/sail/runtime/src/mount.ts";
/// Compiler-emitted `bind(signal)` paint writer. Not `bind.ts` wholesale.
const BIND_PAINT_WRITER: &str = "/repo/packages/sail/runtime/src/generated/bind-paint-writer.ts";
const BIND_TS_WHOLESALE: &str = "/repo/packages/sail/runtime/src/bind.ts";

/// CSSOM holes the floor must catch. The access is the crime — not only
/// `.style.x =`.
const CSSOM_HOLES: &[(&str, &str)] = &[
    ("style-assign", "el.style.opacity = '0.5'"),
    (
        "object-assign-style",
        "Object.assign(el.style, { color: 'red' })",
    ),
    ("create-element-style", "document.createElement('style')"),
    ("attribute-style-map", "el.attributeStyleMap"),
    ("css-style-sheet-ctor", "new CSSStyleSheet()"),
    ("adopted-style-sheets", "root.adoptedStyleSheets = [sheet]"),
    ("insert-rule", "sheet.insertRule('.x{color:red}', 0)"),
];

const CLEAN_STAMP: &str = r#"
export const observe = (node: Element) => {
  node.addEventListener('click', () => {})
  return { destroy() {} }
}
"#;

fn has_style_floor(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.rule == STYLE_FLOOR)
}

fn style_findings(findings: &[Finding]) -> Vec<&str> {
    findings
        .iter()
        .filter(|f| f.rule == STYLE_FLOOR || f.rule.contains("style") || f.rule.contains("cssom"))
        .map(|f| f.rule)
        .collect()
}

/// Write `rel` under a tempdir and run [`lint_path`] (the product dispatch).
fn lint_rel(rel: &str, source: &str) -> Vec<Finding> {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, source).unwrap();
    lint_path(&path)
}

fn assert_style_floor(label: &str, findings: &[Finding]) {
    assert!(
        has_style_floor(findings),
        "{label}: expected {STYLE_FLOOR} (access is the crime). findings: {findings:?}"
    );
    assert!(
        !findings.iter().any(|f| f.rule == "sail-l1-cssom-smell"),
        "{label}: do not invent a weaker sibling id. findings: {findings:?}"
    );
}

// ── A. Dispatch — catalog stamps are not trusted ───────────────────────────

#[test]
fn catalog_stamp_path_is_action_surface() {
    let catalog = Path::new(CATALOG_STAMP);
    assert!(
        is_action_source_path(catalog),
        "packages/sail/runtime/src/actions/* is an action surface — no trusted caste"
    );
}

#[test]
fn lint_path_catalog_stamp_style_assign_is_no_imperative_style() {
    let findings = lint_rel(
        "packages/sail/runtime/src/actions/grid-tracks.ts",
        "el.style.opacity = '0.5'",
    );
    assert_style_floor("lint_path catalog `.style`", &findings);
}

#[test]
fn lint_action_source_catalog_cssom_holes_are_no_imperative_style() {
    let catalog = Path::new(CATALOG_STAMP);
    for (name, src) in CSSOM_HOLES {
        let findings = lint_action_source(catalog, src);
        assert_style_floor(&format!("lint_action_source catalog {name}"), &findings);
    }
}

#[test]
fn lint_path_catalog_cssom_holes_are_no_imperative_style() {
    for (name, src) in CSSOM_HOLES {
        let findings = lint_rel("packages/sail/runtime/src/actions/grid-tracks.ts", src);
        assert_style_floor(&format!("lint_path catalog {name}"), &findings);
    }
}

#[test]
fn clean_catalog_stamp_has_no_style_findings() {
    let catalog = Path::new(CATALOG_STAMP);
    let via_source = lint_action_source(catalog, CLEAN_STAMP);
    let via_path = lint_rel(
        "packages/sail/runtime/src/actions/grid-tracks.ts",
        CLEAN_STAMP,
    );
    assert!(
        style_findings(&via_source).is_empty(),
        "observation-only stamp must not trip the style floor: {via_source:?}"
    );
    assert!(
        style_findings(&via_path).is_empty(),
        "observation-only stamp via lint_path must not trip the style floor: {via_path:?}"
    );
}

#[test]
fn author_action_cssom_holes_are_no_imperative_style() {
    let author = Path::new(AUTHOR_ACTION);
    for (name, src) in CSSOM_HOLES {
        let findings = lint_action_source(author, src);
        assert_style_floor(&format!("author {name}"), &findings);
    }
    // Author `createElement('style')` is the access-is-the-crime hole the
    // weaker `sail-action-no-style` floor does not close.
    let create = lint_action_source(author, "document.createElement('style')");
    assert_style_floor("author createElement('style')", &create);
    let via_path = lint_rel(
        "apps/demo/src/actions/autofocus.ts",
        "document.createElement('style')",
    );
    assert_style_floor("lint_path author createElement('style')", &via_path);
}

// ── B. Writer allowlist — Full DOM only here ───────────────────────────────

fn writer_theme_src() -> &'static str {
    r#"
export function preview(name: string, value: string) {
  document.documentElement.style.setProperty(name, value)
  const root = document.documentElement
  root.style.setProperty('--brand-hue', value)
}
"#
}

fn writer_mount_src() -> &'static str {
    r#"
export function injectCompilerCss(css: string) {
  const style = document.createElement('style')
  style.textContent = css
  document.head.appendChild(style)
}
"#
}

fn writer_bind_src() -> &'static str {
    r#"
export function writeBoundProperty(el: Element, name: string, value: string) {
  (el as HTMLElement).style.setProperty(name, value)
}
"#
}

#[test]
fn theme_writer_allowlist_does_not_fail_no_imperative_style() {
    let path = Path::new(THEME_WRITER);
    let findings = no_imperative_style::check(path, writer_theme_src());
    assert!(
        !has_style_floor(&findings),
        "theme.ts :root token writes are the writer allowlist — must not fail {STYLE_FLOOR}: {findings:?}"
    );
    let via_path = lint_rel("packages/sail/runtime/src/theme.ts", writer_theme_src());
    assert!(
        !has_style_floor(&via_path),
        "lint_path theme.ts must not fail {STYLE_FLOOR}: {via_path:?}"
    );
}

#[test]
fn mount_writer_allowlist_compiler_css_does_not_fail() {
    let path = Path::new(MOUNT_WRITER);
    let findings = no_imperative_style::check(path, writer_mount_src());
    assert!(
        !has_style_floor(&findings),
        "mount.ts injecting compiler CSS is the writer allowlist — must not fail {STYLE_FLOOR}: {findings:?}"
    );
    let via_path = lint_rel("packages/sail/runtime/src/mount.ts", writer_mount_src());
    assert!(
        !has_style_floor(&via_path),
        "lint_path mount.ts must not fail {STYLE_FLOOR}: {via_path:?}"
    );
}

#[test]
fn generated_bind_paint_writer_may_set_bound_css_property() {
    let path = Path::new(BIND_PAINT_WRITER);
    let findings = no_imperative_style::check(path, writer_bind_src());
    assert!(
        !has_style_floor(&findings),
        "compiler-emitted bind(signal) paint writer may set the bound CSS property: {findings:?}"
    );
    let via_path = lint_rel(
        "packages/sail/runtime/src/generated/bind-paint-writer.ts",
        writer_bind_src(),
    );
    assert!(
        !has_style_floor(&via_path),
        "lint_path generated bind writer must not fail {STYLE_FLOOR}: {via_path:?}"
    );
}

#[test]
fn bind_ts_wholesale_is_not_a_blanket_exemption() {
    let path = Path::new(BIND_TS_WHOLESALE);
    let findings = no_imperative_style::check(path, "el.style.opacity = '0.5'");
    assert!(
        has_style_floor(&findings),
        "packages/sail/runtime/src/bind.ts is not a blanket writer exemption: {findings:?}"
    );
}

#[test]
fn spotlight_stamp_create_element_style_fails() {
    let path = Path::new(SPOTLIGHT_STAMP);
    assert!(
        is_action_source_path(path),
        "spotlight.ts is a catalog stamp, not the writer allowlist"
    );
    let findings = lint_action_source(path, "document.createElement('style')");
    assert_style_floor("spotlight stamp createElement('style')", &findings);
    let via_path = lint_rel(
        "packages/sail/runtime/src/actions/spotlight.ts",
        "document.createElement('style')",
    );
    assert_style_floor("lint_path spotlight stamp", &via_path);
}

#[test]
fn action_rules_include_no_imperative_style_not_a_sibling() {
    let ids: Vec<&str> = crate::rules::ACTION_RULES
        .iter()
        .map(|(id, _)| *id)
        .collect();
    assert!(
        ids.contains(&STYLE_FLOOR),
        "promote {STYLE_FLOOR} onto the action surface; do not invent a sibling. ACTION_RULES = {ids:?}"
    );
    assert!(
        !ids.iter()
            .any(|id| id.contains("cssom-smell") || id.contains("l1-cssom")),
        "no weaker sibling rule id on ACTION_RULES: {ids:?}"
    );
}

/// Keep the allowlist path list honest for the implementer: only the three
/// writer homes (generated bind, theme.ts, mount.ts) skip the floor.
#[test]
fn writer_allowlist_is_the_three_named_homes() {
    let allow: &[(&str, &str)] = &[
        (THEME_WRITER, writer_theme_src()),
        (MOUNT_WRITER, writer_mount_src()),
        (BIND_PAINT_WRITER, writer_bind_src()),
    ];
    for (p, src) in allow {
        let findings = no_imperative_style::check(Path::new(p), src);
        assert!(
            !has_style_floor(&findings),
            "allowlist {p} must pass {STYLE_FLOOR}: {findings:?}"
        );
    }
    let refuse: &[(&str, &str)] = &[
        (CATALOG_STAMP, "el.style.opacity = '0.5'"),
        (SPOTLIGHT_STAMP, "document.createElement('style')"),
        (AUTHOR_ACTION, "el.style.opacity = '0.5'"),
        (BIND_TS_WHOLESALE, "el.style.opacity = '0.5'"),
        (
            "/repo/packages/sail/runtime/src/signal.ts",
            "el.style.opacity = '0.5'",
        ),
    ];
    for (p, src) in refuse {
        let findings = no_imperative_style::check(Path::new(p), src);
        assert!(
            has_style_floor(&findings),
            "non-allowlist {p} must fail {STYLE_FLOOR}: {findings:?}"
        );
    }
}
