//! Per-app serving — the host-driven replacement for the old "scaffold by name"
//! model. The previous API read a named scaffold from
//! `Extension<Arc<DeckConfig>>::scaffolds[]` keyed on a hardcoded name
//! (`"shell"`, `"space"`). The new per-app API takes a list of
//! [`AppServeConfig`] (host-injected) and serves each at the route the app
//! declared. The framework is per-app; the host owns the list.
//!
//! ## Router shape
//!
//! For each app, [`serve_apps`] mounts three routes:
//!
//! - `GET <app.route>` → [`serve_app_index`]
//!   Serve the app's `index.html` with `inject_globals` substituted from
//!   the URL path captures. Path captures are extracted as a
//!   `HashMap<String, String>` so any `{name}` segment in the route is
//!   available for substitution.
//! - `GET /<app.name>/{file}` → [`serve_app_asset`]
//!   Serve a per-app static asset (`<app.source>/assets/{file}`).
//!   The app's `name` is the URL namespace — no separate prefix field.
//! - `GET /{file}` (the app's slice) → [`serve_app_public`]
//!   Serve a per-app public asset. The bundler copies `<project>/public/`
//!   to `<project>/<source>/` at build time, so single-segment root
//!   paths land here.
//!
//! All three handlers read their app config from
//! `Extension<Arc<AppServeConfig>>`. The host layers each app's routes
//! on top of the others via the axum router, so each handler sees only
//! the app that owns the route it serves.
use std::collections::HashMap;
use std::sync::Arc;

use axum::{Extension, Router, extract::Path, response::Response, routing::get};

use deck_config::AppServeConfig;
use deck_core::DaemonMode;

use crate::{into_axum, mode_from_ext};

// ── Handlers ───────────────────────────────────────────────────────────────────

/// `GET <app.route>` — serve the app's `index.html` with `inject_globals`
/// substituted from the URL path captures.
///
/// The axum `Path` extractor is `HashMap<String, String>` because axum
/// supports it natively: every `{name}` segment in the route is captured
/// into the map. The framework's `serve_scaffold` substitutes `$<key>`
/// tokens in `inject_globals` values from the map's entries.
pub async fn serve_app_index(
    mode: Option<Extension<DaemonMode>>,
    app: Option<Extension<Arc<AppServeConfig>>>,
    resolution: Option<Extension<Arc<deck_core::Resolution>>>,
    renderer: Option<Extension<Arc<dyn deck_core::synthetic_ssr::SyntheticSsr>>>,
    uri: axum::http::Uri,
    // Static routes like `/_studio` have no path params — Path rejects those,
    // so accept Result and treat rejection as empty captures.
    params: Result<Path<HashMap<String, String>>, axum::extract::rejection::PathRejection>,
) -> Response {
    let mode_val = mode_from_ext(mode);
    let Some(app) = app.as_ref().map(|e| e.0.as_ref()) else {
        return into_axum(deck_core::DeckResponse::not_found(
            "no app configured for this route",
            mode_val,
        ));
    };
    let empty = HashMap::new();
    let params_map = params.as_ref().map(|p| &p.0).unwrap_or(&empty);
    let params_refs: Vec<(&str, &str)> = params_map
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    // Thread the request path so `render: ssr` apps (e.g. Studio at `/_studio`)
    // first-paint. Overlay paths map into studio's own route table:
    // `/_studio` → `/`, `/_studio/edit` → `/edit`, etc.
    let path = uri.path();
    let ssr_owned: Option<String> = if path == "/_studio" {
        Some("/".to_string())
    } else {
        path.strip_prefix("/_studio/").map(|rest| {
            if rest.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", rest.trim_start_matches('/'))
            }
        })
    };
    let ssr_path = ssr_owned.as_deref().unwrap_or(path);
    into_axum(deck_core::serve_scaffold(
        mode_val,
        app,
        &params_refs,
        Some(ssr_path),
        resolution.as_ref().map(|e| e.0.as_ref()),
        renderer.as_ref().map(|e| e.0.as_ref()),
    ))
}

/// `GET /<app.name>/{file}` — serve a per-app static asset
/// (`<app.source>/assets/{file}`). The app's name is its URL namespace.
pub async fn serve_app_asset(
    mode: Option<Extension<DaemonMode>>,
    app: Option<Extension<Arc<AppServeConfig>>>,
    resolution: Option<Extension<Arc<deck_core::Resolution>>>,
    Path(file): Path<String>,
) -> Response {
    let mode_val = mode_from_ext(mode);
    let Some(app) = app.as_ref().map(|e| e.0.as_ref()) else {
        return into_axum(deck_core::DeckResponse::not_found(
            "no app configured for this asset path",
            mode_val,
        ));
    };
    into_axum(deck_core::serve_scaffold_asset(
        mode_val,
        app,
        &file,
        resolution.as_ref().map(|e| e.0.as_ref()),
    ))
}

/// `GET /{file}` (the app's slice) — serve a per-app public asset.
/// The bundler copies `<project>/public/` to `<project>/<source>/` at
/// build time, so a `favicon.ico` lives at
/// `<project>/<source>/favicon.ico`. Reads from the per-app source dir
/// (NOT `<source>/assets/`, which is the per-app bundle asset path —
/// public files are siblings, not children, of `assets/`).
pub async fn serve_app_public(
    mode: Option<Extension<DaemonMode>>,
    app: Option<Extension<Arc<AppServeConfig>>>,
    resolution: Option<Extension<Arc<deck_core::Resolution>>>,
    Path(file): Path<String>,
) -> Response {
    let mode_val = mode_from_ext(mode);
    let Some(app) = app.as_ref().map(|e| e.0.as_ref()) else {
        return into_axum(deck_core::DeckResponse::not_found(
            "no app configured for this public asset",
            mode_val,
        ));
    };
    into_axum(deck_core::serve_public_asset(
        mode_val,
        &app.source,
        &file,
        resolution.as_ref().map(|e| e.0.as_ref()),
    ))
}

// ── Router ─────────────────────────────────────────────────────────────────────
/// Build the per-app router for a list of apps. The host calls this once
/// at startup; the resulting router merges into the host's root alongside
/// the Connect fallback + the deck framework surface (`/_deck/*`).
///
/// ## Layering rules
///
/// - The PRIMARY app (the first one in the list, conventionally the
///   one at route `/`) gets:
///   - `GET /<app.name>/{file}` (per-app asset namespace)
///   - `GET /{file}` (per-app public-asset catch-all, primary only)
///   - The PRIMARY app's `GET /` (the index) is NOT mounted here —
///     `serve_index` owns `/`. Both layers converge: `serve_index`
///     serves the same primary app's `index.html`. The framework
///     deliberately does not mount `<primary.route>` for the primary
///     app to avoid the route conflict (and the bug where `/` would
///     404). Deck owns no first-run / `/setup` convention; a fresh-
///     install redirect is a host concern.
/// - Each NON-PRIMARY app (e.g. spaces at `/space/{id}`) gets:
///   - `GET <app.route>` (e.g. `/space/{id}`)
///   - `GET /<app.name>/{file}` (per-app asset namespace)
pub fn serve_apps(apps: &[AppServeConfig]) -> Router {
    let mut router = Router::new();
    for (i, app) in apps.iter().enumerate() {
        let is_primary = i == 0;
        // The primary app's `/` is served by `serve_index`, not here.
        // Skip route mounting for the primary app at `/`.
        if !(is_primary && app.route == "/") {
            let app_ext = Arc::new(app.clone());
            router = router.route(&app.route, get(serve_app_index).layer(Extension(app_ext)));
        }
        // The asset namespace is the app's name. The route is
        // `/<app.name>/{file}` and the handler reads the app config
        // from the layer extension. No `assetPrefix` config field —
        // the name IS the namespace.
        let app_ext = Arc::new(app.clone());
        router = router.route(
            &format!("/{}/{{file}}", app.name),
            get(serve_app_asset).layer(Extension(app_ext)),
        );
    }
    // Public-asset catch-all: the primary app's `serve_app_public`
    // handles single-segment root paths (`/favicon.ico`, `/logo.svg`).
    if let Some(primary) = apps.first() {
        let primary_ext = Arc::new(primary.clone());
        router = router.route(
            "/{file}",
            get(serve_app_public).layer(Extension(primary_ext)),
        );
    }
    router
}
