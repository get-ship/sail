//! First-run `/setup` placeholder + the redirect helper shared by `/` and
//! `/setup`.
//!
//! Both routes are unauthenticated — a freshly installed daemon has not yet
//! paired with any browser or CLI session, so the setup wizard must be
//! reachable without a token. The provisioned-vs-fresh decision is made by the
//! host through `DataPort::is_provisioned()`; deck owns only the HTML/redirect
//! shapes here, naming no Ship concept.

use crate::{CachePolicy, DaemonMode, DeckResponse};

/// Placeholder HTML served on a fresh install before the SUI runtime mounts.
const PLACEHOLDER_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Ship — Setup</title>
  <style>
    body{font-family:system-ui,-apple-system,sans-serif;background:#0b0d10;color:#e6e8eb;margin:0;min-height:100vh;display:grid;place-items:center;padding:24px}
    main{max-width:560px;width:100%;text-align:center}
    h1{font-size:28px;margin:0 0 12px;letter-spacing:-0.01em}
    p{color:#9aa1a9;line-height:1.5;margin:0 0 20px}
    code{background:#1a1d21;padding:2px 6px;border-radius:4px;font-family:ui-monospace,Menlo,monospace;font-size:13px}
  </style>
</head>
<body>
  <main data-ship-setup>
    <h1>Welcome to Ship</h1>
    <p>The setup wizard is on its way. In the meantime, point a CLI or MCP client at <code>http://localhost:9315</code> to start configuring this daemon.</p>
    <p>From your project directory, run <code>ship use my-agent</code> to activate an agent and compile its provider config.</p>
  </main>
</body>
</html>
"#;

/// Serve the first-run placeholder HTML (no-cache).
pub fn serve_setup_placeholder(mode: DaemonMode) -> DeckResponse {
    DeckResponse::html_view(PLACEHOLDER_HTML, mode)
}

/// A `303 See Other` redirect to `location` (empty body, no-cache).
pub fn redirect(location: impl Into<String>, mode: DaemonMode) -> DeckResponse {
    DeckResponse::new(303, "text/plain; charset=utf-8", CachePolicy::None, mode, Vec::new())
        .with_header("Location", location)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_html_contains_expected_shell() {
        let r = serve_setup_placeholder(DaemonMode::Development);
        assert_eq!(r.status, 200);
        let body = std::str::from_utf8(&r.body).unwrap();
        assert!(body.contains("<title>Ship — Setup</title>"));
        assert!(body.contains("data-ship-setup"));
    }

    #[test]
    fn redirect_sets_location_and_303() {
        let r = redirect("/setup", DaemonMode::Development);
        assert_eq!(r.status, 303);
        let loc = r.extra_headers.iter().find(|(k, _)| *k == "Location").map(|(_, v)| v.as_str());
        assert_eq!(loc, Some("/setup"));
    }
}
