//! JS module emitter.
//!
//! Layout of the emitted module:
//!
//! ```js
//! import { component, … } from "@ship/sail-runtime";
//! import setup from "./controller.ts";  // when paired
//!
//! export default component((ctx) => {
//!   const $scope = Object.create(null);
//!   $scope.ship = ctx.ship; …
//!   //  state signals come from the controller's `setup:` factory:
//!   //  Object.assign($scope, setup.setup(ctx)) seeds them here.
//!   …
//!   const __wireBoundary_b3 = (anchor) => bindIf({ … });
//!   …
//!   return Object.assign({}, $scope, {
//!     __mount(container) {
//!       …
//!       cleanups.push(bindText(t, () => $scope.count()));
//!       …
//!     },
//!     __b3_attach: (target) => { … },
//!   });
//! });
//! ```
//!
//! Template expressions are rewritten at compile time so free
//! identifiers are qualified with `$scope.` (see `expr_rewrite`) and
//! pasted directly into emitted closures. No `with($scope)`, no
//! shared expression bank, no `new Function` eval at module load.
//!
//! Phase 2 client-side control flow: each `{#if}` / `{#for}` / `{#match}`
//! emits a hoisted `__wireBoundary_b<N>` (used by both `__mount` and
//! `__b<N>_attach`) plus a `__b<N>_attach` named property on the route's
//! setup return. The compiled module exports no manifest entries for
//! `if` / `for` / `match` / `await` kinds — `attach()` finds these
//! boundaries by walking SSR'd `<sail-{kind} key="…">` elements and
//! dispatching to the matching attach handler on the setup return.
//!
//! Internal structure:
//!  * `imports` — runtime/dom/setup import block
//!  * `mount` — `export default component((ctx) => …)` mount-form wrapper
//!  * `component` — `export default component(tag, …)` wrapper + props schema
//!  * `setup` — setup body, $scope seeding, lifecycle wiring

use super::writer::Writer;
use crate::CompileOpts;
use crate::ir::*;
use crate::keys::BindingRegistry;
use crate::lower::Lowered;

mod component;
mod imports;
mod mount;
mod setup;

pub fn emit_module(
    program: &Program,
    lowered: &Lowered,
    bindings: &BindingRegistry,
    opts: &CompileOpts,
    sx_var_bindings: &[crate::check::binding::StyleBinding],
) -> String {
    let mut lowered = lowered.clone();
    if !sx_var_bindings.is_empty() {
        lowered.runtime_imports.insert("effect");
        lowered.runtime_imports.insert("writeCssVar");
        lowered.dom_imports.insert("registerCustomProperties");
    }
    let lowered = &lowered;
    let mut w = Writer::new();
    // Leading `_compilerVersion` legal comment. Lets the shell compare
    // its loaded component bundle against the daemon's `/_deck/meta` and
    // surface drift loudly instead of silently rendering against a
    // stale export surface (cf. the `fade` regression that prompted
    // this).
    //
    // We also attach `_compilerVersion` to the default export inside
    // `mount.rs` / `component.rs` — that property travels through
    // tree-shaking bundlers like Bun.build, which would otherwise drop
    // unused top-level exports and module-level comments.
    let cv = crate::compiler_version();
    let stamp_json = format!(
        "{{\"cargo\":\"{}\",\"git\":\"{}\",\"dirty\":{},\"builtAt\":\"{}\"}}",
        cv.cargo, cv.git, cv.dirty, cv.built_at,
    );
    w.line(&format!("/*! _compilerVersion {stamp_json} */"));

    // The BODY is emitted first, because only the emitter knows which subtrees it
    // chose to template — `Lowered` is computed by the lower pass and cannot. The
    // import list then reflects what the body actually used, rather than guessing.
    let mut body = Writer::new();
    body.enter();
    let module_hoists = setup::emit_setup_body(&mut body, program, bindings, opts, sx_var_bindings);
    body.leave();
    let hoisted = module_hoists.into_string();
    let uses_templates = hoisted.contains("__tpl(");
    let uses_delegation = hoisted.contains("delegateEvents(");

    imports::emit_imports(&mut w, lowered, opts, uses_templates, uses_delegation);

    // The `__tpl(...)` cloners go at MODULE scope, above the component: each static
    // subtree's markup is parsed once per program (lazily, on first render) — not
    // once per component instance, and certainly not once per row.
    if !hoisted.is_empty() {
        w.push_str(&hoisted);
    }

    // The mount/register split is keyed on the tag (`component_meta`), not a
    // mode flag: a tagged source registers a custom element (`component(tag,
    // def)`); a tagless one emits the mount-form (`component(setup)` — a deck
    // page). Both call `component(...)`; the tag decides which overload.
    if let Some(meta) = opts.component_meta.as_ref() {
        component::emit_registered_component(&mut w, program, body, meta);
    } else {
        mount::emit_mount_export(&mut w, program, body);
    }
    // Phase 3b.5 (gate I): the `bindings` manifest is gone — every
    // binding kind is now wired by the compiled route's
    // `hydrate(root, ctx)` directly, so the runtime walker has nothing
    // left to dispatch through. Components never used the manifest;
    // route modules no longer need it either. Saves ~9 KB ungzipped across the
    // ported primitives + per-route overhead.
    w.into_string()
}
