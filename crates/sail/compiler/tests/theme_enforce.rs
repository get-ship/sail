//! THEMES-1-ENFORCE + THEMES-2 §1: component-token invisibility + substrate.

use std::path::PathBuf;

use sail_compiler::{CompileMode, CompileOpts, compile};
use sail_lint::rules::no_component_token;
use sail_lint::rules::no_data_attr;

fn app_opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Auto,
        source_path: Some(PathBuf::from("apps/demo/src/page.sail")),
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn compile_err(src: &str) -> String {
    let err = compile(src, app_opts()).expect_err("expected compile error");
    format!("{err}")
}

#[test]
fn style_bg_button_is_theme_owned() {
    let msg = compile_err(r#"<div style({ paint: bg(button) })>x</div>"#);
    assert!(
        msg.contains("theme-owned") && msg.contains("theme="),
        "teaching missing: {msg}"
    );
    assert!(msg.contains("button"), "must name the token: {msg}");
}

#[test]
fn style_bg_button_dot_bg_is_theme_owned() {
    let msg = compile_err(r#"<div style({ paint: bg(button.bg) })>x</div>"#);
    assert!(
        msg.contains("theme-owned") && msg.contains("button.bg"),
        "{msg}"
    );
}

#[test]
fn style_bg_card_is_theme_owned() {
    let msg = compile_err(r#"<div style({ paint: bg(card) })>x</div>"#);
    assert!(msg.contains("theme-owned") && msg.contains("card"), "{msg}");
}

#[test]
fn style_bg_primary_ok() {
    compile(r#"<div style({ paint: bg(primary) })>x</div>"#, app_opts())
        .expect("bg(primary) must compile");
}

#[test]
fn theme_eq_id_ok() {
    compile(r#"<column theme="ship">x</column>"#, app_opts()).expect("theme= id must compile");
}

#[test]
fn unsanctioned_attr_button_bg_is_red() {
    let f = no_component_token::check(
        std::path::Path::new("apps/demo/src/page.sail"),
        r#"<div button.bg="primary">x</div>"#,
    );
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!(f[0].rule, no_component_token::RULE_ID);
    assert!(
        f[0].message.contains("theme-owned") && f[0].message.contains("theme="),
        "{}",
        f[0].message
    );
}

#[test]
fn sanctioned_theme_json_component_path_is_known() {
    assert!(sail_lint::component_surfaces::is_known_component_storage_path("component.button.bg"));
    assert!(
        sail_lint::component_surfaces::is_known_component_storage_path(
            "modes.light.component.card.bg.$value"
        )
    );
}

#[test]
fn data_theme_product_mode_fire_sail_no_data_attr() {
    for (src, name) in [
        (r#"<column data-theme="ship">x</column>"#, "data-theme"),
        (r#"<column data-product="sail">x</column>"#, "data-product"),
        (r#"<column data-mode="dark">x</column>"#, "data-mode"),
    ] {
        let f = no_data_attr::check(std::path::Path::new("apps/demo/src/page.sail"), src);
        assert_eq!(f.len(), 1, "{name}: {f:?}");
        assert_eq!(f[0].rule, no_data_attr::RULE_ID);
        assert!(
            f[0].message.contains("theme="),
            "{name} must teach theme=: {}",
            f[0].message
        );
        let msg = compile_err(src);
        assert!(
            msg.contains("theme=") || msg.contains("data-"),
            "{name} must fail compile: {msg}"
        );
    }
}
