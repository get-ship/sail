//! Integration tests for `check_all` — the multi-diagnostic entry point
//! behind the LSP + agent tooling.
//!
//! The defining contract vs `compile()`: `compile()` short-circuits on the
//! first error (a failed build needs one reason); `check_all` collects every
//! issue so an author/agent fixes them in one pass.

use sail_compiler::{CompileMode, CompileOpts, DiagSeverity, Policy, check_all, compile};

fn opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn collects_multiple_errors_where_compile_stops_at_one() {
    // Two distinct renderable-subset violations on two lines: `new` and `this`.
    // Content tags (<p>/<em>) — not raw <div>/<span> — so the only diagnostics
    // are the two expr-subset Errors, keeping this test about multi-collection.
    let src = "<p>{new Date()}</p>\n<em>{this.value}</em>";

    // compile() stops at the first.
    compile(src, opts()).expect_err("compile should fail");

    // check_all surfaces both.
    let diags = check_all(src, None, &[], None);
    assert!(
        diags.len() >= 2,
        "expected >=2 diagnostics, got {}: {diags:?}",
        diags.len()
    );
    assert!(
        diags.iter().any(|d| d.line == 1),
        "missing line-1 diag: {diags:?}"
    );
    assert!(
        diags.iter().any(|d| d.line == 2),
        "missing line-2 diag: {diags:?}"
    );
    assert!(
        diags
            .iter()
            .all(|d| matches!(d.severity, DiagSeverity::Error))
    );
}

#[test]
fn clean_source_yields_no_diagnostics() {
    // Bare interpolation is renderable; with no controller path, scope-bound
    // checks are skipped, so this is clean.
    let diags = check_all("<p>{name}</p>", None, &[], None);
    assert!(diags.is_empty(), "expected no diagnostics, got {diags:?}");
}

#[test]
fn parse_error_is_a_single_diagnostic() {
    // Nothing can run without an AST, so a parse failure is exactly one diag.
    let diags = check_all("<p>{", None, &[], None);
    assert_eq!(
        diags.len(),
        1,
        "parse error should yield one diag: {diags:?}"
    );
}

#[test]
fn diagnostics_are_sorted_by_position() {
    let src = "<em>{this.value}</em>\n<p>{new Date()}</p>";
    let diags = check_all(src, None, &[], None);
    let positions: Vec<(usize, usize)> = diags.iter().map(|d| (d.line, d.col)).collect();
    let mut sorted = positions.clone();
    sorted.sort();
    assert_eq!(
        positions, sorted,
        "diagnostics must be position-sorted: {diags:?}"
    );
}

// ----------------------------------------------------------------------------
// The stylesheet permit — `sail-no-undeclared-stylesheet`.
//
// These tests use the **real dispatch path** the compiler uses: a tempdir
// laid out like `packages/components/sail/<pack>/` (a `sail.config.json` at
// the pack root, a `.sail` source under `src/`, an undeclared `.css` next
// to it), then `check_all()` with the overrides the CLI applies. That is
// the same call shape the daemon and CLI take — `compile()` invokes
// `check::gate` → `check::collect` → `sail_lint::lint_sail_template_source`
// → `no_undeclared_stylesheet::check`; `check_all` skips the structural
// passes and goes straight to `collect`, which is the path that matters
// for the lint rules.
//
// THE BAR: a violation that FAILS the test when the offending `.css` is
// deleted. If you change this and `undeclared_stylesheet_fails_real_compile_path`
// still passes when the `undeclared.css` file is removed, the test is
// broken — the rule is then the fourth check that cannot fail. Delete the
// file, watch the assertion fail; restore it.
// ----------------------------------------------------------------------------

/// Write a synthetic pack into a tempdir: `<pack>/sail.config.json`,
/// `<pack>/src/widget.sail`, plus the css files named in `css_files` as
/// `<pack>/src/<rel>` (e.g. `"x.css"`, `"widget/widget.css"`). The `.sail`
/// source is a renderable component (`<p>{name}</p>`) so the structural
/// passes produce nothing and the only diagnostic the compiler can emit is
/// the stylesheet-permit one.
fn write_fixture(css_files: &[(&str, &str)], config: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("tmpdir");
    std::fs::write(tmp.path().join("sail.config.json"), config).unwrap();
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let sail_path = src_dir.join("widget.sail");
    std::fs::write(&sail_path, "<p>{name}</p>").unwrap();
    for (rel, body) in css_files {
        let p = src_dir.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, body).unwrap();
    }
    (tmp, sail_path)
}

#[test]
fn undeclared_stylesheet_fails_real_compile_path() {
    // A pack that declares ONE sheet and then smuggles in a second one
    // that isn't in the manifest. The dispatch path the CLI uses
    // (`Policy::overrides_for` → floor + capabilities, then
    // `check_all` → `lint_sail_template_source`) MUST surface an Error.
    let (_tmp, sail_path) = write_fixture(
        &[
            ("declared.css", ".x { color: red; }"),
            ("undeclared.css", ".y { color: blue; }"),
        ],
        r#"{"stylesheets": ["src/declared.css"]}"#,
    );
    let src = std::fs::read_to_string(&sail_path).unwrap();
    let overrides = Policy::overrides_for(&sail_path);
    let diags = check_all(&src, Some(&sail_path), &overrides, None);
    let permit_errors: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && d.rule.as_deref() == Some("sail-no-undeclared-stylesheet")
        })
        .collect();
    assert_eq!(
        permit_errors.len(),
        1,
        "expected exactly one undeclared-stylesheet Error, got {diags:?}"
    );
    assert!(
        permit_errors[0].message.contains("undeclared.css"),
        "diagnostic must name the undeclared file: {}",
        permit_errors[0].message
    );
}

#[test]
fn declared_stylesheet_passes_real_compile_path() {
    // A pack that declares its ONLY sheet. No stylesheet-permit Error.
    // (Other diagnostics may exist; we filter to the rule.)
    let (_tmp, sail_path) = write_fixture(
        &[("only.css", ".x { color: red; }")],
        r#"{"stylesheets": ["src/only.css"]}"#,
    );
    let src = std::fs::read_to_string(&sail_path).unwrap();
    let overrides = Policy::overrides_for(&sail_path);
    let diags = check_all(&src, Some(&sail_path), &overrides, None);
    let permit_errors: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && d.rule.as_deref() == Some("sail-no-undeclared-stylesheet")
        })
        .collect();
    assert!(
        permit_errors.is_empty(),
        "declared sheet must not produce a stylesheet-permit Error: {diags:?}"
    );
}

#[test]
fn empty_stylesheet_array_still_permits_zero() {
    // A pack that ships no CSS — `stylesheets: []` is a valid declaration
    // of "no companion sheets". An undeclared sheet still fails; the
    // empty array is not a back door. Pins both ends: declared-empty
    // passes, an undeclared sheet with declared-empty fails.
    let (_tmp, sail_path) = write_fixture(
        &[("undeclared.css", ".y { color: blue; }")],
        r#"{"stylesheets": []}"#,
    );
    let src = std::fs::read_to_string(&sail_path).unwrap();
    let overrides = Policy::overrides_for(&sail_path);
    let diags = check_all(&src, Some(&sail_path), &overrides, None);
    let permit_errors: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && d.rule.as_deref() == Some("sail-no-undeclared-stylesheet")
        })
        .collect();
    assert_eq!(
        permit_errors.len(),
        1,
        "empty stylesheets[] + an undeclared sheet = Error: {diags:?}"
    );
    assert!(permit_errors[0].message.contains("undeclared.css"));
}

#[test]
fn no_class_fails_real_compile_path() {
    // The disease: a `class=` in a pack source that lives under a
    // `sail.config.json` used to be silently allowed because the rule
    // walked the parent chain looking for ANY config. With the opt-in
    // removed, `class=` refuses everywhere — packs, routes, apps.
    //
    // The dispatch path the CLI uses (`Policy::overrides_for` → floor
    // + capabilities, then `check_all` → `lint_sail_template_source`)
    // MUST surface an Error here, and the diagnostic's rule id MUST be
    // `sail-no-class`. The path is real (a tempdir laid out like a
    // pack) — no isolation.
    let (_tmp, sail_path) = write_fixture(&[], r#"{"stylesheets": []}"#);
    // Replace the default widget.sail with one that has a class=.
    let src = r#"<p class="sneaky-hook">{name}</p>"#;
    std::fs::write(&sail_path, src).unwrap();
    let overrides = Policy::overrides_for(&sail_path);
    let diags = check_all(src, Some(&sail_path), &overrides, None);
    let class_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == DiagSeverity::Error && d.rule.as_deref() == Some("sail-no-class"))
        .collect();
    assert_eq!(
        class_errors.len(),
        1,
        "expected exactly one no-class Error, got {diags:?}"
    );
    assert!(
        class_errors[0].message.contains("class") || class_errors[0].message.contains("`class`"),
        "diagnostic should mention class: {}",
        class_errors[0].message
    );
}

#[test]
fn sx_only_source_passes_real_compile_path() {
    // The negative direction: a pack source that styles with `sx=`
    // instead of `class=` is clean. The full floor surface is
    // available: sx=, role, aria-*, on:event, data-*.
    let (_tmp, sail_path) = write_fixture(&[], r#"{"stylesheets": []}"#);
    let src = r#"<p role="listbox" aria-label="x" on:click={noop} style({ box: pad(4), paint: bg(surface-2) })> {name}</p>"#;
    std::fs::write(&sail_path, src).unwrap();
    let overrides = Policy::overrides_for(&sail_path);
    let diags = check_all(src, Some(&sail_path), &overrides, None);
    let class_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == DiagSeverity::Error && d.rule.as_deref() == Some("sail-no-class"))
        .collect();
    assert!(
        class_errors.is_empty(),
        "s-only source must not produce a no-class Error: {diags:?}"
    );
}

#[test]
fn class_directive_fails_real_compile_path() {
    // The `class:name={value}` directive form is retired at the parser
    // (same shape as `style:`). Floor lint still matches `class:` for
    // defense in depth; either path must surface an Error naming the
    // retirement — never emit a class toggle binding.
    let (_tmp, sail_path) = write_fixture(&[], r#"{"stylesheets": []}"#);
    let src = r#"<a class:active={flag()} href="/x">x</a>"#;
    std::fs::write(&sail_path, src).unwrap();
    let overrides = Policy::overrides_for(&sail_path);
    let diags = check_all(src, Some(&sail_path), &overrides, None);
    let class_errors: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && (d.rule.as_deref() == Some("sail-no-class")
                    || d.message.contains("`class:`")
                    || d.message.contains("class:"))
        })
        .collect();
    assert!(
        !class_errors.is_empty(),
        "class:active= must be an Error naming retirement, got {diags:?}"
    );
    assert!(
        class_errors.iter().any(|d| {
            d.message.contains("removed")
                || d.message.contains("retired")
                || d.message.contains("sx")
                || d.rule.as_deref() == Some("sail-no-class")
        }),
        "error should teach retirement / sx, got {class_errors:?}"
    );
}
