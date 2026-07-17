//! Gated, additive injection seam for the ship-js (synthetic-view) SSR path.
//!
//! deck-core declares the **result** it wants — "first-paint HTML for this
//! resolved view + request" — and owns the **gate**. It does not own the
//! mechanism: the ship-js renderer lives in `deck-ssr` (which depends on
//! deck-core and pulls V8 + Rolldown), so deck-core can't depend on it without
//! a cycle. Instead deck-ssr **registers** an implementation here at host
//! startup, and the serve path calls through this seam.
//!
//! ship-js is the one first-paint engine. It produces HTML whenever a renderer
//! is CARRIED for the request AND it renders without error. When no renderer is
//! carried (e.g. a V8-less context, or a CSR-only mount) or a render fails,
//! there is no server HTML and the caller serves an empty `#sail-root` for the
//! client to mount (CSR) — there is no second rendering path.
//!
//! ## Mount-scoped, not a process global
//!
//! The renderer is carried per-mount — an `Arc<dyn SyntheticSsr>` the host
//! layers as an axum `Extension` and the serve path threads into
//! [`first_paint`] — NOT a process-global `OnceLock` install. So one process
//! can host two projects with different (or no) SSR engines without cross-talk
//! (`inv-deck-mount-scoped-resolution` names `RENDERER` among the globals it
//! removes).
//!
//! FLAG (render-ownership, deferred): this de-globalizes the RENDERER handle but
//! does NOT decide who OWNS SSR rendering long-term. If first-paint moves into
//! SUI (the "SSR render moves to SUI" arc), this seam is where the ship-js
//! engine gets swapped for the SUI-owned one — the per-mount handle makes that a
//! host-wiring change, not another global rewrite. Not resolved here.

use std::path::Path;

use serde_json::Value;

/// The resolved view + request a first-paint render needs. deck-core fills
/// this from what the serve path already resolved — no new IO, no new config.
pub struct SsrRequest<'a> {
    /// The view name as routed (e.g. `greet`, or `studio/agents` folded).
    pub view_name: &'a str,
    /// Nested sub-path, when the view was served at `/_deck/view/<name>/<sub>`.
    pub sub_path: Option<&'a str>,
    /// Absolute path to the source `.sui` (the synth input). The renderer
    /// discovers the app manifest + sibling controller relative to this.
    pub source_path: &'a Path,
    /// The request pathname the deck router matched.
    pub pathname: &'a str,
    /// Route params the router captured from `[param]` segments.
    pub params: Value,
    /// Query params for this request (empty object on the payload endpoint).
    pub query: Value,
}

/// The capability deck-core declares: produce first-paint HTML for a resolved
/// view through ship-js. The implementor (deck-ssr) owns the whole procedure —
/// locate the app manifest, synthesize, bundle, run in a pooled isolate — and
/// returns only the result.
pub trait SyntheticSsr: Send + Sync + 'static {
    /// First-paint HTML for `req`, or an error the caller logs and falls back on.
    fn first_paint(&self, req: &SsrRequest) -> anyhow::Result<String>;
}

/// First-paint HTML through the CARRIED `renderer` when it succeeds; otherwise
/// `None`, and the caller serves an empty `#sail-root` for the client to mount
/// (CSR). A render error is logged loudly (it is a real failure, not a quiet
/// downgrade) — it does not fall back to a second engine. The renderer is a
/// per-mount handle the serve path threads (see the module doc), never a
/// process-global install.
pub fn first_paint(renderer: &dyn SyntheticSsr, req: &SsrRequest) -> Option<String> {
    match renderer.first_paint(req) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(
                view = req.view_name,
                "ship-js SSR failed; serving an empty #sail-root (client mounts via CSR): {e:#}"
            );
            None
        }
    }
}
