//! TEMPLATE CLONING — the compiler's biggest lever on DOM creation.
//!
//! A `createElement` chain costs one JS->DOM round-trip per node AND per attribute.
//! A `cloneNode` costs one, total: the browser copies an already-parsed structure in
//! C++. Measured on the js-framework-benchmark row, 1,000 rows:
//!
//!     createElement chain   7.2 ms
//!     template cloneNode    3.4 ms     2.1x
//!
//! That 2.1x was the entire creation gap against Svelte and Solid — both of whom do
//! this, and we did not. On the benchmark it took replace1k from 92ms to 55ms.
//!
//! Two hazards this pins:
//!
//!   * NAVIGATION must use firstChild/nextSibling, NOT `childNodes[i]`. `childNodes`
//!     is a live NodeList; at ~6 lookups per row x 10,000 rows its overhead ate the
//!     entire cloning win (create10k script went UP, 165ms -> 194ms, with the clone
//!     already in place). Switching to pointer walks took it to 117ms.
//!
//!   * ADJACENT TEXT MERGES. `<td>Hello {name}</td>` serializes to `<td>Hello  </td>`,
//!     which the parser collapses into ONE text node — the binding would have no node
//!     of its own and every index after it would shift. Such an element falls back to
//!     the createElement path rather than emitting silently-wrong navigation.
//!
//! Note: the original tests used `class=` to mark elements; with the floor now
//! banning `class=`, the markup hooks here are `data-test` (the standard test
//! selector the rest of the suite uses). The assertions are about what the
//! compiler emits, not about class=.

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn js(src: &str) -> String {
    compile(src, CompileOpts { mode: CompileMode::Component, ..Default::default() })
        .expect("compiles")
        .js
}

/// A static subtree is hoisted to module scope and cloned — not built node by node.
#[test]
fn a_static_subtree_is_hoisted_and_cloned() {
    let out = js(r#"<div data-actions><button data-test="go">go</button></div>"#);
    assert!(out.contains("__tpl("), "the markup is hoisted as a template:\n{out}");
    assert!(
        out.contains(r#"<button data-test=\"go\">go</button>"#),
        "static attributes bake into the markup — which also deletes the setAttribute call:\n{out}"
    );
    assert!(!out.contains("createElement"), "nothing is built by hand:\n{out}");
    assert!(!out.contains("setAttribute"), "no attribute is replayed at runtime:\n{out}");
}

/// The cloner lives at MODULE scope, so the markup is parsed once per program — not
/// once per component instance, and certainly not once per row.
#[test]
fn the_cloner_is_hoisted_above_the_component() {
    let out = js(r#"<div data-x><span>y</span></div>"#);
    let tpl = out.find("const __tpl0").expect("a cloner is emitted");
    let comp = out.find("component(").expect("the component is emitted");
    assert!(tpl < comp, "the cloner must be hoisted ABOVE the component:\n{out}");
}

/// Bindings navigate the clone with pointer walks. `childNodes[i]` is a live NodeList
/// and its overhead is what ate the cloning win the first time round.
#[test]
fn bindings_navigate_with_pointers_not_childnodes() {
    let out = js(r#"<div><button on:click={go}>go</button><span>{label()}</span></div>"#);
    assert!(out.contains(".firstChild"), "navigates by pointer:\n{out}");
    assert!(
        !out.contains("childNodes["),
        "childNodes is a LIVE NodeList — its lookup cost erases the win:\n{out}"
    );
}

/// A dynamic text slot becomes a single space in the markup, so the parser creates a
/// text node for the binding to own. (Exactly what Svelte emits, for the same reason.)
#[test]
fn a_dynamic_text_slot_reserves_a_text_node() {
    let out = js(r#"<span>{label()}</span>"#);
    assert!(
        out.contains(r#"__tpl("<span> </span>")"#),
        "the placeholder space forces a text node into the clone:\n{out}"
    );
    assert!(out.contains("bindText"), "and the binding still wires to it:\n{out}");
}

/// ADJACENT TEXT would merge on parse. Rather than emit navigation that silently
/// points at the wrong node, such an element falls back to building by hand.
#[test]
fn adjacent_text_falls_back_rather_than_emitting_wrong_navigation() {
    let out = js(r#"<span>Hello {name()}</span>"#);
    assert!(
        out.contains("createTextNode"),
        "static text + dynamic text adjacent => two nodes that HTML would merge into \
         one, so this must NOT be templated:\n{out}"
    );
}

/// Control flow needs a runtime anchor, so an element containing it is not templated.
/// Its static SIBLINGS still are — the fallback is per-element, not per-file.
#[test]
fn control_flow_is_not_templated_but_its_siblings_still_are() {
    let out = js(r#"<div><p data-test="a">static</p><ul>{#if ok()}<li>x</li>{/if}</ul></div>"#);
    assert!(out.contains("__tpl("), "the static <li> subtree is still hoisted:\n{out}");
    assert!(out.contains("bindIf"), "and the control flow still wires:\n{out}");
}

/// Model-A stamps `sui-root` on the component's root element. It is baked into the
/// markup, not added with `classList.add` at runtime.
///
/// (Needs a real tag: a tagless source is a mount-form view, and Model-A's root
/// convention is a COMPONENT convention — the host is the transparent pass-through.)
#[test]
fn the_model_a_root_class_bakes_into_the_markup() {
    let out = compile(
        r#"<div data-card="card">x</div>"#,
        CompileOpts {
            mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "tpl-probe".into() }),
            ..Default::default()
        },
    )
    .expect("compiles")
    .js;
    // The root class is framework-stamped and joins the author's data-* attribute.
    assert!(
        out.contains(r#"class=\"sui-root\""#),
        "sui-root bakes into the markup, alongside the author's data-card attribute:\n{out}"
    );
    assert!(!out.contains("classList.add"), "not stamped at runtime:\n{out}");
}
