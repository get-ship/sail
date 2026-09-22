//! The `303` redirect helper shared by the first-run flow.
//!
//! `/` is unauthenticated — a freshly installed daemon has not yet paired with
//! any browser or CLI session — and on a fresh install it redirects to
//! `/setup`. `/setup` itself has NO server route: it falls through to the SPA
//! fallback, which serves the app shell so the app's own client-side `/setup`
//! wizard renders. The provisioned-vs-fresh decision is made by the host
//! through `DataPort::is_provisioned()`; deck owns only the redirect shape
//! here, naming no Ship concept.

use crate::{CachePolicy, DaemonMode, DeckResponse};

/// A `303 See Other` redirect to `location` (empty body, no-cache).
pub fn redirect(location: impl Into<String>, mode: DaemonMode) -> DeckResponse {
    DeckResponse::new(
        303,
        "text/plain; charset=utf-8",
        CachePolicy::None,
        mode,
        Vec::new(),
    )
    .with_header("Location", location)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_sets_location_and_303() {
        let r = redirect("/setup", DaemonMode::Development);
        assert_eq!(r.status, 303);
        let loc = r
            .extra_headers
            .iter()
            .find(|(k, _)| *k == "Location")
            .map(|(_, v)| v.as_str());
        assert_eq!(loc, Some("/setup"));
    }
}
