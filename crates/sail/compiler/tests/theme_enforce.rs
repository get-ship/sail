//! Theme substrate after component.* deletion.

use std::path::PathBuf;

use sail_compiler::{CompileMode, CompileOpts, compile};
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
fn style_bg_card_is_legal_global() {
    compile(r#"<div style({ bg({ color: card }) })>x</div>"#, app_opts())
        .expect("card is a global color");
}

#[test]
fn style_bg_primary_ok() {
    compile(
        r#"<div style({ bg({ color: primary }) })>x</div>"#,
        app_opts(),
    )
    .expect("bg({ color: primary }) must compile");
}

#[test]
fn theme_eq_id_ok() {
    compile(r#"<column theme="sail">x</column>"#, app_opts()).expect("theme= id must compile");
}

#[test]
fn component_storage_paths_are_gone() {
    assert!(!spinnaker::component_surfaces::is_known_component_storage_path("component.button.bg"));
    assert!(!spinnaker::component_surfaces::is_theme_owned_token_name(
        "button"
    ));
    assert!(!spinnaker::component_surfaces::is_theme_owned_token_name(
        "card"
    ));
}

#[test]
fn data_theme_product_mode_fire_sail_no_data_attr() {
    for (src, name) in [
        (r#"<column data-theme="sail">x</column>"#, "data-theme"),
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
