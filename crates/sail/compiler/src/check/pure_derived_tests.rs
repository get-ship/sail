//! Unit tests for `pure_derived`. Sibling module so the implementation
//! file stays under the 300-line cap.

use std::path::PathBuf;

use super::controller_parse::{ParsedController, parse};
use super::pure_derived::check;

fn parse_src(src: &str) -> ParsedController {
    parse(PathBuf::from("t.ts"), src.into()).expect("controller call expected")
}

#[test]
fn allows_pure_arrow_body() {
    let src = r#"
export default controller((ctx) => {
  const items = derived(({ agents }) => agents.value?.items ?? [])
  const total = derived((s) => s.items.length)
  return { items, total }
})
"#;
    check(&parse_src(src)).expect("pure derived should pass");
}

#[test]
fn rejects_date_now() {
    let src = r#"
export default controller((ctx) => {
  const now = derived(() => Date.now())
  return { now }
})
"#;
    let err = check(&parse_src(src)).expect_err("Date.now() should fail");
    assert!(err.message.contains("Derived 'now'"));
    assert!(err.message.contains("Date.now"));
}

#[test]
fn rejects_math_random() {
    let src = r#"
export default controller((ctx) => {
  const r = derived(() => Math.random())
  return { r }
})
"#;
    let err = check(&parse_src(src)).expect_err("Math.random() should fail");
    assert!(err.message.contains("Math.random"));
}

#[test]
fn rejects_await() {
    let src = r#"
export default controller((ctx) => {
  const x = derived(async (s) => await s.thing())
  return { x }
})
"#;
    let err = check(&parse_src(src)).expect_err("await should fail");
    assert!(err.message.contains("await"));
}

#[test]
fn rejects_ship_call() {
    let src = r#"
export default controller(({ ship }) => {
  const x = derived(() => ship.git.overview())
  return { x }
})
"#;
    let err = check(&parse_src(src)).expect_err("ship.* should fail");
    assert!(err.message.contains("ship"));
}

#[test]
fn ignores_string_with_forbidden_word() {
    let src = r#"
export default controller((ctx) => {
  const tag = derived(() => "Date.now is fine inside a string")
  return { tag }
})
"#;
    check(&parse_src(src)).expect("string contents should not match");
}

#[test]
fn does_not_match_method_named_date_nowish() {
    let src = r#"
export default controller((ctx) => {
  const x = derived((s) => s.dateNow)
  return { x }
})
"#;
    check(&parse_src(src)).expect("dateNow != Date.now");
}
