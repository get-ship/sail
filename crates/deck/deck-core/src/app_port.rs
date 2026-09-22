//! The host's app registry — deck's seam to "which apps does the host serve,
//! and how is each one configured to serve".
//!
//! Sibling to [`crate::data_port::DataPort`]: a small port the HOST implements.
//! Deck declares the RESULT it needs — the set of [`AppServeConfig`] to mount —
//! and the host owns the MECHANISM (read its orchestration list, load each
//! app's `deck.jsonc`, bind the file locations inside the impl). Deck names no
//! Ship path; the host binds its own state, exactly as `DataPort` does.
//!
//! Why a port and not a bare `Vec`: the app list is a host capability ("the
//! apps I serve"), not framework state. The framework consumes the enumerated
//! result; it must not learn the host's file layout, its config filename, or
//! its orchestration shape. The seam keeps that knowledge on the host side.

use deck_config::AppServeConfig;

/// Enumerates the apps a host serves.
///
/// The host's impl reads whatever orchestration source it owns (for shipd: the
/// monorepo `apps.jsonc` list → each `apps/<app>/deck.jsonc`) and produces the
/// fully-resolved per-app serve configs. A load failure degrades to an empty
/// list (serve no apps — a clean 404, never a panic); the impl logs the cause.
pub trait AppRegistry: Send + Sync + 'static {
    /// The apps to serve, each a fully-resolved [`AppServeConfig`] loaded from
    /// that app's `deck.jsonc`.
    fn serve_configs(&self) -> Vec<AppServeConfig>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use deck_config::RenderMode;

    /// A host binds its own enumeration inside the impl; deck consumes only the
    /// `Vec<AppServeConfig>` result through the trait object — never the host's
    /// file layout. This stand-in proves the seam is object-safe and returns
    /// the host-resolved configs.
    struct StubRegistry(Vec<AppServeConfig>);
    impl AppRegistry for StubRegistry {
        fn serve_configs(&self) -> Vec<AppServeConfig> {
            self.0.clone()
        }
    }

    #[test]
    fn registry_is_object_safe_and_enumerates() {
        let hub = AppServeConfig {
            name: "hub".into(),
            route: "/".into(),
            source: "apps/hub/dist".into(),
            render: RenderMode::Csr,
            ..Default::default()
        };
        let reg: std::sync::Arc<dyn AppRegistry> = std::sync::Arc::new(StubRegistry(vec![hub]));
        let apps = reg.serve_configs();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "hub");
        assert_eq!(apps[0].render, RenderMode::Csr);
    }
}
