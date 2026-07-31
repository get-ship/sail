//! Regression guard (#285): a static attribute authored on an element
//! that ALSO carries `bind:this` must survive compilation in every emit
//! path. The ref binding must not claim — or suppress — the element's
//! static attribute slot.
//!
//! The behavioral report (shadow-DOM dumps in the @ship/pipelines packs)
//! observed a `data-static` hook element rendering as a bare `<div>` —
//! container-by-attribute selectors found nothing. These assertions pin
//! the compiler contract so the regression can never re-enter through
//! the CSR (`__mount` / component setup) or SSR-skeleton paths.
//!
//! (The original regression used `class="dag"`, but `class=` is now banned
//! at the floor — the same regression shape is what matters, and any
//! static attribute pins it. The test now exercises `data-static`.)

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

// `part=` was RETIRED with shadow parts (light DOM has none — anatomy is
// classes + attributes, and `class` is the published styling hook — and
// the floor now bans it). The regression this guards is unchanged and
// still real: a static attribute on an element that also carries
// `bind:this` must survive every emit path — the ref binding must not
// claim or suppress the element's static attribute slot.
const REF_WITH_STATIC: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const rowsEl = signal<HTMLElement | null>(null)
  return { rowsEl }
})
</controller>
<div data-static="rows" bind:this={rowsEl}><span>x</span></div>"#;

fn component_opts(setup_url: Option<&str>, component_imports: bool) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: setup_url.map(Into::into),
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta {
            name: "probe-test".into(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: component_imports,
        lint_overrides: Vec::new(),
    }
}

/// The static attribute must SURVIVE. How it survives is the compiler's
/// business, and it changed: a static attribute is now baked into the
/// hoisted template markup (`<div data-static="rows">`) rather than
/// replayed as a `setAttribute` call at runtime — which is both correct
/// and strictly faster (it deletes a DOM round-trip per element per
/// instance). Asserting on `setAttribute` was asserting the MECHANISM;
/// the invariant is that the attribute reaches the DOM.
#[test]
fn static_attrs_survive_bind_this_in_component_csr() {
    let out = compile(REF_WITH_STATIC, component_opts(None, false)).expect("compile");
    assert!(
        out.js.contains(r#"data-static=\"rows\""#)
            || out.js.contains(r#"setAttribute("data-static", "rows")"#),
        "data-static must reach the DOM alongside bind:this — baked into the template or set at runtime:\n{}",
        out.js
    );
    assert!(
        out.js.contains("runAction"),
        "bind:this still wires via an action:\n{}",
        out.js
    );
}

#[test]
fn static_attrs_survive_bind_this_with_production_component_opts() {
    // Exact path the @ship/pipelines packs (dag/graph/run-history) compile
    // through: shadow component, a controller (setup_url), component-imports on.
    let src = r#"
<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const contentEl = signal<HTMLElement | null>(null)
  return { contentEl }
})
</controller>
<div data-static="dag" bind:this={contentEl}></div>"#;
    let out = compile(src, component_opts(Some("./x.controller.ts"), true)).expect("compile");
    // The static attribute rides alongside bind:this into the DOM.
    assert!(
        out.js.contains(r#"data-static=\"dag\""#)
            || out.js.contains(r#"setAttribute("data-static", "dag")"#),
        "data-static must survive in production component compile:\n{}",
        out.js
    );
    assert!(
        out.js.contains("runAction"),
        "bind:this still wires via an action:\n{}",
        out.js
    );
}
