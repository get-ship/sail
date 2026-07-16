//! Fixture-based tests for the `.sui.d.ts` declaration emitter.
//!
//! Each test compiles a small `.sui` snippet with `emit_dts: true` and
//! asserts the resulting d.ts contains the expected type lines.

use std::fs;
use sui_compiler::{compile, CompileMode, CompileOpts};

fn compile_dts(source: &str) -> String {
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: true,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    compile(source, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

/// Compile a `.sui` source whose controller is inlined as a `<controller>`
/// block — the ONE source (the paired `.controller.ts` fallback is gone) — so
/// the d.ts emitter can pull the scope factory's fields. Returns the d.ts text.
fn compile_dts_with_controller(_stem: &str, sui_src: &str, controller_ts: &str) -> String {
    let src = format!("<controller>\n{controller_ts}\n</controller>\n{sui_src}");
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: true,
        emit_component_imports: false,

        lint_overrides: Vec::new(),
    };
    compile(&src, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

fn compile_component_dts(source: &str) -> String {
    use sui_compiler::ComponentMeta;
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta { name: "test-el".into(), shadow: false }),
        emit_dts: true,
        emit_component_imports: false,
    
            lint_overrides: Vec::new(),};
    compile(source, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

#[test]
fn data_live_emits_live_state() {
    let dts = compile_dts(r#"<data name="agents" source="ship.agents.list()" live />"#);
    assert!(dts.contains("agents: Signal<LiveState<unknown>>"), "got: {dts}");
    assert!(dts.contains("import type { Signal, LiveState }"), "got: {dts}");
}

#[test]
fn data_once_emits_signal_unknown() {
    let dts = compile_dts(r#"<data name="config" source="ship.config.get()" />"#);
    assert!(dts.contains("config: Signal<unknown>"), "got: {dts}");
    assert!(!dts.contains("LiveState"), "got: {dts}");
}

#[test]
fn prop_union_emits_union_type() {
    let dts = compile_component_dts(
        r#"<component>{ props: { tone: 'primary' | 'neutral' } }</component><span>x</span>"#,
    );
    assert!(
        dts.contains("tone: Signal<'primary' | 'neutral'>"),
        "got: {dts}"
    );
}

#[test]
fn prop_bool_emits_boolean() {
    let dts = compile_component_dts(
        r#"<component>{ props: { loading: false } }</component><span>x</span>"#,
    );
    assert!(dts.contains("loading: Signal<boolean>"), "got: {dts}");
}

#[test]
fn use_hook_with_as_emits_return_type() {
    let dts = compile_dts(
        r#"<data name="items" source="ship.list()" />
<use-filter-list as="filter"><span>x</span></use-filter-list>"#,
    );
    assert!(
        dts.contains("filter: ReturnType<typeof useFilterList>"),
        "got: {dts}"
    );
}

#[test]
fn emit_dts_false_returns_none() {
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
    
            lint_overrides: Vec::new(),};
    let out = compile(r#"<p>hi</p>"#, opts).expect("compile");
    assert!(out.dts.is_none(), "expected no dts when emit_dts=false");
}

#[test]
fn no_declarations_emits_empty_scope() {
    let dts = compile_dts("<p>Hello world</p>");
    assert!(dts.contains("export interface Scope {}"), "got: {dts}");
}

#[test]
fn bind_this_on_button_emits_html_button_element_ref() {
    let dts = compile_dts(r#"<button bind:this={btnEl}>x</button>"#);
    assert!(
        dts.contains("btnEl: Signal<HTMLButtonElement | null>"),
        "got: {dts}"
    );
}

#[test]
fn bind_this_on_input_emits_html_input_element_ref() {
    let dts = compile_dts(r#"<input bind:this={field} />"#);
    assert!(
        dts.contains("field: Signal<HTMLInputElement | null>"),
        "got: {dts}"
    );
}

#[test]
fn bind_this_on_custom_element_falls_back_to_html_element() {
    let dts = compile_dts(r#"<sui-button bind:this={el}>x</sui-button>"#);
    assert!(
        dts.contains("el: Signal<HTMLElement | null>"),
        "got: {dts}"
    );
}

#[test]
fn bind_this_on_div_emits_html_div_element_ref() {
    let dts = compile_dts(r#"<div bind:this={panel}>x</div>"#);
    assert!(
        dts.contains("panel: Signal<HTMLDivElement | null>"),
        "got: {dts}"
    );
}

#[test]
fn reflect_empty_does_not_add_scope_entry() {
    // The slot lives inside a component since slots are component-only.
    // The reflect-empty side-effect must not surface in Scope.
    let dts = compile_component_dts(
        r#"<button><slot reflect-empty="data-icon-only"></slot></button>"#,
    );
    assert!(dts.contains("export interface Scope {}"), "got: {dts}");
    assert!(!dts.contains("data-icon-only"), "got: {dts}");
    assert!(!dts.contains("dataIconOnly"), "got: {dts}");
}

#[test]
fn scope_factory_fields_omitted_from_scope() {
    // Scope-factory fields are typed solely by the controller's inferred SC.
    // Re-declaring them in Scope put the same key in both Ctx and SC, so
    // `$signals.x` collapsed to `unknown` — the root cause of the state casts
    // (#121/#122). They are OMITTED from Scope now. Template expression reads
    // (`{foo()}`) do not add Scope members; only `<data>`/props/use/ref do.
    let sui = r#"<p>{foo()} {label()}</p>"#;
    let controller = r#"
        import { controller, signal } from '@ship/sui-runtime'
        export default controller({
          setup: () => ({
            foo: signal<number>(0),
            label: signal(''),
          }),
        })
    "#;
    let dts = compile_dts_with_controller("page", sui, controller);
    assert!(
        dts.contains("export interface Scope {}"),
        "factory-only scope over an empty template must yield an empty Scope, got: {dts}"
    );
    assert!(
        !dts.contains("foo:") && !dts.contains("label:"),
        "factory fields must NOT be re-declared in Scope, got: {dts}"
    );
    assert!(
        !dts.contains("// From controller scope"),
        "factory section header must be gone, got: {dts}"
    );
}

#[test]
fn scope_factory_shorthand_fields_omitted_from_scope() {
    // Block-body factory with shorthand return (the sui-error-boundary pattern)
    // — same omit rule: caughtError/reset are SC-typed, absent from Scope.
    let sui = r#"<p>x</p>"#;
    let controller = r#"
        import { controller, signal } from '@ship/sui-runtime'
        export default controller({
          setup: (_b: unknown, host?: Element) => {
            const caughtError = signal<Error | null>(null)
            const reset = (): void => { caughtError.set(null); host?.removeAttribute('x') }
            return { caughtError, reset }
          },
        })
    "#;
    let dts = compile_dts_with_controller("page", sui, controller);
    assert!(dts.contains("export interface Scope {}"), "got: {dts}");
    assert!(
        !dts.contains("caughtError:") && !dts.contains("reset:"),
        "factory fields must be omitted from Scope, got: {dts}"
    );
}

#[test]
fn empty_scope_factory_leaves_scope_unchanged() {
    let sui = r#"<p>x</p>"#;
    let controller = r#"
        import { controller } from '@ship/sui-runtime'
        export default controller({ setup: () => ({}) })
    "#;
    let dts = compile_dts_with_controller("page", sui, controller);
    assert!(
        dts.contains("export interface Scope {}"),
        "expected empty Scope when factory adds nothing, got: {dts}"
    );
    assert!(
        !dts.contains("// From controller setup: factory"),
        "should not emit the factory header when factory is empty, got: {dts}"
    );
}

#[test]
fn no_scope_section_leaves_scope_unchanged() {
    // Controller without a `setup:` section — falls back to template-derived
    // Scope only. Existing controllers must continue typechecking.
    let sui = r#"<p>x</p>"#;
    let controller = r#"
        import { controller } from '@ship/sui-runtime'
        export default controller({
          mount: (_state, _host) => {},
        })
    "#;
    let dts = compile_dts_with_controller("page", sui, controller);
    assert!(
        dts.contains("export interface Scope {}"),
        "expected empty Scope when no scope: section, got: {dts}"
    );
}

#[test]
fn template_member_clashing_with_factory_is_dropped_from_scope() {
    // A name in BOTH a template `<data>` AND the factory: the template-derived
    // `Signal<unknown>` is dropped (clash-retain) and the factory type is NOT
    // re-declared (omit) — so `config` is absent from Scope entirely and typed
    // solely by the inferred SC. (Was: factory type re-declared in Scope.)
    let sui = r#"<data name="config" source="ship.config.get()" /><p>x</p>"#;
    let controller = r#"
        import { controller, signal } from '@ship/sui-runtime'
        export default controller({
          setup: () => ({ config: signal<{ url: string }>({ url: '' }) }),
        })
    "#;
    let dts = compile_dts_with_controller("page", sui, controller);
    assert!(
        !dts.contains("config:"),
        "clashing field must be omitted from Scope (SC-sourced), got: {dts}"
    );
}

#[test]
fn multiple_declarations_all_appear() {
    let src = r#"
<data name="overview" source="ship.git.overview()" live />
<data name="config" source="ship.config.get()" />
"#;
    let dts = compile_dts(src);
    assert!(dts.contains("overview: Signal<LiveState<unknown>>"), "got: {dts}");
    assert!(dts.contains("config: Signal<unknown>"), "got: {dts}");
    assert!(dts.contains("import type { Signal, LiveState }"), "got: {dts}");
}
