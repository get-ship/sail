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
    // Motion attach / query / virtualizer live on their own singletons.
    for sym in [
        "bindFor",
        "bindIf",
        "bindText",
        "runAction",
        "__tpl",
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

/// Hello-world does not import query / motion / virtual / actions.
#[test]
fn hello_world_does_not_import_batteries() {
    let src = "<column><h1>Hi</h1><p>Sail app ready.</p></column>";
    let out = compile(src, opts()).expect("compile");
    for banned in [
        "@ship/sail-runtime/query",
        "@ship/sail-runtime/motion",
        "@ship/sail-runtime/virtual",
        "@ship/sail-runtime/actions",
    ] {
        assert!(
            !out.js.contains(banned),
            "hello-world must not import {banned}:\n{}",
            out.js
        );
    }
    assert!(
        out.js.contains(ABI),
        "hello-world still needs the ABI barrel:\n{}",
        out.js
    );
}

/// `fx:` pulls motion, not the kitchen-sink ABI barrel for attach.
#[test]
fn fx_imports_motion_singleton() {
    let src = r#"<button style({ animate: { hover: play(fade) } })>hi</button>"#;
    let out = compile(src, opts()).expect("compile");
    assert!(
        out.js.contains("@ship/sail-runtime/motion"),
        "fx: must import the motion singleton:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("__sail_motion_attach")
            || out.js.contains("from \"@ship/sail-runtime/motion\""),
        "motion attach must come from /motion:\n{}",
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
    use sail_compiler::action_manifest::{ACTIONS, PUBLIC_ACTION_COUNT};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let index = root.join("packages/sail/runtime/src/index.ts");
    let Ok(src) = std::fs::read_to_string(&index) else {
        return;
    };

    assert_eq!(PUBLIC_ACTION_COUNT, 7);

    let leaked: Vec<&str> = ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|a| src.contains(&format!("\n  {a},")))
        .collect();

    assert!(
        leaked.is_empty(),
        "action values must not re-export from the language barrel \
         (they load `@ship/sail-runtime/actions`):\n  {}",
        leaked.join("\n  ")
    );
}

/// The action table is the single source of truth. A consumer
/// that disagrees with the table must fail this test.
///
/// - ALL actions must appear in `actions/index.ts` (full auto-import surface)
/// - NO action values on the language barrel (`index.ts`) — that file is
///   every page's runtime. Public and internal actions load via
///   `@ship/sail-runtime/actions`.
#[test]
fn action_table_has_no_phantom_consumers() {
    use sail_compiler::action_manifest::{ACTIONS, PUBLIC_ACTION_COUNT};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let runtime_actions_barrel = root.join("packages/sail/runtime/src/actions/index.ts");
    let runtime_top_barrel = root.join("packages/sail/runtime/src/index.ts");
    let Ok(actions_src) = std::fs::read_to_string(&runtime_actions_barrel) else {
        return;
    };
    let Ok(top_src) = std::fs::read_to_string(&runtime_top_barrel) else {
        return;
    };

    assert_eq!(PUBLIC_ACTION_COUNT, 7);

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

    let leaked_on_language_barrel: Vec<&str> = ACTIONS
        .iter()
        .map(|a| a.name)
        .filter(|n| top_src.contains(&format!("\n  {n},")))
        .collect();

    assert!(
        missing_actions_barrel.is_empty() && leaked_on_language_barrel.is_empty(),
        "action table disagrees with the runtime barrels — the table is the \
         single source; update the barrels, not the table. \
         Missing from actions/index.ts: {:?}. \
         Leaked onto language barrel: {:?}.",
        missing_actions_barrel,
        leaked_on_language_barrel,
    );
}

/// Public `use:` is the generic toolkit (Job E). Widget brains are not
/// auto-imported `use:` names.
#[test]
fn public_action_surface_is_curated() {
    use sail_compiler::action_manifest::{
        ACTIONS, PUBLIC_ACTION_COUNT, PUBLIC_ACTION_NAMES, is_public_action, public_actions,
    };

    assert_eq!(PUBLIC_ACTION_COUNT, 7);
    assert_eq!(PUBLIC_ACTION_NAMES.len(), 7);
    assert_eq!(public_actions().count(), 7);

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
    for widget in [
        "tabs",
        "icon",
        "combobox",
        "themeSelect",
        "syntaxHighlight",
        "contextMenu",
        "anchor",
        "tipDescribe",
        "dropzone",
    ] {
        assert!(
            !is_public_action(widget),
            "{widget} is widget-tied — not the public toolkit"
        );
    }

    // Toolkit `use:` auto-imports from the actions subpath — never the
    // author front door.
    let src = r#"<div use:clickOutside use:focus use:hotkeys></div>"#;
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
    for name in ["clickOutside", "focus", "hotkeys"] {
        assert!(
            out.contains(name),
            "expected auto-import of toolkit action `{name}`:\n{out}"
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
        let src = format!(r#"<div use:{a}></div>"#);
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
