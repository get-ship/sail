//! BLOCKS-1: template match arm syntax + LiveState exhaustiveness.
//!
//! - Missing arm on a kinded LiveState match is a compile **error** naming
//!   the arm (`error`).
//! - `{ok as x => …}` binds the unwrapped payload (scope_bound + emit).

use sail_compiler::ir::{Instr, MatchTest};
use sail_compiler::parse::parse;
use sail_compiler::{DiagSeverity, Diagnostic, check_all};

const TEMPLATE_RULE: &str = "sail-match-exhaustive";

fn template_errors(src: &str) -> Vec<Diagnostic> {
    check_all(src, None, &[], None)
        .into_iter()
        .filter(|d| d.rule.as_deref() == Some(TEMPLATE_RULE))
        .collect()
}

#[test]
fn missing_error_arm_is_compile_error_naming_error() {
    let src = r#"
<controller>
import { controller, query } from '@ship/sail-runtime'
export default controller(() => ({
  q: query(async () => ({ name: 'a' })),
}))
</controller>
<match {q()}>
  {loading => <p>…</p>}
  {ok as v => <p>{v.name}</p>}
</match>
"#;
    let diags = template_errors(src);
    assert_eq!(diags.len(), 1, "one missing-arm error: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, DiagSeverity::Error);
    assert!(
        d.message.contains("`error`"),
        "must name missing arm `error`: {}",
        d.message
    );
}

#[test]
fn full_live_state_match_is_silent() {
    let src = r#"
<controller>
import { controller, query } from '@ship/sail-runtime'
export default controller(() => ({
  q: query(async () => ({ name: 'a' })),
}))
</controller>
<match {q()}>
  {loading => <p>…</p>}
  {error as err => <p>{err.message}</p>}
  {ok as v => <p>{v.name}</p>}
</match>
"#;
    assert!(
        template_errors(src).is_empty(),
        "full coverage must not error"
    );
}

#[test]
fn else_arm_covers_remainder() {
    let src = r#"
<controller>
import { controller, query } from '@ship/sail-runtime'
export default controller(() => ({
  q: query(async () => 1),
}))
</controller>
<match {q()}>
  {ok as n => <p>{n}</p>}
  {else => <p>pending</p>}
</match>
"#;
    assert!(
        template_errors(src).is_empty(),
        "else arm makes match exhaustive"
    );
}

#[test]
fn ok_as_binding_parses_and_typechecks_scope() {
    let src = r#"
<controller>
import { controller, query } from '@ship/sail-runtime'
export default controller(() => ({
  q: query(async () => ({ name: 'a' })),
}))
</controller>
<match {q()}>
  {loading => <p>…</p>}
  {error as err => <p>{err.message}</p>}
  {ok as names => <p>{names.name}</p>}
</match>
"#;
    // Parse must accept arm syntax.
    let program = parse(src).expect("parse ok-as arm");
    let m = program
        .template
        .iter()
        .find_map(|i| match i {
            Instr::Match(m) => Some(m),
            _ => None,
        })
        .expect("match instr");
    let ok = m
        .cases
        .iter()
        .find(|c| matches!(&c.test, MatchTest::Kind(k) if k == "ok"))
        .expect("ok arm");
    assert_eq!(ok.binding.as_deref(), Some("names"));
    // scope_bound must accept `names` inside the arm body (no unknown-ident).
    let diags = check_all(src, None, &[], None);
    let scope_errs: Vec<_> = diags
        .iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && (d.message.contains("names") || d.message.contains("unknown"))
        })
        .collect();
    assert!(
        scope_errs.is_empty(),
        "payload binding must be in scope: {scope_errs:?}"
    );
}

#[test]
fn for_in_brace_binding_parses() {
    let src = r#"
<ul>
  <for {items() as item} key={item.id}>
    <li>{item.label}</li>
  </for>
</ul>
"#;
    // items() is unbound without controller — parse only.
    let program = parse(src).expect("parse for as-binding");
    let f = program
        .template
        .iter()
        .find_map(|i| match i {
            Instr::Element(el) => el.children.iter().find_map(|c| match c {
                Instr::For(f) => Some(f),
                _ => None,
            }),
            _ => None,
        })
        .expect("for instr");
    assert_eq!(f.var_name, "item");
    assert!(f.iterable.source.contains("items"));
}

#[test]
fn if_else_arm_parses() {
    let src = r#"
<if {open()}>
  <p>yes</p>
  {else => <p>no</p>}
</if>
"#;
    let program = parse(src).expect("parse if else arm");
    let i = program
        .template
        .iter()
        .find_map(|i| match i {
            Instr::If(i) => Some(i),
            _ => None,
        })
        .expect("if");
    assert!(i.else_body.is_some());
}
