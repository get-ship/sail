//! JS module emitter.
//!
//! Layout of the emitted module:
//!
//! ```js
//! import { view, … } from "@ship/sui-runtime";
//! import setup from "./view.ts";        // when paired
//!
//! export default view((ctx) => {
//!   const $scope = Object.create(null);
//!   $scope.ship = ctx.ship; …
//!   //  state signals come from the controller's `scope:` factory:
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
//! `__b<N>_attach`) plus a `__b<N>_attach` named property on the view's
//! setup return. The compiled module exports no manifest entries for
//! `if` / `for` / `match` / `await` kinds — `attach()` finds these
//! boundaries by walking SSR'd `<sui-{kind} key="…">` elements and
//! dispatching to the matching attach handler on the setup return.
//!
//! Internal structure:
//!  * `imports` — runtime/dom/setup import block
//!  * `view` — `export default view(…)` wrapper
//!  * `component` — `export default component(…)` wrapper + props schema
//!  * `setup` — setup body, $scope seeding, lifecycle wiring

use super::writer::Writer;
use crate::ir::*;
use crate::keys::BindingRegistry;
use crate::lower::Lowered;
use crate::CompileOpts;

mod component;
mod imports;
mod setup;
mod view;

pub fn emit_module(
    program: &Program,
    lowered: &Lowered,
    bindings: &BindingRegistry,
    opts: &CompileOpts,
) -> String {
    let mut w = Writer::new();
    // Leading `_compilerVersion` legal comment. Lets the shell compare
    // its loaded view bundle against the daemon's `/_deck/meta` and
    // surface drift loudly instead of silently rendering against a
    // stale export surface (cf. the `fade` regression that prompted
    // this).
    //
    // We also attach `_compilerVersion` to the default export inside
    // `view.rs` / `component.rs` — that property travels through
    // tree-shaking bundlers like Bun.build, which would otherwise drop
    // unused top-level exports and module-level comments.
    let cv = crate::compiler_version();
    let stamp_json = format!(
        "{{\"cargo\":\"{}\",\"git\":\"{}\",\"dirty\":{},\"builtAt\":\"{}\"}}",
        cv.cargo, cv.git, cv.dirty, cv.built_at,
    );
    w.line(&format!("/*! _compilerVersion {stamp_json} */"));
    imports::emit_imports(&mut w, lowered, opts);

    let mut body = Writer::new();
    body.enter();
    setup::emit_setup_body(&mut body, program, bindings, opts, lowered);
    body.leave();

    // The view/component split is keyed on the tag (`component_meta`), not a
    // mode flag: a tagged source registers a custom element; a tagless one
    // emits the mount-form ("view"/page). `CompileMode::View` is gone.
    if opts.component_meta.is_some() {
        component::emit_component_export(&mut w, program, body, opts);
    } else {
        view::emit_view_export(&mut w, program, body);
    }
    // Phase 3b.5 (gate I): the `bindings` manifest is gone — every
    // binding kind is now wired by the compiled view's
    // `hydrate(root, ctx)` directly, so the runtime walker has nothing
    // left to dispatch through. Components never used the manifest;
    // views no longer need it either. Saves ~9 KB ungzipped across the
    // ported primitives + per-view overhead.
    w.into_string()
}
