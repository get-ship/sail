//! `<error-boundary>` — RENDER-FAULT ISOLATION, as an L1 element.
//!
//! The runtime has always had the catch machinery: `attachBoundary` /
//! `findNearestBoundary` / `runProtected`, called from component mount, `bindIf`,
//! and the `{#each}` row renderer. What it did NOT have was an element to find.
//! `findNearestBoundary` walks up the DOM looking for a host carrying a handle;
//! with no such host in the tree it returns null and every caught error degrades
//! to a `console.error`. The mechanism was live and unreachable.
//!
//! It used to be reachable through a COMPONENT — a controller holding a
//! `caughtError` signal with `{#if caught}fallback{:else}<slot/>{/if}`. The only
//! reason to pay for a controller would be REAL unmount: disposing the broken
//! subtree's reactive scope. It never did that. The protected content arrives
//! through a `<slot>`, so it belongs to the PARENT's scope; swapping the branch
//! merely stopped projecting it. Nothing was disposed, and the custom element,
//! the signal, and the scope bought nothing an action cannot do.
//!
//! So it is an element, and this file pins the three things that makes true.

use sui_compiler::{compile, compile_element, CompileMode, CompileOpts};

fn js(src: &str) -> String {
    compile(src, CompileOpts { mode: CompileMode::Component, ..Default::default() })
        .expect("compiles")
        .js
}

/// A HEALTHY boundary must be INVISIBLE TO LAYOUT.
///
/// This is the invariant that makes the element usable at all: you wrap a
/// boundary around markup you have ALREADY laid out, and it must not disturb it.
/// `display: contents` on both the root and the content wrapper means the
/// protected children lay out against the boundary's parent exactly as if the
/// boundary were not in the tree. Get this wrong and every boundary silently
/// breaks a flex row.
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

/// The BRANCH SWAP is pure CSS, gated on `[data-caught]` — there is no signal and
/// no `{#if}`. That is the whole argument for demoting this from a component: the
/// reactive branch was never buying disposal, so a CSS gate does the same job for
/// no JS at all.
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

/// The fallback is REPLACEABLE. An author who supplies `slot="fallback"` gets
/// their markup instead of the built-in one — otherwise a boundary is only ever
/// as good as our default, and nobody would use it in a real design.
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

/// Using the tag from a component lowers it away — the element is REACHABLE
/// through the stdlib registry (the thing that was missing) and costs no custom
/// element, only the action import.
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
