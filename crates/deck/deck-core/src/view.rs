use sail_compiler::{CompileOpts, compile};
use deck_host::resolver::ResolvedView;

use crate::{DaemonMode, DeckResponse};
use crate::assets::resolve_view_js;
use crate::resolver::{resolve, Resolution};

#[derive(Debug, serde::Deserialize)]
pub struct ViewSetupQuery {
    pub sub: Option<String>,
}

/// Serve `/_deck/views/{name}/view.js` — the built bundle when present, else an
/// on-demand compile. Mount-scoped: the per-request [`Resolution`] carries the
/// project root the built bundle + source `.sui` are resolved under, so one
/// process serving many apps reads each app's views from ITS own root (no
/// process global).
pub fn serve_sui_view_setup(
    mode: DaemonMode,
    resolution: &Resolution,
    name: &str,
    sub: Option<&str>,
) -> DeckResponse {
    if !is_valid_view_name(name) {
        return DeckResponse::not_found("invalid view name", mode);
    }
    if sub.is_none() {
        if let Some(js) = resolve_view_js(Some(resolution), name) {
            return DeckResponse::js_view(js, mode);
        }
    }
    let Some(root) = resolution.project_root() else {
        return DeckResponse::not_found("view: no project root (host must carry a Resolution)", mode);
    };
    match resolve(root, name, sub) {
        Ok(r) => compile_view_js(name, &r, mode),
        Err(e) => DeckResponse::not_found(e.message, mode),
    }
}

fn compile_view_js(name: &str, resolved: &ResolvedView, mode: DaemonMode) -> DeckResponse {
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
        setup_url: ts_path.as_ref().map(|_| format!("./{ts_filename}")),
        ..CompileOpts::default()
    };
    let out = match compile(&resolved.source, opts) {
        Ok(o) => o,
        Err(err) => return DeckResponse::render_error(err.render(&resolved.source), mode),
    };

    // A view with a paired controller must be served as a bundle (the
    // compiled module imports `./<controller>`). Deck used to bundle
    // that in-process via Rolldown to `.deck/views/<name>/view.js`
    // (#167); the runtime read of that path was purged (deck-purge-root).
    // Until view chunk serving has a host-supplied path, controller
    // views fall back to this honest error. The in-process bundler
    // path (when wired via `Extension<Bundler>`) can produce a bundle
    // and serve it; the runtime read step is what was purged.
    let final_js = match (&ts_path, resolved.source_path.as_deref()) {
        (Some(ts), Some(_sui)) => {
            return DeckResponse::render_error(
                format!(
                    "view `{name}` has a paired controller ({}) but no compiled \
                     bundle is available and no in-process bundler produced one",
                    ts.display()
                ),
                mode,
            );
        }
        _ => out.js,
    };

    DeckResponse::js_view(final_js, mode).with_header(
        "X-Sui-View",
        name.chars().filter(|c| c.is_ascii()).collect::<String>(),
    )
}

// The view-name validator is a single source of truth in deck-host so the
// render/payload path (here) and the SSE/mutation handlers share one rule. This
// path used to carry its own (lenient) copy while the re-exported deck-host one
// was stricter — a view could render + be handed a subscribe/action URL the
// SSE/mutation path then rejected. Delegate down to the one canonical rule.
pub(crate) use deck_host::render_helpers::is_valid_view_name;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_name_accepts_digits_upper_under_dash() {
        assert!(is_valid_view_name("v2"));
        assert!(is_valid_view_name("MyView"));
        assert!(is_valid_view_name("my_view"));
        assert!(is_valid_view_name("my-view"));
        assert!(is_valid_view_name("v2-dashboard"));
        assert!(is_valid_view_name("agents"));
    }

    #[test]
    fn view_name_rejects_traversal_and_dotted_prefixes() {
        assert!(!is_valid_view_name("../etc"));
        assert!(!is_valid_view_name(".."));
        assert!(!is_valid_view_name(".hidden"));
        assert!(!is_valid_view_name("-leading-dash"));
    }

    #[test]
    fn view_name_rejects_empty_and_oversize() {
        assert!(!is_valid_view_name(""));
        let too_long: String = "a".repeat(65);
        assert!(!is_valid_view_name(&too_long));
        let max_ok: String = "a".repeat(64);
        assert!(is_valid_view_name(&max_ok));
    }

    #[test]
    fn view_name_rejects_invalid_chars() {
        assert!(!is_valid_view_name("with space"));
        assert!(!is_valid_view_name("with.dot"));
        assert!(!is_valid_view_name("with$dollar"));
    }

    #[test]
    fn view_name_segment_limits() {
        assert!(is_valid_view_name("a/b/c"));
        let too_deep = "a/b/c/d/e/f/g/h/i";
        assert!(!is_valid_view_name(too_deep));
        assert!(!is_valid_view_name("foo//bar"));
        assert!(!is_valid_view_name("foo/../bar"));
    }
}
