use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use serde::Serialize;
use sui_compiler::{bindings, parse, render};
pub use deck_host::view_payload::ViewPayload;
use deck_host::resolver::read_view_artifact;

use crate::{CachePolicy, DaemonMode, DeckResponse};
use crate::form_inject::inject_form_actions;
use crate::resolver::resolve;
use crate::routes::subscribe_url;

/// Memoizes the per-request compile (`parse` + `bindings::build` +
/// `render_split`) keyed on the source `.sui`'s mtime. Same pattern as
/// `view_host::SSR_CACHE`: a hit re-stats the source and only reuses the split
/// when the mtime is unchanged, so an edited view (advanced mtime) is a MISS and
/// is never stale-pinned. Without this, every page load re-lexes/parses/renders
/// the view on the hot path, so CPU/latency scaled with traffic, not with edits.
struct SplitEntry {
    mtime: SystemTime,
    split: render::RenderSplit,
    /// Declared `<client>` names, captured at parse time so a cache hit
    /// still serves the payload's `clients` list without re-parsing.
    clients: Vec<String>,
}

static SPLIT_CACHE: OnceLock<Mutex<HashMap<String, SplitEntry>>> = OnceLock::new();

fn split_cache() -> &'static Mutex<HashMap<String, SplitEntry>> {
    SPLIT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn clone_split(s: &render::RenderSplit) -> render::RenderSplit {
    render::RenderSplit { html: s.html.clone(), init: s.init.clone(), has_live_data: s.has_live_data }
}

/// Cache hit only when we have a source mtime AND it matches the cached entry.
fn split_cache_get(
    key: &str,
    mtime: Option<SystemTime>,
) -> Option<(render::RenderSplit, Vec<String>)> {
    let mt = mtime?;
    let cache = split_cache().lock().unwrap_or_else(|e| e.into_inner());
    let entry = cache.get(key)?;
    (entry.mtime == mt).then(|| (clone_split(&entry.split), entry.clients.clone()))
}

fn split_cache_put(key: &str, mtime: SystemTime, split: &render::RenderSplit, clients: &[String]) {
    let mut cache = split_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.insert(
        key.to_string(),
        SplitEntry { mtime, split: clone_split(split), clients: clients.to_vec() },
    );
}

/// Test-only: drop a cached split so a fresh fixture isn't shadowed by a prior
/// test's entry. Production invalidation is automatic via the mtime gate.
#[cfg(test)]
pub(crate) fn evict_split_cache(key: &str) {
    let mut cache = split_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.remove(key);
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
    // at /_deck/views/<name>/<sub>/sse, not /_deck/views/<name>/sse. Fold the sub
    // into the resolved view_name so render_split emits the right __subscribe__
    // URL (mirrors the module_url ?sub= handling below) and the cache keys per
    // (name, sub).
    let view_name = match sub_path {
        Some(s) => format!("{name}/{s}"),
        None => name.to_string(),
    };
    // Hot path: compile is mtime-cached on the source (see SPLIT_CACHE). A hit
    // skips parse/build/render_split entirely; a source whose mtime advanced is
    // a MISS, so an edited view is never stale-pinned.
    let source_mtime = resolved
        .source_path
        .as_deref()
        .and_then(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    let (split, clients) = if let Some(hit) = split_cache_get(&view_name, source_mtime) {
        hit
    } else {
        let program = match parse::parse(&resolved.source) {
            Ok(p) => p,
            Err(err) => return compile_error_response(name, &resolved.source, err, mode),
        };
        let registry = bindings::build(&program);
        let opts = render::RenderOpts { view_name: view_name.clone() };
        let split = match render::render_split(&program, &registry, &opts) {
            Ok(r) => r,
            Err(err) => return compile_error_response(name, &resolved.source, err, mode),
        };
        let clients: Vec<String> =
            program.clients.iter().map(|c| c.name.clone()).collect();
        if let Some(mt) = source_mtime {
            split_cache_put(&view_name, mt, &split, &clients);
        }
        (split, clients)
    };
    let css = collect_view_css(name, resolved.source_path.as_deref());
    let module_url = crate::routes::view_module_url(name, sub_path);
    // First-paint source, in precedence order:
    //   1. ship-js synthetic SSR — the DEFAULT: selected whenever a renderer is
    //      registered and it succeeds. `DECK_SSR=render-eval` opts back out.
    //   2. a precompiled `<stem>.ssr.html` mirror artifact.
    //   3. the archived Rust render-eval split (`render_eval_archived`) — the
    //      fallback when ship-js is unregistered/errored or opted out of. The
    //      `render_split` engine itself is shared with the CSR hydration init
    //      (`split.init` below), so it stays in the compiler; this is its
    //      first-paint use.
    let raw_html = synthetic_first_paint(&view_name, sub_path, &resolved)
        .or_else(|| read_precompiled_ssr(resolved.source_path.as_deref()))
        .unwrap_or(split.html);
    // Host-side injection: bake the daemon-relative mutation action URL into
    // any <form mutation="bN"> that doesn't already carry an explicit action=.
    // Idempotent — safe to call on pre-baked .ssr.html artifacts.
    let html = inject_form_actions(&raw_html, &view_name);
    // Host-side live-data: inject __subscribe__ from the host URL builder so
    // the compiler never has to bake a served URL. has_live_data is the signal.
    let scope = if split.has_live_data {
        let mut obj = split.init;
        if let Some(map) = obj.as_object_mut() {
            map.entry("__subscribe__")
                .or_insert_with(|| serde_json::Value::String(subscribe_url(&view_name)));
        }
        obj
    } else {
        split.init
    };
    let payload = ViewPayload {
        name: name.to_string(),
        sub: sub_path.map(str::to_string),
        html,
        css,
        scope,
        module_url,
        params: resolved.params,
        clients,
    };
    json_ok(&payload, mode)
}

/// Bridge to the gated ship-js SSR seam. Returns first-paint HTML only when
/// the gate is on, a renderer is registered, the view has a source path, and
/// the render succeeds — otherwise `None`, and the caller falls back to
/// render-eval. `view_name` is the (sub-folded) routed name; `resolved`
/// carries the source path + captured route params.
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

fn read_precompiled_ssr(source_path: Option<&std::path::Path>) -> Option<String> {
    let p = source_path?;
    let stem = p.file_stem()?.to_str()?;
    let stem_clean = stem.trim_end_matches(".html");
    let mirror = sui_compiler::out_layout::resolve_out_dir(p);
    let ssr_path = mirror.join(format!("{stem_clean}.ssr.html"));
    std::fs::read_to_string(&ssr_path).ok()
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
