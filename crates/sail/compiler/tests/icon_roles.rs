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

#[test]
fn lucide_id_alias_still_compiles() {
    let src = r#"<column><icon name="refresh-cw"></icon><icon name="trash-2"></icon></column>"#;
    compile(src, CompileOpts::default()).expect("resolve aliases must compile");
}

#[test]
fn named_pack_icon_html_contains_svg() {
    let src = r#"<icon name="settings"></icon>"#;
    let out = compile(src, CompileOpts::default()).expect("named pack role compiles");
    assert!(
        out.js.contains("<svg"),
        "compiled <icon name> HTML contains <svg> (compiler inline, not fetch):\n{}",
        out.js
    );
    assert!(
        !out.js.contains("/_deck/icons"),
        "pack roles do not fetch /_deck/icons:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("primeIconCache"),
        "pack roles do not use primeIconCache:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("use:icon"),
        "no use:icon stamp survives (icon is pure structure):\n{}",
        out.js
    );
}

#[test]
fn unlabeled_icon_is_decorative_and_hidden() {
    let out =
        compile(r#"<icon name="settings"></icon>"#, CompileOpts::default()).expect("compiles");
    assert!(
        out.js.contains("aria-hidden=\\\"true\\\""),
        "decorative icon has aria-hidden at compile time:\n{}",
        out.js
    );
    assert!(!out.js.contains("role=\\\"img\\\""), "{}", out.js);
}

#[test]
fn labeled_icon_is_an_image_with_name() {
    let out = compile(
        r#"<icon label="Close" name="settings"></icon>"#,
        CompileOpts::default(),
    )
    .expect("compiles");
    assert!(
        out.js.contains("role=\\\"img\\\""),
        "labeled icon is role=img (anatomy, not runtime):\n{}",
        out.js
    );
    assert!(
        out.js.contains("aria-label=\\\"Close\\\""),
        "label lowers to aria-label:\n{}",
        out.js
    );
    assert!(!out.js.contains("aria-hidden"), "{}", out.js);
}

#[test]
fn catalog_roles_compile() {
    let src = r#"<column>
      <icon name="sparkles"></icon>
      <icon name="heart"></icon>
      <icon name="inbox"></icon>
      <icon name="plus"></icon>
      <icon name="download"></icon>
      <icon name="user"></icon>
      <icon name="bell"></icon>
      <icon name="mail"></icon>
    </column>"#;
    compile(src, CompileOpts::default()).expect("catalog roles must compile");
}
