//! Deck's host context — the framework-provided names a `.sail` may use without
//! the app declaring them in its `app.manifest.jsonc`.
//!
//! SAIL names no host: its synthesize path resolves every free name through the
//! app manifest, with no built-in convention. Deck declares its context **here**
//! — the one place `@ship/deck/router` is named into the synthesize seam — and
//! merges it into each app's manifest at build/synthesize time (the app's own
//! entries win on conflict). Adding a new well-known handle is a single-line,
//! single-file change.

use sail_compiler::emit::dts::ScopeCapability;
use sail_compiler::emit::synthetic::{AppManifest, ManifestClient};

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

/// The same context, as SCOPE CAPABILITIES — the typed members the compiler
/// writes into a view's generated `Scope`, so an authored controller reads
/// `ctx.route.params.id()` / `ctx.navigate('/x')` precisely and a name the app
/// never declared is a type error.
///
/// SAIL types every manifest client through its own `Clients` port
/// (`sail_compiler::emit::synthetic::scope_capabilities`); deck's two names are
/// NOT data clients, so deck declares their types here — the same one-file,
/// one-line seam as the imports above. `RouteSignals` is the SAIL-local route
/// port, not a deck type: the compiler still names no provider.
pub fn host_context_capabilities(manifest: &AppManifest) -> Vec<ScopeCapability> {
    let mut caps = sail_compiler::emit::synthetic::scope_capabilities(manifest);
    for cap in &mut caps {
        match cap.name.as_str() {
            "route" => {
                cap.ts_type = "RouteSignals".to_string();
                cap.ts_import =
                    Some(("RouteSignals".to_string(), "@ship/sail-runtime".to_string()));
            }
            "navigate" => {
                cap.ts_type = "(path: string) => void".to_string();
                cap.ts_import = None;
            }
            _ => {}
        }
    }
    caps
}
