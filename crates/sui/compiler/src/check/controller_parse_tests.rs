//! Unit tests for `controller_parse`. Lives in a sibling module to
//! keep the implementation file under the 300-line cap.

use std::path::PathBuf;

use super::controller_parse::{parse, ParsedController};

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
fn extracts_simple_sections() {
    let src = r#"
import { controller } from '@ship/sui-runtime'
export default controller<Scope>({
  derived: { x: () => 1 },
  on: { go: (s) => {} },
})
"#;
    let r = p(src);
    let names: Vec<&str> = r.sections.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["derived", "on"]);
}

#[test]
fn flags_unknown_section_for_caller() {
    let src = r#"
export default controller({
  derived: {},
  effects: {},
})
"#;
    let r = p(src);
    let names: Vec<&str> = r.sections.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["derived", "effects"]);
}

#[test]
fn handles_method_shorthand_section() {
    let src = r#"
export default controller({
  mount(s, host) { host.x = 1 },
  unmount(s) {}
})
"#;
    let r = p(src);
    let names: Vec<&str> = r.sections.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["mount", "unmount"]);
}

#[test]
fn ignores_keys_inside_nested_literal() {
    let src = r#"
export default controller({
  derived: {
    pretender: () => ({ effects: 1 }),
  },
})
"#;
    let r = p(src);
    let names: Vec<&str> = r.sections.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["derived"]);
}

#[test]
fn slices_value_span() {
    let src = r#"export default controller({ derived: { x: () => 1 } })"#;
    let r = p(src);
    let s = r.section("derived").unwrap();
    let value = r.slice(s.value_span);
    assert!(value.starts_with('{'));
    assert!(value.ends_with('}'));
}
