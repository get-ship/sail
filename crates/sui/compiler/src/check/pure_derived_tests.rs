//! Unit tests for `pure_derived`. Sibling module so the implementation
//! file stays under the 300-line cap.

use std::path::PathBuf;

use super::controller_parse::{parse, ParsedController};
use super::pure_derived::check;

fn parse_src(src: &str) -> ParsedController {
    parse(PathBuf::from("t.ts"), src.into()).expect("controller call expected")
}

#[test]
fn allows_pure_arrow_body() {
    let src = r#"
export default controller({
  derived: {
    items: ({ agents }) => agents.value?.items ?? [],
    total: (s) => s.items.length,
  },
})
"#;
    check(&parse_src(src)).expect("pure derived should pass");
}

#[test]
fn rejects_date_now() {
    let src = r#"
export default controller({
  derived: { now: () => Date.now() },
})
"#;
    let err = check(&parse_src(src)).expect_err("Date.now() should fail");
    assert!(err.message.contains("Derived 'now'"));
    assert!(err.message.contains("Date.now"));
}

#[test]
fn rejects_math_random() {
    let src = r#"
export default controller({
  derived: { r: () => Math.random() },
})
"#;
    let err = check(&parse_src(src)).expect_err("Math.random() should fail");
    assert!(err.message.contains("Math.random"));
}

#[test]
fn rejects_await() {
    let src = r#"
export default controller({
  derived: { x: async (s) => await s.thing() },
})
"#;
    let err = check(&parse_src(src)).expect_err("await should fail");
    assert!(err.message.contains("await"));
}

#[test]
fn rejects_ship_call() {
    let src = r#"
export default controller({
  derived: { x: ({ ship }) => ship.git.overview() },
})
"#;
    let err = check(&parse_src(src)).expect_err("ship.* should fail");
    assert!(err.message.contains("ship"));
}

#[test]
fn ignores_string_with_forbidden_word() {
    let src = r#"
export default controller({
  derived: { tag: () => "Date.now is fine inside a string" },
})
"#;
    check(&parse_src(src)).expect("string contents should not match");
}

#[test]
fn does_not_match_method_named_date_nowish() {
    let src = r#"
export default controller({
  derived: { x: (s) => s.dateNow },
})
"#;
    check(&parse_src(src)).expect("dateNow != Date.now");
}

#[test]
fn rejects_method_shorthand_body() {
    let src = r#"
export default controller({
  derived: { x(s) { return Date.now() } },
})
"#;
    check(&parse_src(src)).expect_err("method-shorthand body should also be scanned");
}
