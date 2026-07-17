//! Unit tests for `controller_parse`. Lives in a sibling module to
//! keep the implementation file under the 300-line cap.

use std::path::PathBuf;

use super::controller_parse::{parse, ParsedController, ScopeKind};

fn p(src: &str) -> ParsedController {
    parse(PathBuf::from("test.ts"), src.into()).expect("expected controller call")
}

#[test]
fn ignores_legacy_setup_shape() {
    let r = parse(
        PathBuf::from("legacy.ts"),
        "export default function setup() { return {} }".into(),
    );
    assert!(r.is_none());
}

#[test]
fn ignores_null_default() {
    let r = parse(PathBuf::from("n.ts"), "export default null\n".into());
    assert!(r.is_none());
}

#[test]
fn extracts_block_body_returned_members() {
    let src = r#"
import { controller, derived } from '@ship/sui-runtime'
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
