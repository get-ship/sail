//! The ABI boundary: the COMPILER imports from `@ship/sui-runtime/internal`,
//! HUMANS import from `@ship/sui-runtime`.
//!
//! The runtime's front door used to advertise ~80 symbols — `bindFor`, `bindIf`,
//! `attachRef`, `runAction`, `applyPatch`, `__sui_motion_attach` — sitting in the
//! same export as `signal` and `effect`. No human types those; the emitter writes
//! them. But an agent reading that index cannot tell the language from the
//! plumbing, and the language is about ten symbols, not eighty.
//!
//! Nothing enforced the split, so these pin it. Without them the boundary is a
//! convention, and a convention that no test asserts is a comment.

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

const ABI: &str = "@ship/sui-runtime/internal";
const AUTHOR: &str = "@ship/sui-runtime";

/// A component exercising the emit paths that pull ABI symbols: text binding,
/// an event handler (`on`), and an action (`runAction` + the auto-imported
/// builtin). All three are things the emitter writes, none are things a human does.
const SRC: &str = r#"<button on:click={() => go()} use:tooltip>{label()}</button>"#;

fn opts(shadow: bool) -> CompileOpts {
    CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta { name: "abi-probe".into(), shadow }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

/// Every runtime symbol the emitter writes resolves through the ABI subpath —
/// never the package root, which is the author's front door.
#[test]
fn emitter_imports_the_abi_subpath_not_the_root() {
    let out = compile(SRC, opts(false)).expect("compile");
    let js = &out.js;

    assert!(
        js.contains(&format!("from \"{ABI}\"")),
        "the emitter must import the ABI from `{ABI}`:\n{js}"
    );

    // The root must not appear as an import SOURCE in generated code. (Checking
    // the closing quote matters: `@ship/sui-runtime/internal` and
    // `@ship/sui-runtime/template-hooks` both start with the root's text.)
    assert!(
        !js.contains(&format!("from \"{AUTHOR}\"")),
        "generated code must never import the author front door — that is the \
         boundary this test exists to hold:\n{js}"
    );
}

/// The symbols in question really are ABI: emitter-written, never hand-typed.
#[test]
fn the_abi_import_carries_emitter_symbols() {
    let out = compile(SRC, opts(false)).expect("compile");
    let js = &out.js;
    for sym in ["component", "on", "runAction"] {
        assert!(js.contains(sym), "expected emitter symbol `{sym}`:\n{js}");
    }
}

/// The shadow path pulls `adoptSuiSheets` — a second emit branch, same boundary.
/// Shadow is SOURCE-authoritative (light DOM is the default), so the opt-in has to
/// be declared in the `<component>` block: passing `ComponentMeta { shadow: true }`
/// alone does nothing, which is itself the contract worth pinning.
#[test]
fn the_shadow_emit_path_holds_the_boundary_too() {
    const SHADOW_SRC: &str = r#"<component>{ shadow: true }</component>
<button on:click={() => go()} use:tooltip>{label()}</button>"#;
    let out = compile(SHADOW_SRC, opts(true)).expect("compile");
    let js = &out.js;
    assert!(js.contains("adoptSuiSheets"), "shadow emit pulls the sheet adopter:\n{js}");
    assert!(
        !js.contains(&format!("from \"{AUTHOR}\"")),
        "shadow emit leaked the author front door:\n{js}"
    );
}

/// `@ship/sui-runtime/template-hooks` is a legitimate subpath and must not be
/// mistaken for the root by a naive prefix rewrite. Guards the rewrite itself.
#[test]
fn sibling_subpaths_are_not_rewritten_into_the_abi_path() {
    let out = compile(SRC, opts(false)).expect("compile");
    assert!(
        !out.js.contains("@ship/sui-runtime/internal/template-hooks"),
        "the ABI rewrite corrupted a sibling subpath:\n{}",
        out.js
    );
}
