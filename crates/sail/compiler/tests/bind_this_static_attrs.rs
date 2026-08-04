//! Regression guard (#285): a static attribute authored on an element
//! that ALSO carries an element-capture `use:` action must survive
//! compilation in every emit path. The capture must not claim — or
//! suppress — the element's static attribute slot.
//!
//! (Historically this used `bind:this`; that directive is retired in
//! favour of a local `use:` action with the same lifecycle.)

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

const REF_WITH_STATIC: &str = r#"<controller>
import { controller, signal, type Action } from '@ship/sail-runtime'
export default controller((_ctx) => {
  const rowsEl = signal<HTMLElement | null>(null)
  const captureRows: Action<void> = (node) => {
    rowsEl.set(node as HTMLElement)
    return { destroy: () => rowsEl.set(null) }
  }
  return { captureRows }
})
</controller>
<div data-static="rows" use:captureRows><span>x</span></div>"#;

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

#[test]
fn static_attrs_survive_capture_action_in_component_csr() {
    let out = compile(REF_WITH_STATIC, component_opts(None, false)).expect("compile");
    assert!(
        out.js.contains(r#"data-static=\"rows\""#)
            || out.js.contains(r#"setAttribute("data-static", "rows")"#),
        "data-static must reach the DOM alongside use: capture — baked into the template or set at runtime:\n{}",
        out.js
    );
    assert!(
        out.js.contains("runAction"),
        "use: capture still wires via an action:\n{}",
        out.js
    );
}

#[test]
fn static_attrs_survive_capture_action_with_production_component_opts() {
    let src = r#"
<controller>
import { controller, signal, type Action } from '@ship/sail-runtime'
export default controller((_ctx) => {
  const contentEl = signal<HTMLElement | null>(null)
  const captureContent: Action<void> = (node) => {
    contentEl.set(node as HTMLElement)
    return { destroy: () => contentEl.set(null) }
  }
  return { captureContent }
})
</controller>
<div data-static="dag" use:captureContent></div>"#;
    let out = compile(src, component_opts(Some("./x.controller.ts"), true)).expect("compile");
    assert!(
        out.js.contains(r#"data-static=\"dag\""#)
            || out.js.contains(r#"setAttribute("data-static", "dag")"#),
        "data-static must survive in production component compile:\n{}",
        out.js
    );
    assert!(
        out.js.contains("runAction"),
        "use: capture still wires via an action:\n{}",
        out.js
    );
}

#[test]
fn bind_this_is_retired_with_use_migration() {
    let src = r#"<button bind:this={btnEl}>x</button>"#;
    let err = compile(src, component_opts(None, false)).expect_err("bind:this must refuse");
    assert!(
        err.message.contains("retired") && err.message.contains("use:"),
        "expected bind:this retirement with use: hint, got: {}",
        err.message
    );
}
