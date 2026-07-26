//! The FRAMEWORK STYLESHEET — **sail-base** (document floor) + stdlib element skins.
//!
//! Deck serves this so a page is styled WITHOUT running the runtime's JS
//! injection (SSR / static / L1-in-page / the showcase site). Two halves,
//! two SSOTs, concatenated at compile time:
//!
//!   - **sail-base** (`sail-base.css`) — platform document floor: host
//!     baseline, interactive-surface, focus/disabled, framework `@font-face`,
//!     keyframes catalog. Token-driven rules; users do not author these —
//!     they theme via tokens only. Generated from the runtime's
//!     `LIGHT_CONTRACTS_CSS` (`packages/sail/runtime/scripts/emit-contracts-css.ts`).
//!     (Historical name "contracts"; package still exports `./contracts.css` as
//!     a deprecated alias of `./sail-base.css`.)
//!   - `element-skins.css` — the L1 element skins (spinner/separator/icon/
//!     label/…). Generated from the sail-compiler stdlib registry
//!     (`cargo run -p sail-compiler -- framework-css`). Do NOT fold L1 paint
//!     into sail-base.
//!
//! Both are drift-locked by tests in their source crates, so this concat can
//! never serve stale framework CSS. The shell links it with
//! `data-sail-contracts`, and the runtime's `ensureLightContracts` yields to
//! that link — injection is the JS-only-host fallback (dies at full cutover).
//!
//! Production responses are minified via [`crate::css_minify::minify_css`];
//! development keeps the readable source for debugging.

use std::sync::LazyLock;

use crate::css_minify::minify_css;
use crate::{CachePolicy, DaemonMode, DeckResponse};

/// sail-base + stdlib element skins, concatenated at compile time (unminified SSOT).
const SAIL_CSS: &str = concat!(
    include_str!("../../../../packages/sail/tokens/src/sail-base.css"),
    "\n",
    include_str!("../../../../packages/sail/tokens/src/element-skins.css"),
);

/// Cached production minify of [`SAIL_CSS`].
static SAIL_CSS_MINIFIED: LazyLock<Vec<u8>> =
    LazyLock::new(|| minify_css(SAIL_CSS).into_bytes());

/// `GET /api/assets/sail.css` — the framework stylesheet (sail-base + element-skins).
///
/// No-cache so contract/skin edits land on the next load without a hard
/// refresh (mirrors `view-tokens.css`). Production bodies are minified;
/// development bodies stay readable.
pub fn serve_sail_css(mode: DaemonMode) -> DeckResponse {
    let body = match mode {
        DaemonMode::Development => SAIL_CSS.as_bytes().to_vec(),
        DaemonMode::Production => SAIL_CSS_MINIFIED.clone(),
    };
    let mut r = DeckResponse::new(
        200,
        "text/css; charset=utf-8",
        CachePolicy::None,
        mode,
        body,
    );
    r.cache_control = "no-cache";
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sail_css_served_with_css_content_type() {
        let r = serve_sail_css(DaemonMode::Development);
        assert_eq!(r.status, 200);
        assert!(r.content_type.starts_with("text/css"));
        assert_eq!(r.cache_control, "no-cache");
        assert!(!r.body.is_empty(), "framework stylesheet must be non-empty");
    }

    #[test]
    fn sail_css_carries_both_halves() {
        let css = String::from_utf8(serve_sail_css(DaemonMode::Development).body).unwrap();
        // A sail-base marker (size attribute channel) and a skin marker (spinner).
        // Note: tone is no longer a universal channel in sail-base; size attrs remain.
        assert!(
            css.contains("[size=") || css.contains("@font-face") || css.contains("@layer sail"),
            "sail-base half present: {css:.120}"
        );
        assert!(css.contains(".sail-spinner"), "element-skins half present");
    }

    #[test]
    fn sail_css_production_is_minified() {
        let dev = serve_sail_css(DaemonMode::Development).body;
        let prod = serve_sail_css(DaemonMode::Production).body;
        assert!(
            prod.len() < dev.len(),
            "production must shrink vs development: prod={} dev={}",
            prod.len(),
            dev.len()
        );
        let prod_s = String::from_utf8(prod.clone()).unwrap();
        let dev_s = String::from_utf8(dev).unwrap();
        // Comments present in the generated source should be gone in prod.
        if dev_s.contains("/*") {
            assert!(
                !prod_s.contains("/*"),
                "production should strip CSS comments"
            );
        }
        // Both halves still present after minify.
        assert!(
            prod_s.contains(".sail-spinner"),
            "element-skins survives minify"
        );
        assert!(
            prod_s.contains("@font-face") || prod_s.contains("@layer") || prod_s.contains("[size"),
            "sail-base survives minify: {prod_s:.80}"
        );
    }

    #[test]
    fn sail_css_dev_stays_readable() {
        let css = String::from_utf8(serve_sail_css(DaemonMode::Development).body).unwrap();
        assert!(
            css.contains('\n'),
            "development path keeps multi-line CSS"
        );
        assert!(
            css.contains("/*") || css.contains("  "),
            "development path keeps comments or indentation"
        );
    }
}
