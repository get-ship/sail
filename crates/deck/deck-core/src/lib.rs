//! deck-core — transport-agnostic SAIL hosting logic.
//!
//! This crate owns *what* deck serves (compiled routes, bundles, the catalog)
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
pub mod css_minify;
pub mod host_context;
pub mod render_helpers;
pub mod resolver;
pub mod route;
pub mod route_tokens;
pub mod routes;
pub mod sail_css;
pub mod setup;
pub mod synthetic_ssr;
pub mod tag;
pub mod themes;
// `route_host` + `route_payload` (the named-route SERVING CORE) moved to the
// `ship-views` crate (deck ⊥ ship-views). deck-core keeps the SHARED infra both
// paths use: `resolver`, `synthetic_ssr` (the SSR seam), `route::serve_sail_route_setup`,
// `error_page`, `form_inject`. ship-views calls into those.
pub mod window_registry;

#[cfg(test)]
mod lucide_drift;
#[cfg(test)]
mod mode_tests;

// (formerly `mod test_lock` with a `ProjectRootGuard` RAII helper — removed with
// the process-global resolver it pinned. Resolution is now mount-scoped, so a
// test simply constructs a `Resolution::new().with_root(dir)` and passes it in;
// there is no global to guard or serialize.)
pub use app_port::AppRegistry;
pub use change_source::{ChangeFeed, ChangeSource};
pub use data_port::DataPort;
pub use live_wire::{
    PatchElements, PatchElementsMode, format_keepalive, format_mux_close, format_mux_data,
    format_mux_heartbeat, format_patch_elements, format_patch_set, format_patch_signals,
};
pub use response::{
    CachePolicy, DeckResponse, HTML_SECURITY_HEADERS, compute_etag, connect_src_directive,
    html_security_headers_for,
};
pub use window_registry::{
    Subscription, SubscriptionId, WindowBucket, WindowId, WindowSubscriptionRegistry,
};

pub use assets::{
    baked_framework_stamps, lucide_icon_names, lucide_icon_svg, parse_baked_stamp,
    resolve_route_js, serve_deck_route, serve_framework_font, serve_public_asset,
    serve_sail_compiler_wasm, serve_sail_dom, serve_sail_error_overlay, serve_sail_icon,
    serve_sail_live_reload, serve_sail_lsp_completions, serve_sail_lsp_diagnostics,
    serve_sail_runtime, serve_sail_sdk_client, serve_sail_studio_bridge, serve_scaffold,
    serve_scaffold_asset,
};
pub use form_inject::inject_form_actions;
pub use meta::{DaemonBuildInfo, serve_meta};
pub use mode_html::{
    inject_effective_themes_meta, inject_mode_meta, inject_project_themes_meta,
    inject_render_meta, inject_theme_defaults_meta, mode_str, sail_project_rel_from_root,
};
pub use resolver::{Resolution, resolve_project_root_with};
// (formerly `packs::serve_catalog_workspace`) — purged with the `.deck/` runtime read.
pub use css_minify::minify_css;
pub use route::{RouteSetupQuery, serve_sail_route_setup};
pub use route_tokens::serve_route_tokens;
pub use sail_css::serve_sail_css;
pub use setup::redirect;
pub use tag::{SAIL_TAG_PREFIX, strip_sail_tag_prefix};
pub use themes::{
    ThemeMeta, ThemeSource, list_themes, resolve_theme_css, serve_theme_css, serve_themes_list,
};
// `serve_route_host` / `serve_route_payload[_subpath]` moved to `ship-views`.

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
