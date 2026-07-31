//! Production CSS minification via [lightningcss].
//!
//! Used by the framework stylesheet (`serve_sail_css`) and re-exported for the
//! bundler's `ShipAssetCssPlugin` so both production CSS paths share one
//! minifier. Dev stays unminified (readable for debugging).

use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};

/// Minify a CSS string for production delivery.
///
/// On parse/minify/print failure, returns the original source unchanged and
/// logs a warning — a minifier bug must never blank a page's styles.
pub fn minify_css(source: &str) -> String {
    let mut stylesheet = match StyleSheet::parse(source, ParserOptions::default()) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("css minify: parse failed, serving unminified: {e}");
            return source.to_string();
        }
    };
    if let Err(e) = stylesheet.minify(MinifyOptions::default()) {
        tracing::warn!("css minify: minify pass failed, serving unminified: {e}");
        return source.to_string();
    }
    match stylesheet.to_css(PrinterOptions {
        minify: true,
        ..Default::default()
    }) {
        Ok(r) => r.code,
        Err(e) => {
            tracing::warn!("css minify: print failed, serving unminified: {e}");
            source.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minify_strips_comments_and_whitespace() {
        let src = "/* banner */\n.foo  {\n  color:  red;\n}\n";
        let out = minify_css(src);
        assert!(!out.contains("/*"), "comments stripped: {out}");
        assert!(!out.contains("  "), "runs of whitespace collapsed: {out}");
        assert!(out.contains(".foo"), "selector preserved: {out}");
        assert!(
            out.contains("red") || out.contains("#f00") || out.contains("#ff0000"),
            "color preserved: {out}"
        );
        assert!(
            out.len() < src.len(),
            "size shrinks {} → {}",
            src.len(),
            out.len()
        );
    }

    #[test]
    fn minify_preserves_url_paths() {
        let src = r#"@font-face { src: url("./dm-sans-HASH.woff2") format("woff2"); }"#;
        let out = minify_css(src);
        assert!(
            out.contains("dm-sans-HASH.woff2"),
            "hashed url() must survive minify: {out}"
        );
        assert!(out.contains("url("), "url() wrapper preserved: {out}");
    }

    #[test]
    fn minify_invalid_css_falls_back_to_source() {
        // Unclosed block — lightningcss may reject; we must not panic or empty.
        let src = "/* keep me if parse fails */ .broken { color:";
        let out = minify_css(src);
        assert!(!out.is_empty());
        // Either minified partial success or original fallback — both ok as long
        // as we return something usable.
        assert!(out.contains("broken") || out.contains("keep me"));
    }
}
