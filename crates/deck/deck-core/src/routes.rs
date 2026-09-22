//! Deck's own `/_deck/...` serve-scheme URL builders.
//!
//! Deck owns the URL contract for the assets and routes it hosts. Anything that
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
/// the framework (SAIL) does not.
///
/// Product SPA is `routes/` + `mountRouter` (hashed entry from `dist/`).
/// Deck does not mint `/_deck/views/{name}/view.js` — `views/` is not a
/// public Deck URL surface.
pub const DECK_PREFIX: &str = "/_deck";

/// URL the daemon serves for form-mutation POST submission.
/// Matches the daemon's `POST /_deck/mutation/{name}` route exactly.
pub fn mutation_action(name: &str, form_key: &str) -> String {
    format!("{DECK_PREFIX}/mutation/{name}?form={form_key}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_prefix_is_the_canonical_literal() {
        assert_eq!(DECK_PREFIX, "/_deck");
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
}

/// Deck's file-location routing convention, as a QUESTION the compile
/// pipeline can ask: is this `.sail` file a route, and with which params?
///
/// A file is a route iff it is a `page.sail` under the nearest `routes/`
/// ancestor. Params are the `[seg]` directory names between that ancestor
/// and the file, in path order — `routes/tutorial/[slug]/page.sail` →
/// `["slug"]`. `views/` is not a product route tree.
///
/// This is deck's convention in deck's crate: the sail compiler receives the
/// result as a structured GRANT (`CompileOpts::route_params`) and never
/// sniffs the filesystem for a routing scheme it does not own.
pub fn route_params_for(route_path: &std::path::Path) -> Option<Vec<String>> {
    let file = route_path.file_name()?.to_str()?;
    if file != "page.sail" {
        return None;
    }
    let comps: Vec<&str> = route_path
        .parent()?
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let anchor = comps.iter().rposition(|c| *c == "routes")?;
    let params = comps[anchor + 1..]
        .iter()
        .filter_map(|seg| seg.strip_prefix('[').and_then(|s| s.strip_suffix(']')))
        .map(str::to_string)
        .collect();
    Some(params)
}

#[cfg(test)]
mod route_params_tests {
    use super::route_params_for;
    use std::path::Path;

    #[test]
    fn page_under_routes_is_a_route_with_bracket_params() {
        assert_eq!(
            route_params_for(Path::new("/x/apps/site/routes/tutorial/[slug]/page.sail")),
            Some(vec!["slug".to_string()])
        );
        assert_eq!(
            route_params_for(Path::new("/x/app/routes/greet/[id]/page.sail")),
            Some(vec!["id".to_string()])
        );
        // Static routes are still routes — an empty grant legalizes `route`
        // (e.g. `route.path`) with no params.
        assert_eq!(
            route_params_for(Path::new("/x/routes/about/page.sail")),
            Some(vec![])
        );
    }

    #[test]
    fn non_route_files_get_no_grant() {
        assert_eq!(
            route_params_for(Path::new("/x/packs/sail-badge/badge.sail")),
            None
        );
        assert_eq!(
            route_params_for(Path::new("/x/routes/tutorial/[slug]/lesson.ts")),
            None
        );
        // `views/` is not a product route tree — no grant even for page.sail.
        assert_eq!(
            route_params_for(Path::new("/x/app/views/tutorial/[slug]/page.sail")),
            None
        );
    }
}
