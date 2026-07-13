//! End-to-end: a `.controller.ts` that writes `.innerHTML` must fail
//! the compile, not just lint. Mirrors the existing
//! handler_complexity test's filesystem shape.

use std::fs;
use sui_compiler::{compile, CompileMode, CompileOpts};

fn view_opts(source_path: Option<std::path::PathBuf>) -> CompileOpts {
    CompileOpts {
        setup_url: Some("./view.controller.ts".into()),
        mode: CompileMode::Component,
        source_path,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn impure_controller_fails_compile() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(
        dir.path().join("view.controller.ts"),
        r#"
            export default {
              mount: (_state, host) => {
                host.innerHTML = '<p>nope</p>'
                return () => {}
              },
            }
        "#,
    )
    .unwrap();
    let sui_path = dir.path().join("view.sui");
    let src = r#"<div></div>"#;
    fs::write(&sui_path, src).unwrap();

    let err = compile(src, view_opts(Some(sui_path)))
        .expect_err("controller writing .innerHTML must fail compile");
    assert_eq!(
        err.category.as_deref(),
        Some("sui-controller-purity"),
        "expected rule id as the error category, got: {:?}",
        err.category
    );
    assert!(
        err.message.contains("innerHTML"),
        "expected message to name the bad property, got: {}",
        err.message
    );
}

#[test]
fn clean_controller_compiles() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(
        dir.path().join("view.controller.ts"),
        r#"
            import { controller } from '@ship/sui-runtime'
            export default controller({
              mount: (_state, host) => {
                host.addEventListener('click', () => {})
                return () => {}
              },
            })
        "#,
    )
    .unwrap();
    let sui_path = dir.path().join("view.sui");
    let src = r#"<div></div>"#;
    fs::write(&sui_path, src).unwrap();

    compile(src, view_opts(Some(sui_path)))
        .expect("clean controller pair should compile");
}

#[test]
fn no_controller_paired_skips_rule() {
    // Sanity: a .sui without a paired .controller.ts compiles fine.
    let dir = tempfile::tempdir().expect("tempdir");
    let sui_path = dir.path().join("view.sui");
    fs::write(&sui_path, r#"<div></div>"#).unwrap();

    compile(r#"<div></div>"#, view_opts(Some(sui_path)))
        .expect("no controller — rule must be skipped");
}
