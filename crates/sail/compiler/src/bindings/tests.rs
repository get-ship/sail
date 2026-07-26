//! Unit tests for the binding-manifest pre-pass.

use super::*;
use crate::keys::BindingKind;
use crate::parse::parse;

fn build_for(src: &str) -> BindingRegistry {
    let program = parse(src).expect("parse");
    build(&program)
}

#[test]
fn dynamic_text_emits_text_binding() {
    let r = build_for("<p>{x}</p>");
    assert_eq!(r.len(), 1);
    let b = r.get(0).unwrap();
    assert_eq!(b.key, "b0");
    assert_eq!(b.kind, BindingKind::Text);
    assert_eq!(b.expr, "x");
    assert_eq!(b.deps, vec!["x".to_string()]);
}

#[test]
fn event_handler_emits_on_binding() {
    let r = build_for("<button on:click={incr}></button>");
    assert_eq!(r.len(), 1);
    let b = r.get(0).unwrap();
    assert_eq!(b.kind, BindingKind::On("click".into()));
    assert_eq!(b.expr, "incr");
    assert_eq!(b.deps, vec!["incr".to_string()]);
}

#[test]
fn bind_value_emits_bind_value_binding() {
    let r = build_for("<input bind:value={query} />");
    assert_eq!(r.len(), 1);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::BindValue);
    assert_eq!(r.get(0).unwrap().deps, vec!["query".to_string()]);
}

#[test]
fn bind_checked_emits_bind_checked_binding() {
    let r = build_for("<input type=\"checkbox\" bind:checked={on} />");
    assert_eq!(r.len(), 1);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::BindChecked);
}

#[test]
fn attribute_interp_emits_attr_binding() {
    let r = build_for("<a href={u}>x</a>");
    assert_eq!(r.len(), 1);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::Attr("href".into()));
}

#[test]
fn class_directive_is_rejected_at_parse_time() {
    // `class:` has been deleted from the language — same floor as bare
    // `class=`. Styling is sx / named use: actions; state is aria-* /
    // closed sx prefixes / layout. The compiler refuses so the rejection
    // names the replacement.
    let err = crate::parse::parse("<p class:active={on}>x</p>").unwrap_err();
    let msg = format!("{err:?}");
    assert!(
        msg.contains("class:") && (msg.contains("removed") || msg.contains("retired")),
        "the rejection should name class: retirement; got: {msg}"
    );
}

#[test]
fn style_directive_is_rejected_at_parse_time() {
    // `style:` has been deleted from the language. Runtime geometry is a
    // named, typed action (`use:virtualRow`, `use:eventSpan`, …); the
    // compiler refuses the form so the rejection names the replacement
    // rather than silently compiling to an untokened write.
    let err = crate::parse::parse("<p style:color={c}>x</p>").unwrap_err();
    let msg = format!("{err:?}");
    assert!(
        msg.contains("style:") && msg.contains("removed"),
        "the rejection should name the mechanism that replaced style:; got: {msg}"
    );
}

#[test]
fn prop_and_attr_prefixes_are_rejected() {
    // Author surface no longer has prop:/attr:; plain fields only.
    assert!(crate::parse::parse("<a prop:href={x}>x</a>").is_err());
    assert!(crate::parse::parse("<a attr:href={x}>x</a>").is_err());
}

#[test]
fn if_emits_if_binding_then_walks_body() {
    let r = build_for("<if cond={x}><p>{x}</p></if>");
    assert_eq!(r.len(), 2);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::If);
    assert_eq!(r.get(0).unwrap().expr, "x");
    assert_eq!(r.get(1).unwrap().kind, BindingKind::Text);
}

#[test]
fn if_walks_else_body() {
    let r = build_for("<if cond={a}><p>{a}</p><else><p>{b}</p></else></if>");
    // bindings: if, text(a), text(b)
    assert_eq!(r.len(), 3);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::If);
    assert_eq!(r.get(1).unwrap().expr, "a");
    assert_eq!(r.get(2).unwrap().expr, "b");
}

#[test]
fn for_iterable_emits_for_binding() {
    let r = build_for(r#"<for as="n" of={xs}><li>{n}</li></for>"#);
    assert_eq!(r.len(), 2);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::For);
    assert_eq!(r.get(0).unwrap().expr, "xs");
    assert_eq!(r.get(1).unwrap().kind, BindingKind::Text);
}

#[test]
fn for_body_yields_one_binding_not_per_iteration() {
    // Even if the body has many bindings, the registry walks it once.
    let r = build_for(r#"<for as="n" of={xs}><li aria-current={n}>{n}</li></for>"#);
    // for boundary, aria-current attr binding, dynamic text — 3 entries total.
    assert_eq!(r.len(), 3);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::For);
    assert_eq!(r.get(1).unwrap().kind, BindingKind::Attr("aria-current".into()));
    assert_eq!(r.get(2).unwrap().kind, BindingKind::Text);
}

#[test]
fn match_emits_match_binding_and_walks_cases() {
    let r = build_for(
        "<match on={k}><case of={'a'}><p>{k}</p></case><case of={'b'}><p>B</p></case><default><p>D</p></default></match>",
    );
    // match boundary, text(k), no bindings in case 'b' or default.
    assert_eq!(r.len(), 2);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::Match);
    assert_eq!(r.get(0).unwrap().expr, "k");
    assert_eq!(r.get(1).unwrap().kind, BindingKind::Text);
}

#[test]
fn nested_control_flow_walks_outer_then_inner() {
    let r = build_for(r#"<if cond={x}><for as="n" of={xs}><li>{n}</li></for></if>"#);
    // bindings: if, for, text
    assert_eq!(r.len(), 3);
    assert_eq!(r.get(0).unwrap().kind, BindingKind::If);
    assert_eq!(r.get(1).unwrap().kind, BindingKind::For);
    assert_eq!(r.get(2).unwrap().kind, BindingKind::Text);
}

#[test]
fn keys_are_sequential_b_strings() {
    let r = build_for("<p>{a}-{b}</p>");
    let keys: Vec<&str> = r.iter().map(|b| b.key.as_str()).collect();
    assert_eq!(keys, vec!["b0", "b1"]);
}

#[test]
fn deps_collect_root_idents_only() {
    // `agents().items.length` should resolve to deps=["agents"].
    let r = build_for("<p>{agents().items.length}</p>");
    assert_eq!(r.get(0).unwrap().deps, vec!["agents".to_string()]);
}

#[test]
fn deps_skip_method_names() {
    // `count.set(1)` reads `count` only, not `set`.
    let r = build_for("<button on:click={count.set(1)}></button>");
    assert_eq!(r.get(0).unwrap().deps, vec!["count".to_string()]);
}

#[test]
fn arrow_handler_dep_walk_uses_arrow_parser() {
    // Arrow handlers are parsed with arrows enabled. The body reads `count`
    // (twice — dedup'd); `set` is a property name and must not appear.
    let r = build_for("<button on:click={() => count.set(count() + 1)}></button>");
    let deps = &r.get(0).unwrap().deps;
    assert!(deps.contains(&"count".to_string()), "deps={deps:?}");
    // `set` is after a `.` and must not appear as a root dep.
    assert!(!deps.contains(&"set".to_string()), "deps={deps:?}");
}

#[test]
fn empty_program_yields_empty_registry() {
    let r = build_for("");
    assert_eq!(r.len(), 0);
    assert!(r.is_empty());
}
