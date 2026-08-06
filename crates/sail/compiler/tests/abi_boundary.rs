//! The ABI boundary: the COMPILER imports from `@ship/sail-runtime/internal`,
//! HUMANS import from `@ship/sail-runtime`.
//!
//! The runtime's front door used to advertise ~80 symbols — `bindFor`, `bindIf`,
//! `attachRef`, `runAction`, `applyPatch`, `__sail_motion_attach` — sitting in the
//! same export as `signal` and `effect`. No human types those; the emitter writes
//! them. But an agent reading that index cannot tell the language from the
//! plumbing, and the language is about ten symbols, not eighty.
//!
//! Nothing enforced the split, so these pin it. Without them the boundary is a
//! convention, and a convention that no test asserts is a comment.

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

const ABI: &str = "@ship/sail-runtime/internal";
const AUTHOR: &str = "@ship/sail-runtime";

/// A component exercising the emit paths that pull ABI symbols: text binding,
/// an event handler (`on`), and an action (`runAction` + the auto-imported
/// builtin). All three are things the emitter writes, none are things a human does.
const SRC: &str = r#"<button on:click={() => go()} use:tipDescribe>{label()}</button>"#;

fn opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta {
            name: "abi-probe".into(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
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

/// The author front door must not re-export the worst ABI symbols. Those live
/// on `/internal` only — an agent reading the root index should not see
/// `bindFor` / `runAction` / `__tpl` offered as language.
#[test]
fn root_barrel_does_not_export_abi_plumbing() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let index = root.join("packages/sail/runtime/src/index.ts");
    let internal = root.join("packages/sail/runtime/src/internal.ts");
    let Ok(index_src) = std::fs::read_to_string(&index) else {
        return;
    };
    let Ok(internal_src) = std::fs::read_to_string(&internal) else {
        return;
    };

    // Must be more than a lone `export * from './index'` — curated ABI blocks
    // must follow the language re-export.
    let only_total_reexport = internal_src
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("//")
        })
        .all(|l| l.trim() == "export * from './index'");
    assert!(
        !only_total_reexport,
        "`internal.ts` must be a curated ABI list (not only `export * from './index'`)"
    );
    // Curated ABI list must name the emitters' worst symbols.
    for sym in [
        "bindFor",
        "bindIf",
        "bindText",
        "runAction",
        "__tpl",
        "__sail_motion_attach",
        "delegateEvents",
        "mountComponent",
        "markHydrateText",
        "wireFormMutation",
        "createScope",
        "withScope",
    ] {
        assert!(
            internal_src.contains(sym),
            "internal ABI barrel must export/mention `{sym}`:\n(check packages/sail/runtime/src/internal.ts)"
        );
    }

    // Root must not value-export these (comments may still mention the names).
    let mut leaked = Vec::new();
    for name in [
        "bindFor",
        "bindIf",
        "bindText",
        "runAction",
        "__tpl",
        "__sail_motion_attach",
        "delegateEvents",
        "mountComponent",
        "wireFormMutation",
        "createMountState",
        "componentMountState",
        "createScope",
        "withScope",
        "bindOpen",
        "bindPopover",
        "morph",
        "bindRawHtml",
        "markHydrateText",
    ] {
        let leaked_here = index_src.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("export ")
                && !t.starts_with("export type ")
                && (t.contains(&format!(" {name}"))
                    || t.contains(&format!("{{{name}"))
                    || t.contains(&format!(", {name}"))
                    || t.contains(&format!("{name},"))
                    || t.contains(&format!("{name} }}"))
                    || t.contains(&format!("{{ {name} }}"))
                    || t.contains(&format!("{{ {name},"))
                    || t.contains(&format!(", {name} }}"))
                    || t.contains(&format!("{name} as ")))
        });
        if leaked_here {
            leaked.push(name);
        }
    }
    assert!(
        leaked.is_empty(),
        "root index.ts still value-exports ABI symbols (move to /internal):\n  {}",
        leaked.join("\n  ")
    );
}

/// `@ship/sail-runtime/template-hooks` is a legitimate subpath and must not be
/// mistaken for the root by a naive prefix rewrite. Guards the rewrite itself.
#[test]
fn sibling_subpaths_are_not_rewritten_into_the_abi_path() {
    let out = compile(SRC, opts()).expect("compile");
    assert!(
        !out.js
            .contains("@ship/sail-runtime/internal/template-hooks"),
        "the ABI rewrite corrupted a sibling subpath:\n{}",
        out.js
    );
}
/// Every *public* action the action table declares must be exported by
/// the runtime front door (`index.ts`). Internal stamps are NOT on the
/// front door — they live on `@ship/sail-runtime/actions` only (see
/// `action_table_has_no_phantom_consumers` / `public_action_surface_is_curated`).
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
/// (`sail_compiler::action_manifest::ACTIONS` + `PUBLIC_ACTION_NAMES`);
/// the runtime barrel is checked against the public subset here.
#[test]
fn action_table_is_exported_by_the_runtime() {
    use sail_compiler::action_manifest::{ACTIONS, PUBLIC_ACTION_COUNT, is_public_action};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let index = root.join("packages/sail/runtime/src/index.ts");
    let Ok(src) = std::fs::read_to_string(&index) else {
        return;
    };

    assert_eq!(PUBLIC_ACTION_COUNT, 11);

    let missing: Vec<&str> = ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|a| is_public_action(a))
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
        "PUBLIC_ACTION_NAMES lists these names and the runtime's top-level \
         `index.ts` does not export them — the taught public surface must \
         match the front door:\n  {}",
        missing.join("\n  ")
    );
}

/// The action table is the single source of truth. A consumer
/// that disagrees with the table must fail this test.
///
/// - ALL actions must appear in `actions/index.ts` (full auto-import surface)
/// - ONLY public actions may appear as `\n  {name},` in the top-level
///   `export { … } from './actions'` block
#[test]
fn action_table_has_no_phantom_consumers() {
    use sail_compiler::action_manifest::{ACTIONS, PUBLIC_ACTION_COUNT, is_public_action};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let runtime_actions_barrel = root.join("packages/sail/runtime/src/actions/index.ts");
    let runtime_top_barrel = root.join("packages/sail/runtime/src/index.ts");
    let Ok(actions_src) = std::fs::read_to_string(&runtime_actions_barrel) else {
        return;
    };
    let Ok(top_src) = std::fs::read_to_string(&runtime_top_barrel) else {
        return;
    };

    assert_eq!(PUBLIC_ACTION_COUNT, 11);

    // Every action in the table must be re-exported from the full
    // actions barrel. Drift here is the class of bug this test exists
    // to catch (`portEdge`) — the compiler auto-imports from this path.
    let missing_actions_barrel: Vec<&str> = ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|n| {
            !actions_src.contains(&format!(" {n}\n"))
                && !actions_src.contains(&format!("{n} "))
                && !actions_src.contains(&format!(",{n},"))
                && !actions_src.contains(&format!(" {n},"))
                && !actions_src.contains(&format!("{n},"))
        })
        .collect();

    // Top-level front door: only public names as `\n  {name},` inside
    // the actions re-export block. Non-public actions must not appear
    // in that form (careful not to false-positive other exports).
    let leaked_internal: Vec<&str> = ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|n| !is_public_action(n))
        .filter(|n| top_src.contains(&format!("\n  {n},")))
        .collect();

    let missing_public_top: Vec<&str> = ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|n| is_public_action(n))
        .filter(|n| !top_src.contains(&format!("\n  {n},")))
        .collect();

    assert!(
        missing_actions_barrel.is_empty()
            && leaked_internal.is_empty()
            && missing_public_top.is_empty(),
        "action table disagrees with the runtime barrels — the table is the \
         single source; update the barrels, not the table. \
         Missing from actions/index.ts: {:?}. \
         Non-public leaked into top-level actions export: {:?}. \
         Public missing from top-level actions export: {:?}.",
        missing_actions_barrel,
        leaked_internal,
        missing_public_top,
    );
}

/// The curated public `use:` surface is exactly 11 names (ACTIONS-1 diet;
/// icon is widget-private with the L1 `<icon>` element), all present in
/// ACTIONS, sorted, and the only action names on the runtime front door.
/// The compiler still auto-imports *all* actions from
/// `@ship/sail-runtime/actions` (stdlib needs internal stamps for
/// tabs/menu/pagination/radioGroup/combobox/icon closed widget impl).
#[test]
fn public_action_surface_is_curated() {
    use sail_compiler::action_manifest::{
        ACTIONS, PUBLIC_ACTION_COUNT, PUBLIC_ACTION_NAMES, is_public_action, public_actions,
    };

    assert_eq!(PUBLIC_ACTION_COUNT, 11);
    assert_eq!(PUBLIC_ACTION_NAMES.len(), 11);
    assert_eq!(public_actions().count(), 11);

    let mut sorted = PUBLIC_ACTION_NAMES.to_vec();
    sorted.sort();
    assert_eq!(
        PUBLIC_ACTION_NAMES.as_slice(),
        sorted.as_slice(),
        "PUBLIC_ACTION_NAMES must be sorted alphabetically"
    );

    for name in *PUBLIC_ACTION_NAMES {
        assert!(
            ACTIONS.iter().any(|a| a.name == name),
            "public action `{name}` missing from ACTIONS"
        );
        assert!(is_public_action(name));
    }

    // Emitter pulls every builtin action (including internal) from the
    // dedicated actions subpath — never from the front door or ABI barrel
    // alone, so stripping internal names from index.ts does not break stdlib.
    let src = r#"<div use:recoverySurface use:tabs use:toggleGroup></div>"#;
    let out = sail_compiler::compile(
        src,
        sail_compiler::CompileOpts {
            mode: sail_compiler::CompileMode::Component,
            ..Default::default()
        },
    )
    .expect("compiles")
    .js;
    assert!(
        out.contains("from \"@ship/sail-runtime/actions\""),
        "auto-imported actions must come from `@ship/sail-runtime/actions`:\n{out}"
    );
    for name in ["recoverySurface", "tabs", "toggleGroup"] {
        assert!(
            out.contains(name),
            "expected auto-import of internal/public action `{name}`:\n{out}"
        );
    }
    // Must not pull actions through the author front door (exact root
    // specifier — subpaths like /actions and /internal are fine).
    assert!(
        !out.contains("from \"@ship/sail-runtime\";")
            && !out.contains("from \"@ship/sail-runtime\"\n"),
        "generated code must not import the author front door for actions:\n{out}"
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
        // Pairing checks require the sx bridge on the same node for some actions.
        let src = match *a {
            "virtualRow" => {
                r#"<div use:virtualRow> style({ paint: translateY(bind(rowY) })) </div>"#.to_string()
            }
            "virtualSpacer" => {
                r#"<div use:virtualSpacer> style({ box: h(bind(rowsH) })) </div>"#.to_string()
            }
            _ => format!(r#"<div use:{a}></div>"#),
        };
        let out = sail_compiler::compile(
            &src,
            sail_compiler::CompileOpts {
                mode: sail_compiler::CompileMode::Component,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("compiles for use:{a}: {e:?}"))
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
