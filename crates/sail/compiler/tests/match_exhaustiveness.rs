//! Style match exhaustiveness — a `match(prop, {…})` on a closed-union scope
//! prop that leaves members without an arm warns (never errors); full
//! coverage and non-union scrutinees stay silent.

use sail_compiler::{DiagSeverity, Diagnostic, check_all};

const RULE: &str = "sail-style-match-exhaustive";

fn warnings(src: &str) -> Vec<Diagnostic> {
    check_all(src, None, &[], None)
        .into_iter()
        .filter(|d| d.rule.as_deref() == Some(RULE))
        .collect()
}

#[test]
fn partial_match_on_union_prop_warns_naming_missing_members() {
    let src = r#"
<scope>
props: {
  variant: 'a' | 'b' | 'c' = 'a'
}
</scope>
<div> match(variant, { a: style.bg(primary), b: style.bg(muted) }) x</div>
"#;
    let diags = warnings(src);
    assert_eq!(diags.len(), 1, "one warning for the missing arm: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, DiagSeverity::Warning);
    assert!(
        d.message.contains("`variant`"),
        "names the prop: {}",
        d.message
    );
    assert!(
        d.message.contains("`c`"),
        "names the missing member: {}",
        d.message
    );
    assert!(
        !d.message.contains("`a`") && !d.message.contains("`b`"),
        "covered members not named: {}",
        d.message
    );
}

#[test]
fn fully_covered_union_match_is_silent() {
    let src = r#"
<scope>
props: {
  variant: 'a' | 'b' | 'c' = 'a'
}
</scope>
<div> match(variant, { a: style.bg(primary), b: style.bg(muted), c: style.bg(accent) }) x</div>
"#;
    assert!(warnings(src).is_empty(), "full coverage must not warn");
}

#[test]
fn non_union_scrutinee_is_silent() {
    let src = r#"
<scope>
props: {
  label = 'x'
  count = 3
}
</scope>
<div> match(label, { x: style.bg(primary) }) x</div>
<p> match(count, { 3: style.bg(muted) }) y</p>
"#;
    assert!(
        warnings(src).is_empty(),
        "string/number scrutinees must not warn"
    );
}

#[test]
fn scope_style_const_match_is_checked() {
    let src = r#"
<scope>
props: {
  variant: 'a' | 'b' | 'c' = 'a'
}
const tones = match(variant, { a: style.bg(primary) })
</scope>
<div> style(tones) x</div>
"#;
    let diags = warnings(src);
    assert_eq!(diags.len(), 1, "const-declared match warns once: {diags:?}");
    assert!(
        diags[0].message.contains("`b`") && diags[0].message.contains("`c`"),
        "names both missing members: {}",
        diags[0].message
    );
}
