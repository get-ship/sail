//! EVENT DELEGATION.
//!
//! `addEventListener` is not free. A 1,000-row table with two handlers per row costs
//! TWO THOUSAND listener registrations — each allocating a record in the browser — and
//! two thousand cleanup closures to tear them down again.
//!
//! Delegation installs ONE listener per event TYPE on the document and stores the
//! handler as a property on the node (`el.$$click`). Creating a row costs a property
//! assignment; REMOVING one costs nothing at all, because the property dies with the
//! node — no cleanup closure, no removeEventListener. Measured, 1,000 rows:
//!
//!     no handlers          2.2 ms
//!     el.$$click = fn      2.2 ms     <- free
//!     addEventListener     2.7 ms
//!
//! It is an OPTIMIZATION WITH A FALLBACK, not a second way to write a handler: the
//! author writes `on:click` either way and never sees the seam. What cannot be
//! delegated falls back to a real listener, and this file pins exactly what that is.

use sail_compiler::{CompileMode, CompileOpts, compile};

fn js(src: &str) -> String {
    compile(
        src,
        CompileOpts {
            element_prefix: "sail".into(),
            host_data_attr: "data-sail-host".into(),
            mode: CompileMode::Component,
            ..Default::default()
        },
    )
    .expect("compiles")
    .js
}

/// A plain bubbling handler is delegated: a property on the node, plus ONE root
/// listener hoisted to module scope.
#[test]
fn a_bubbling_handler_is_delegated() {
    let out = js(r#"<button on:click={go}>go</button>"#);
    assert!(
        out.contains("$$click = "),
        "the handler lives on the node:\n{out}"
    );
    assert!(
        out.contains(r#"delegateEvents(["click"])"#),
        "one root listener is installed for the type:\n{out}"
    );
    assert!(
        !out.contains("push(on("),
        "no per-element listener, and no cleanup slot:\n{out}"
    );
}

/// Each event TYPE is installed once, however many elements use it.
#[test]
fn one_root_listener_per_type_no_matter_how_many_handlers() {
    let out = js(r#"<div><button on:click={a}>a</button><button on:click={b}>b</button></div>"#);
    assert_eq!(out.matches("$$click = ").count(), 2, "two handlers:\n{out}");
    assert_eq!(
        out.matches("delegateEvents(").count(),
        1,
        "but ONE delegateEvents call:\n{out}"
    );
}

/// NON-BUBBLING events cannot be delegated — there is nothing at the root to catch.
/// `focus` is the classic trap (its bubbling twin `focusin` IS delegated).
#[test]
fn a_non_bubbling_event_falls_back_to_a_real_listener() {
    let out = js(r#"<input on:focus={f}>"#);
    assert!(
        out.contains("push(on(") && !out.contains("$$focus"),
        "`focus` does not bubble, so it must NOT be delegated:\n{out}"
    );

    let bubbling = js(r#"<input on:focusin={f}>"#);
    assert!(
        bubbling.contains("$$focusin = "),
        "`focusin` is the bubbling twin and IS delegated:\n{bubbling}"
    );
}

/// A MODIFIED handler keeps its own listener.
///
/// The modifier set is `prevent` / `stop` / `stopimmediate` / `self` / `trusted` /
/// `key`, and their semantics are implemented by the `on()` runtime. Several of them
/// (`stopimmediate` especially) mean something specific about listener ORDER on a
/// single element, which a delegated dispatch — where there is exactly one listener,
/// at the root — cannot faithfully reproduce. Rather than delegate the easy ones and
/// silently change the meaning of the hard ones, a modified handler keeps its real
/// listener. Delegation is an optimization; it may not alter semantics.
#[test]
fn a_modified_handler_keeps_its_own_listener() {
    let out = js(r#"<button on:click|stop={go}>go</button>"#);
    assert!(
        out.contains("push(on(") && !out.contains("$$click"),
        "a modified handler must keep its real listener:\n{out}"
    );
}

/// The delegated dispatch must reproduce bubbling — including the part authors rely
/// on, which is that `stopPropagation()` stops the handlers above. That is runtime
/// behavior (see delegate.ts); what the compiler must not do is delegate a handler
/// whose modifiers say otherwise, which the test above covers.
#[test]
fn a_mixed_template_delegates_what_it_can_and_no_more() {
    let out = js(r#"<div on:click={a}><input on:focus={b}><span on:mouseover={c}>x</span></div>"#);
    assert!(out.contains("$$click = "), "click delegates:\n{out}");
    assert!(
        out.contains("$$mouseover = "),
        "mouseover bubbles, so it delegates:\n{out}"
    );
    assert!(
        out.contains("push(on("),
        "focus does not, so it keeps a listener:\n{out}"
    );
    // ...and exactly the two delegated types are installed.
    assert!(
        out.contains(r#"delegateEvents(["click", "mouseover"])"#),
        "the root installs precisely the delegated types:\n{out}"
    );
}
