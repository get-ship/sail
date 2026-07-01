//! `emit_component_imports` — the per-view resolver's compiler half. When set,
//! the compiled module side-effect-imports `@sui-tag/<tag>` for every
//! custom-element tag the template uses, so components enter the import graph
//! and the bundler tree-shakes exactly what a view needs. Off by default: the
//! umbrella/islands registry path is untouched until the bundler opts in.

use sui_compiler::{compile, CompileOpts};

#[test]
fn off_by_default_emits_no_component_imports() {
    let out = compile(
        "<div><sui-table></sui-table></div>",
        CompileOpts::default(),
    )
    .expect("compile");
    assert!(
        !out.js.contains("@sui-tag/"),
        "no component imports unless opted in; got:\n{}",
        out.js
    );
}

#[test]
fn flag_emits_registration_import_per_used_custom_tag() {
    let opts = CompileOpts { emit_component_imports: true,
            lint_overrides: Vec::new(), ..CompileOpts::default() };
    let out = compile(
        "<div><sui-table></sui-table><ship-card></ship-card><span>plain</span></div>",
        opts,
    )
    .expect("compile");
    assert!(
        out.js.contains("import \"@sui-tag/sui-table\";"),
        "side-effect import for sui-table; got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("import \"@sui-tag/ship-card\";"),
        "side-effect import for ship-card; got:\n{}",
        out.js
    );
    assert!(!out.js.contains("@sui-tag/span"), "plain HTML tags are not components");
    assert!(!out.js.contains("@sui-tag/div"), "plain HTML tags are not components");
}

#[test]
fn flag_collects_tags_nested_in_control_flow() {
    let opts = CompileOpts { emit_component_imports: true,
            lint_overrides: Vec::new(), ..CompileOpts::default() };
    let out = compile(
        "<div>{#if cond()}<sui-a></sui-a>{:else}<sui-b></sui-b>{/if}</div>",
        opts,
    )
    .expect("compile");
    assert!(out.js.contains("import \"@sui-tag/sui-a\";"), "tag in #if branch; got:\n{}", out.js);
    assert!(out.js.contains("import \"@sui-tag/sui-b\";"), "tag in :else branch");
}
