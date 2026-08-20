//! Job C — `<controller>` is legal in an L1 `.sail`.
//!
//! No `customElements.define`, no hyphen CE, no sibling factory.
//! No `<controller>` → static splice; with a real block → render machinery.
//! Button has none. Theme-picker stays a tag. Stdlib widgets do not
//! trampoline `@ship/sail-runtime/actions` from a controller — that is
//! not a brain.

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile, compile_element};

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
        el.controller_js.is_some(),
        "with a controller, the element carries the same setup module as any component"
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
fn stdlib_tabs_is_not_ce_and_not_a_tabs_action() {
    let src = stdlib_src("tabs");
    assert!(!src.contains("use:tabs"), "no widget use:tabs:\n{src}");
    assert!(
        src.contains("<controller") && src.contains("use:hotkeys={keys}"),
        "tabs state is a controller; listen is toolkit hotkeys:\n{src}"
    );
    assert!(
        !src.contains("querySelector")
            && !src.contains("setAttribute")
            && !src.contains("Symbol.for")
            && !src.contains("provideContext")
            && !src.contains("injectContext")
            && !src.contains("contextKey"),
        "tabs state is this controller's return, not context:\n{src}"
    );
    let tab_src = stdlib_src("tab");
    assert!(
        tab_src.contains("init.active")
            && tab_src.contains("init.select")
            && !tab_src.contains("injectContext")
            && !tab_src.contains("contextKey"),
        "child tabs take parent active/select as props:\n{tab_src}"
    );
    let el = compile_element(src, "tabs").unwrap_or_else(|e| panic!("tabs: {e:?}"));
    assert!(
        !el.structure_html.contains("use:tabs") && el.controller_js.is_some(),
        "tabs is a component with a controller, not a use:tabs action: {}",
        el.structure_html
    );
    let js = el.controller_js.as_deref().expect("tabs controller module");
    assert!(
        !js.contains("querySelector") && !js.contains("HTMLElement") && !js.contains("ctx.host"),
        "author controller stays platform-agnostic:\n{js}"
    );
}

#[test]
fn stdlib_radio_group_is_not_ce_and_not_a_radioGroup_action() {
    let src = stdlib_src("radio-group");
    assert!(
        !src.contains("use:radioGroup"),
        "no widget use:radioGroup:\n{src}"
    );
    assert!(
        src.contains("<controller")
            && src.contains("type=\"radio\"") == false
            && !src.contains("use:hotkeys"),
        "radio-group state is a controller; listen is native radio, not hotkeys:\n{src}"
    );
    assert!(
        !src.contains("querySelector")
            && !src.contains("setAttribute")
            && !src.contains("MutationObserver")
            && !src.contains("Symbol.for")
            && !src.contains("provideContext")
            && !src.contains("injectContext")
            && !src.contains("contextKey"),
        "radio-group state is this controller's return, not context:\n{src}"
    );
    let radio_src = stdlib_src("radio");
    assert!(
        radio_src.contains("init.groupValue")
            && radio_src.contains("init.select")
            && radio_src.contains("type=\"radio\"")
            && !radio_src.contains("use:hotkeys")
            && !radio_src.contains("injectContext")
            && !radio_src.contains("contextKey"),
        "child radios take parent groupValue/select as props; listen is native radio:\n{radio_src}"
    );
    let el = compile_element(src, "radio-group").unwrap_or_else(|e| panic!("radio-group: {e:?}"));
    assert!(
        !el.structure_html.contains("use:radioGroup") && el.controller_js.is_some(),
        "radio-group is a component with a controller, not a use:radioGroup action: {}",
        el.structure_html
    );
    let js = el.controller_js.as_deref().expect("radio-group controller module");
    assert!(
        !js.contains("querySelector") && !js.contains("HTMLElement") && !js.contains("ctx.host"),
        "author controller stays platform-agnostic:\n{js}"
    );
}

#[test]
fn stdlib_context_menu_is_not_a_widget_action() {
    let src = stdlib_src("context-menu");
    let code = src.rsplit("-->").next().unwrap_or(src);
    assert!(
        !code.contains("use:contextMenu") && !code.contains("use:menu"),
        "no widget use:contextMenu / use:menu:\n{src}"
    );
    assert!(
        src.contains("<controller") && src.contains("use:hotkeys={keys}"),
        "context-menu state is a controller; listen is toolkit hotkeys:\n{src}"
    );
    assert!(
        !src.contains("querySelector")
            && !src.contains("setAttribute")
            && !src.contains("Symbol.for")
            && !src.contains("provideContext")
            && !src.contains("injectContext")
            && !src.contains("contextKey"),
        "context-menu state is this controller's return, not context:\n{src}"
    );
    let item = stdlib_src("menu-item");
    assert!(
        item.contains("init.select")
            && item.contains("on:click={onSelect}")
            && !item.contains("injectContext")
            && !item.contains("contextKey"),
        "menu-item takes parent select as a prop:\n{item}"
    );
    let el = compile_element(src, "context-menu").unwrap_or_else(|e| panic!("context-menu: {e:?}"));
    assert!(
        !el.structure_html.contains("use:contextMenu")
            && !el.structure_html.contains("use:menu")
            && el.controller_js.is_some(),
        "context-menu is a component with a controller, not a widget action: {}",
        el.structure_html
    );
    let js = el.controller_js.as_deref().expect("context-menu controller");
    assert!(
        !js.contains("querySelector") && !js.contains("HTMLElement") && !js.contains("ctx.host"),
        "author controller stays platform-agnostic:\n{js}"
    );
}

#[test]
fn page_attaches_hotkeys_from_controller_instance() {
    let out = compile(
        r#"<tabs><tab title="A">one</tab></tabs>"#,
        page_opts(),
    )
    .expect("page with <tabs> compiles");
    assert!(
        out.js.contains("mountComponent") && !out.js.contains("runController"),
        "{}",
        out.js
    );
    let tabs = compile(
        stdlib_src("tabs"),
        CompileOpts {
            setup_url: Some("./tabs.sail.controller.ts".into()),
            source_path: Some(sail_compiler::stdlib::source_file("tabs")),
            emit_dts: false,
            ..page_opts()
        },
    )
    .expect("tabs compiles as tagless mount-form");
    let js = &tabs.js;
    assert!(
        js.contains("runAction") && js.contains("hotkeys") && js.contains(".keys"),
        "tabs module attaches hotkeys against its controller:\n{js}"
    );
    assert!(
        !js.contains("customElements.define") && !js.contains("component(\"tabs\""),
        "tabs is not a CE:\n{js}"
    );
    let tab = compile(
        stdlib_src("tab"),
        CompileOpts {
            setup_url: Some("./tab.sail.controller.ts".into()),
            source_path: Some(sail_compiler::stdlib::source_file("tab")),
            emit_dts: false,
            ..page_opts()
        },
    )
    .expect("tab compiles as mount-form");
    assert!(
        tab.js.contains("role") && tab.js.contains("tab"),
        "tab trigger is role=tab in the mount-form module:\n{}",
        tab.js
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
        "L1 is not CE:\n{js}"
    );
    assert!(!js.contains("use:tabs"), "{js}");
    assert!(
        js.contains("tabs.sail")
            && js.contains("mountComponent")
            && js.contains("__mod_tabs")
            && !js.contains("runController")
            && !js.contains("sail-l1:")
            && !js.contains("__ctrl_"),
        "page composes tabs as mount-form via mountComponent:\n{js}"
    );
}

#[test]
fn page_using_chip_is_not_ce() {
    let src = stdlib_src("chip");
    assert!(
        src.contains("<controller")
            && src.contains("use:hotkeys={keys}")
            && src.contains("selected={selected()}")
            && !src.contains("data-dismiss")
            && !src.contains("e.stopPropagation")
            && !src.contains("use:pressable")
            && !src.contains("use:pressed")
            && !src.contains("use:dismissible"),
        "chip is controller + toolkit, selected on the host:\n{src}"
    );
    let out = compile(r#"<chip>Design</chip>"#, page_opts()).expect("page with <chip> compiles");
    let js = &out.js;
    assert!(
        !js.contains("customElements.define") && !js.contains("component(\"chip\""),
        "L1 is not CE:\n{js}"
    );
    assert!(
        js.contains("chip.sail")
            && js.contains("mountComponent")
            && js.contains("__mod_chip")
            && !js.contains("runController")
            && !js.contains("sail-l1:")
            && !js.contains("__ctrl_"),
        "page composes chip as mount-form via mountComponent:\n{js}"
    );
}

#[test]
fn hyphenated_stdlib_tag_is_not_a_ce() {
    let out = compile(
        stdlib_src("tab-panel"),
        CompileOpts {
            setup_url: Some("./tab-panel.sail.controller.ts".into()),
            source_path: Some(sail_compiler::stdlib::source_file("tab-panel")),
            component_meta: Some(ComponentMeta {
                name: "tab-panel".into(),
            }),
            emit_dts: false,
            ..page_opts()
        },
    )
    .expect("tab-panel compiles");
    assert!(
        !out.js.contains("customElements.define")
            && !out.js.contains("component(\"tab-panel\""),
        "stdlib tag is mount-form even if the host passed a hyphenated name:\n{}",
        out.js
    );
}

#[test]
fn nested_tab_gets_parent_signal_props() {
    let out = compile(
        r#"<tabs><tab title="A">one</tab><tab title="B">two</tab></tabs>"#,
        page_opts(),
    )
    .expect("page with <tabs> compiles");
    let js = &out.js;
    assert!(
        js.contains("mountComponent")
            && js.contains(".instance.active")
            && js.contains(".instance.select")
            && !js.contains(".instance.active()"),
        "child tab gets parent active/select from the mount instance:\n{js}"
    );
    assert!(
        !js.contains("provideContext") && !js.contains("injectContext") && !js.contains("contextKey"),
        "compound L1 does not use context:\n{js}"
    );
}

#[test]
fn nested_radio_gets_parent_signal_props() {
    let out = compile(
        r#"<radio-group label="Billing period" name="plan" value="monthly">
  <radio value="monthly">Monthly</radio>
  <radio value="yearly">Yearly</radio>
</radio-group>"#,
        page_opts(),
    )
    .expect("page with <radio-group> compiles");
    let js = &out.js;
    assert!(
        js.contains("mountComponent")
            && js.contains(".instance.groupValue")
            && js.contains(".instance.select")
            && !js.contains(".instance.groupValue()"),
        "child radio gets parent groupValue/select from the mount instance:\n{js}"
    );
    assert!(
        !js.contains("provideContext")
            && !js.contains("injectContext")
            && !js.contains("contextKey")
            && !js.contains("use:radioGroup"),
        "compound L1 does not use context or use:radioGroup:\n{js}"
    );
}

#[test]
fn parameterized_use_attaches_in_controller_module() {
    let src = r#"
<scope></scope>
<controller>
export default controller(() => {
  const keys = { Escape: () => {} }
  return { keys }
})
</controller>
<div use:hotkeys={keys}></div>
"#;
    let el = compile_element(src, "probe").expect("controller + use:hotkeys={keys}");
    assert!(
        el.controller_js.is_some() && !el.structure_html.contains("use:tabs"),
        "{}",
        el.structure_html
    );
    let js = el.controller_js.expect("author controller");
    assert!(
        !js.contains("querySelector") && !js.contains("runAction"),
        "DOM attach is page emit, not the controller module:\n{js}"
    );
}

#[test]
fn controller_less_parameterized_use_still_refuses() {
    let err = compile_element("<div use:hotkeys={keys}></div>", "probe").unwrap_err();
    let msg = format!("{err:?}");
    assert!(
        msg.contains("use:hotkeys") || msg.contains("per-instance") || msg.contains("controller"),
        "{msg}"
    );
}

#[test]
fn page_using_button_does_not_attach_controller() {
    let out = compile("<button>Go</button>", page_opts()).expect("button page compiles");
    let js = &out.js;
    assert!(
        !js.contains("sail-l1:") && !js.contains("__ctrl_"),
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
fn no_l1_controller_runner_in_the_runtime() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let compose = std::fs::read_to_string(root.join("packages/sail/runtime/src/compose.ts"))
        .expect("compose");
    let internal = std::fs::read_to_string(root.join("packages/sail/runtime/src/internal.ts"))
        .expect("runtime internal");
    assert!(
        !compose.contains("runController") && !internal.contains("runController"),
        "controller().setup is enough — do not grow a second attach runner"
    );
}
