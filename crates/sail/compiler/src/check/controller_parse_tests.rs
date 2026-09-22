//! Unit tests for `controller_parse`. Lives in a sibling module to
//! keep the implementation file under the 300-line cap.

use std::path::PathBuf;

use super::controller_parse::{ParsedController, ScopeKind, parse};

fn p(src: &str) -> ParsedController {
    parse(PathBuf::from("test.ts"), src.into()).expect("expected controller call")
}

#[test]
fn ignores_null_default() {
    let r = parse(PathBuf::from("n.ts"), "export default null\n".into());
    assert!(r.is_none());
}

#[test]
fn extracts_block_body_returned_members() {
    let src = r#"
import { controller, derived } from '@ship/sail-runtime'
export default controller((ctx) => {
  const x = derived(() => 1)
  const go = (s) => {}
  return { x, go }
})
"#;
    let r = p(src);
    let names: Vec<&str> = r.returned.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["x", "go"]);
    assert_eq!(r.member("x").unwrap().kind, ScopeKind::Derived);
    assert_eq!(r.member("go").unwrap().kind, ScopeKind::Action);
}

#[test]
fn extracts_concise_body_returned_members() {
    let src = r#"export default controller((ctx) => ({ count: signal(0) }))"#;
    let r = p(src);
    let names: Vec<&str> = r.returned.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["count"]);
    assert_eq!(r.member("count").unwrap().kind, ScopeKind::Signal);
}

#[test]
fn captures_ctx_and_host_params() {
    let src = r#"export default controller((ctx, host) => ({ n: signal(0) }))"#;
    let r = p(src);
    assert_eq!(r.ctx_param, "ctx");
    assert_eq!(r.host_param.as_deref(), Some("host"));
}

#[test]
fn captures_destructured_ctx_param() {
    let src = r#"export default controller(({ ship, route }) => ({ n: signal(0) }))"#;
    let r = p(src);
    assert_eq!(r.ctx_param, "{ ship, route }");
    assert!(r.host_param.is_none());
}

#[test]
fn ignores_keys_inside_nested_literal() {
    let src = r#"
export default controller((ctx) => {
  return {
    pretender: () => ({ effects: 1 }),
  }
})
"#;
    let r = p(src);
    let names: Vec<&str> = r.returned.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["pretender"]);
}

#[test]
fn empty_return_is_legal() {
    let src = r#"export default controller((ctx) => ({}))"#;
    let r = p(src);
    assert!(r.returned.is_empty());
}

// ── regex literals + multi-declarator lists ────────────────────────────────

/// A regex literal is valid JS whose brackets are TEXT. The scanner used to
/// count them as structure, so `return {...}` became unfindable and the whole
/// contributed scope came back EMPTY — the template then blamed its first
/// binding with "Available: ctx.", a phantom error at a consumer line.
#[test]
fn regex_literal_does_not_collapse_the_returned_scope() {
    let src = r#"
export default controller((ctx) => {
  const bez = (v) => v.match(/cubic-bezier\(([^)]+)\)/)
  const rail = signal(0)
  return { rail, bez }
})
"#;
    let r = p(src);
    let names: Vec<&str> = r.returned.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["rail", "bez"]);
    assert_eq!(r.member("rail").unwrap().kind, ScopeKind::Signal);
    assert_eq!(r.member("bez").unwrap().kind, ScopeKind::Action);
}

/// The other half of the ambiguity: a `/` that really is division must stay
/// division, or the scanner swallows the rest of the file as a "regex".
#[test]
fn division_is_still_division() {
    let src = r#"
export default controller((ctx) => {
  const half = derived(() => ctx.total() / 2)
  const n = signal(0)
  return { half, n }
})
"#;
    let r = p(src);
    let names: Vec<&str> = r.returned.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["half", "n"]);
    assert_eq!(r.member("n").unwrap().kind, ScopeKind::Signal);
}

/// `const a = signal(0), b = signal(1)` is ONE statement with TWO declarators.
/// Only the first used to register; `b` fell through to `Other`, so `(--b)`
/// was refused with a did-you-mean pointing at its own sibling.
#[test]
fn every_declarator_in_a_comma_list_classifies() {
    let src = r#"
export default controller((ctx) => {
  const a = signal(0), b = signal(1), c = derived(() => 2)
  return { a, b, c }
})
"#;
    let r = p(src);
    assert_eq!(r.member("a").unwrap().kind, ScopeKind::Signal);
    assert_eq!(r.member("b").unwrap().kind, ScopeKind::Signal);
    assert_eq!(r.member("c").unwrap().kind, ScopeKind::Derived);
}

/// Each declarator's value span must stay its OWN — an over-long span would
/// make `pure_derived` and the type inference read a sibling's initializer.
#[test]
fn declarator_spans_do_not_overrun_their_sibling() {
    let src = "export default controller((ctx) => {\n  const a = signal(0), b = signal(1)\n  return { a, b }\n})";
    let r = p(src);
    assert_eq!(r.slice(r.member("a").unwrap().value_span), "signal(0)");
    assert_eq!(r.slice(r.member("b").unwrap().value_span), "signal(1)");
}

#[test]
fn declarator_list_mixes_patterns_and_uninitialized_names() {
    let src = r#"
export default controller((ctx) => {
  let seen, { x } = point, n = signal(3)
  return { n, x }
})
"#;
    let r = p(src);
    assert_eq!(r.member("n").unwrap().kind, ScopeKind::Signal);
}
