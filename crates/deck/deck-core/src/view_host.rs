use crate::{DaemonMode, DeckResponse};
use crate::form_inject::inject_form_actions;
use crate::resolver::resolve;

const VIEW_HOST_SHELL: &str = include_str!("../assets/view-host.html");
const SSR_PLACEHOLDER: &str = "<!-- SUI_SSR_CONTENT -->";
const VIEW_NAME_PLACEHOLDER: &str = "<!-- SUI_VIEW_NAME -->";

fn view_name_from_path(path: &str) -> Option<String> {
    // The view-host surface is at `/_deck/view/{name}[/{*sub_path}]`.
    // Segment layout: ["_deck", "view", name, ...sub]. Name is at index 2.
    path.trim_start_matches('/')
        .split('/')
        .nth(2)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// First-paint HTML for the view-host shell from the ship-js SSR seam. Returns
/// `None` when no renderer is registered (e.g. a V8-less context), the view has
/// no source path, or the render fails — the shell then ships an empty
/// `#sui-root` and the client mounts the component (CSR). There is no
/// render-eval fallback: ship-js is the one SSR engine.
fn synthetic_first_paint_host(name: &str) -> Option<String> {
    let resolved = resolve(name, None).ok()?;
    let source_path = resolved.source_path.as_deref()?;
    let params = serde_json::to_value(&resolved.params).unwrap_or(serde_json::Value::Null);
    let pathname = format!("/{name}");
    let req = crate::synthetic_ssr::SsrRequest {
        view_name: name,
        sub_path: None,
        source_path,
        pathname: &pathname,
        params,
        query: serde_json::json!({}),
    };
    crate::synthetic_ssr::first_paint(&req)
}

pub fn serve_view_host(mode: DaemonMode, path: &str) -> DeckResponse {
    let view_name = view_name_from_path(path);

    // A named-but-unresolvable view is a human-facing 404: serve the rendered
    // not-found page rather than a 200 shell that boots, fetches the payload,
    // gets a 404, and dumps raw JSON into a div. The bare `/_deck/view/` landing
    // (no name) keeps serving the shell. The programmatic payload endpoint
    // (`/_deck/views/<name>`) is untouched and still answers JSON.
    if let Some(name) = view_name.as_deref()
        && (!crate::view::is_valid_view_name(name) || resolve(name, None).is_err())
    {
        return crate::error_page::view_not_found(mode, name);
    }
    // First-paint HTML comes solely from ship-js. No renderer registered (or a
    // render error) → empty #sui-root and the client mounts the component (CSR);
    // never a degraded partial render.
    let ssr_content = view_name
        .as_deref()
        .map(|name| {
            let raw = synthetic_first_paint_host(name).unwrap_or_default();
            // Host-side injection: augment <form mutation="bN"> with the
            // daemon-relative action URL. No-op on empty (CSR) markup.
            inject_form_actions(&raw, name)
        })
        .unwrap_or_default();
    let body = VIEW_HOST_SHELL
        .replace(
            VIEW_NAME_PLACEHOLDER,
            &sui_compiler::escape::escape_attr_value(view_name.as_deref().unwrap_or_default()),
        )
        .replace(SSR_PLACEHOLDER, &ssr_content);
    // Reflect the daemon mode so the live-reload client self-gates: it only
    // subscribes when content="development".
    let body = crate::mode_html::inject_mode_meta(&body, mode);
    DeckResponse::html_view(body, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_html_with_correct_headers() {
        let resp = serve_view_host(DaemonMode::Development, "/_deck/view/");
        assert_eq!(resp.status, 200);
        assert!(resp.content_type.contains("text/html"));
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("<!doctype html>"));
        assert!(body.contains("import { mount } from '@ship/sui-runtime'"));
        assert!(body.contains("/_deck/views/"));
        let imap = body.find("type=\"importmap\"").expect("importmap present");
        let module = body.find("type=\"module\"").expect("module script present");
        assert!(imap < module, "importmap must come before any module script");
    }

    #[test]
    fn unresolvable_view_serves_html_not_found_page() {
        // A named view the daemon can't resolve renders the styled 404 page
        // (text/html, 404), not a 200 shell that boots only to dump the
        // payload endpoint's raw JSON. Pin the resolver at an empty tempdir.
        let tmp = tempfile::tempdir().unwrap();
        let _root = crate::test_lock::ProjectRootGuard::new(tmp.path());

        let resp = serve_view_host(DaemonMode::Development, "/_deck/view/no-such-view-xyzzy");

        assert_eq!(resp.status, 404);
        assert!(resp.content_type.contains("text/html"));
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("<!doctype html>"), "must be a rendered page; got: {body}");
        assert!(body.contains("no-such-view-xyzzy"), "must name the missing view; got: {body}");
        assert!(body.contains("href=\"/\""), "must link back to the shell root; got: {body}");
        assert!(
            !body.contains("\"view_not_found\""),
            "must NOT leak the raw JSON error shape; got: {body}"
        );
    }

    #[test]
    fn resolvable_view_serves_shell_with_root_name() {
        // A view that resolves on disk serves the 200 shell, with the root
        // advertising the view name (from the shell template, independent of
        // whether SSR ran). The bare `/_deck/view/` landing leaves it empty.
        let tmp = tempfile::tempdir().unwrap();
        let _root = crate::test_lock::ProjectRootGuard::new(tmp.path());
        let view_dir = tmp.path().join("views/some-view");
        std::fs::create_dir_all(&view_dir).unwrap();
        std::fs::write(view_dir.join("page.sui"), "<p>hi</p>").unwrap();

        let resp = serve_view_host(DaemonMode::Development, "/_deck/view/some-view/sub");
        let bare = serve_view_host(DaemonMode::Development, "/_deck/view/");

        assert_eq!(resp.status, 200);
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(
            body.contains("data-sui-root data-view=\"some-view\""),
            "root must advertise the resolved view name; got: {body}"
        );
        assert_eq!(bare.status, 200);
        let body = std::str::from_utf8(&bare.body).unwrap();
        assert!(
            body.contains("data-sui-root data-view=\"\""),
            "bare /_deck/view/ must leave data-view empty; got: {body}"
        );
    }

    #[test]
    fn view_name_extracted_from_various_paths() {
        assert_eq!(view_name_from_path("/_deck/view/counter"), Some("counter".to_string()));
        assert_eq!(view_name_from_path("/_deck/view/counter/sub"), Some("counter".to_string()));
        assert_eq!(view_name_from_path("/_deck/view/"), None);
        assert_eq!(view_name_from_path("/_deck/view"), None);
        assert_eq!(view_name_from_path("/"), None);
    }
}
