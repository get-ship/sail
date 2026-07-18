//! Shared design tokens + base styles for every deck-served view.
//!
//! Canonical OKLCH palette sourced from `packages/sail/tokens/src/tokens.css`
//! so SAIL components, deck-served views, and the theme studio can never drift.
//! View-scoped base rules live alongside in `deck-core/assets/view-tokens.css`
//! (deck owns this file — relocated out of shipd in the UI-purge so deck is
//! self-contained). Per-theme overrides come from `themes.css`.

use crate::{CachePolicy, DaemonMode, DeckResponse};

/// Design tokens + view base styles + per-theme overrides, concatenated at
/// compile time. The theme picker applies a theme by toggling `data-theme`
/// on `<html>` — the `:root[data-theme='paper']` etc. blocks come from
/// `themes.css`.
const VIEW_TOKENS_CSS: &str = concat!(
    include_str!("../../../../packages/sail/tokens/src/tokens.css"),
    "\n",
    include_str!("../assets/view-tokens.css"),
    "\n",
    include_str!("../../../../packages/sail/tokens/src/themes.css"),
);

/// `GET /api/assets/view-tokens.css` — the design-token + base stylesheet.
///
/// No-cache so token/theme edits land on the next load without a hard refresh.
pub fn serve_route_tokens(mode: DaemonMode) -> DeckResponse {
    let mut r = DeckResponse::new(
        200,
        "text/css; charset=utf-8",
        CachePolicy::None,
        mode,
        VIEW_TOKENS_CSS.as_bytes().to_vec(),
    );
    r.cache_control = "no-cache";
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_tokens_served_with_css_content_type() {
        let r = serve_route_tokens(DaemonMode::Development);
        assert_eq!(r.status, 200);
        assert!(r.content_type.starts_with("text/css"));
        assert_eq!(r.cache_control, "no-cache");
        assert!(!r.body.is_empty(), "concatenated tokens must be non-empty");
    }
}
