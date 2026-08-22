//! SPEC-STYLE-BUCKETS-1 invariant 7 — scope-poisoning cascade.
//!
//! One bad Style table row must not unregister sibling style consts.
//! Repro 08-16: a sibling const that spends unknown `size.wide` made
//! `<page> style(base)` an "unknown vocabulary `base`" error.

use sail_compiler::CompileMode;
use sail_compiler::CompileOpts;
use sail_compiler::check_all;
use sail_compiler::compile;
use sail_compiler::parse::preprocess::preprocess;

fn compile_component(src: &str) -> Result<String, String> {
    let out = compile(
        src,
        CompileOpts {
            element_prefix: "sail".into(),
            host_data_attr: "data-sail-host".into(),
            setup_url: None,
            mode: CompileMode::Component,
            source_path: None,
            component_meta: None,
            route_params: None,
            scope_capabilities: Vec::new(),
            emit_dts: false,
            emit_component_imports: false,
            lint_overrides: Vec::new(),
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    Ok(out.js)
}

fn joined_diags(src: &str) -> String {
    check_all(src, None, &[], None)
        .into_iter()
        .map(|d| format!("{} @{}:{}", d.message, d.line, d.col))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The 08-16 poison: legal `style base` plus a sibling that spends unknown
/// `size.wide`. `style(base)` must still resolve; the error names the bad token.
#[test]
fn sibling_unknown_identity_does_not_poison_style_base() {
    let src = r#"
<scope>
  style base = { pad(page) }
  style wide = { size(wide) }
</scope>
<column style(base)>x</column>
"#;

    let out = preprocess(src).expect("legal style(base) must still lower");
    assert!(
        out.contains("s-padpage") || out.contains("--pad-page"),
        "style(base) must resolve pad(page):\n{out}"
    );
    assert!(
        !out.to_lowercase().contains("unknown vocabulary `base`"),
        "preprocess must not report unknown vocabulary base:\n{out}"
    );

    let err = compile_component(src).expect_err("illegal identity is terminal");
    assert!(
        err.contains("wide") || err.contains("size.wide"),
        "error must name the bad identity: {err}"
    );
    assert!(
        !err.contains("unknown vocabulary `base`"),
        "must not poison sibling `base`: {err}"
    );

    let diags = check_all(src, None, &[], None);
    let joined = diags
        .iter()
        .map(|d| format!("{} @{}:{}", d.message, d.line, d.col))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("wide") || joined.contains("size.wide"),
        "check_all must name the bad identity: {joined}"
    );
    assert!(
        !joined.contains("unknown vocabulary `base`"),
        "check_all must not poison `base`: {joined}"
    );
    assert!(
        diags.iter().any(|d| {
            (d.message.contains("wide") || d.message.contains("size.wide")) && d.line == 4
        }),
        "error must sit on `style wide`, not the scope/file: {joined}"
    );
}

/// page.sail shape: `style base` refs sibling insets; a later unknown
/// identity must not unregister `base` or the when-refs.
#[test]
fn page_shape_sibling_unknown_does_not_unregister_base() {
    let src = r#"
<scope>
  style insetMd = { pad(page-md) }
  style insetLg = { pad(page-lg) }
  style base = { pad(page), when: { tablet: insetMd, desktop: insetLg } }
  style wide = { size(wide) }
</scope>
<column gap="6" style(base)>
  x
</column>
"#;

    let out = preprocess(src).expect("page-shaped style(base) must still lower");
    assert!(
        out.contains("s-padpage") || out.contains("--pad-page"),
        "style(base) must resolve:\n{out}"
    );
    assert!(
        out.contains("pad-page-md") || out.contains("s-tablet-padpagemd"),
        "insetMd when-ref must stay registered:\n{out}"
    );

    let err = compile_component(src).expect_err("illegal identity is terminal");
    assert!(
        err.contains("wide") || err.contains("size.wide"),
        "error must name the bad identity: {err}"
    );
    assert!(
        !err.contains("unknown vocabulary `base`")
            && !err.contains("unknown vocabulary `insetMd`")
            && !err.contains("unknown vocabulary `insetLg`"),
        "must not unregister sibling names: {err}"
    );
}

/// `<page style(base)>` use site — the 08-16 report. Local `base` stays
/// legal when an unrelated const in the same scope is unknown.
#[test]
fn page_tag_style_base_stays_legal_when_sibling_is_unknown() {
    let src = r#"
<scope>
  style base = { pad(page) }
  style wide = { size(wide) }
</scope>
<page style(base)>x</page>
"#;

    let out = preprocess(src).expect("<page style(base)> must still lower");
    assert!(
        out.contains("s-padpage") || out.contains("--pad-page"),
        "use-site style(base) must resolve:\n{out}"
    );
    assert!(
        out.contains("data-sail-host=\"page\"") || out.contains("sail-page"),
        "<page> must still expand:\n{out}"
    );

    let err = compile_component(src).expect_err("illegal identity is terminal");
    assert!(
        err.contains("wide") || err.contains("size.wide"),
        "error must name the bad identity: {err}"
    );
    assert!(
        !err.contains("unknown vocabulary `base`"),
        "<page> style(base) must not become unknown vocabulary: {err}"
    );

    let diags = joined_diags(src);
    assert!(
        !diags.contains("unknown vocabulary `base`"),
        "check_all must not report unknown vocabulary base: {diags}"
    );
    assert!(
        diags.contains("wide") || diags.contains("size.wide"),
        "check_all must name size.wide at the const: {diags}"
    );
}

/// Off-scale pad in a named const must fire the two-doors sentence at the
/// declaration. Sibling `base` stays registered (same cascade law).
#[test]
fn named_const_off_scale_pad_fires_scale_sentence() {
    let src = r#"
<scope>
  style base = { pad(page) }
  style box = { pad({ x: 3.5 }) }
</scope>
<column style(base)>x</column>
"#;

    let out = preprocess(src).expect("legal style(base) must still lower");
    assert!(
        out.contains("s-padpage") || out.contains("--pad-page"),
        "style(base) must resolve pad(page):\n{out}"
    );
    assert!(
        !out.to_lowercase().contains("unknown vocabulary `base`"),
        "preprocess must not report unknown vocabulary base:\n{out}"
    );

    let err = compile_component(src).expect_err("off-scale named const is terminal");
    assert!(
        err.contains("3.5 is not a space step") && err.contains("raw(3.5)"),
        "compile must carry the two-doors sentence: {err}"
    );
    assert!(
        !err.contains("unknown vocabulary `base`"),
        "must not poison sibling `base`: {err}"
    );

    let diags = check_all(src, None, &[], None);
    let joined = diags
        .iter()
        .map(|d| format!("{} @{}:{}", d.message, d.line, d.col))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("3.5 is not a space step") && joined.contains("raw(3.5)"),
        "check_all must carry the two-doors sentence: {joined}"
    );
    assert!(
        !joined.contains("unknown vocabulary `base`"),
        "check_all must not poison `base`: {joined}"
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("3.5 is not a space step") && d.line == 4),
        "scale sentence must sit on `style box`: {joined}"
    );
}
