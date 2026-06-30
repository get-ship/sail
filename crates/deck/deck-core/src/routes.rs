//! Deck's own `/_deck/...` serve-scheme URL builders.
//!
//! Deck owns the URL contract for the assets and views it hosts. Anything that
//! needs a `/_deck/...` URL on the Rust side builds it here rather than hand-
//! writing the literal, so the scheme has one source of truth in the layer that
//! owns serving. (The TypeScript side derives the same scheme from the
//! `packages/deck/sdk-client` `DECK_PREFIX` export — see #157/#158
//! "deck owns the serve contract".)

/// URL prefix for the daemon-served design-token / theme / shared CSS surface
/// (e.g. `<DECK_API_ASSETS>/view-tokens.css`, `<DECK_API_ASSETS>/theme.css`).
/// The TypeScript mirror is `DECK_API_ASSETS` in `@ship/sdk-client`; a
/// `ts_constants_pinned_to_rust` test in this module asserts they match
/// exactly so the two halves of the SSOT cannot drift.
pub const DECK_API_ASSETS: &str = "/api/assets";

/// Single source of truth for the serve-scheme URL prefix. Every Rust literal
/// that names `/_deck/...` (and the companion TS `DECK_PREFIX` export in
/// `@ship/sdk-client`) routes through this constant so the prefix has ONE
/// mint point on the server side. deck, the host, names the URL namespace;
/// the framework (SUI) does not.
pub const DECK_PREFIX: &str = "/_deck";

/// URL of a view's compiled ESM module. `sub` is the optional sub-route segment
/// for nested/parameterized views (`/x/:id`), carried as a `?sub=` query so the
/// host resolves the right leaf.
pub fn view_module_url(name: &str, sub: Option<&str>) -> String {
    match sub.filter(|s| !s.is_empty()) {
        Some(s) => format!("{DECK_PREFIX}/views/{name}/view.js?sub={s}"),
        None => format!("{DECK_PREFIX}/views/{name}/view.js"),
    }
}

/// URL the daemon serves for form-mutation POST submission.
/// Matches the daemon's `POST /_deck/mutation/{name}` route exactly.
pub fn mutation_action(name: &str, form_key: &str) -> String {
    format!("{DECK_PREFIX}/mutation/{name}?form={form_key}")
}

/// URL the daemon serves for live-data SSE streaming.
/// Matches the daemon's `GET /_deck/views/{name}/sse` route exactly.
pub fn subscribe_url(name: &str) -> String {
    format!("{DECK_PREFIX}/views/{name}/sse")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_prefix_is_the_canonical_literal() {
        assert_eq!(DECK_PREFIX, "/_deck");
    }

    #[test]
    fn view_module_url_with_and_without_sub() {
        assert_eq!(view_module_url("demo", None), "/_deck/views/demo/view.js");
        assert_eq!(
            view_module_url("demo", Some("anything")),
            "/_deck/views/demo/view.js?sub=anything",
        );
        // Empty sub is treated as no sub.
        assert_eq!(view_module_url("demo", Some("")), "/_deck/views/demo/view.js");
    }

    #[test]
    fn mutation_action_builds_correct_url() {
        assert_eq!(
            mutation_action("todos", "b0"),
            "/_deck/mutation/todos?form=b0",
        );
        assert_eq!(
            mutation_action("my-view", "b3"),
            "/_deck/mutation/my-view?form=b3",
        );
    }

    #[test]
    fn subscribe_url_builds_correct_url() {
        assert_eq!(subscribe_url("clock"), "/_deck/views/clock/sse");
        assert_eq!(subscribe_url("studio/agents"), "/_deck/views/studio/agents/sse");
    }
}
