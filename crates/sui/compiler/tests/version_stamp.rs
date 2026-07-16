//! Every artifact emitted by `compile()` must carry a `_compilerVersion`
//! marker — the shell uses it (via `/_deck/meta`) to detect when its
//! loaded bundle drifts from the daemon's compiled output. This test
//! locks the contract per output kind (JS leading comment, CEM, schema,
//! meta sidecar, dts comment).

use sui_compiler::{compile, compiler_version, CompileMode, CompileOpts, ComponentMeta};

const VIEW_SRC: &str = r#"<data name="rows" source="ship.rows()" live />
<ul>{#for (row, _i) of rows().value key={row.id}}<li>{row.label}</li>{/for}</ul>
"#;

const COMPONENT_SRC: &str = r#"<scope>
  props: {
    label = ''
  }
</scope><button><slot /></button>
"#;

fn view_opts() -> CompileOpts {
    CompileOpts {
        mode: CompileMode::Component,
        emit_dts: true,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
        ..Default::default()
    }
}

fn component_opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        mode: CompileMode::Component,
        component_meta: Some(ComponentMeta {
            name: "vs-button".into(),
        }),
        emit_dts: true,
        emit_component_imports: false,
        ..Default::default()
    }
}
#[test]
fn compiler_version_struct_is_populated() {
    let cv = compiler_version();
    assert_eq!(cv.cargo, env!("CARGO_PKG_VERSION"));
    // git_sha is either a short hex sha or the sentinel "unknown" when
    // the source tarball was extracted outside a git checkout.
    assert!(
        cv.git == "unknown" || cv.git.chars().all(|c| c.is_ascii_hexdigit()),
        "unexpected git sha shape: {:?}",
        cv.git
    );
    assert!(!cv.built_at.is_empty());
}

#[test]
fn js_carries_compiler_version_marker() {
    let out = compile(VIEW_SRC, view_opts()).expect("compile view");
    let cv = compiler_version();
    let first_real = out
        .js
        .lines()
        .find(|l| !l.trim().is_empty())
        .expect("non-empty output");
    // Legal comment `/*! ... */` (bundlers preserve it) on the first
    // non-blank line.
    assert!(
        first_real.starts_with("/*! _compilerVersion ") && first_real.ends_with("*/"),
        "JS must lead with the /*! _compilerVersion */ legal comment, got: {first_real:?}"
    );
    assert!(
        first_real.contains(&format!("\"cargo\":\"{}\"", cv.cargo)),
        "stamp must carry cargo version, got: {first_real:?}"
    );
    assert!(
        first_real.contains(&format!("\"git\":\"{}\"", cv.git)),
        "stamp must carry git sha, got: {first_real:?}"
    );
    // Property on the default export — survives bundling under
    // Bun.build / esbuild even when leading comments get rewritten by
    // the bundler. For views the property hangs off the `view(...)`
    // result via `Object.assign(..., { _compilerVersion })`.
    assert!(
        out.js.contains("_compilerVersion:"),
        "JS must attach `_compilerVersion` to the default export, got:\n{}",
        out.js
    );
}

#[test]
fn component_default_export_carries_compiler_version() {
    let out = compile(COMPONENT_SRC, component_opts()).expect("compile component");
    // Components stamp the config object passed to component(): the
    // factory ignores unknown fields and the property survives
    // bundling alongside `props`, `shadow`, `setup`, etc.
    assert!(
        out.js.contains("_compilerVersion: {") || out.js.contains("_compilerVersion:{"),
        "component config must carry _compilerVersion, got:\n{}",
        out.js
    );
}

#[test]
fn cem_carries_compiler_version_field() {
    let out = compile(COMPONENT_SRC, component_opts()).expect("compile component");
    let cem = out.cem.expect("CEM emitted in component mode");
    let stamp = cem
        .get("_compilerVersion")
        .expect("CEM must include _compilerVersion field");
    assert!(stamp.get("cargo").is_some());
    assert!(stamp.get("git").is_some());
    assert!(stamp.get("builtAt").is_some());
}

#[test]
fn schema_carries_compiler_version_field() {
    let out = compile(VIEW_SRC, view_opts()).expect("compile view");
    let schema = out.schema.expect("schema emitted");
    let stamp = schema
        .get("_compilerVersion")
        .expect("schema must include _compilerVersion field");
    assert!(stamp.get("cargo").is_some());
}

#[test]
fn meta_carries_compiler_version_field() {
    let out = compile(VIEW_SRC, view_opts()).expect("compile view");
    let meta = out.meta.expect("meta emitted");
    let stamp = meta
        .get("_compilerVersion")
        .expect("meta must include _compilerVersion field");
    assert!(stamp.get("cargo").is_some());
}

#[test]
fn dts_carries_compiler_version_comment() {
    let out = compile(VIEW_SRC, view_opts()).expect("compile view");
    let dts = out.dts.expect("dts emitted");
    assert!(
        dts.lines().any(|l| l.starts_with("// _compilerVersion ")),
        "dts must include version-stamp comment, got:\n{dts}"
    );
}
