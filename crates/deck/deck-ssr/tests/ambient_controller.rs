//! Ambient inject on SSR inline-controller materialize.
//!
//! Production path: `synthesize_and_bundle_with` extracts `<controller>`, then
//! `materialize_inline_controller` (query_keys rewrite → wrap_controller_body)
//! writes `page.inline.controller.ts`. Bare `signal()` / `controller()` must
//! land as imports from `@ship/sail-runtime/internal`.

use deck_ssr::build::{materialize_inline_controller, repo_root, synthesize_and_bundle};
use sail_compiler::CompileMode;
use sail_compiler::parse::preprocess::extract_controller_block;

const BARE_AMBIENT_ROUTE: &str = r#"
<controller>
export default controller(() => {
  const count = signal(0)
  const doubled = derived(() => count() * 2)
  return {
    count,
    doubled,
    inc: () => count.set(count() + 1),
  }
})
</controller>
<button on:click={inc}>{count()} / {doubled()}</button>
"#;

const MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";

fn ambient_import_line(body: &str) -> Option<&str> {
    body.lines().find(|l| {
        let t = l.trim_start();
        t.starts_with("import") && t.contains("@ship/sail-runtime/internal")
    })
}

fn assert_ambient(body: &str, idents: &[&str]) {
    let line = ambient_import_line(body)
        .unwrap_or_else(|| panic!("expected ambient import from /internal, got module:\n{body}"));
    for ident in idents {
        assert!(
            line.contains(ident),
            "ambient `{ident}` missing from inject line `{line}`\nmodule:\n{body}"
        );
    }
}

#[test]
fn materialize_inline_controller_injects_bare_ambient() {
    assert!(
        !BARE_AMBIENT_ROUTE.contains("from '@ship/sail-runtime'")
            && !BARE_AMBIENT_ROUTE.contains("from \"@ship/sail-runtime\""),
        "fixture must use bare ambient only"
    );
    let (_, inline) = extract_controller_block(BARE_AMBIENT_ROUTE);
    let inline = inline.expect("inline <controller>");
    let body = materialize_inline_controller(&inline).expect("materialize");
    assert_ambient(&body, &["controller", "signal", "derived"]);
}

/// Full synthesize path: written `page.inline.controller.ts` is ambient-wrapped.
#[test]
fn synthesize_writes_ambient_wrapped_inline_controller() {
    let repo = repo_root().expect("repo root");
    let route_rel = "target/deck-ssr-fixtures/bare-ambient-page/page.sail";
    let route_path = repo.join(route_rel);
    std::fs::create_dir_all(route_path.parent().unwrap()).expect("mkdir fixture");
    std::fs::write(&route_path, BARE_AMBIENT_ROUTE).expect("write fixture");

    let build_dir = repo.join("target/deck-ssr-test/bare-ambient");
    let _ = std::fs::remove_dir_all(&build_dir);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio");
    let _bundle = rt
        .block_on(synthesize_and_bundle(
            &repo,
            route_rel,
            MANIFEST,
            CompileMode::Component,
            &[],
            &build_dir,
        ))
        .expect("synthesize + bundle bare ambient route");

    let written = std::fs::read_to_string(build_dir.join("page.inline.controller.ts"))
        .expect("page.inline.controller.ts written by materialize path");
    assert_ambient(&written, &["controller", "signal", "derived"]);
}
