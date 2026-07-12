//! `<error-boundary>` — render-fault isolation, as an element + `use:` action.
//!
//! `runProtected` routes a throw by walking up the DOM for a registered boundary
//! handle. A boundary hides a broken subtree rather than disposing one (the
//! protected content comes in through a `<slot>`, so it belongs to the enclosing
//! component's scope), which is why it needs no controller and no scope of its own.

use sui_compiler::{compile, compile_element, CompileMode, CompileOpts};

fn js(src: &str) -> String {
    compile(src, CompileOpts { mode: CompileMode::Component, ..Default::default() })
        .expect("compiles")
        .js
}

/// A healthy boundary must be invisible to layout. You wrap one around markup you
/// have already laid out, and it must not disturb it — get this wrong and every
/// boundary silently breaks a flex row.
#[test]
fn a_healthy_boundary_does_not_participate_in_layout() {
    let el = compile_element(
        include_str!("../stdlib/error-boundary.sui"),
        "error-boundary",
    )
    .expect("element compiles");

    assert!(
        el.skin_css.contains(".sui-error-boundary { display: contents; }")
            || el.skin_css.contains(".sui-error-boundary{display:contents;}"),
        "the root must not box: {}",
        el.skin_css
    );
    assert!(
        el.skin_css.contains("> .content { display: contents; }")
            || el.skin_css.contains(">.content{display:contents;}"),
        "nor may the content wrapper: {}",
        el.skin_css
    );
}

/// The branch swap is pure CSS, gated on `[data-caught]` — no signal, no `{#if}`.
#[test]
fn the_swap_is_a_css_gate_not_a_reactive_branch() {
    let el = compile_element(
        include_str!("../stdlib/error-boundary.sui"),
        "error-boundary",
    )
    .expect("element compiles");

    assert!(
        el.skin_css.contains("[data-caught]"),
        "the caught state is an attribute the CSS keys on: {}",
        el.skin_css
    );
    // Both halves of the swap must be gated, or a boundary renders its fallback
    // and its broken content at the same time.
    assert!(
        el.skin_css.contains(".sui-error-boundary > .fallback"),
        "the fallback is hidden until caught: {}",
        el.skin_css
    );
}

/// The fallback is replaceable: `slot="fallback"` overrides the built-in one.
/// Otherwise a boundary is only ever as good as our default.
#[test]
fn the_fallback_is_a_slot_with_a_built_in_default() {
    let el = compile_element(
        include_str!("../stdlib/error-boundary.sui"),
        "error-boundary",
    )
    .expect("element compiles");

    assert!(
        el.structure_html.contains(r#"<slot name="fallback">"#),
        "the fallback is an override point: {}",
        el.structure_html
    );
    assert!(
        el.structure_html.contains("class=\"retry\""),
        "and it ships a default with a recovery control: {}",
        el.structure_html
    );
    assert!(
        el.structure_html.contains("<slot></slot>"),
        "the protected subtree is the default slot: {}",
        el.structure_html
    );
}

/// The tag lowers away: it is reachable through the stdlib registry and costs no
/// custom element, only the action import.
#[test]
fn the_tag_lowers_and_auto_imports_its_action() {
    let out = js(r#"<error-boundary message="Panel failed."><p>content</p></error-boundary>"#);

    assert!(
        out.contains("sui-error-boundary"),
        "the tag lowers to the element's root class — it is registered:\n{out}"
    );
    assert!(
        out.contains("errorBoundary"),
        "and its behavior rides the auto-imported action:\n{out}"
    );
    assert!(
        !out.contains("customElements.define"),
        "a boundary is not a component — it defines no custom element:\n{out}"
    );
}
