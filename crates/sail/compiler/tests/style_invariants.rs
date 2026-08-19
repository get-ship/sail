//! P6 S6 — BUCKETS §4 convictions: application-default, const-eq-name, dead-const.

use std::path::PathBuf;

use sail_compiler::check_all;
use sail_compiler::edit::{
    AutofixMode, apply_ops_to_source, apply_text_patches, is_auto_apply_plan,
    suggest_fixes_for_source,
};
use sail_compiler::parse::preprocess::preprocess;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../packages/sail/l0/fixtures")
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(fixtures_dir().join(name)).unwrap_or_else(|e| {
        panic!("read fixture {name}: {e}");
    })
}

fn joined(src: &str) -> String {
    check_all(src, None, &[], None)
        .into_iter()
        .map(|d| d.message)
        .collect::<Vec<_>>()
        .join("\n")
}

fn apply_rule(src: &str, rule: &str) -> String {
    let plans = suggest_fixes_for_source(src, "t.sail", None);
    let p = plans
        .iter()
        .find(|p| p.rule == rule)
        .unwrap_or_else(|| panic!("expected FixPlan for {rule}: {plans:?}"));
    assert!(is_auto_apply_plan(p), "must be fix=apply: {p:?}");
    let patch = p.text_patch.as_ref().expect("text patch");
    let (out, skipped) =
        apply_text_patches(src, &[(patch.start, patch.end, patch.replacement.clone())]);
    assert_eq!(skipped, 0, "{out}");
    out
}

#[test]
fn application_default_fixture_sentence_and_apply() {
    let src = fixture("application-default.bad.sail");
    let diags = check_all(&src, None, &[], None);
    assert!(
        diags.iter().any(|d| {
            d.rule.as_deref() == Some("sail-application-default")
                && d.message.contains(
                    "`type(heading)` is the default for `<h1>` — omit it, or is this not an h1?",
                )
        }),
        "expected stamped sentence: {}",
        joined(&src)
    );
    let out = apply_rule(&src, "sail-application-default");
    assert!(out.contains("<h1>"), "tag must stay: {out}");
    assert!(
        !out.contains("type(heading)"),
        "restated Style must be gone: {out}"
    );
    preprocess(&out).unwrap_or_else(|e| panic!("fixed source must compile: {e:?}\n{out}"));
    assert!(
        !check_all(&out, None, &[], None)
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-application-default")),
        "apply must clear the diagnostic: {}",
        joined(&out)
    );
}

#[test]
fn application_default_extra_pad_is_not_this_error() {
    let src = "<h1 style({ type(heading), pad(4) })>x</h1>";
    assert!(
        !check_all(src, None, &[], None)
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-application-default")),
        "pad is not this error: {}",
        joined(src)
    );
}

#[test]
fn const_eq_name_fixture_sentence_and_apply() {
    let src = fixture("const-eq-name.bad.sail");
    let diags = check_all(&src, None, &[], None);
    assert!(
        diags.iter().any(|d| {
            d.rule.as_deref() == Some("sail-const-eq-name")
                && d.message.contains(
                    "`heading` already names `type(heading)` — spend the name, don't redeclare it.",
                )
        }),
        "expected stamped sentence: {}",
        joined(&src)
    );
    assert!(
        !diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-dead-const")),
        "alias is not dead-const: {}",
        joined(&src)
    );
    let out = apply_rule(&src, "sail-const-eq-name");
    assert!(
        !out.contains("style heading ="),
        "alias must be gone: {out}"
    );
    assert!(out.contains("<h1>"), "tag must stay: {out}");
    preprocess(&out).unwrap_or_else(|e| panic!("fixed source must compile: {e:?}\n{out}"));
    assert!(
        !check_all(&out, None, &[], None)
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-const-eq-name")),
        "apply must clear the diagnostic: {}",
        joined(&out)
    );
}

#[test]
fn const_eq_name_extra_slots_are_not_this_error() {
    let src = r#"
<scope>
  style heading = { type(heading), pad(4) }
</scope>
<h1 style(heading)>x</h1>
"#;
    assert!(
        !check_all(src, None, &[], None)
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-const-eq-name")),
        "extra slots are not this error: {}",
        joined(src)
    );
}

#[test]
fn dead_const_fixture_sentence_and_apply() {
    let src = fixture("dead-const.bad.sail");
    let diags = check_all(&src, None, &[], None);
    assert!(
        diags.iter().any(|d| {
            d.rule.as_deref() == Some("sail-dead-const")
                && d.message
                    .contains("`unused` is never spent — delete it, or apply `style(unused)`.")
        }),
        "expected stamped sentence: {}",
        joined(&src)
    );
    let out = apply_rule(&src, "sail-dead-const");
    assert!(
        !out.contains("style unused ="),
        "unused const must be gone: {out}"
    );
    assert!(out.contains("<column>"), "host must stay: {out}");
    preprocess(&out).unwrap_or_else(|e| panic!("fixed source must compile: {e:?}\n{out}"));
    assert!(
        !check_all(&out, None, &[], None)
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-dead-const")),
        "apply must clear the diagnostic: {}",
        joined(&out)
    );
}

#[test]
fn dead_const_spent_is_not_this_error() {
    let src = r#"
<scope>
  style unused = { bg(muted) }
</scope>
<column style(unused)>x</column>
"#;
    assert!(
        !check_all(src, None, &[], None)
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-dead-const")),
        "spent const is not this error: {}",
        joined(src)
    );
}

#[test]
fn fix_apply_consumes_the_three_plans() {
    for (src, rule, gone) in [
        (
            "<h1 style({ type(heading) })>x</h1>",
            "sail-application-default",
            "type(heading)",
        ),
        (
            "<scope>\n  style heading = { type(heading) }\n</scope>\n<h1>x</h1>\n",
            "sail-const-eq-name",
            "style heading =",
        ),
        (
            "<scope>\n  style unused = { bg(muted) }\n</scope>\n<column>x</column>\n",
            "sail-dead-const",
            "style unused =",
        ),
    ] {
        let result =
            apply_ops_to_source(src, "t.sail", &[], None, AutofixMode::Apply, Some("test"));
        assert!(
            result
                .auto_fixed
                .iter()
                .any(|p| p.rule == rule && is_auto_apply_plan(p)),
            "fix=apply must consume {rule}: {:?}",
            result.auto_fixed
        );
        assert!(
            !result.source.contains(gone),
            "{rule} apply must drop `{gone}`: {}",
            result.source
        );
        preprocess(&result.source)
            .unwrap_or_else(|e| panic!("{rule} result must compile: {e:?}\n{}", result.source));
    }
}

#[test]
fn two_doors_stays_suggest_only() {
    let src = r#"<div style({ pad({ x: 3.5 }) })>x</div>"#;
    let result = apply_ops_to_source(src, "t.sail", &[], None, AutofixMode::Apply, Some("test"));
    assert!(
        result.auto_fixed.is_empty(),
        "fix=apply must not rewrite two-doors: {:?}",
        result.auto_fixed
    );
    assert!(
        result.source.contains("3.5"),
        "source must stay unwrapped: {}",
        result.source
    );
}
