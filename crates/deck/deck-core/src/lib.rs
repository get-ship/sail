//! deck-core — transport-agnostic SUI hosting logic.
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
pub mod themes;
pub mod view;
pub mod view_tokens;
pub mod view_host;
pub mod view_payload;
pub mod window_registry;

#[cfg(test)]
mod view_payload_tests;
#[cfg(test)]
mod view_payload_shape_tests;
#[cfg(test)]
mod view_payload_injection_tests;
#[cfg(test)]
mod mode_tests;

#[cfg(test)]
pub(crate) mod test_lock {
    use std::sync::Mutex;
    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());
    pub fn lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Test-only guard that pins the resolver's project root to `dir` for
    /// its lifetime. On Drop, the override is cleared. Holds the env lock
    /// so two tests can't race on the resolver.
    pub struct ProjectRootGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
    }
    impl ProjectRootGuard {
        pub fn new(dir: &std::path::Path) -> Self {
            let lock = lock();
            super::resolver::set_project_root(dir.to_path_buf());
            Self { _lock: lock }
        }
    }
    impl Drop for ProjectRootGuard {
        fn drop(&mut self) {
            super::resolver::reset_project_root_override_for_test();
        }
    }
// P2-11 canary — should trip the gate
pub const P2_11_CANARY: &str = "apps/shell";
}
pub use response::{CachePolicy, DeckResponse, HTML_SECURITY_HEADERS, compute_etag};
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
    resolve_view_js, serve_deck_route, serve_public_asset, serve_scaffold, serve_scaffold_asset,
    serve_sui_dom, serve_sui_icon,
    serve_sui_live_reload, serve_sui_runtime, serve_sui_sdk_client,
};
pub use form_inject::inject_form_actions;
pub use resolver::{register_view_output, set_project_root};
pub use meta::{serve_meta, DaemonBuildInfo};
pub use mode_html::{inject_mode_meta, mode_str};
// (formerly `packs::serve_catalog_workspace`) — purged with the `.deck/` runtime read.
pub use setup::redirect;
pub use themes::{list_themes, resolve_theme_css, serve_theme_css, serve_themes_list, ThemeMeta, ThemeSource};
pub use view_tokens::serve_view_tokens;
pub use view::{serve_sui_view_setup, ViewSetupQuery};
pub use view_host::serve_view_host;
pub use view_payload::{serve_view_payload, serve_view_payload_subpath};

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
