//! Snapshot tests. For each fixture, compile and compare against a
//! checked-in `<fixture>.expected.js`. Set `UPDATE_SNAPSHOTS=1` in
//! the environment to overwrite the expectations.

use std::fs;
use std::path::{Path, PathBuf};

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

mod common;
use common::strip_version_stamp;

struct Fixture {
    name: &'static str,
    setup_url: Option<&'static str>,
    mode: CompileMode,
    component_meta: Option<ComponentMeta>,
}

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture { name: "hello",       setup_url: None,               mode: CompileMode::Component,      component_meta: None },
        Fixture { name: "counter",     setup_url: Some("./counter.controller.ts"), mode: CompileMode::Component,     component_meta: None },
        Fixture { name: "ship-button", setup_url: None,               mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "ship-button-test".into() }) },
        Fixture { name: "bind-this-button", setup_url: None,           mode: CompileMode::Component,      component_meta: None },
        Fixture { name: "reflect-empty-slot", setup_url: None,         mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "reflect-empty-slot-test".into() }) },
        Fixture { name: "comment-and-whitespace", setup_url: None,    mode: CompileMode::Component,      component_meta: None },
        Fixture { name: "for-update",  setup_url: None,               mode: CompileMode::Component,      component_meta: None },
        Fixture { name: "for-static",  setup_url: None,               mode: CompileMode::Component,      component_meta: None },
        Fixture { name: "for-static-text", setup_url: None,           mode: CompileMode::Component,      component_meta: None },
        Fixture { name: "for-static-attr", setup_url: None,           mode: CompileMode::Component,      component_meta: None },
        Fixture { name: "for-dynamic-mixed", setup_url: None,         mode: CompileMode::Component,      component_meta: None },
    ]
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn compile_fixture(f: &Fixture) -> String {
    let path = fixtures_dir().join(format!("{}.sui", f.name));
    let source = fs::read_to_string(&path).expect("read fixture");
    let opts = CompileOpts {
        setup_url: f.setup_url.map(|s| s.to_string()),
        mode: f.mode,
        source_path: Some(path),
        component_meta: f.component_meta.clone(),
        emit_dts: false,
        ..CompileOpts::default()
    };
    let out = compile(&source, opts).expect("compile");
    out.js
}
fn assert_snapshot(name: &str, actual: &str, expected_path: &Path) {
    let normalized_actual = strip_version_stamp(actual);
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        fs::write(expected_path, &normalized_actual).expect("write snapshot");
        return;
    }
    let expected = match fs::read_to_string(expected_path) {
        Ok(s) => s,
        Err(_) => {
            fs::write(expected_path, &normalized_actual).expect("write snapshot");
            panic!(
                "snapshot {} did not exist; wrote it now — re-run to confirm",
                expected_path.display()
            );
        }
    };
    assert_eq!(
        strip_version_stamp(&expected), normalized_actual,
        "snapshot mismatch for {name}. Set UPDATE_SNAPSHOTS=1 to refresh."
    );
}


fn fixture_named(name: &str) -> Fixture {
    fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no fixture named {name}"))
}

#[test]
fn hello_snapshot() {
    let fx = &fixture_named("hello");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn counter_snapshot() {
    let fx = &fixture_named("counter");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn ship_button_snapshot() {
    let fx = &fixture_named("ship-button");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn bind_this_button_snapshot() {
    let fx = &fixture_named("bind-this-button");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn bind_this_button_direct_emits_hydrate_ref() {
    use sui_compiler::compile;
    let path = fixtures_dir().join("bind-this-button.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = compile(&source, opts).expect("compile");
    // Phase 3b.4: the bind:this leaf is direct-emit — the view's
    // `hydrate(root, ctx)` calls `hydrateRef` with the bare scope
    // variable name; the manifest no longer carries a `bind:this` row.
    assert!(
        out.js.contains("hydrateRef(__m[\"b0\"], $scope, \"btnEl\")"),
        "expected hydrateRef call in hydrate(), got:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("kind: \"bind:this\""),
        "did not expect bind:this manifest entry post-direct-emit, got:\n{}",
        out.js
    );
}

#[test]
fn reflect_empty_slot_snapshot() {
    let fx = &fixture_named("reflect-empty-slot");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}


#[test]
fn comment_and_whitespace_snapshot() {
    let fx = &fixture_named("comment-and-whitespace");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

/// The `{#for}` render closure must return `{ nodes, cleanup, update }`.
/// The `update` function mutates the child scope's item + index and
/// increments the per-row version signal so bindings re-evaluate.
#[test]
fn for_update_snapshot() {
    let fx = &fixture_named("for-update");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

/// D3: when a `{#for}` body contains no expression that reads the row
/// or index binder, the compiler omits the per-row `__v` version
/// signal, the `void $scope.__v()` reads, the `__v.set` increment, and
/// the `update` field on the render-closure return. The `signal`
/// runtime import is also dropped (nothing else here uses it).
#[test]
fn for_static_body_omits_v_sentinel() {
    let path = fixtures_dir().join("for-static.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    assert!(
        !out.js.contains("signal(0)"),
        "static row body must not allocate __v: \n{}",
        out.js
    );
    assert!(
        !out.js.contains("__v"),
        "static row body must not reference __v anywhere: \n{}",
        out.js
    );
    assert!(
        !out.js.contains("update:"),
        "static row body must not emit an update field: \n{}",
        out.js
    );
    // The only `signal` usage in this fixture was the per-row sentinel.
    // The runtime import list must drop it.
    assert!(
        !out.js.contains(" signal,") && !out.js.contains(", signal "),
        "signal import must be dropped when no for-body needs __v: \n{}",
        out.js
    );
}

/// D3: a `{#for}` whose body holds a binding reading the index alone
/// (no row-field reference) must still emit `__v` — reorders shift the
/// index for surviving keys, so the binding has to re-fire.
#[test]
fn for_body_referencing_index_refreshes_on_row_reuse() {
    // Index-only bodies (`{i}` text, no row-field reads) used to
    // allocate a whole-row `__v` signal and bump it on every update.
    // After per-field __v split, no reactive binding subscribes to a
    // per-row signal here (the `{i}` text takes D5's inline path, not
    // bindText), so the per-row signal would be dead weight. What
    // STILL matters: the `update()` callback must re-fire the inline
    // textContent assignment when bindFor reuses the row at a new
    // index. Index reads escalate to RowUse::Whole in the analyzer,
    // which makes partition_inline_updates fall back to the unguarded
    // join — so the inline stmt always runs on update, no per-field
    // diff needed.
    use sui_compiler::compile;
    let src = r#"<ul>
  {#for (row, i) of [{id: 1}, {id: 2}] key={row.id}}
    <li>{i}</li>
  {/for}
</ul>
"#;
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = compile(src, opts).expect("compile");
    assert!(
        out.js.contains("update:"),
        "expected an `update:` callback so row-reuse refreshes the index DOM, got:\n{}",
        out.js
    );
    // The inline textContent assignment must appear in BOTH the
    // initial render (where it sets the text on row creation) and the
    // update body (where it re-sets when the row is reused at a new
    // index). Two occurrences is the contract.
    let occurrences = out.js.matches("textContent = ($scope.i)").count();
    assert!(
        occurrences >= 2,
        "expected textContent assignment in both render + update bodies, got {occurrences} in:\n{}",
        out.js
    );
}

#[test]
fn for_static_snapshot() {
    let fx = &fixture_named("for-static");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn for_static_text_snapshot() {
    let fx = &fixture_named("for-static-text");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn for_static_attr_snapshot() {
    let fx = &fixture_named("for-static-attr");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn for_dynamic_mixed_snapshot() {
    let fx = &fixture_named("for-dynamic-mixed");
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

/// D5: a `{#for}` body whose only DynamicText reads a row property
/// (no `()`) must inline the assignment as `node.textContent = …`
/// and never call `bindText` for that read.
#[test]
fn for_static_text_inlines_text_content() {
    let path = fixtures_dir().join("for-static-text.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    assert!(
        out.js.contains(".textContent = ($scope.row.label)"),
        "expected inlined textContent assignment, got:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("bindText("),
        "static row text must not emit bindText call, got:\n{}",
        out.js
    );
}

/// D5: a `{#for}` body whose only AttrInstr::Expr reads a row property
/// must inline the assignment as `el.setAttribute(...)` and never call
/// `bindAttr` for that read.
#[test]
fn for_static_attr_inlines_set_attribute() {
    let path = fixtures_dir().join("for-static-attr.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    assert!(
        out.js.contains("setAttribute(\"id\""),
        "expected inlined setAttribute, got:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("bindAttr("),
        "static row attr must not emit bindAttr call, got:\n{}",
        out.js
    );
}

/// D5: a `{#for}` body that mixes a reactive binding (`()` present)
/// with a static row read inlines only the static one. The reactive
/// binding still emits its `bind*` effect.
#[test]
fn for_dynamic_mixed_inlines_only_static() {
    let path = fixtures_dir().join("for-dynamic-mixed.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    assert!(
        out.js.contains(".textContent = ($scope.row.label)"),
        "expected inlined textContent for row.label, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains(r#"bindAttr(__e5, "aria-selected""#),
        "expected bindAttr effect for the row's selected state:\n{}",
        out.js
    );
}

#[test]
fn for_render_closure_emits_update_field() {
    let path = fixtures_dir().join("for-update.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    assert!(
        out.js.contains("update:"),
        "expected `update:` field in render closure return, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("signal(0)"),
        "expected per-row version signal, got:\n{}",
        out.js
    );
    // After per-field __v split, fixture's `{row.label.toUpperCase()}`
    // body reads only `row.label` reactively — so the closure subscribes
    // to `__v_label` (per-field) instead of the legacy whole-row `__v`.
    // Accept either form for forward-compat; both shapes preserve the
    // reactive-on-row-update contract.
    assert!(
        out.js.contains("void $scope.__v(") || out.js.contains("void $scope.__v_label("),
        "expected per-row __v subscription in binding closure, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("$scope.__v.set(c => c + 1)")
            || out.js.contains("$scope.__v_label.set(c => c + 1)"),
        "expected version signal increment in update, got:\n{}",
        out.js
    );
}

/// Routing is deck's: a view that references `route` reads the handle from
/// `ctx.route` (deck injects it via host-context). SUI vends no route factory,
/// so the compiled output must NOT import `@ship/sui-runtime/route`, and assigns
/// `$scope.route = route` straight from the ctx destructure.
#[test]
fn route_view_reads_route_from_ctx() {
    let path = fixtures_dir().join("route-view.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: Some(vec!["name".to_string()]),
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    assert!(
        !out.js.contains("@ship/sui-runtime/route") && !out.js.contains("__routeFactory"),
        "routing is deck: no SUI route-factory import or fallback, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("$scope.route = route"),
        "the route handle must come straight from ctx, got:\n{}",
        out.js
    );
}

#[test]
fn routeless_view_omits_route_import() {
    let path = fixtures_dir().join("counter.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    assert!(
        !out.js.contains("@ship/sui-runtime/route"),
        "routeless view must not import @ship/sui-runtime/route, got:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("__routeFactory"),
        "routeless view must not reference __routeFactory, got:\n{}",
        out.js
    );
}

/// Fix 1: source HTML comments must never appear as createComment() in output.
#[test]
fn comment_nodes_not_emitted() {
    let path = fixtures_dir().join("comment-and-whitespace.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    assert!(
        !out.js.contains("createComment(\"Developer comment"),
        "source HTML comment must not appear in compiled output:\n{}",
        out.js
    );
}

/// Fix 2: whitespace-only text nodes between elements must not be emitted;
/// intentional inline whitespace (text-only parent) must be preserved.
#[test]
fn whitespace_text_nodes_dropped_between_elements() {
    let path = fixtures_dir().join("comment-and-whitespace.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = sui_compiler::compile(&source, opts).expect("compile");
    // Inter-element indentation whitespace must be gone.
    assert!(
        !out.js.contains("createTextNode(\"\\n"),
        "whitespace-only text nodes between elements must not be emitted:\n{}",
        out.js
    );
    // Intentional space inside <span> (text-only parent) must be kept. With
    // template cloning the space rides inside the hoisted skeleton HTML.
    assert!(
        out.js.contains("createTextNode(\" \")") || out.js.contains("> <"),
        "intentional inline whitespace must be preserved:\n{}",
        out.js
    );
}


#[test]
fn reflect_empty_slot_emits_bind_slot_empty_call() {
    use sui_compiler::compile;
    let path = fixtures_dir().join("reflect-empty-slot.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: Some(ComponentMeta {
            name: "reflect-empty-slot-test".into(),

        }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = compile(&source, opts).expect("compile");
    assert!(
        out.js.contains("bindSlotEmpty(") && out.js.contains("\"data-icon-only\""),
        "expected bindSlotEmpty(...) call referencing data-icon-only:\n{}",
        out.js
    );
    // Imports must include bindSlotEmpty.
    assert!(
        out.js.contains("bindSlotEmpty"),
        "expected bindSlotEmpty import, got:\n{}",
        out.js
    );
}

#[test]
fn reflect_empty_slot_in_conditional_threads_container() {
    // A `<slot reflect-empty>` inside an `{#if}` lands in a hoisted
    // `__wireBoundary` helper at FACTORY scope, where `container` (the
    // `__mount(container)` param) is NOT bound. The helper must therefore TAKE
    // `container` as a param and every call site must PASS it — otherwise
    // mounting the conditional branch throws `ReferenceError: container is not
    // defined` (the bug behind every `<sui-*>` with a conditional reflect-empty
    // slot, e.g. sui-input's `<slot name="trailing">`).
    use sui_compiler::compile;
    let path = fixtures_dir().join("reflect-empty-slot-conditional.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: Some(ComponentMeta {
            name: "reflect-empty-slot-conditional-test".into(),

        }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = compile(&source, opts).expect("compile");
    // The hoisted helper THREADS `container` (so the slot binding inside it
    // resolves the host).
    assert!(
        out.js.contains("(anchor, container) => bindIf("),
        "expected the hoisted __wireBoundary helper to take `container`:\n{}",
        out.js
    );
    // The slot binding lives inside that helper and reflects via `container`.
    assert!(
        out.js.contains("bindSlotEmpty(") && out.js.contains(", container, "),
        "expected bindSlotEmpty(slot, container, attr) inside the helper:\n{}",
        out.js
    );
    // The `__mount` call site PASSES `container` to the helper.
    assert!(
        out.js.contains(", container));"),
        "expected the __mount call site to pass `container`:\n{}",
        out.js
    );
}

#[test]
fn ship_button_emits_cem() {
    let path = fixtures_dir().join("ship-button.sui");
    let source = fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: Some(ComponentMeta { name: "ship-button-test".into() }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = compile(&source, opts).expect("compile");
    let cem = out.cem.expect("CEM emitted for component-mode");
    let cem_str = serde_json::to_string_pretty(&cem).unwrap();
    let expected_path = fixtures_dir().join("ship-button.expected.cem.json");
    assert_snapshot("ship-button.cem", &cem_str, &expected_path);
}

#[test]
fn schema_emitted_for_each_fixture() {
    for fx in &fixtures() {
        let path = fixtures_dir().join(format!("{}.sui", fx.name));
        let source = fs::read_to_string(&path).unwrap();
        let opts = CompileOpts {
        route_params: None,
            setup_url: None,
            mode: fx.mode,
            source_path: Some(path),
            component_meta: fx.component_meta.clone(),
            emit_dts: false,
            emit_component_imports: false,
        lint_overrides: Vec::new(),
        };
        let out = compile(&source, opts).expect("compile");
        let schema = out.schema.expect("schema emitted");
        let s = serde_json::to_string_pretty(&schema).unwrap();
        let expected = fixtures_dir().join(format!("{}.expected.schema.json", fx.name));
        assert_snapshot(&format!("{}.schema", fx.name), &s, &expected);
    }
}
/// Phase 3b.5 (gate L): every fixture emits a `.sui.meta.json` sidecar
/// with binding info. The runtime never loads it; it's purely
/// developer-tooling fuel (LSP hover, error overlay, HMR diff).
#[test]
fn meta_sidecar_emitted_for_each_fixture() {
    for fx in &fixtures() {
        let path = fixtures_dir().join(format!("{}.sui", fx.name));
        let source = fs::read_to_string(&path).unwrap();
        let opts = CompileOpts {
        route_params: None,
            setup_url: None,
            mode: fx.mode,
            source_path: Some(path),
            component_meta: fx.component_meta.clone(),
            emit_dts: false,
            emit_component_imports: false,
        lint_overrides: Vec::new(),
        };
        let out = compile(&source, opts).expect("compile");
        let meta = out.meta.expect("meta emitted");
        let bindings = meta
            .get("bindings")
            .and_then(|v| v.as_array())
            .expect("bindings array");
        for entry in bindings {
            assert!(entry.get("key").and_then(|v| v.as_str()).is_some(),
                "binding entry missing key: {entry}");
            assert!(entry.get("kind").and_then(|v| v.as_str()).is_some(),
                "binding entry missing kind: {entry}");
        }
    }
}
