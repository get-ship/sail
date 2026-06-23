//! Gated, additive injection seam for the ship-js (synthetic-view) SSR path.
//!
//! deck-core declares the **result** it wants — "first-paint HTML for this
//! resolved view + request" — and owns the **gate**. It does not own the
//! mechanism: the ship-js renderer lives in `deck-ssr` (which depends on
//! deck-core and pulls V8 + Rolldown), so deck-core can't depend on it without
//! a cycle. Instead deck-ssr **registers** an implementation here at host
//! startup, and the serve path calls through this seam.
//!
//! ship-js is now the **default** first-paint engine. It is selected whenever a
//! renderer has been registered AND it renders without error — either failing
//! falls straight back to the **archived** Rust render-eval (see
//! [`crate::render_eval_archived`]). So a daemon built without deck-ssr, or one
//! that opts back out via `DECK_SSR=render-eval`, keeps the byte-for-byte
//! render-eval output. The path stays additive: the fallback is always
//! render-eval; ship-js only swaps the first-paint source when it is available.

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

/// The gate. Default ON (ship-js). `DECK_SSR=render-eval` opts BACK to the
/// archived render-eval engine; every other value (including unset) selects
/// ship-js. The escape hatch's single source of truth is
/// [`crate::render_eval_archived::requested`] — this is its inverse.
pub fn enabled() -> bool {
    !crate::render_eval_archived::requested()
}

/// First-paint HTML through ship-js when the gate is on AND a renderer is
/// registered AND it succeeds; otherwise `None`, and the caller renders via
/// the Rust render-eval path exactly as before.
pub fn first_paint(req: &SsrRequest) -> Option<String> {
    if !enabled() {
        return None;
    }
    let r = renderer()?;
    match r.first_paint(req) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::warn!(
                view = req.view_name,
                "ship-js SSR failed; falling back to render-eval: {e:#}"
            );
            None
        }
    }
}
