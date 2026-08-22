//! Fixture-based tests for the `.sail.d.ts` declaration emitter.
//!
//! Each test compiles a small `.sail` snippet with `emit_dts: true` and
//! asserts the resulting d.ts contains the expected type lines.

use sail_compiler::{CompileMode, CompileOpts, compile};

fn compile_dts(source: &str) -> String {
    let opts = CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: true,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    compile(source, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

/// Compile a `.sail` source whose controller is inlined as a `<controller>`
/// block — the ONE source (the paired `.controller.ts` fallback is gone) — so
/// the d.ts emitter can pull the scope factory's fields. Returns the d.ts text.
fn compile_dts_with_controller(_stem: &str, sail_src: &str, controller_ts: &str) -> String {
    let src = format!("<controller>\n{controller_ts}\n</controller>\n{sail_src}");
    let tmp = tempfile::tempdir().expect("dts project");
    let project = tmp.path();
    std::fs::create_dir_all(project.join("src/widget")).unwrap();
    std::fs::write(project.join("package.json"), r#"{"name":"@ship/dts-emit"}"#).unwrap();
    std::fs::write(
        project.join("sail.config.jsonc"),
        "{ \"raw_containers\": true }\n",
    )
    .unwrap();
    let path = project.join("src/widget/widget.sail");
    let opts = CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: true,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,

        lint_overrides: Vec::new(),
    };
    compile(&src, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

fn compile_component_dts(source: &str) -> String {
    use sail_compiler::ComponentMeta;
    let opts = CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta {
            name: "test-el".into(),
        }),
        emit_dts: true,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,

        lint_overrides: Vec::new(),
    };
    compile(source, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

#[test]
fn prop_union_emits_union_type() {
    let dts = compile_component_dts(
        r#"<scope>
  props: {
    tone: 'primary' | 'neutral' = 'primary'
  }
</scope><span>x</span>"#,
    );
    assert!(
        dts.contains("tone: Signal<'primary' | 'neutral'>"),
        "got: {dts}"
    );
}

#[test]
fn prop_bool_emits_boolean() {
    let dts = compile_component_dts(
        r#"<scope>
  props: {
    loading = false
  }
</scope><span>x</span>"#,
    );
    assert!(dts.contains("loading: Signal<boolean>"), "got: {dts}");
}

#[test]
fn use_hook_with_as_emits_return_type() {
    let dts = compile_dts(r#"<use-filter-list as="filter"><span>x</span></use-filter-list>"#);
    assert!(
        dts.contains("filter: ReturnType<typeof useFilterList>"),
        "got: {dts}"
    );
}

#[test]
fn emit_dts_false_returns_none() {
    let opts = CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,

        lint_overrides: Vec::new(),
    };
    let out = compile(r#"<p>hi</p>"#, opts).expect("compile");
    assert!(out.dts.is_none(), "expected no dts when emit_dts=false");
}

#[test]
fn no_declarations_emits_empty_scope() {
    let dts = compile_dts("<p>Hello world</p>");
    assert!(dts.contains("export interface Scope {}"), "got: {dts}");
}

#[test]
fn reflect_empty_does_not_add_scope_entry() {
    // The slot lives inside a component since slots are component-only.
    // The reflect-empty side-effect must not surface in Scope.
    let dts =
        compile_component_dts(r#"<button><slot reflect-empty="data-icon-only"></slot></button>"#);
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
    // (`{foo()}`) do not add Scope members; only props/use/ref do.
    let sail = r#"<p>{foo()} {label()}</p>"#;
    let controller = r#"
        import { controller, signal } from '@ship/sail-runtime'
        export default controller((ctx) => ({
            foo: signal<number>(0),
            label: signal(''),
          }))
    "#;
    let dts = compile_dts_with_controller("page", sail, controller);
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
    // Block-body factory with shorthand return (the sail-error-boundary pattern)
    // — same omit rule: caughtError/reset are SC-typed, absent from Scope.
    let sail = r#"<p>x</p>"#;
    let controller = r#"
        import { controller, signal } from '@ship/sail-runtime'
        export default controller((_b: unknown, host?: { clear?: () => void }) => {
            const caughtError = signal<string | null>(null)
            const reset = (): void => { caughtError.set(null); host?.clear?.() }
            return { caughtError, reset }
          })
    "#;
    let dts = compile_dts_with_controller("page", sail, controller);
    assert!(dts.contains("export interface Scope {}"), "got: {dts}");
    assert!(
        !dts.contains("caughtError:") && !dts.contains("reset:"),
        "factory fields must be omitted from Scope, got: {dts}"
    );
}

#[test]
fn empty_scope_factory_leaves_scope_unchanged() {
    let sail = r#"<p>x</p>"#;
    let controller = r#"
        import { controller } from '@ship/sail-runtime'
        export default controller((ctx) => {
          mount((_host) => {})
          return {}
        })
    "#;
    let dts = compile_dts_with_controller("page", sail, controller);
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
    let sail = r#"<p>x</p>"#;
    let controller = r#"
        import { controller } from '@ship/sail-runtime'
        export default controller((ctx) => {
          mount((_host) => {})
          return {}
        })
    "#;
    let dts = compile_dts_with_controller("page", sail, controller);
    assert!(
        dts.contains("export interface Scope {}"),
        "expected empty Scope when no scope: section, got: {dts}"
    );
}

#[test]
fn template_member_clashing_with_factory_is_dropped_from_scope() {
    // A name in BOTH a template-derived member (`use-filter-list as="config"`)
    // AND the factory: the template-derived type is dropped (clash-retain) and
    // the factory type is NOT re-declared (omit) — so `config` is absent from
    // Scope entirely and typed solely by the inferred SC.
    // (Historically this used bind:this; that directive is retired.)
    let sail = r#"<use-filter-list as="config"><span>x</span></use-filter-list>"#;
    let controller = r#"
        import { controller, signal } from '@ship/sail-runtime'
        export default controller((ctx) => ({ config: signal<string | null>(null) }))
    "#;
    let dts = compile_dts_with_controller("page", sail, controller);
    assert!(
        !dts.contains("config:"),
        "clashing field must be omitted from Scope (SC-sourced), got: {dts}"
    );
}

// ---------------------------------------------------------------------------
// Host-granted capabilities (#109) — the resolved client list, typed into Scope
// ---------------------------------------------------------------------------

/// Compile with a host grant, as deck hands one in: its own context
/// (`route`/`navigate`, typed by the SAIL-local ports) plus an app-declared
/// data client (typed through the `Clients` port).
fn compile_dts_with_capabilities(source: &str) -> String {
    compile_dts_with_capabilities_as(source, None)
}

fn compile_dts_with_capabilities_as(
    source: &str,
    component_meta: Option<sail_compiler::ComponentMeta>,
) -> String {
    use sail_compiler::emit::dts::ScopeCapability;
    let opts = CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        component_meta,
        scope_capabilities: vec![
            ScopeCapability::client("api"),
            ScopeCapability {
                name: "navigate".into(),
                ts_type: "(path: string) => void".into(),
                ts_import: None,
            },
            ScopeCapability {
                name: "route".into(),
                ts_type: "RouteSignals".into(),
                ts_import: Some(("RouteSignals".into(), "@ship/sail-runtime".into())),
            },
        ],
        ..base_opts()
    };
    compile(source, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

fn base_opts() -> CompileOpts {
    CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: true,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn granted_capabilities_are_typed_members_of_scope() {
    let dts = compile_dts_with_capabilities("<p>hi</p>");
    // Flat Host members — no nested clients bag.
    assert!(dts.contains("api: Host['api']"), "got: {dts}");
    assert!(!dts.contains("clients:"), "got: {dts}");
    assert!(dts.contains("route: RouteSignals"), "got: {dts}");
    assert!(
        dts.contains("navigate: (path: string) => void"),
        "got: {dts}"
    );
    assert_eq!(
        dts.matches("from '@ship/sail-runtime'").count(),
        1,
        "capability imports must collapse to one line per module; got: {dts}"
    );
    assert!(
        dts.contains("import type { Host, RouteSignals }"),
        "got: {dts}"
    );
}

#[test]
fn no_grant_leaves_scope_exactly_as_before() {
    // A file compiled with no host context gets NO ambient members — reading
    // `ctx.route` there is a type error, which is the point.
    let dts = compile_dts("<p>hi</p>");
    assert!(dts.contains("export interface Scope {}"), "got: {dts}");
    assert!(!dts.contains("clients"), "got: {dts}");
    assert!(!dts.contains("route"), "got: {dts}");
}

#[test]
fn a_declared_member_shadows_the_granted_capability() {
    // The runtime spreads `...(ctx.scope ?? {})` LAST, over the client map, so
    // a template/controller declaration of the same name wins. The type must
    // agree — one member, the declared one, not a duplicate key.
    let dts = compile_dts_with_capabilities_as(
        "<scope>\n  props: {\n    api = ''\n  }\n</scope>\n<p>hi</p>",
        Some(sail_compiler::ComponentMeta {
            name: "test-el".into(),
        }),
    );
    assert!(dts.contains("api: Signal<string>"), "got: {dts}");
    assert!(!dts.contains("api: Clients['api']"), "got: {dts}");
    assert_eq!(
        dts.matches("  api:").count(),
        1,
        "exactly one `api` member; got: {dts}"
    );
}
