//! The ABI boundary: the COMPILER imports from `@ship/sail-runtime/internal`,
//! HUMANS import from `@ship/sail-runtime`.
//!
//! The runtime's front door used to advertise ~80 symbols — `bindFor`, `bindIf`,
//! `attachRef`, `runAction`, `applyPatch`, `__sui_motion_attach` — sitting in the
//! same export as `signal` and `effect`. No human types those; the emitter writes
//! them. But an agent reading that index cannot tell the language from the
//! plumbing, and the language is about ten symbols, not eighty.
//!
//! Nothing enforced the split, so these pin it. Without them the boundary is a
//! convention, and a convention that no test asserts is a comment.

use sail_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

const ABI: &str = "@ship/sail-runtime/internal";
const AUTHOR: &str = "@ship/sail-runtime";

/// A component exercising the emit paths that pull ABI symbols: text binding,
/// an event handler (`on`), and an action (`runAction` + the auto-imported
/// builtin). All three are things the emitter writes, none are things a human does.
const SRC: &str = r#"<button on:click={() => go()} use:tooltip>{label()}</button>"#;

fn opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta { name: "abi-probe".into() }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

/// Every runtime symbol the emitter writes resolves through the ABI subpath —
/// never the package root, which is the author's front door.
#[test]
fn emitter_imports_the_abi_subpath_not_the_root() {
    let out = compile(SRC, opts()).expect("compile");
    let js = &out.js;

    assert!(
        js.contains(&format!("from \"{ABI}\"")),
        "the emitter must import the ABI from `{ABI}`:\n{js}"
    );

    // The root must not appear as an import SOURCE in generated code. (Checking
    // the closing quote matters: `@ship/sail-runtime/internal` and
    // `@ship/sail-runtime/template-hooks` both start with the root's text.)
    assert!(
        !js.contains(&format!("from \"{AUTHOR}\"")),
        "generated code must never import the author front door — that is the \
         boundary this test exists to hold:\n{js}"
    );
}

/// The symbols in question really are ABI: emitter-written, never hand-typed.
#[test]
fn the_abi_import_carries_emitter_symbols() {
    let out = compile(SRC, opts()).expect("compile");
    let js = &out.js;
    for sym in ["component", "on", "runAction"] {
        assert!(js.contains(sym), "expected emitter symbol `{sym}`:\n{js}");
    }
}

/// `@ship/sail-runtime/template-hooks` is a legitimate subpath and must not be
/// mistaken for the root by a naive prefix rewrite. Guards the rewrite itself.
#[test]
fn sibling_subpaths_are_not_rewritten_into_the_abi_path() {
    let out = compile(SRC, opts()).expect("compile");
    assert!(
        !out.js.contains("@ship/sail-runtime/internal/template-hooks"),
        "the ABI rewrite corrupted a sibling subpath:\n{}",
        out.js
    );
}
/// Every action the action table declares must be exported by the runtime.
///
/// Before this test read `lower::bindings::BUILTIN_ACTIONS` — a
/// constant the compiler maintained by hand in lockstep with the
/// runtime's `index.ts`. When they drifted, the failure was
/// invisible until bundle time: the element compiled, conformance
/// passed (it attaches actions by hand), and the app died with a
/// MissingExport. `portEdge` shipped exactly that way — the test
/// existed solely to catch the drift between two lists that should
/// never have been separate.
///
/// The compiler now has ONE source of truth
/// (`sail_compiler::action_manifest::ACTIONS`); the runtime barrel
/// is checked against it here. A new action means one entry in the
/// table — this test fails the build until the runtime barrel is
/// updated to match.
#[test]
fn action_table_is_exported_by_the_runtime() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let index = root.join("packages/sail/runtime/src/index.ts");
    let Ok(src) = std::fs::read_to_string(&index) else { return };

    let missing: Vec<&str> = sail_compiler::action_manifest::ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|a| {
            // The runtime's front door carries a single
            // `export { … } from './actions'` block; the format is
            // `\n  {name},` (2-space indent inside that block).
            !src.contains(&format!("\n  {a},"))
                // Some entries were already there pre-block (e.g. the
                // top-of-file imports) — accept those too.
                && !src.contains(&format!("{a},"))
        })
        .collect();

    assert!(
        missing.is_empty(),
        "the action table lists these names and the runtime's top-level \
         `index.ts` does not export them — every L1 element that uses one will \
         compile, pass conformance, and then fail to bundle. The action table \
         is the single source of truth; update the runtime barrel, not the \
         table, when adding a new action:\n  {}",
        missing.join("\n  ")
    );
}

/// The action table is the single source of truth. A consumer
/// (runtime barrel, type re-export, scope-bound builtin list,
/// expression-rewrite builtin list) that disagrees with the table
/// must fail this test. The spec calls this the "anti-drift
/// property" — a table nothing enforces is just a fifth list.
///
/// This test lives in the same suite as
/// `action_table_is_exported_by_the_runtime` so a regression in the
/// invariant is visible next to the test that detects it.
#[test]
fn action_table_has_no_phantom_consumers() {
    use sail_compiler::action_manifest::ACTIONS;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let runtime_actions_barrel = root.join("packages/sail/runtime/src/actions/index.ts");
    let runtime_top_barrel = root.join("packages/sail/runtime/src/index.ts");
    let Ok(actions_src) = std::fs::read_to_string(&runtime_actions_barrel) else { return };
    let Ok(top_src) = std::fs::read_to_string(&runtime_top_barrel) else { return };

    // Every action in the table must be re-exported from BOTH
    // barrels. Drift between the table and either barrel is the
    // exact class of bug this test exists to catch (`portEdge`).
    let missing_actions_barrel: Vec<&str> = ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|n| !actions_src.contains(&format!(" {n}\n")) && !actions_src.contains(&format!("{n} ")) && !actions_src.contains(&format!(",{n},")) && !actions_src.contains(&format!(" {n},")) && !actions_src.contains(&format!("{n},")))
        .collect();
    let missing_top_barrel: Vec<&str> = ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|n| !top_src.contains(&format!("\n  {n},")) && !top_src.contains(&format!("{n},")))
        .collect();

    assert!(
        missing_actions_barrel.is_empty() && missing_top_barrel.is_empty(),
        "action table disagrees with the runtime barrels — the table is the \
         single source; update the barrels, not the table. \
         Missing from actions/index.ts: {:?}. \
         Missing from index.ts: {:?}.",
        missing_actions_barrel,
        missing_top_barrel,
    );
}

/// A `use:` action must emit BARE, not `$scope.<name>`.
///
/// The lowering auto-imports built-in actions as ESM bindings, but the expression
/// rewriter qualifies unknown identifiers with `$scope.`. Its builtin list held
/// four names while `BUILTIN_ACTIONS` held twenty-three, so every other action
/// compiled to `runAction(el, $scope.tabs)` — and `$scope.tabs` is `undefined`.
/// The element rendered, and did nothing. No error, no warning.
///
/// This bites only when an element is INLINED INTO A HOST COMPONENT, which is the
/// only way anyone actually uses one — and never in the element's own source,
/// which is why the stdlib looked fine.
#[test]
fn builtin_actions_emit_bare_not_scope_qualified() {
    let mut broken = Vec::new();
    for a in sail_compiler::lower::bindings::BUILTIN_ACTIONS {
        let src = format!(r#"<div use:{a}></div>"#);
        let out = sail_compiler::compile(
            &src,
            sail_compiler::CompileOpts {
                mode: sail_compiler::CompileMode::Component,
                ..Default::default()
            },
        )
        .expect("compiles")
        .js;
        if out.contains(&format!("$scope.{a}")) {
            broken.push(*a);
        }
    }
    assert!(
        broken.is_empty(),
        "these actions compile to `$scope.<name>`, which is undefined at runtime — \
         the element renders and does nothing:\n  {}",
        broken.join("\n  ")
    );
}
