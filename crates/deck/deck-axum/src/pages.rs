//! deck-owned root + shared-asset handlers: design tokens, theme CSS/list,
//! the root `/` handler, the single-segment public-asset dispatch, and
//! the SPA fallback. All thin axum shims over `deck-core`.
//!
//! The root + public-asset / SPA-fallback dispatch read the host's
//! per-app serve config from `Extension<Arc<Vec<AppServeConfig>>>` — the
//! "primary" app is the one at route `/` in the host's list.
use std::sync::Arc;

use axum::{
    Router,
    extract::{Extension, Path},
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
};
use deck_config::AppServeConfig;
use deck_core::DaemonMode;

use crate::{into_axum, mode_from_ext};

// ── Shared CSS / theme assets ─────────────────────────────────────────────────

/// `GET /api/assets/view-tokens.css`
pub async fn serve_view_tokens(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_view_tokens(mode_from_ext(mode)))
}

/// `GET /api/assets/sui.css` — the framework stylesheet (contracts + stdlib
/// element skins), so a page is styled without the runtime's JS injection.
pub async fn serve_sui_css(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_css(mode_from_ext(mode)))
}

/// `GET /api/assets/theme.css`
pub async fn serve_theme_css(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_theme_css(mode_from_ext(mode)))
}

/// `GET /api/v1/themes`
pub async fn serve_themes_list(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_themes_list(mode_from_ext(mode)))
}

// ── Root handler ────────────────────────────────────────────────────────────

/// The "primary" app — the one at route `/`. `serve_index` and the SPA
/// fallback serve its `index.html` for any non-backend GET. Returns `None`
/// when no app is configured (the host didn't inject one — a 404 is the
/// right answer).
fn primary_app(apps: &Option<Extension<Arc<Vec<AppServeConfig>>>>) -> Option<Arc<AppServeConfig>> {
    let apps = apps.as_ref()?;
    apps.0.iter().find(|a| a.route == "/").map(|a| Arc::new(a.clone()))
}

/// `GET /` — serve the primary app's `index.html`. Deck owns no first-run
/// convention: a fresh-install redirect (e.g. to a host `/setup` wizard) is
/// the HOST's concern. `/setup` itself has no server route here — like any
/// bare client-route segment it falls through to the SPA fallback, which
/// serves the app shell so the app's own client-side routing renders it.
pub async fn serve_index(
    mode: Option<Extension<DaemonMode>>,
    apps: Option<Extension<Arc<Vec<AppServeConfig>>>>,
    resolution: Option<Extension<Arc<deck_core::Resolution>>>,
    renderer: Option<Extension<Arc<dyn deck_core::synthetic_ssr::SyntheticSsr>>>,
) -> Response {
    match primary_app(&apps) {
        Some(app) => into_axum(deck_core::serve_scaffold(
            mode_from_ext(mode),
            app.as_ref(),
            &[],
            Some("/"),
            resolution.as_ref().map(|e| e.0.as_ref()),
            renderer.as_ref().map(|e| e.0.as_ref()),
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
    resolution: Option<Extension<Arc<deck_core::Resolution>>>,
    renderer: Option<Extension<Arc<dyn deck_core::synthetic_ssr::SyntheticSsr>>>,
    Path(file): Path<String>,
) -> Response {
    let Some(app) = primary_app(&apps) else {
        return into_axum(deck_core::DeckResponse::not_found(
            "no primary app configured",
            mode_from_ext(mode),
        ));
    };
    if file.contains('.') {
        // Per-app public asset. deck-core reads `<project>/<app.source>/<file>`,
        // resolving `<project>` from the mount-scoped Resolution.
        return into_axum(deck_core::serve_public_asset(
            mode_from_ext(mode),
            &app.source,
            &file,
            resolution.as_ref().map(|e| e.0.as_ref()),
        ));
    }
    // Bare segment — SPA navigation, serve the app's `index.html`. Pass the
    // request path so an `ssr` app first-paints this exact route server-side.
    let req_path = format!("/{file}");
    into_axum(deck_core::serve_scaffold(
        mode_from_ext(mode),
        app.as_ref(),
        &[],
        Some(&req_path),
        resolution.as_ref().map(|e| e.0.as_ref()),
        renderer.as_ref().map(|e| e.0.as_ref()),
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
    host_backend_prefixes: Option<Extension<Arc<Vec<String>>>>,
    resolution: Option<Extension<Arc<deck_core::Resolution>>>,
    renderer: Option<Extension<Arc<dyn deck_core::synthetic_ssr::SyntheticSsr>>>,
) -> Response {
    let path = uri.path();
    // deck owns ONLY its own framework surface: `/api/*` (token + theme CSS),
    // `/_deck/*` (the framework routes), and any path carrying an extension (a
    // static asset, or a Connect-RPC `*.*` path). Every OTHER backend prefix is
    // the HOST's — `/auth`, `/daemon`, `/space`, `/shell` are ship-PRODUCT
    // routes, not deck's, so deck must not name them (inv-deck-standalone). The
    // host declares its backend prefixes via `host_backend_prefixes`; absent ⇒
    // none (the standalone default — deck dev has no host routes). A host's
    // real routes are matched before the fallback fires, so this list only
    // guards UNMATCHED paths under a host prefix from being served the SPA shell.
    let deck_owns =
        path.starts_with("/api") || path.starts_with("/_deck") || path.contains('.');
    let host_owns = host_backend_prefixes
        .as_ref()
        .is_some_and(|hp| hp.0.iter().any(|p| path.starts_with(p.as_str())));
    if method == Method::GET && !deck_owns && !host_owns {
        if let Some(app) = primary_app(&apps) {
            return into_axum(deck_core::serve_scaffold(
                mode_from_ext(mode),
                app.as_ref(),
                &[],
                Some(path),
                resolution.as_ref().map(|e| e.0.as_ref()),
                renderer.as_ref().map(|e| e.0.as_ref()),
            ));
        }
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}
/// Router helper for the host to mount the root `/` handler. The L1
/// framework exports the handler, the host chooses how to mount it.
/// shipd merges this into its root. Deck owns no first-run / `/setup`
/// convention: `/setup` is an app client-route that falls through to the
/// SPA fallback (which serves the app shell), so the app's own client-side
/// routing renders it. A first-run redirect is a host concern.
pub fn serve_index_route() -> Router {
    Router::new().route("/", get(serve_index))
}
