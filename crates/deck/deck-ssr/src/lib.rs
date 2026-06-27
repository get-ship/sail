//! Server-render a SYNTHESIZED SUI view through ship-js (deno_core/V8).
//!
//! The pipeline ([`build`] + [`run`]):
//!   1. [`build::synthesize_and_bundle`] runs the synthetic emitter
//!      (`emit_synthetic`, pure) for a `.sui` + app manifest, then drives
//!      Rolldown into ONE self-contained ES module — `@ship/sui-runtime`
//!      and `@ship/deck/router` aliased to their **server** entries, app
//!      `<client>` modules to their server entries, and only `import "ship"`
//!      (the host wire) left external.
//!   2. [`run::render`] runs that module in the deno_core/V8 isolate via
//!      `ship_js::deno::run_module`, with a [`host::SsrDispatch`] answering
//!      `route.current` (the matched request) and `<data>`/client ops over
//!      the real op surface. A minimal server-DOM lets the SAME synthesized
//!      render run unchanged and serialize to a first-paint HTML string.
//!
//! ship-js is THE SSR path: the Rust render-eval engine was removed in the
//! JS-only SSR cutover, so there is no second rendering path.

pub mod build;
pub mod host;
pub mod pool;
pub mod run;
pub mod serve;

/// The SDK served for `import "ship"` — the deny-by-default loader's one
/// importable module. The thinnest possible door: `call` is the host-
/// dispatch wire (`op_ship_call` → the host's `Dispatch`), `emit` surfaces
/// a value out of the headless run. The host decides what `call` is allowed
/// to reach; this is just the wire.
///
/// `call` also maintains an in-flight counter exposed as
/// `globalThis.__shipPending()`, so the SSR entry can flush async `live()`
/// sources DETERMINISTICALLY — drain while host work is pending instead of
/// spinning a fixed number of microtask ticks (see `ssr/ssr-entry.ts`).
pub const SHIP_SDK: &str = "\
let __pending = 0;
globalThis.__shipPending = () => __pending;
export const call = (op, args) => {
  __pending++;
  return Promise.resolve(Deno.core.ops.op_ship_call(op, args ?? {}))
    .finally(() => { __pending--; });
};
export const emit = (v) => Deno.core.ops.op_emit(v);";

/// What one SSR run produces.
#[derive(Debug, Clone)]
pub struct RenderOut {
    /// First-paint HTML produced by running the synthesized TS in ship-js.
    pub html: String,
    /// The route params resolved through host-dispatch (proves no browser).
    pub route_params: serde_json::Value,
    /// The recorded host-dispatch call path (op names, in order).
    pub call_path: Vec<String>,
    /// Bytes of the bundled, self-contained ES module fed to the isolate.
    pub bundle_bytes: usize,
}
