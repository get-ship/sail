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
//! has been registered AND it renders without error. When no renderer is
//! registered (e.g. a V8-less context) or a render fails, there is no server
//! HTML and the caller serves an empty `#sui-root` for the client to mount
//! (CSR) — there is no second rendering path.

use std::path::Path;
use std::sync::OnceLock;

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

static RENDERER: OnceLock<Box<dyn SyntheticSsr>> = OnceLock::new();

/// Register the ship-js SSR renderer ONCE at host startup. A second
/// registration is ignored (the first wins) — registration is a process-wide
/// install, not a per-request handle.
pub fn register(renderer: impl SyntheticSsr) {
    let _ = RENDERER.set(Box::new(renderer));
}

fn renderer() -> Option<&'static dyn SyntheticSsr> {
    RENDERER.get().map(|b| b.as_ref())
}

/// First-paint HTML through ship-js when a renderer is registered AND it
/// succeeds; otherwise `None`, and the caller serves an empty `#sui-root` for
/// the client to mount (CSR). A render error is logged loudly (it is a real
/// failure, not a quiet downgrade) — it does not fall back to a second engine.
pub fn first_paint(req: &SsrRequest) -> Option<String> {
    let r = renderer()?;
    match r.first_paint(req) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(
                view = req.view_name,
                "ship-js SSR failed; serving an empty #sui-root (client mounts via CSR): {e:#}"
            );
            None
        }
    }
}
