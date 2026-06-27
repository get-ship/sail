use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use serde::Serialize;
use sui_compiler::ViewInit;
pub use deck_host::view_payload::ViewPayload;
use deck_host::resolver::read_view_artifact;

use crate::{CachePolicy, DaemonMode, DeckResponse};
use crate::form_inject::inject_form_actions;
use crate::resolver::resolve;
use crate::routes::subscribe_url;

/// Memoizes the per-request parse (`view_init`) keyed on the source `.sui`'s
/// mtime. A hit re-stats the source and only reuses the seed when the mtime is
/// unchanged, so an edited view (advanced mtime) is a MISS and is never
/// stale-pinned. Without this, every page load re-lexes/parses the view on the
/// hot path, so CPU/latency scaled with traffic, not with edits.
struct InitEntry {
    mtime: SystemTime,
    init: ViewInit,
}

static INIT_CACHE: OnceLock<Mutex<HashMap<String, InitEntry>>> = OnceLock::new();

fn init_cache() -> &'static Mutex<HashMap<String, InitEntry>> {
    INIT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn clone_init(s: &ViewInit) -> ViewInit {
    ViewInit { init: s.init.clone(), has_live_data: s.has_live_data, clients: s.clients.clone() }
}

/// Cache hit only when we have a source mtime AND it matches the cached entry.
fn init_cache_get(key: &str, mtime: Option<SystemTime>) -> Option<ViewInit> {
    let mt = mtime?;
    let cache = init_cache().lock().unwrap_or_else(|e| e.into_inner());
    let entry = cache.get(key)?;
    (entry.mtime == mt).then(|| clone_init(&entry.init))
}

fn init_cache_put(key: &str, mtime: SystemTime, init: &ViewInit) {
    let mut cache = init_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.insert(key.to_string(), InitEntry { mtime, init: clone_init(init) });
}

pub fn serve_view_payload(mode: DaemonMode, name: &str) -> DeckResponse {
    serve(name, None, mode)
}

pub fn serve_view_payload_subpath(mode: DaemonMode, name: &str, sub_path: &str) -> DeckResponse {
    serve(name, Some(sub_path), mode)
}

fn serve(name: &str, sub_path: Option<&str>, mode: DaemonMode) -> DeckResponse {
    if !crate::view::is_valid_view_name(name) {
        return json_error(404, PayloadErrorBody::InvalidName { name: name.to_string() });
    }
    let resolved = match resolve(name, sub_path) {
        Ok(r) => r,
        Err(_) => {
            return json_error(404, PayloadErrorBody::ViewNotFound { name: name.to_string() });
        }
    };
    // For a nested route (e.g. /studio/agents) the live-data SSE channel lives
    // at /_deck/views/<name>/<sub>/sse. Fold the sub into the routed name so the
    // host emits the right __subscribe__ URL and the cache keys per (name, sub).
    let view_name = match sub_path {
        Some(s) => format!("{name}/{s}"),
        None => name.to_string(),
    };
    // Hot path: the parse is mtime-cached on the source (see INIT_CACHE). A hit
    // skips the parse; a source whose mtime advanced is a MISS, so an edited
    // view is never stale-pinned.
    let source_mtime = resolved
        .source_path
        .as_deref()
        .and_then(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    let vi = if let Some(hit) = init_cache_get(&view_name, source_mtime) {
        hit
    } else {
        let vi = match sui_compiler::view_init(&resolved.source) {
            Ok(vi) => vi,
            Err(err) => return compile_error_response(name, &resolved.source, err, mode),
        };
        if let Some(mt) = source_mtime {
            init_cache_put(&view_name, mt, &vi);
        }
        vi
    };
    let css = collect_view_css(name, resolved.source_path.as_deref());
    let module_url = crate::routes::view_module_url(name, sub_path);
    // First-paint HTML comes solely from the JS SSR engine (ship-js). When no
    // renderer is registered (e.g. a V8-less context), there is no server HTML
    // and the client mounts the component itself (CSR) — never a degraded
    // partial render.
    let raw_html = synthetic_first_paint(&view_name, sub_path, &resolved).unwrap_or_default();
    // Host-side injection: bake the daemon-relative mutation action URL into
    // any <form mutation="bN"> that doesn't already carry an explicit action=.
    let html = inject_form_actions(&raw_html, &view_name);
    // Host-side live-data: inject __subscribe__ from the host URL builder so
    // the compiler never has to bake a served URL. has_live_data is the signal.
    let scope = if vi.has_live_data {
        let mut obj = vi.init;
        if let Some(map) = obj.as_object_mut() {
            map.entry("__subscribe__")
                .or_insert_with(|| serde_json::Value::String(subscribe_url(&view_name)));
        }
        obj
    } else {
        vi.init
    };
    let payload = ViewPayload {
        name: name.to_string(),
        sub: sub_path.map(str::to_string),
        html,
        css,
        scope,
        module_url,
        params: resolved.params,
        clients: vi.clients,
    };
    json_ok(&payload, mode)
}

/// Bridge to the ship-js SSR seam. Returns first-paint HTML only when a
/// renderer is registered, the view has a source path, and the render
/// succeeds — otherwise `None`, and the client mounts the component (CSR).
/// `view_name` is the (sub-folded) routed name; `resolved` carries the source
/// path + captured route params.
fn synthetic_first_paint(
    view_name: &str,
    sub_path: Option<&str>,
    resolved: &deck_host::resolver::ResolvedView,
) -> Option<String> {
    let source_path = resolved.source_path.as_deref()?;
    let params = serde_json::to_value(&resolved.params).unwrap_or(serde_json::Value::Null);
    let pathname = format!("/{view_name}");
    let req = crate::synthetic_ssr::SsrRequest {
        view_name,
        sub_path,
        source_path,
        pathname: &pathname,
        params,
        query: serde_json::json!({}),
    };
    crate::synthetic_ssr::first_paint(&req)
}

fn collect_view_css(name: &str, source_path: Option<&std::path::Path>) -> String {
    let Some(p) = source_path else { return String::new() };
    let mut buf = String::new();
    if let Some(top) = top_level_view_dir(p, name) {
        push_css_from(&mut buf, &top);
        if let Some(parent) = p.parent()
            && parent != top
        {
            push_css_from(&mut buf, parent);
        }
    }
    buf
}

fn push_css_from(buf: &mut String, dir: &std::path::Path) {
    if let Some(s) = read_view_artifact(&dir.join("style.css"))
        .or_else(|| read_view_artifact(&dir.join("index.css")))
        .or_else(|| read_view_artifact(&dir.join("view.css")))
    {
        if !buf.is_empty() && !buf.ends_with('\n') {
            buf.push('\n');
        }
        buf.push_str(&s);
    }
}

fn top_level_view_dir(p: &std::path::Path, view_name: &str) -> Option<std::path::PathBuf> {
    let mut cur = p.parent()?;
    loop {
        if cur.file_name().and_then(|s| s.to_str()) == Some(view_name) {
            return Some(cur.to_path_buf());
        }
        cur = cur.parent()?;
    }
}

fn compile_error_response(
    name: &str,
    source: &str,
    err: sui_compiler::diagnostics::CompileError,
    mode: DaemonMode,
) -> DeckResponse {
    if mode == DaemonMode::Production {
        return json_error(500, PayloadErrorBody::RedactedError {});
    }
    let (line, col) = position_from(&err, source);
    let message = err.render(source);
    json_error(
        500,
        PayloadErrorBody::CompileError {
            name: name.to_string(),
            line,
            col,
            message,
        },
    )
}

fn position_from(err: &sui_compiler::diagnostics::CompileError, source: &str) -> (usize, usize) {
    let Some(span) = err.span else { return (0, 0) };
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in source.char_indices() {
        if i >= span.start {
            return (line, col);
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn json_ok<T: Serialize>(body: &T, mode: DaemonMode) -> DeckResponse {
    let body_str = serde_json::to_string(body).unwrap_or_else(|_| "{}".into());
    DeckResponse::json_view(body_str, mode)
}

#[derive(Debug, Serialize)]
struct PayloadError {
    error: PayloadErrorBody,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum PayloadErrorBody {
    ViewNotFound { name: String },
    CompileError { name: String, line: usize, col: usize, message: String },
    InvalidName { name: String },
    RedactedError {},
}

fn json_error(status: u16, body: PayloadErrorBody) -> DeckResponse {
    let body = serde_json::to_string(&PayloadError { error: body })
        .unwrap_or_else(|_| "{\"error\":{\"code\":\"serialize_failed\"}}".into());
    // Errors are never cached and carry no ETag — CachePolicy::None handles
    // both regardless of mode.
    DeckResponse::new(status, "application/json; charset=utf-8", CachePolicy::None, DaemonMode::Development, body.into_bytes())
}
