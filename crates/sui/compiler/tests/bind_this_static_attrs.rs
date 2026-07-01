//! Regression guard (#285): a static `class` / `part` attribute authored
//! on an element that ALSO carries `bind:this` must survive compilation
//! in every emit path. The ref binding must not claim — or suppress —
//! the element's static attribute slot.
//!
//! The behavioral report (shadow-DOM dumps in the @ship/pipelines packs)
//! observed `<div class="dag" bind:this={contentEl}>` rendering as a bare
//! `<div>` — container-by-class selectors found nothing. These assertions
//! pin the compiler contract so the regression can never re-enter through
//! the CSR (`__mount` / component setup) or SSR-skeleton paths.

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

const REF_WITH_STATIC: &str =
    r#"<div class="rows" part="rows" bind:this={rowsEl}><span>x</span></div>"#;

fn component_opts(setup_url: Option<&str>, component_imports: bool) -> CompileOpts {
    CompileOpts {
        setup_url: setup_url.map(Into::into),
        mode: CompileMode::Component,
        source_path: None,
        styles: None,
        component_meta: Some(ComponentMeta { name: "probe-test".into(), shadow: true }),
        emit_dts: false,
        emit_component_imports: component_imports,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn static_attrs_survive_bind_this_in_component_csr() {
    let out = compile(REF_WITH_STATIC, component_opts(None, false)).expect("compile");
    assert!(
        out.js.contains(r#"setAttribute("class", "rows")"#),
        "class must be emitted alongside bind:this:\n{}",
        out.js
    );
    assert!(
        out.js.contains(r#"setAttribute("part", "rows")"#),
        "part must be emitted alongside bind:this:\n{}",
        out.js
    );
}

#[test]
fn static_attrs_survive_bind_this_with_production_component_opts() {
    // Exact path the @ship/pipelines packs (dag/graph/run-history) compile
    // through: shadow component, a controller (setup_url), component-imports on.
    let src = r#"<div class="dag" part="dag" bind:this={contentEl}></div>"#;
    let out = compile(src, component_opts(Some("./x.controller.ts"), true)).expect("compile");
    assert!(
        out.js.contains(r#"setAttribute("class", "dag")"#),
        "class must survive in production component compile:\n{}",
        out.js
    );
    assert!(
        out.js.contains(r#"setAttribute("part", "dag")"#),
        "part must survive in production component compile:\n{}",
        out.js
    );
}
