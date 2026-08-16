//! deck-conformance — mounting deck on axum, in one place.
//!
//! Two entry points share one definition of "how to mount deck":
//! - [`deck_app`] — a ready router, NO filesystem watcher (read-only host /
//!   tests). On-demand bundling still works; nothing watches for edits.
//! - [`deck_dev_server`] — the same, plus the bundler's FS watcher running, so
//!   editing a `.sail` rebuilds the affected target and pushes an HMR event.
//!
//! The host contract:
//! 1. Point the resolver at an app dir via `DECK_PROJECT_DIR` (leave
//!    `DECK_FIRST_PARTY_VIEWS` unset — non-ship host, no first-party bucket).
//!    The resolver also reads built bundles from `<app_dir>/dist/` (deploy)
//!    and the generate tree under `<app_dir>/.sail/`.
//! 2. Mount `deck::router()`. (The named-route surface — `/_deck/view(s)/*` —
//!    was extracted out of deck into the separate `ship-views` repo and is no
//!    longer served here; its conformance tests moved out with it.)
//! 3. Layer the extensions deck's handlers read — `DaemonMode` and a
//!    `Bundler` (so a controller route gets its Rolldown bundle on demand).
//! 4. Mount the data port (`data_port::router`) for the live-data endpoints
//!    backed by the in-memory [`DemoStore`]. Tokens, the HMR SSE route, and
//!    the mutation tombstone all come from `deck::router()` itself.
//!
//! Decoupling gaps surfaced (see README): env config is still process-global
//! (now `DECK_*`-named with a `SHIP_*` fallback — Fix C);
//! `Bundler::start` only needs a repo root + config; `router()` carries no
//! bundler and no HMR (both host-wired); the bundler out-dir must equal the
//! resolver's project root.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{Extension, Router};
use deck::bundler::Bundler;
use deck::{DaemonMode, router};

pub mod data_port;
pub use data_port::DemoStore;

/// Resolve the daemon mode from the `DECK_DEMO_MODE` environment variable.
/// `"prod"` or `"production"` → [`DaemonMode::Production`]; anything else
/// (including absent) → [`DaemonMode::Development`].
pub fn demo_mode() -> DaemonMode {
    match std::env::var("DECK_DEMO_MODE").as_deref() {
        Ok("prod") | Ok("production") => DaemonMode::Production,
        _ => DaemonMode::Development,
    }
}

/// This crate's example app: `app/routes/<name>/page.sail`.
pub fn default_app_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app")
}

/// Construct the bundler for `app_dir`. repo_root == app_dir so bundle output
/// lands where the resolver reads it. Zero-config host: no `deck.jsonc` — use
/// built-in defaults (no consumer targets, conventional route buckets).
fn make_bundler(app_dir: &Path) -> Bundler {
    let cfg = deck::config::DeckConfig::defaults();
    Bundler::start(app_dir.to_path_buf(), None, cfg)
}

/// deck's hosting surface + the two extensions its handlers read + the HMR SSE
/// route + the in-memory data port. Shared by both entry points.
///
/// `app_dir` becomes the mount-scoped `Resolution` root, layered as an
/// Extension so deck's handlers (and the data port's live-data SSE) resolve
/// route sources against THIS app's root — not a process global.
fn build_router(
    mode: DaemonMode,
    app_dir: &Path,
    bundler: Bundler,
    cfg: Arc<deck::config::DeckConfig>,
    store: Arc<DemoStore>,
) -> Router {
    let resolution = Arc::new(deck::Resolution::new().with_root(app_dir.to_path_buf()));
    router()
        // The named-route surface (`/_deck/view(s)/*`) was extracted out of deck
        // into the separate `ship-views` repo; deck's own `router()` no longer
        // carries it and this harness no longer merges it back. The remaining
        // conformance tests cover deck's own surface (assets, mutation
        // tombstone, prod caching) plus the local data port.
        // Tokens (/api/assets/view-tokens.css), HMR (/_deck/dev/hmr), and the
        // mutation tombstone come from deck's own router since the shipd
        // UI-purge — no host wiring.
        // Data port: live-data SSE + data endpoints (ship-free, in-memory,
        // mirrors shipd handlers with the kernel removed).
        .merge(data_port::router(store))
        .layer(Extension(mode))
        // DeckConfig Extension — scaffold handlers read scaffold declarations
        // and inject_globals from this. Zero-config hosts (no deck.jsonc) get
        // built-in defaults and degrade gracefully.
        .layer(Extension(cfg))
        .layer(Extension(bundler))
        // Mount-scoped resolution root — the replacement for the removed
        // `set_project_root` process global.
        .layer(Extension(resolution))
}

/// A ready router with NO filesystem watcher (read-only host / tests).
/// Reads the daemon mode from [`demo_mode`] (`DECK_DEMO_MODE` env var).
pub fn deck_app(app_dir: PathBuf) -> Router {
    deck_app_with_mode(app_dir, demo_mode())
}

/// Like [`deck_app`] but with an explicit mode — for tests that need to
/// verify production vs development behavior without reaching into the env.
pub fn deck_app_with_mode(app_dir: PathBuf, mode: DaemonMode) -> Router {
    deck_app_with_store(app_dir, mode, DemoStore::new())
}

/// Like [`deck_app_with_mode`] but with a caller-owned [`DemoStore`]. Tests
/// use this to mutate the store directly (the progressive-enhancement form endpoint is a 410
/// tombstone in deck's router) and assert the live SSE surface reflects it.
pub fn deck_app_with_store(app_dir: PathBuf, mode: DaemonMode, store: Arc<DemoStore>) -> Router {
    let cfg = Arc::new(deck::config::DeckConfig::defaults());
    build_router(mode, &app_dir, make_bundler(&app_dir), cfg, store)
}

/// A dev server with the FS watcher running: edit a `.sail` → rebuild the
/// affected target → push an HMR event. Async because `start_watching` spawns
/// the watch task. Reads the daemon mode from [`demo_mode`].
pub async fn deck_dev_server(app_dir: PathBuf) -> Router {
    let mode = demo_mode();
    let cfg = Arc::new(deck::config::DeckConfig::defaults());
    let bundler = make_bundler(&app_dir);
    if let Err(e) = bundler.start_watching().await {
        eprintln!("deck-conformance: watcher failed to start: {e}");
    }
    let store = DemoStore::new();
    build_router(mode, &app_dir, bundler, cfg, store)
}
