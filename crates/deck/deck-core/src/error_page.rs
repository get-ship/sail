//! Human-facing error pages for the view host.
//!
//! The browser-facing view route (`/_deck/view/<name>`) serves an HTML shell.
//! When the requested view can't be resolved, the daemon must return a
//! *rendered* page — not the raw `{"error":{"view_not_found":…}}` JSON that
//! the programmatic `/_deck/views/<name>` payload endpoint emits for API
//! callers. This module owns that page: it fills the baked
//! `view-not-found.html` shell with the requested name and the views the
//! daemon can actually serve, then returns a `404 text/html` `DeckResponse`.
//!
//! Styling rides the daemon-served design tokens (`/api/assets/view-tokens.css`,
//! linked by the shell) so the page tracks the active theme — the same
//! convention `view-host.html` uses. No color/hex literals live here.

use crate::resolver::list_view_names;
use crate::response::CachePolicy;
use crate::{DaemonMode, DeckResponse};

const NOT_FOUND_SHELL: &str = include_str!("../assets/view-not-found.html");
const NAME_PLACEHOLDER: &str = "<!-- SAIL_NF_NAME -->";
const ROUTES_PLACEHOLDER: &str = "<!-- SAIL_NF_ROUTES -->";

/// Render the `404 view-not-found` HTML page for `name`.
///
/// `text/html`, status 404, never cached. The available-view list is a
/// best-effort scan of the mount's project `root` (`list_view_names`) — empty
/// when the caller has no root (`None`), in which case the routes block is
/// omitted gracefully rather than rendering an empty list.
pub fn view_not_found(mode: DaemonMode, name: &str, root: Option<&std::path::Path>) -> DeckResponse {
    let routes = root.map(list_view_names).unwrap_or_default();
    let body = NOT_FOUND_SHELL
        .replace(NAME_PLACEHOLDER, &escape_html(name))
        .replace(ROUTES_PLACEHOLDER, &routes_block(&routes));
    let body = crate::mode_html::inject_mode_meta(&body, mode);
    DeckResponse::new(
        404,
        "text/html; charset=utf-8",
        CachePolicy::None,
        mode,
        body.into_bytes(),
    )
}

/// Build the "available views" block: a labelled list of links to
/// `/_deck/view/<name>`. Empty input yields an empty string so the shell shows
/// no stray "available routes" heading over a blank list.
fn routes_block(routes: &[String]) -> String {
    if routes.is_empty() {
        return String::new();
    }
    let mut items = String::new();
    for r in routes {
        let safe = escape_html(r);
        items.push_str(&format!(
            "<li><a href=\"/_deck/view/{safe}\">/_deck/view/{safe}</a></li>"
        ));
    }
    format!(
        "<p class=\"nf-routes-label\">Views this daemon can serve</p>\
         <ul class=\"nf-routes\">{items}</ul>"
    )
}

/// Minimal HTML-text/attribute escape. The host path only reaches this with a
/// name `is_valid_view_name` already accepted (alphanumeric, `_`, `-`, `/`),
/// but the page is also served for *invalid* names, so escape defensively —
/// the name lands in both element text and an `href` attribute.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_angle_brackets_and_quotes() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a\"b'c&d"), "a&quot;b&#x27;c&amp;d");
    }

    #[test]
    fn routes_block_empty_is_blank() {
        assert_eq!(routes_block(&[]), "");
    }

    #[test]
    fn routes_block_lists_links() {
        let block = routes_block(&["home".into(), "studio".into()]);
        assert!(block.contains("href=\"/_deck/view/home\""));
        assert!(block.contains("href=\"/_deck/view/studio\""));
        assert!(block.contains("nf-routes-label"));
    }
}
