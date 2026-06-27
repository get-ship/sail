//! Deck's host context — the framework-provided names a `.sui` may use without
//! the app declaring them in its `app.manifest.jsonc`.
//!
//! SUI names no host: its synthesize path resolves every free name through the
//! app manifest, with no built-in convention. Deck declares its context **here**
//! — the one place `@ship/deck/router` is named into the synthesize seam — and
//! merges it into each app's manifest at build/synthesize time (the app's own
//! entries win on conflict). Adding a new well-known handle is a single-line,
//! single-file change.

use sui_compiler::emit::synthetic::ManifestClient;

/// The names deck injects into every app manifest: `route` (constructed from the
/// `route()` factory) and `navigate` (a plain function reference), both from
/// `@ship/deck/router`. Callers merge these with the app manifest's clients,
/// letting an app override a name by declaring it itself.
pub fn host_context_clients() -> Vec<(String, ManifestClient)> {
    vec![
        (
            "route".to_string(),
            ManifestClient {
                from: "@ship/deck/router".to_string(),
                import: "route".to_string(),
                construct: true,
            },
        ),
        (
            "navigate".to_string(),
            ManifestClient {
                from: "@ship/deck/router".to_string(),
                import: "navigate".to_string(),
                construct: false,
            },
        ),
    ]
}
