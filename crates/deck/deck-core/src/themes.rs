//! Theme discovery and resolution — a deck-owned serving concern.
//!
//! Themes override the design tokens served by `view-tokens.css`. Two on-disk
//! sources are searched (global wins over project on a name collision):
//!
//! 1. **Project** — `<project_root>/.ship/themes/*.css`, resolved through the
//!    deck resolver's one project root.
//! 2. **Global** — `<home>/.ship/themes/*.css` in the user's home directory.
//!
//! The active theme is the `DECK_ACTIVE_THEME` value (default `"ship"`).
//! `"ship"` means no override — the base tokens are used as-is, so the
//! theme.css endpoint returns an empty body. This is a deck-owned seam; the
//! host injects nothing and deck names no Ship concept (the `.ship/themes`
//! directory is a path convention, not a domain import).

use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::{CachePolicy, DaemonMode, DeckResponse};

/// Metadata for a discovered theme.
#[derive(Debug, Clone, Serialize)]
pub struct ThemeMeta {
    pub name: String,
    pub source: ThemeSource,
    pub has_light: bool,
    pub has_dark: bool,
}

/// Where a theme was loaded from.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSource {
    Project,
    Global,
}

/// `GET /api/assets/theme.css` — the active theme's CSS overrides.
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

/// `GET /api/v1/themes` — JSON list of all discovered themes.
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

/// Return metadata for all discovered themes across both sources.
pub fn list_themes() -> Vec<ThemeMeta> {
    let mut themes = Vec::new();
    if let Some(dir) = project_themes_dir() {
        collect_dir_themes(&dir, ThemeSource::Project, &mut themes);
    }
    if let Some(dir) = global_themes_dir() {
        collect_dir_themes(&dir, ThemeSource::Global, &mut themes);
    }
    themes
}

/// Resolve the CSS for the currently active theme.
///
/// Returns an empty string when the active theme is `"ship"` (the default)
/// or when the requested theme cannot be found.
pub fn resolve_theme_css() -> String {
    let name = active_theme_name();
    if name == "ship" {
        return String::new();
    }
    resolve_css_by_name(&name)
}

// ── Internals ────────────────────────────────────────────────────────────────

fn active_theme_name() -> String {
    std::env::var("DECK_ACTIVE_THEME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "ship".to_string())
}

fn project_themes_dir() -> Option<PathBuf> {
    crate::resolver::resolve_project_root().map(|r| r.join(".ship").join("themes"))
}

fn global_themes_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".ship").join("themes"))
}

/// Best-effort home directory via the standard env vars — no external crate.
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}

fn resolve_css_by_name(name: &str) -> String {
    // Global > project (last-write-wins for overrides).
    if let Some(dir) = global_themes_dir() {
        if let Ok(css) = std::fs::read_to_string(dir.join(format!("{name}.css"))) {
            return css;
        }
    }
    if let Some(dir) = project_themes_dir() {
        if let Ok(css) = std::fs::read_to_string(dir.join(format!("{name}.css"))) {
            return css;
        }
    }
    String::new()
}

fn collect_dir_themes(dir: &Path, source: ThemeSource, out: &mut Vec<ThemeMeta>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("css") {
            continue;
        }
        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let css = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        out.push(ThemeMeta {
            name,
            source: source.clone(),
            has_light: css.contains(":root"),
            has_dark: css.contains(".dark"),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_unknown_returns_empty() {
        assert!(resolve_css_by_name("nonexistent-theme-xyz").is_empty());
    }

    #[test]
    fn themes_list_serializes_to_json_array() {
        let r = serve_themes_list(DaemonMode::Development);
        assert_eq!(r.status, 200);
        assert!(r.content_type.starts_with("application/json"));
        let body = std::str::from_utf8(&r.body).unwrap();
        assert!(body.starts_with('['), "themes list must be a JSON array; got {body}");
    }
}
