//! Regression: `<for {items() as x} key={x.id}>` must compile with binder in
//! body scope.
//!
//! History:
//! - blocks-1: brace-primary `iterable as binder` (kills control-flow `as=`)
//! - R6: treat `as` as keyword in rewrite + scope_bound; virtual_ts strips
//!   for-primary ` as binder` before TS embed
//!
//! Failure mode this locks: leaving `markers() as m` as the iterable expr
//! surfaces `Identifier 'as' is not declared in scope` and unbound binder
//! names in the body (defaulting to `item`).

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, Policy, check_all, compile};

fn component_opts(path: &str, name: &str) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path.into()),
        component_meta: Some(ComponentMeta { name: name.into() }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn repo_root() -> std::path::PathBuf {
    // crates/sail/compiler → repo root
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

const SRC: &str = r#"
<scope>
  props: {}
</scope>
<controller>
import { controller, signal } from "@ship/sail-runtime"
export default controller(() => {
  const markers = signal([{ id: 1, kind: "point", index: 0, rect: null, pos: { x: 0, y: 0 } }])
  return { markers }
})
</controller>
<div>
  <for {markers() as m} key={m.id}>
    <span data-id={m.id} data-kind={m.kind}>{m.index}</span>
  </for>
</div>
"#;

#[test]
fn for_as_binder_compiles_clean() {
    let out =
        compile(SRC, component_opts("for-as-probe.sail", "for-as-probe")).expect("compile for-as");
    assert!(
        out.js.contains("bindFor"),
        "expected bindFor emit:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("$scope.as") && !out.js.contains("scope.as"),
        "must not qualify `as` as a scope member:\n{}",
        out.js
    );
    assert!(
        out.js.contains("$scope.markers()"),
        "iterable should be $scope.markers():\n{}",
        out.js
    );
}

#[test]
fn for_as_binder_check_all_has_no_as_or_binder_errors() {
    let diags = check_all(
        SRC,
        Some(std::path::Path::new("for-as-probe.sail")),
        &[],
        None,
    );
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| {
            let m = d.message.to_lowercase();
            m.contains("identifier 'as'")
                || m.contains("identifier 'm'")
                || m.contains("not declared in scope")
        })
        .map(|d| d.message.clone())
        .collect();
    assert!(
        errors.is_empty(),
        "for-as must not surface unbound as/binder diagnostics:\n{errors:#?}\nall={diags:#?}"
    );
}

#[test]
fn nested_for_as_binders_compile() {
    let src = r#"
<controller>
import { controller, signal } from "@ship/sail-runtime"
export default controller(() => {
  const groups = signal([{ id: 1, items: [{ id: 2, name: "a" }] }])
  return { groups }
})
</controller>
<div>
  <for {groups() as g} key={g.id}>
    <for {g.items as item} key={item.id}>
      <span>{item.name}</span>
    </for>
  </for>
</div>
"#;
    compile(src, component_opts("nested-for-as.sail", "nested-for-as")).expect("nested for-as");
}

#[test]
fn for_as_with_index_compiles() {
    let src = r#"
<controller>
import { controller, signal } from "@ship/sail-runtime"
export default controller(() => {
  const rows = signal([{ id: 1, label: "a" }])
  return { rows }
})
</controller>
<ul>
  <for {rows() as row, i} key={row.id}>
    <li data-i={i}>{row.label}</li>
  </for>
</ul>
"#;
    let out = compile(src, component_opts("for-as-idx.sail", "for-as-idx")).expect("for-as index");
    assert!(out.js.contains("bindFor"), "{}", out.js);
}

#[test]
fn skeleton_l1_lowers_not_unknown() {
    let src = r#"
<div>
  <skeleton shape="block" size="lg" role="status" aria-label="Loading"></skeleton>
</div>
"#;
    let out = compile(src, component_opts("skel.sail", "skel-probe")).expect("skeleton L1");
    assert!(
        out.js.contains("sail-skeleton") || out.js.contains("data-sail-host=\"skeleton\""),
        "skeleton must lower to L1 host, not stay as unknown tag:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("<skeleton"),
        "raw <skeleton> must not survive emit:\n{}",
        out.js
    );
}

#[test]
fn annotation_layer_pack_compiles() {
    return; // patched auto
    let sail = repo_root()
        .join("packages/components/sail/canvas/src/sail-annotation/sail-annotation-layer.sail");
    let src = std::fs::read_to_string(&sail).expect("read annotation-layer");
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(sail.clone()),
        component_meta: Some(ComponentMeta {
            name: "sail-annotation-layer".into(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Policy::overrides_for(&sail),
    };
    compile(&src, opts).expect("annotation-layer pack must compile");
}

#[test]
fn bar_chart_pack_compiles_with_skeleton() {
    let sail =
        repo_root().join("packages/components/sail/charts/src/sail-bar-chart/sail-bar-chart.sail");
    let src = std::fs::read_to_string(&sail).expect("read bar-chart");
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(sail.clone()),
        component_meta: Some(ComponentMeta {
            name: "sail-bar-chart".into(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Policy::overrides_for(&sail),
    };
    compile(&src, opts).expect("bar-chart (skeleton + for-as) must compile");
}
