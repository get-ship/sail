//! Theme viewport breakpoints for the structural `<when {token}>` block.
//!
//! Names and media queries match Style condition keys
//! (`sail_lint::style::conditions`) so paint-level `.when({ tablet: … })`
//! and tree-level `<when {tablet}>` share one scale. Widths are the
//! design-token `breakpoint-*` lengths.

/// Responsive breakpoint tokens authors may pass to `<when {…}>`.
/// Order is mobile-first (smallest min-width first).
pub const BREAKPOINT_TOKENS: &[&str] = &["mobile", "tablet", "desktop"];

/// Resolve a breakpoint token to its media query. `None` when the name is
/// not on the theme scale (raw px, unknown id, paint-only conditions).
pub fn breakpoint_media(token: &str) -> Option<&'static str> {
    let t = token.trim();
    if !BREAKPOINT_TOKENS.contains(&t) {
        return None;
    }
    sail_lint::style::conditions::condition_media(t)
}

/// Human-readable list for diagnostics: `mobile, tablet, desktop`.
pub fn breakpoint_token_list() -> String {
    BREAKPOINT_TOKENS.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_resolve_to_min_width_media() {
        for name in BREAKPOINT_TOKENS {
            let q = breakpoint_media(name).unwrap_or_else(|| panic!("missing {name}"));
            assert!(
                q.contains("min-width"),
                "{name} media must be min-width: {q}"
            );
        }
    }

    #[test]
    fn rejects_raw_px_and_paint_conditions() {
        assert!(breakpoint_media("768px").is_none());
        assert!(breakpoint_media("(min-width: 768px)").is_none());
        assert!(breakpoint_media("motion-reduce").is_none());
        assert!(breakpoint_media("dark").is_none());
        assert!(breakpoint_media("bp-md").is_none());
    }

    #[test]
    fn matches_style_condition_table() {
        for name in BREAKPOINT_TOKENS {
            assert_eq!(
                breakpoint_media(name),
                sail_lint::style::conditions::condition_media(name),
                "{name} must equal Style condition media"
            );
        }
    }
}
