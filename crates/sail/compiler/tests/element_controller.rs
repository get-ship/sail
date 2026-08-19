//! Job C — `<controller>` is legal in an L1 `.sail`.
//!
//! No `customElements.define`, no hyphen CE, no sibling factory.
//! No `<controller>` → static splice; with a real block → render machinery.
//! Button has none. Theme-picker stays a tag. Stdlib widgets do not
//! trampoline `@ship/sail-runtime/actions` from a controller — that is
//! not a brain.

use sail_compiler::{CompileMode, CompileOpts, compile, compile_element};

fn page_opts() -> CompileOpts {
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

fn stdlib_src(tag: &str) -> &'static str {
    sail_compiler::stdlib::sources()
        .iter()
        .find(|(t, _)| *t == tag)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("{tag} registered in stdlib"))
}

const CHIP_WITH_CTRL: &str = r#"
<scope>
  props: { selected = false }
</scope>
<controller>
export default controller(() => {
  const selected = signal(false)
  return { selected }
})
</controller>
<span>label</span>
"#;

#[test]
fn element_accepts_controller_block() {
    let el = compile_element(CHIP_WITH_CTRL, "chip").expect(
        "element mode: `<controller>` is the L1 state home — same one-file form as app components",
    );
    assert_eq!(el.root_class, "sail-chip");
    assert!(
        el.structure_html.contains("data-sail-ctrl=\"chip\""),
        "with a controller, the opaque host stamps data-sail-ctrl so the page can attach: {}",
        el.structure_html
    );
    assert!(
        !el.structure_html.contains("customElements"),
        "element compile is not a custom element: {}",
        el.structure_html
    );
}

#[test]
fn element_without_controller_stays_static_splice() {
    let el = compile_element(include_str!("../stdlib/button/button.sail"), "button")
        .expect("button compiles");
    assert!(
        !el.structure_html.contains("data-sail-ctrl"),
        "button is not state — static splice, no controller stamp: {}",
        el.structure_html
    );
}

#[test]
fn button_source_has_no_controller() {
    let src = stdlib_src("button");
    assert!(
        !src.contains("<controller"),
        "button is not a controller widget:\n{src}"
    );
}

#[test]
fn page_using_tabs_is_not_ce() {
    let out = compile(
        r#"<tabs><tab title="A">one</tab><tab title="B">two</tab></tabs>"#,
        page_opts(),
    )
    .expect("page with <tabs> compiles");
    let js = &out.js;
    assert!(
        !js.contains("customElements.define") && !js.contains("component(\"tabs\""),
        "L1 is not CE — no define, no hyphen registration:\n{js}"
    );
    assert!(
        js.contains("data-sail-host=\\\"tabs\\\"") || js.contains("data-sail-host=\"tabs\""),
        "same tag at the use site (opaque host):\n{js}"
    );
}

#[test]
fn page_using_button_does_not_attach_controller() {
    let out = compile("<button>Go</button>", page_opts()).expect("button page compiles");
    let js = &out.js;
    assert!(
        !js.contains("runController") && !js.contains("sail-l1:"),
        "button stays static splice:\n{js}"
    );
    assert!(
        !js.contains("customElements.define"),
        "button is not a custom element:\n{js}"
    );
}

#[test]
fn theme_picker_stays_a_tag() {
    let out = compile("<theme-picker></theme-picker>", page_opts())
        .expect("<theme-picker> compiles as a tag");
    let js = &out.js;
    assert!(
        !js.contains("customElements.define"),
        "theme-picker is framework chrome as a tag, not a CE:\n{js}"
    );
    assert!(
        js.contains("theme-picker") && !js.contains("component(\"sail-theme-picker\""),
        "same tag at the use site:\n{js}"
    );
}

#[test]
fn run_controller_is_abi_not_author_surface() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let index = std::fs::read_to_string(root.join("packages/sail/runtime/src/index.ts"))
        .expect("runtime index");
    let internal = std::fs::read_to_string(root.join("packages/sail/runtime/src/internal.ts"))
        .expect("runtime internal");
    assert!(
        internal.contains("runController"),
        "runController is ABI next to runAction — export it from /internal"
    );
    let leaked = index.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with("export ")
            && !t.starts_with("export type ")
            && (t.contains(" runController")
                || t.contains("{runController")
                || t.contains(", runController")
                || t.contains("runController,")
                || t.contains("runController }"))
    });
    assert!(
        !leaked,
        "runController must not leak onto the author front door"
    );
}
