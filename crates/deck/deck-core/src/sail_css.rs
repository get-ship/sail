//! Framework stylesheet: sail-base.css + element-skins.css (include_str concat).

use std::sync::LazyLock;

use crate::css_minify::minify_css;
use crate::{CachePolicy, DaemonMode, DeckResponse};

const SAIL_CSS: &str = concat!(
    include_str!("../../../../packages/sail/tokens/src/sail-base.css"),
    "\n",
    include_str!("../../../../packages/sail/tokens/src/element-skins.css"),
);

static SAIL_CSS_MINIFIED: LazyLock<Vec<u8>> = LazyLock::new(|| minify_css(SAIL_CSS).into_bytes());

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
        assert!(css.contains('\n'), "development path keeps multi-line CSS");
        assert!(
            css.contains("/*") || css.contains("  "),
            "development path keeps comments or indentation"
        );
    }
}
