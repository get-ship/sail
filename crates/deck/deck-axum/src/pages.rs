//! deck-owned root + shared-asset handlers: design tokens, theme CSS/list,
//! first-run `/` + `/setup`, the single-segment public-asset dispatch, and
//! the SPA fallback. All thin axum shims over `deck-core`.
//!
//! The first-run flow + public-asset / SPA-fallback dispatch read the host's
//! per-app serve config from `Extension<Arc<Vec<AppServeConfig>>>` (the
//! the "primary" app is the one at route `/` in the host's list.
use std::sync::Arc;

use axum::{
    Router,
    extract::{Extension, Path},
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
};
use deck_config::AppServeConfig;
use deck_core::{DaemonMode, DataPort};

use crate::{into_axum, mode_from_ext};

// ── Shared CSS / theme assets ─────────────────────────────────────────────────

/// `GET /api/assets/view-tokens.css`
pub async fn serve_view_tokens(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_view_tokens(mode_from_ext(mode)))
}

/// `GET /api/assets/theme.css`
pub async fn serve_theme_css(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_theme_css(mode_from_ext(mode)))
}

/// `GET /api/v1/themes`
pub async fn serve_themes_list(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_themes_list(mode_from_ext(mode)))
}

// ── First-run flow ────────────────────────────────────────────────────────────

/// Read the host's provisioning state through the `DataPort`. When no port
/// is wired the deck default is "not provisioned" (the fresh-install posture).
fn is_provisioned(port: &Option<Extension<Arc<dyn DataPort>>>) -> bool {
    port.as_ref().map(|e| e.0.is_provisioned()).unwrap_or(false)
}

/// The "primary" app — the one at route `/`. The first-run flow serves its
/// `index.html` once provisioning completes; the SPA fallback serves it
/// for any non-backend GET. Returns `None` when no app is configured (the
/// host didn't inject one — a 404 is the right answer).
fn primary_app(apps: &Option<Extension<Arc<Vec<AppServeConfig>>>>) -> Option<Arc<AppServeConfig>> {
    let apps = apps.as_ref()?;
    apps.0.iter().find(|a| a.route == "/").map(|a| Arc::new(a.clone()))
}

/// `GET /` — serve the primary app's `index.html` once provisioned; on a
/// fresh install redirect to `/setup`. `/setup` itself has no server route —
/// it falls through to the SPA fallback, which serves the app shell so the
/// app's own client-side `/setup` wizard renders. The first-run redirect here
/// is the only server-side first-run signal deck keeps; the wizard lives in
/// the app.
pub async fn serve_index(
    mode: Option<Extension<DaemonMode>>,
    apps: Option<Extension<Arc<Vec<AppServeConfig>>>>,
    port: Option<Extension<Arc<dyn DataPort>>>,
) -> Response {
    if !is_provisioned(&port) {
        return into_axum(deck_core::redirect("/setup", mode_from_ext(mode)));
    }
    match primary_app(&apps) {
        Some(app) => into_axum(deck_core::serve_scaffold(
            mode_from_ext(mode),
            app.as_ref(),
            &[],
        )),
        None => into_axum(deck_core::DeckResponse::not_found(
            "no primary app configured for /",
            mode_from_ext(mode),
        )),
    }
}

// ── SPA / public-asset dispatch ───────────────────────────────────────────────

/// `GET /{file}` — a single-segment path. A dotted name (`favicon.ico`,
/// `ship-logo.svg`) is a real public asset, served from the primary app's
/// source. A bare segment (`/git`, `/agents`, `/settings`) is a client-
/// side route, so serve the primary app's `index.html` and let client-
/// side routing resolve it.
pub async fn spa_or_public_asset(
    mode: Option<Extension<DaemonMode>>,
    apps: Option<Extension<Arc<Vec<AppServeConfig>>>>,
    Path(file): Path<String>,
) -> Response {
    let Some(app) = primary_app(&apps) else {
        return into_axum(deck_core::DeckResponse::not_found(
            "no primary app configured",
            mode_from_ext(mode),
        ));
    };
    if file.contains('.') {
        // Per-app public asset. deck-core reads `<project>/<app.source>/<file>`.
        return into_axum(deck_core::serve_public_asset(
            mode_from_ext(mode),
            &app.source,
            &file,
        ));
    }
    // Bare segment — SPA navigation, serve the app's `index.html`.
    into_axum(deck_core::serve_scaffold(
        mode_from_ext(mode),
        app.as_ref(),
        &[],
    ))
}

/// Router-level fallback for unmatched requests. A GET navigation to a
/// multi-segment client route (`/studio/themes`, …) has no server route,
/// so serve the primary app's `index.html` and let client-side routing
/// take over. Every other unmatched request keeps an honest 404 — the
/// fallback never shadows a real backend path or static asset.
pub async fn spa_fallback(
    method: Method,
    uri: Uri,
    mode: Option<Extension<DaemonMode>>,
    apps: Option<Extension<Arc<Vec<AppServeConfig>>>>,
) -> Response {
    let path = uri.path();
    let is_backend_or_asset = path.starts_with("/api")
        || path.starts_with("/_deck")
        || path.starts_with("/hooks")
        || path.starts_with("/auth")
        || path.starts_with("/daemon")
        || path.starts_with("/space")
        || path.starts_with("/shell")
        // Connect-RPC paths are `/ship.ops.v1.*` and assets carry an
        // extension — both contain a `.`, so this single check covers
        // them.
        || path.contains('.');
    if method == Method::GET && !is_backend_or_asset {
        if let Some(app) = primary_app(&apps) {
            return into_axum(deck_core::serve_scaffold(
                mode_from_ext(mode),
                app.as_ref(),
                &[],
            ));
        }
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}
/// Router helper for the host to mount the first-run flow. The L1
/// framework exports the handler, the host chooses how to mount it.
/// shipd merges this into its root. There is no `/setup` route helper:
/// `/setup` is an app client-route that falls through to the SPA fallback
/// (which serves the app shell), so the app's own `/setup` wizard renders.
pub fn serve_index_route() -> Router {
    Router::new().route("/", get(serve_index))
}
