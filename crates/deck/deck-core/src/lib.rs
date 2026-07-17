//! deck-core — transport-agnostic SAIL hosting logic.
//!
//! This crate owns *what* deck serves (compiled views, bundles, the catalog)
//! and produces a [`DeckResponse`] for each. It never touches axum or any
//! HTTP framework. Server adapters (`deck-axum`, a future `deck-bun`)
//! translate `DeckResponse` onto their own response type — the
//! ports-and-adapters seam.

pub mod app_port;
pub mod change_source;
pub mod data_port;
pub mod response;

// (formerly `pub mod asset_graph;`) — purged with the `.deck/` runtime read.
pub mod assets;
pub mod error_page;
pub mod form_inject;
pub mod live_wire;
pub mod meta;
pub mod mode_html;
// (formerly `pub mod packs;`) — purged with the `.deck/` runtime read.
pub mod render_helpers;
pub mod resolver;
pub mod routes;
pub mod setup;
pub mod synthetic_ssr;
pub mod host_context;
pub mod sail_css;
pub mod tag;
pub mod themes;
pub mod view;
pub mod view_tokens;
// `view_host` + `view_payload` (the named-view SERVING CORE) moved to the
// `ship-views` crate (deck ⊥ ship-views). deck-core keeps the SHARED infra both
// paths use: `resolver`, `synthetic_ssr` (the SSR seam), `view::serve_sui_view_setup`,
// `error_page`, `form_inject`. ship-views calls into those.
pub mod window_registry;

#[cfg(test)]
mod mode_tests;

// (formerly `mod test_lock` with a `ProjectRootGuard` RAII helper — removed with
// the process-global resolver it pinned. Resolution is now mount-scoped, so a
// test simply constructs a `Resolution::new().with_root(dir)` and passes it in;
// there is no global to guard or serialize.)
pub use response::{
    CachePolicy, DeckResponse, HTML_SECURITY_HEADERS, compute_etag, connect_src_directive,
    html_security_headers_for,
};
pub use app_port::AppRegistry;
pub use change_source::{ChangeFeed, ChangeSource};
pub use data_port::DataPort;
pub use live_wire::{
    PatchElements, PatchElementsMode,
    format_keepalive, format_mux_close, format_mux_data, format_mux_heartbeat,
    format_patch_elements, format_patch_set, format_patch_signals,
};
pub use window_registry::{
    Subscription, SubscriptionId, WindowBucket, WindowId, WindowSubscriptionRegistry,
};

pub use assets::{
    baked_framework_stamps, parse_baked_stamp, resolve_view_js, serve_deck_route,
    serve_public_asset, serve_scaffold, serve_scaffold_asset, serve_sui_dom, serve_sui_error_overlay,
    serve_sui_icon, serve_sui_live_reload, serve_sui_runtime, serve_sui_sdk_client,
};
pub use form_inject::inject_form_actions;
pub use resolver::{resolve_project_root_with, Resolution};
pub use meta::{serve_meta, DaemonBuildInfo};
pub use mode_html::{inject_mode_meta, mode_str};
// (formerly `packs::serve_catalog_workspace`) — purged with the `.deck/` runtime read.
pub use setup::redirect;
pub use themes::{list_themes, resolve_theme_css, serve_theme_css, serve_themes_list, ThemeMeta, ThemeSource};
pub use sail_css::serve_sail_css;
pub use tag::{strip_sail_tag_prefix, SAIL_TAG_PREFIX};
pub use view_tokens::serve_view_tokens;
pub use view::{serve_sui_view_setup, ViewSetupQuery};
// `serve_view_host` / `serve_view_payload[_subpath]` moved to `ship-views`.

/// Daemon operating mode — controls cache headers, HMR availability, SSR cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DaemonMode {
    Development,
    Production,
}

impl DaemonMode {
    pub fn parse(s: &str) -> Self {
        match s {
            "production" => Self::Production,
            _ => Self::Development,
        }
    }
}
// P2-11 canary: apps/shell
