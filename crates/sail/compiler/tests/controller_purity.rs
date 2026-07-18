//! End-to-end: a `.controller.ts` that writes `.innerHTML` must fail
//! the compile, not just lint. Mirrors the existing
//! handler_complexity test's filesystem shape.

use std::fs;
use sail_compiler::{compile, CompileMode, CompileOpts};

fn view_opts(source_path: Option<std::path::PathBuf>) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: Some("./view.controller.ts".into()),
        mode: CompileMode::Component,
        source_path,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn impure_controller_fails_compile() {
    // The controller is the source's `<controller>` BLOCK — a sibling
    // `.controller.ts` is itself a compile error now (`sail-paired-controller`), so
    // a fixture written that way never reaches the purity gate at all. This test
    // used to pass for the wrong reason and then silently stopped testing anything
    // when the gate was repointed at the block.
    let dir = tempfile::tempdir().expect("tempdir");
    let sui_path = dir.path().join("view.sail");
    let src = r#"<controller>
export default controller((ctx, host) => {
  mount((el) => {
    host.innerHTML = '<p>nope</p>'
    return () => {}
  })
  return {}
})
</controller>
<div></div>"#;
    fs::write(&sui_path, src).unwrap();

    let err = compile(src, view_opts(Some(sui_path)))
        .expect_err("a controller writing .innerHTML must fail the compile");
    assert_eq!(
        err.category.as_deref(),
        Some("sail-controller-purity"),
        "expected the rule id as the error category, got: {:?}",
        err.category
    );
    assert!(
        err.message.contains("innerHTML"),
        "the message must name the bad property, got: {}",
        err.message
    );
}


#[test]
fn clean_controller_compiles() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sui_path = dir.path().join("view.sail");
    let src = r#"<controller>
import { controller } from '@ship/sail-runtime'
export default controller((ctx, host) => {
  mount((el) => {
    host.addEventListener('click', () => {})
    return () => {}
  })
  return {}
})
</controller>
<div></div>"#;
    fs::write(&sui_path, src).unwrap();

    compile(src, view_opts(Some(sui_path))).expect("a clean inline controller compiles");
}

/// The one spelling: a sibling `<stem>.controller.ts` is a compile error, not a
/// second way to author a controller.
#[test]
fn paired_controller_file_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(
        dir.path().join("view.controller.ts"),
        "export default controller((ctx) => ({}))",
    )
    .unwrap();
    let sui_path = dir.path().join("view.sail");
    let src = r#"<div></div>"#;
    fs::write(&sui_path, src).unwrap();

    let err = compile(src, view_opts(Some(sui_path))).unwrap_err();
    assert_eq!(err.category.as_deref(), Some("sail-paired-controller"), "{err:?}");
}

#[test]
fn no_controller_paired_skips_rule() {
    // Sanity: a .sail without a paired .controller.ts compiles fine.
    let dir = tempfile::tempdir().expect("tempdir");
    let sui_path = dir.path().join("view.sail");
    fs::write(&sui_path, r#"<div></div>"#).unwrap();

    compile(r#"<div></div>"#, view_opts(Some(sui_path)))
        .expect("no controller — rule must be skipped");
}
