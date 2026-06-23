//! ARCHIVED — superseded by ship-js (the default SSR engine since this commit).
//!
//! The Rust **render-eval** path renders a view's first paint by walking the
//! parsed `.sui` and evaluating its template expressions in pure Rust
//! (`sui_compiler::render`, whose expression evaluator is `render::eval`).
//! ship-js replaces it as the default first-paint source: it is strictly more
//! capable — it runs the view's controller and resolves host data at first
//! paint, which render-eval cannot (see `deck-ssr/tests/serve_wired.rs`).
//!
//! Reachable ONLY via `DECK_SSR=render-eval` (the surviving escape hatch) or as
//! the automatic fallback when ship-js is unregistered or errors. **Do not
//! extend.**
//!
//! Why the engine proper is not relocated *into* this file: `sui_compiler::
//! render` is shared infrastructure — the SAME `render_split` also produces the
//! always-needed CSR hydration init (`split.init` in `view_payload`) and is
//! consumed by `bindings`. Physically moving it would strand the client
//! hydration path and the compiler (both out of scope, and the compiler is pure
//! transformation). This module is therefore the single, marked home for the
//! deck-side render-eval **first-paint selection**: the gate that requests it
//! ([`requested`]) and the live-emit first paint that is exclusively
//! render-eval's ([`live_first_paint`]). The payload endpoint's render-eval
//! first paint is the `split.html` field of that shared render — documented at
//! its one use site in `view_payload::serve`.

use sui_compiler::{bindings, parse, render};

use crate::resolver::resolve;

/// True when the operator has opted BACK to the archived render-eval engine via
/// `DECK_SSR=render-eval`. The single source of truth for the escape hatch;
/// [`crate::synthetic_ssr::enabled`] is its inverse (ship-js runs unless this is
/// set). Every other value of `DECK_SSR`, and an unset var, leave ship-js on.
pub fn requested() -> bool {
    std::env::var("DECK_SSR").map(|v| v == "render-eval").unwrap_or(false)
}

/// Render-eval first paint at request time, from the source view.
///
/// ARCHIVED — the live-emit half of the render-eval path, relocated here intact
/// from `view_host`. Used as the fallback when no precompiled `<stem>.ssr.html`
/// mirror exists yet and ship-js did not render. Uses the full `render()` so the
/// body carries the `<script id="__set_init__">` seed island the client runtime
/// needs to hydrate initial signal state, and the `__subscribe__` URL that
/// `<data live>` views open their SSE channel with — keeping dev live-emit
/// byte-identical to the prod from-disk artifact.
pub(crate) fn live_first_paint(name: &str) -> Option<String> {
    let resolved = resolve(name, None).ok()?;
    let program = parse::parse(&resolved.source).ok()?;
    let registry = bindings::build(&program);
    let opts = render::RenderOpts { view_name: name.to_string() };
    let output = render::render(&program, &registry, &opts).ok()?;
    Some(output.html)
}
