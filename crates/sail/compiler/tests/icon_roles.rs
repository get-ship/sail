//! SPEC-ICONS-1 — unknown role is a compile error with close matches.

use sail_compiler::{CompileOpts, compile};

#[test]
fn unknown_icon_role_is_compile_error_with_close_match() {
    let src = r#"<column><icon name="setttings"></icon></column>"#;
    let err = compile(src, CompileOpts::default()).expect_err("unknown role must fail");
    assert_eq!(err.category.as_deref(), Some("sail-unknown-icon"));
    assert!(
        err.message.contains("setttings"),
        "names the unknown role: {}",
        err.message
    );
    assert!(
        err.message.contains("settings"),
        "lists close match: {}",
        err.message
    );
}

#[test]
fn known_icon_role_compiles() {
    let src = r#"<column><icon name="settings"></icon></column>"#;
    compile(src, CompileOpts::default()).expect("core role must compile");
}

#[test]
fn pack_prefix_is_compile_error() {
    let src = r#"<column><icon name="lucide:settings"></icon></column>"#;
    let err = compile(src, CompileOpts::default()).expect_err("prefix must fail");
    assert_eq!(err.category.as_deref(), Some("sail-unknown-icon"));
    assert!(
        err.message.contains("roles, not pack prefixes"),
        "{}",
        err.message
    );
}
