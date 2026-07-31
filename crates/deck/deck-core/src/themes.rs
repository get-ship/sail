//! Theme serving — a deck-owned serving concern.
//!
//! Themes are a SAIL-native feature (CSS vars). The layering:
//!   - SAIL ships a default light + dark theme (its base tokens).
//!   - deck ships a default light + dark theme — its base tokens, served as
//!     `view-tokens.css`. deck's default IS that base; the `theme.css` override
//!     is empty by default (the base tokens apply as-is).
//!   - A host (e.g. ship) ships several themes that OVERRIDE the defaults.
//!
//! The override is a BUNDLING concern, and the theme CONTENT is shipped by
//! whoever owns the layer — the host owns NO plumbing. deck-core therefore does
//! NOT discover themes by reaching into a host directory at runtime (the old
//! host-config fs-walk was a ship-product reach + the wrong shape —
//! inv-deck-standalone). deck serves its own default here; the bundler will
//! resolve + bake the theme cascade (SAIL default → deck default → host themes)
//! and deck will serve the baked result, exactly as it serves `view-tokens.css`.
//!
//! TARGET (not yet built — bundler theme-cascade): `serve_theme_css` reads the
//! baked override; `list_themes` reports the baked set. Until then deck serves
//! its default only (empty override, empty override list). Tracked: the deck
//! standalone overhaul (themes model). RELEASE-DEP: a host's themes return via
//! the bundler cascade, not a runtime directory read.

use serde::Serialize;

use crate::{CachePolicy, DaemonMode, DeckResponse};

/// Metadata for a discovered theme. Retained as the public list-endpoint shape;
/// populated by the bundler theme-cascade (the list is empty until then).
#[derive(Debug, Clone, Serialize)]
pub struct ThemeMeta {
    pub name: String,
    pub source: ThemeSource,
    pub has_light: bool,
    pub has_dark: bool,
}

/// Where a theme was baked from (for the future bundler-populated list).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSource {
    /// The framework's own default (SAIL/deck base tokens).
    Default,
    /// A host-shipped override theme (baked by the bundler cascade).
    Host,
}

/// `GET /api/assets/theme.css` — the active theme's CSS overrides.
///
/// deck serves its OWN default, whose override is empty (the base tokens in
/// `view-tokens.css` are the default light/dark). A host's override themes are
/// baked by the bundler cascade and served from there — deck-core does not
/// reach into a host directory at runtime.
pub fn serve_theme_css(mode: DaemonMode) -> DeckResponse {
    let mut r = DeckResponse::new(
        200,
        "text/css; charset=utf-8",
        CachePolicy::None,
        mode,
        resolve_theme_css().into_bytes(),
    );
    r.cache_control = "no-cache";
    r
}

/// `GET /api/v1/themes` — JSON list of override themes. Empty until the bundler
/// theme-cascade bakes a host's themes; deck's default is implicit (not a
/// selectable override).
pub fn serve_themes_list(mode: DaemonMode) -> DeckResponse {
    let json = serde_json::to_string(&list_themes()).unwrap_or_else(|_| "[]".to_string());
    let mut r = DeckResponse::new(
        200,
        "application/json; charset=utf-8",
        CachePolicy::None,
        mode,
        json.into_bytes(),
    );
    r.cache_control = "no-cache";
    r
}

/// Override themes available to select. Empty until the bundler theme-cascade
/// populates it — deck's default is implicit (the base tokens), not a listed
/// override.
pub fn list_themes() -> Vec<ThemeMeta> {
    Vec::new()
}

/// CSS for the active theme override. deck's default is "no override" (the base
/// tokens apply), so this is empty until the bundler bakes a host theme.
pub fn resolve_theme_css() -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_override_is_empty() {
        // deck serves its own default — base tokens apply, no override CSS.
        assert!(resolve_theme_css().is_empty());
    }

    #[test]
    fn themes_list_serializes_to_json_array() {
        let r = serve_themes_list(DaemonMode::Development);
        assert_eq!(r.status, 200);
        assert!(r.content_type.starts_with("application/json"));
        let body = std::str::from_utf8(&r.body).unwrap();
        assert!(
            body.starts_with('['),
            "themes list must be a JSON array; got {body}"
        );
    }
}
