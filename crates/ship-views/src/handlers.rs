//! Thin axum shims for the ship-Views surface. Each extracts axum params,
//! calls the matching ship-views serving fn (or the deck-core view-setup fn),
//! and maps the resulting `DeckResponse` onto an `axum::response::Response`
//! via [`deck_axum::into_axum`]. No hosting logic lives here — it sits in
//! `view_host` / `view_payload` (and deck-core's `serve_sui_view_setup`).
//!
//! These moved out of `deck-axum::handlers` so the named-view feature is no
//! longer wired into deck's own router (the deck ⊥ ship-views split). The
//! `into_axum` / `mode_from_ext` helpers are still deck-axum's — reused, not
//! re-derived.

use axum::{
    Extension,
    extract::{OriginalUri, Path, Query},
    response::Response,
};

use deck_axum::{into_axum, mode_from_ext};
use deck_core::{DaemonMode, ViewSetupQuery};

// ── View-setup handler (compiled `view.js`) ────────────────────────────────────

pub async fn serve_sui_view_setup(
    mode: Option<Extension<DaemonMode>>,
    bundler: Option<Extension<deck_bundler::Bundler>>,
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    let mode = mode_from_ext(mode);
    // Deck owns bundling (#167). If the view bundle isn't on disk yet, build it
    // in-process via Rolldown — the controller is inlined natively, no bun. The
    // daemon's watcher keeps `.deck` warm afterward; this covers the cold miss.
    let no_sub = q.sub.as_deref().filter(|s| !s.is_empty()).is_none();
    if no_sub
        && deck_core::resolve_view_js(&name).is_none()
        && let Some(Extension(b)) = bundler.as_ref()
    {
        // P0-2: when the in-process build FAILS (e.g. a broken paired
        // `controller.ts`), surface the REAL compile error instead of swallowing
        // it with `let _ =` and falling through to deck-core's generic "the
        // in-process bundler must build views/{name}/view" fallback, which masks
        // the actual syntax/type error. Only a `Bundle` failure (the view exists
        // but did not compile) short-circuits; `UnknownTarget`/`NoCompiler` fall
        // through so the resolve path yields the right 404 / diagnostic.
        if let Err(e) = b.build(&format!("views/{name}/view")).await
            && let deck_bundler::BundlerError::Bundle { .. } = &e
        {
            return into_axum(deck_core::DeckResponse::render_error(
                format!("view `{name}` failed to compile:\n{e}"),
                mode,
            ));
        }
    }
    into_axum(deck_core::serve_sui_view_setup(mode, &name, q.sub.as_deref()))
}

// ── View-host handler ─────────────────────────────────────────────────────────

pub async fn serve_view_host(
    mode: Option<Extension<DaemonMode>>,
    OriginalUri(uri): OriginalUri,
) -> Response {
    into_axum(crate::view_host::serve_view_host(mode_from_ext(mode), uri.path()))
}

// ── View-payload handlers ─────────────────────────────────────────────────────

pub async fn serve_view_payload(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
) -> Response {
    into_axum(crate::view_payload::serve_view_payload(mode_from_ext(mode), &name))
}

/// `GET /_deck/views/{name}/{*sub_path}` — the JSON payload for a
/// nested/parameterized view route. The wildcard path segment carries
/// one or more URL segments (e.g. `greet/zelda` or `studio/agents/42`);
/// the runtime folds the sub into the resolved `name` (and the
/// `__subscribe__` URL) and walks the resolver to capture `[param]`
/// segments. Mirrors the path-split pattern `serve_view_host` uses,
/// because axum's `Path<(String, String)>` only captures a fixed
/// 2-tuple and the deeper the nesting the more brittle that becomes.
pub async fn serve_view_payload_subpath(
    mode: Option<Extension<DaemonMode>>,
    OriginalUri(uri): OriginalUri,
) -> Response {
    // The sub-path payload surface is at `/_deck/views/{name}/{*sub_path}`.
    // Segment layout: ["_deck", "views", name, ...sub]. Name is at index 2;
    // the sub path is everything after the name, joined back with `/`.
    let mut segs = uri.path().trim_start_matches('/').split('/');
    let name = segs
        .nth(2)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let sub_path: String = segs.collect::<Vec<_>>().join("/");
    match name {
        Some(name) => into_axum(crate::view_payload::serve_view_payload_subpath(
            mode_from_ext(mode),
            &name,
            &sub_path,
        )),
        // Fallback: degenerate path that didn't match the 3-segment
        // `/_deck/views/<name>/<sub…>` shape — delegate to the top-level
        // payload handler, which produces the canonical 404 / name-error JSON.
        None => {
            let name = uri
                .path()
                .trim_start_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("");
            into_axum(crate::view_payload::serve_view_payload(mode_from_ext(mode), name))
        }
    }
}
