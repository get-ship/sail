use deck_host::resolver::ResolvedRoute;
use sail_compiler::{CompileOpts, compile};

use crate::assets::resolve_route_js;
use crate::resolver::{Resolution, resolve};
use crate::{DaemonMode, DeckResponse};

#[derive(Debug, serde::Deserialize)]
pub struct RouteSetupQuery {
    pub sub: Option<String>,
}

/// On-demand compile-or-read for a named route module (named-view leftover).
///
/// Product SPA is `routes/` + `mountRouter`: hashed entry from `dist/`, HMR
/// via `__deckHmr`. Deck's public router does not serve
/// `/_deck/views/{name}/view.js` — `views/` is not a public Deck URL.
/// This helper remains for ship-views: when a built `routes/<name>/route.js`
/// is present under the mount-scoped [`Resolution`] it is returned;
/// otherwise the source `.sail` compiles in-process.
pub fn serve_sail_route_setup(
    mode: DaemonMode,
    resolution: &Resolution,
    name: &str,
    sub: Option<&str>,
) -> DeckResponse {
    if !is_valid_route_name(name) {
        return DeckResponse::not_found("invalid route name", mode);
    }
    if sub.is_none() {
        if let Some(js) = resolve_route_js(Some(resolution), name) {
            return DeckResponse::js_route(js, mode);
        }
    }
    let Some(root) = resolution.project_root() else {
        return DeckResponse::not_found(
            "route: no project root (host must carry a Resolution)",
            mode,
        );
    };
    match resolve(root, name, sub) {
        Ok(r) => compile_route_js(name, &r, mode),
        Err(e) => DeckResponse::not_found(e.message, mode),
    }
}

fn compile_route_js(name: &str, resolved: &ResolvedRoute, mode: DaemonMode) -> DeckResponse {
    let ts_filename = match resolved
        .source_path
        .as_ref()
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
    {
        Some("page") => "controller.ts",
        _ => "index.controller.ts",
    };
    let ts_path = resolved
        .source_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|d| d.join(ts_filename))
        .filter(|p| p.is_file());

    // emit_dts is held off here on purpose: this path serves JS per
    // HTTP request and never persists outputs. dts lives next to the
    // source and is the build-step's job (the deck bundler + FFI).
    let opts = CompileOpts {
        route_params: None,
        source_path: resolved.source_path.clone(),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        setup_url: ts_path.as_ref().map(|_| format!("./{ts_filename}")),
        ..CompileOpts::default()
    };
    let out = match compile(&resolved.source, opts) {
        Ok(o) => o,
        Err(err) => return DeckResponse::render_error(err.render(&resolved.source), mode),
    };

    // A route with a paired controller must be served as a bundle (the
    // compiled module imports `./<controller>`). Product emit is
    // `routes/<name>/route.js` under the host's bundle root (`<root>/dist`
    // or a registered route-output). Deck used to bundle in-process via
    // Rolldown to `.deck/views/<name>/view.js` (#167); that runtime read
    // was purged (deck-purge-root). Until a host-supplied bundle is on
    // disk, controller routes fall back to this honest error. The
    // in-process bundler (when wired via `Extension<Bundler>`) can still
    // produce a bundle; the `.deck/views/` read is what was purged.
    let final_js = match (&ts_path, resolved.source_path.as_deref()) {
        (Some(ts), Some(_sail)) => {
            return DeckResponse::render_error(
                format!(
                    "route `{name}` has a paired controller ({}) but no compiled \
                     bundle is available and no in-process bundler produced one",
                    ts.display()
                ),
                mode,
            );
        }
        _ => out.js,
    };

    DeckResponse::js_route(final_js, mode)
}

// The route-name validator is a single source of truth in deck-host so the
// render/payload path (here) and the SSE/mutation handlers share one rule. This
// path used to carry its own (lenient) copy while the re-exported deck-host one
// was stricter — a route could render + be handed a subscribe/action URL the
// SSE/mutation path then rejected. Delegate down to the one canonical rule.
pub(crate) use deck_host::render_helpers::is_valid_route_name;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_name_accepts_digits_upper_under_dash() {
        assert!(is_valid_route_name("v2"));
        assert!(is_valid_route_name("MyView"));
        assert!(is_valid_route_name("my_route"));
        assert!(is_valid_route_name("my-view"));
        assert!(is_valid_route_name("v2-dashboard"));
        assert!(is_valid_route_name("agents"));
    }

    #[test]
    fn route_name_rejects_traversal_and_dotted_prefixes() {
        assert!(!is_valid_route_name("../etc"));
        assert!(!is_valid_route_name(".."));
        assert!(!is_valid_route_name(".hidden"));
        assert!(!is_valid_route_name("-leading-dash"));
    }

    #[test]
    fn route_name_rejects_empty_and_oversize() {
        assert!(!is_valid_route_name(""));
        let too_long: String = "a".repeat(65);
        assert!(!is_valid_route_name(&too_long));
        let max_ok: String = "a".repeat(64);
        assert!(is_valid_route_name(&max_ok));
    }

    #[test]
    fn route_name_rejects_invalid_chars() {
        assert!(!is_valid_route_name("with space"));
        assert!(!is_valid_route_name("with.dot"));
        assert!(!is_valid_route_name("with$dollar"));
    }

    #[test]
    fn route_name_segment_limits() {
        assert!(is_valid_route_name("a/b/c"));
        let too_deep = "a/b/c/d/e/f/g/h/i";
        assert!(!is_valid_route_name(too_deep));
        assert!(!is_valid_route_name("foo//bar"));
        assert!(!is_valid_route_name("foo/../bar"));
    }
}
