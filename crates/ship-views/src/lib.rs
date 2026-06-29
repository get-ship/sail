//! ship-views — the named-view serving feature, extracted out of deck.
//!
//! This is the SERVING CORE of the ship-Views feature: the `/_deck/view(s)/*`
//! surface that serves a named view's HTML shell (`view_host`), its JSON
//! payload (`view_payload`), and its compiled `view.js` (via deck-core's
//! `serve_sui_view_setup`). It used to live inside `deck-core` + `deck-axum`;
//! it now lives here so deck no longer carries the feature.
//!
//! Hard invariant: **deck ⊥ ship-views**. ship-views depends DOWN on deck
//! (deck-core / deck-host / deck-bundler / deck-axum); NO deck crate depends on
//! ship-views. The host (and deck-conformance, a test crate) merge
//! [`router`] alongside `deck::router()` to serve views.
//!
//! What STAYS in deck (shared by BOTH the routes/ SPA path and this view path):
//! the resolver (`deck_core::resolver` / `deck_host::resolver`), the SSR seam
//! (`deck_core::synthetic_ssr`), the bundler (`deck-bundler`), the view-name
//! validator (`deck_host::render_helpers::is_valid_view_name`), and the
//! `ViewPayload` wire type (`deck_host::view_payload::ViewPayload` — a shared
//! DTO, not serving logic).

pub mod view_host;
pub mod view_payload;
mod handlers;

// The axum shims, surfaced at the crate root so a host can name them directly
// (and the `view_build_error` integration test drives `serve_sui_view_setup`).
pub use handlers::{
    serve_sui_view_setup, serve_view_host, serve_view_payload, serve_view_payload_subpath,
};

use axum::{Router, routing::get};

/// The ship-Views routes as one mountable `Router`.
///
/// Registers the named-view surface that used to live in `deck-axum::router()`:
///   - `GET /_deck/views/{name}/view.js`        — compiled view module
///   - `GET /_deck/views/{name}/{*sub_path}`    — JSON payload (nested/param)
///   - `GET /_deck/views/{name}`                — JSON payload (top-level)
///   - `GET /_deck/view/{name}`                 — HTML shell (view host)
///   - `GET /_deck/view/{name}/{*sub_path}`     — HTML shell (nested)
///
/// Paths + state match exactly what `deck-axum` registered, so the host merges
/// this into its root the same way it merges `deck::router()`. The handlers
/// read the daemon mode + bundler from `Extension`s the host layers (mirroring
/// deck's own router), so there is no router state to thread.
pub fn router() -> Router {
    Router::new()
        .route("/_deck/views/{name}/view.js", get(serve_sui_view_setup))
        // `GET /_deck/views/{name}/{*sub_path}` — the JSON payload for a
        // nested / parameterized view route. The `view.js` route above
        // accepts `?sub=<path>` for the same payload client-side; the path
        // form here is the canonical API for the runtime + the conformance
        // param test.
        .route(
            "/_deck/views/{name}/{*sub_path}",
            get(serve_view_payload_subpath),
        )
        .route("/_deck/views/{name}", get(serve_view_payload))
        .route("/_deck/view/{name}", get(serve_view_host))
        .route("/_deck/view/{name}/{*sub_path}", get(serve_view_host))
}

#[cfg(test)]
mod view_payload_tests;
#[cfg(test)]
mod view_payload_shape_tests;
#[cfg(test)]
mod view_payload_injection_tests;

/// Test-only harness: pin deck-core's process-global resolver root per test.
///
/// The moved view tests resolve views off a tempdir. deck-core's resolver root
/// is a process-wide override (`set_project_root`), so concurrent tests would
/// race on it — a single `Mutex` serializes them. The guard holds the lock for
/// its whole lifetime and each test sets its OWN root up front, so a prior
/// test's (now-deleted) tempdir is never observed: the next test overwrites the
/// root before it resolves anything.
#[cfg(test)]
pub(crate) mod test_lock {
    use std::sync::{Mutex, MutexGuard};

    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

    pub struct ProjectRootGuard {
        _lock: MutexGuard<'static, ()>,
    }
    impl ProjectRootGuard {
        pub fn new(dir: &std::path::Path) -> Self {
            let lock = TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            deck_core::resolver::set_project_root(dir.to_path_buf());
            Self { _lock: lock }
        }
    }
}
