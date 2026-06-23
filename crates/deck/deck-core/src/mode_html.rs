//! Inject `<meta name="ship-*">` tags into served HTML shells.
//!
//! shell's index.html ships a static <meta name="ship-mode"
//! content="development"> tag. When shipd serves the same HTML in
//! production mode it rewrites that tag via `inject_mode_meta` so the
//! shell's status bar can render the DEV badge appropriately. The same
//! rewrite-or-insert mechanism surfaces the app's render mode via
//! `inject_render_meta` (`<meta name="ship-render">`) so CSR-vs-SSR is
//! legible client-side rather than implicit.

use crate::DaemonMode;
use deck_config::RenderMode;

/// Short string label for the daemon mode. Used as the `content`
/// attribute of `<meta name="ship-mode">`.
pub fn mode_str(mode: DaemonMode) -> &'static str {
    match mode {
        DaemonMode::Development => "development",
        DaemonMode::Production => "production",
    }
}

/// Short string label for the render mode. Used as the `content`
/// attribute of `<meta name="ship-render">` (`csr` / `ssr`).
pub fn render_str(render: RenderMode) -> &'static str {
    match render {
        RenderMode::Csr => "csr",
        RenderMode::Ssr => "ssr",
    }
}

/// Rewrite (or insert) a `<meta name="{name}" content="{value}" />` tag in an
/// HTML string. If a tag with that `name` already exists (with any `content`),
/// replace the whole tag; otherwise insert a fresh one right after `<head>`
/// so it appears before scripts. No `<head>` ⇒ returned unchanged.
fn inject_named_meta(html: &str, name: &str, value: &str) -> String {
    let needle = format!("<meta name=\"{name}\"");
    let replacement = format!("<meta name=\"{name}\" content=\"{value}\" />");
    if let Some(start) = html.find(&needle) {
        // Find the closing `>` of this tag.
        if let Some(rel_end) = html[start..].find('>') {
            let end = start + rel_end + 1;
            let mut out = String::with_capacity(html.len() + replacement.len());
            out.push_str(&html[..start]);
            out.push_str(&replacement);
            out.push_str(&html[end..]);
            return out;
        }
    }
    // Tag missing — insert after <head>.
    let head_open = "<head>";
    if let Some(idx) = html.find(head_open) {
        let insert_at = idx + head_open.len();
        let mut out = String::with_capacity(html.len() + replacement.len() + 4);
        out.push_str(&html[..insert_at]);
        out.push_str("\n  ");
        out.push_str(&replacement);
        out.push_str(&html[insert_at..]);
        return out;
    }
    // No <head> — return as-is.
    html.to_string()
}

/// Rewrite (or insert) the `<meta name="ship-mode" content="…">` tag
/// in an HTML string to reflect the daemon's current mode.
pub fn inject_mode_meta(html: &str, mode: DaemonMode) -> String {
    inject_named_meta(html, "ship-mode", mode_str(mode))
}

/// Rewrite (or insert) the `<meta name="ship-render" content="csr|ssr">` tag
/// so the app's active render mode is observable client-side.
pub fn inject_render_meta(html: &str, render: RenderMode) -> String {
    inject_named_meta(html, "ship-render", render_str(render))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_existing_tag() {
        let html =
            "<head>\n<meta name=\"ship-mode\" content=\"development\" />\n</head>";
        let out = inject_mode_meta(html, DaemonMode::Production);
        assert!(out.contains("content=\"production\""));
        assert!(!out.contains("content=\"development\""));
    }

    #[test]
    fn inserts_when_missing() {
        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_mode_meta(html, DaemonMode::Development);
        assert!(out.contains("<meta name=\"ship-mode\" content=\"development\""));
    }

    #[test]
    fn no_head_returns_unchanged() {
        let html = "<html><body>x</body></html>";
        let out = inject_mode_meta(html, DaemonMode::Production);
        assert_eq!(out, html);
    }

    #[test]
    fn render_meta_inserted_when_missing() {
        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_render_meta(html, RenderMode::Ssr);
        assert!(out.contains("<meta name=\"ship-render\" content=\"ssr\""));
    }

    #[test]
    fn render_meta_rewrites_existing_tag() {
        let html = "<head>\n<meta name=\"ship-render\" content=\"csr\" />\n</head>";
        let out = inject_render_meta(html, RenderMode::Ssr);
        assert!(out.contains("content=\"ssr\""));
        assert!(!out.contains("content=\"csr\""));
    }

    /// The two meta injectors are independent — injecting render leaves an
    /// already-present ship-mode tag intact (different `name`).
    #[test]
    fn render_and_mode_meta_coexist() {
        let html = "<head>\n<meta name=\"ship-mode\" content=\"development\" />\n</head>";
        let out = inject_render_meta(html, RenderMode::Csr);
        assert!(out.contains("name=\"ship-mode\" content=\"development\""));
        assert!(out.contains("name=\"ship-render\" content=\"csr\""));
    }
}
