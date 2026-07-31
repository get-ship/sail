//! `<for>` key-fn emission shape.
//!
//! When the `key={…}` expression is scope-free — every free identifier
//! is the loop binding, the index binding, an expression-local, or a
//! JS/browser builtin — the compiler emits a flat `(row, i) => expr`
//! closure. The legacy form allocated `Object.create($scope)` per call,
//! and the runtime's keyed reconciler (`bindFor`) calls the key fn for
//! every row on every render — one dead allocation per row per swap.
//!
//! Keys that read outer scope (`$scope.…` after rewriting) keep the
//! child-scope fallback so shadowing semantics are untouched.

use sail_compiler::{CompileMode, CompileOpts, compile};

fn route_opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

/// The single emitted `key:` line of the bindFor options object.
fn key_line(js: &str) -> &str {
    js.lines()
        .find(|l| l.trim_start().starts_with("key: "))
        .unwrap_or_else(|| panic!("no `key:` line in emitted JS:\n{js}"))
}

#[test]
fn scope_free_key_emits_flat_closure() {
    let src = r#"<ul><for as="row" of={rows()} key={row.id}><li>{row.label}</li></for></ul>"#;
    let r = compile(src, route_opts()).expect("compile");
    let line = key_line(&r.js);
    assert!(
        line.contains("key: (row, __i) => (row.id),"),
        "expected flat key closure, got: {line}"
    );
    assert!(
        !line.contains("Object.create"),
        "scope-free key must not allocate a child scope: {line}"
    );
}

#[test]
fn index_binding_key_emits_flat_closure() {
    let src = r#"<ul><for as="row" index="i" of={rows()} key={i}><li>{row.label}</li></for></ul>"#;
    let r = compile(src, route_opts()).expect("compile");
    let line = key_line(&r.js);
    assert!(
        line.contains("key: (row, i) => (i),"),
        "expected flat key closure over the index binding, got: {line}"
    );
    assert!(!line.contains("Object.create"), "got: {line}");
}

#[test]
fn template_literal_key_over_loop_bindings_is_flat() {
    let src = r#"<ul><for as="row" index="i" of={rows()} key={`${row.id}-${i}`}><li>{row.label}</li></for></ul>"#;
    let r = compile(src, route_opts()).expect("compile");
    let line = key_line(&r.js);
    assert!(
        line.contains("key: (row, i) => (`${row.id}-${i}`),"),
        "expected flat key closure, got: {line}"
    );
    assert!(!line.contains("Object.create"), "got: {line}");
}

#[test]
fn builtin_ident_key_is_flat() {
    // `Infinity` is a global builtin — left bare by the rewriter, so
    // the key is still scope-free. (Calls with arguments like
    // `String(row.id)` are rejected upstream by the renderable-expr
    // subset and never reach the emitter.)
    let src =
        r#"<ul><for as="row" of={rows()} key={row.id ?? Infinity}><li>{row.label}</li></for></ul>"#;
    let r = compile(src, route_opts()).expect("compile");
    let line = key_line(&r.js);
    assert!(
        line.contains("key: (row, __i) => (row.id ?? Infinity),"),
        "builtins resolve globally, no scope needed, got: {line}"
    );
    assert!(!line.contains("Object.create"), "got: {line}");
}

#[test]
fn scoped_key_falls_back_to_child_scope() {
    let src =
        r#"<ul><for as="row" of={rows()} key={prefix() + row.id}><li>{row.label}</li></for></ul>"#;
    let r = compile(src, route_opts()).expect("compile");
    let line = key_line(&r.js);
    assert!(
        line.contains("Object.create($scope)"),
        "key reading outer scope must keep the child-scope form, got: {line}"
    );
    assert!(
        line.contains("$scope.prefix()"),
        "outer ident must stay $scope-qualified, got: {line}"
    );
    assert!(
        line.contains("(($scope) => ($scope.prefix() + row.id))(__child)"),
        "fallback IIFE shape drifted: {line}"
    );
}
