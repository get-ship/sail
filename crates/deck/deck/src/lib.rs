//! deck — thin re-export facade over deck-axum + deck-core + deck-host.
//!
//! Hosting logic lives in `deck-core` (transport-agnostic, returns
//! `DeckResponse`); `deck-axum` is the axum adapter (`serve_*` handlers +
//! `router`). This crate re-exports both so shipd's imports (`deck::*`)
//! continue to resolve without changes.

pub use deck_axum::{
    apps::serve_apps,
    into_axum, mode_from_ext, pages, router,
    serve_meta,
    serve_deck_route,
    serve_sui_dom, serve_sui_icon, serve_sui_live_reload,
    serve_sui_runtime, serve_sui_sdk_client,
};
// The named-view handlers + router moved to the `ship-views` crate
// (deck ⊥ ship-views). A host serves views by merging `ship_views::router()`;
// the facade no longer re-exports the view handlers (it cannot depend on
// ship-views).
pub use deck_config::{AppServeConfig, RenderMode};
pub use deck_core::AppRegistry;
pub use deck_axum::multiplex::{MultiplexState, multiplex_router, HEARTBEAT_MS};

// The SPA navigation fallback. deck's `router()` deliberately sets NO fallback
// (it merges alongside the Connect-ops router, which sets its own — two merged
// fallbacks panic axum). The host sets this once on the fully-merged root.
pub use deck_axum::pages::spa_fallback;

// Contract types (transport-agnostic), surfaced for hosts that name them.
// `DataPort` is the data seam a host implements (live `<data source>`); the
// facade surfaces it so hosts write `deck::DataPort`, not `deck_core::DataPort`.
pub use deck_core::{
    CachePolicy, DaemonBuildInfo, DaemonMode, DataPort, DeckResponse, ViewSetupQuery,
    inject_mode_meta, mode_str, register_view_output,
};

// SSE envelope encoders — single source of truth for the live-data wire protocol.
// Both shipd and deck-conformance (and any future host) call these; no copies.
pub use deck_core::live_wire::{
    PatchElements, PatchElementsMode,
    format_keepalive, format_patch_elements, format_patch_set, format_patch_signals,
};
pub use deck_core::live_wire;

// Submodule surfaces consumers reach into directly (shipd, xtask SDK gen).
//
// `render_helpers` lives in deck-axum (its `not_found` returns an axum
// response); `resolver` is pure and lives in deck-core. `view_payload` (the
// `ViewPayload` wire DTO) is a shared low-level type that lives in deck-host —
// it stayed put when the view SERVING moved to ship-views, so the facade still
// surfaces `deck::view_payload::ViewPayload` from deck-host (no ship-views dep).
pub use deck_axum::render_helpers;
pub use deck_core::resolver;
pub use deck_host::view_payload;

// The bundler — deck owns bundling (#167). Consumers that named
// `ship_platform::bundler::*` now use `deck::bundler::*`.
pub use deck_bundler as bundler;

// deck.jsonc config — the OPTIONAL host override for view-discovery + watch
// layout. Consumer bundles are self-declared per-package + discovered.
pub use deck_config as config;
