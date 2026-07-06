//! The FRAMEWORK STYLESHEET — the light-DOM contracts + stdlib element skins
//! deck serves so a page is styled WITHOUT running the runtime's JS
//! injection (SSR / static / L1-in-page / the showcase site). Two halves,
//! two SSOTs, concatenated at compile time:
//!
//!   - `contracts.css` — the tone/size channels, interactive-surface
//!     baseline, focus/disabled, button skin. Generated from the runtime's
//!     `LIGHT_CONTRACTS_CSS` (`packages/sui/runtime/scripts/emit-contracts-css.ts`).
//!   - `element-skins.css` — the L1 element skins (spinner/separator/icon/
//!     label/…). Generated from the sui-compiler stdlib registry
//!     (`cargo run -p sui-compiler -- framework-css`).
//!
//! Both are drift-locked by tests in their source crates, so this concat can
//! never serve stale framework CSS. The shell links it with
//! `data-sui-contracts`, and the runtime's `ensureLightContracts` yields to
//! that link — injection is the JS-only-host fallback (dies at full cutover).

use crate::{CachePolicy, DaemonMode, DeckResponse};

/// Framework contracts + stdlib element skins, concatenated at compile time.
const SUI_CSS: &str = concat!(
    include_str!("../../../../packages/sui/tokens/src/contracts.css"),
    "\n",
    include_str!("../../../../packages/sui/tokens/src/element-skins.css"),
);

/// `GET /api/assets/sui.css` — the framework stylesheet.
///
/// No-cache so contract/skin edits land on the next load without a hard
/// refresh (mirrors `view-tokens.css`).
pub fn serve_sui_css(mode: DaemonMode) -> DeckResponse {
    let mut r = DeckResponse::new(
        200,
        "text/css; charset=utf-8",
        CachePolicy::None,
        mode,
        SUI_CSS.as_bytes().to_vec(),
    );
    r.cache_control = "no-cache";
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sui_css_served_with_css_content_type() {
        let r = serve_sui_css(DaemonMode::Development);
        assert_eq!(r.status, 200);
        assert!(r.content_type.starts_with("text/css"));
        assert_eq!(r.cache_control, "no-cache");
        assert!(!r.body.is_empty(), "framework stylesheet must be non-empty");
    }

    #[test]
    fn sui_css_carries_both_halves() {
        let css = String::from_utf8(serve_sui_css(DaemonMode::Development).body).unwrap();
        // A contracts marker (tone channel) and a skin marker (spinner).
        assert!(css.contains("[tone="), "contracts half present: {css:.80}");
        assert!(css.contains(".sui-spinner"), "element-skins half present");
    }
}
