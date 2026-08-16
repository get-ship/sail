use crate::{CachePolicy, DaemonMode, DeckResponse};
use deck_config::AppServeConfig;
use deck_config::RenderMode;

const ASSETS_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// deck's OWN baked product (sail-runtime/dom/sdk-client + primitives umbrella).
/// These ship inside the deck-core crate, like Next shipping its own
/// runtime. App + user content is NOT here — see `read_source_asset`.
///
/// Used for committed assets (live-reload.js) that remain in the crate's
/// `assets/` directory. Framework singletons come from the `include_str!`
/// constants below — the `.deck/framework/` dev-override was purged (see
/// deck-purge-root); the framework no longer reads a monorepo-root
/// `.deck/framework/`. Icon glyphs are never embedded: Deck materializes
/// only used roles into the app out graph.
fn read_asset(rel: &str) -> Option<String> {
    let path = std::path::Path::new(ASSETS_DIR).join("assets").join(rel);
    std::fs::read_to_string(path).ok()
}

/// App + user build output, read from the host-supplied output tree. deck
/// bakes none of this — route + chrome bundles are the project's content,
/// served from the path the host passes in via `scaffold.source` (per-app)
/// or the asset graph (workspace-level). The retired `<root>/.deck/<rel>`
/// read was the framework's hardcoded `.deck/` reach — PURGED
/// (deck-purge-root). The framework no longer reads a monorepo-root
/// `.deck/`; the host owns the output location.
// (formerly `read_deck_asset(rel)` — `<root>/.deck/<rel>`) removed.

/// Project SOURCE / built output, read from an EXPLICIT project `root` (the
/// caller resolves it from the per-request [`Resolution`] — never a process
/// global). When a scaffold has no built artifact under `<root>/<source>/`,
/// this serves the authored source at the same path — the built-or-source
/// pattern route.rs uses.
/// Binary-safe: assets include bundler-emitted woff2, which is not UTF-8 —
/// a text read would 404 every font.
fn read_source_asset(root: &std::path::Path, rel: &str) -> Option<Vec<u8>> {
    std::fs::read(root.join(rel)).ok()
}

const SUI_RUNTIME_JS: &str = include_str!("../assets/sail-runtime.js");
const SUI_DOM_JS: &str = include_str!("../assets/sail-dom.js");
const SUI_SDK_CLIENT_JS: &str = include_str!("../assets/sail-sdk-client.js");
// `deck/route` is the bundler-generated entry's `@ship/deck/router`
// import. The runtime exports `defineRoutes`, `route`, `navigate`,
// `setEditMode` — every app's entry needs this singleton, so the
// framework ships the bundle as an `include_str!` constant.
const DECK_ROUTE_JS: &str = include_str!("../assets/deck-route.js");
// In-browser WASM compiler + browser LSP singletons. The importmap binds
// `@ship/sail-compiler-wasm` → `/_deck/compiler-wasm.js` and the
// `@ship/sail-lsp-browser/*` subpaths → `/_deck/lsp-*.js`; dev serves them
// from the same baked constants as the runtime. The `.wasm` binary itself is
// served by the host app from `public/sail_compiler_bg.wasm`.
const SAIL_COMPILER_WASM_JS: &str = include_str!("../assets/sail-compiler-wasm.js");
const SAIL_LSP_COMPLETIONS_JS: &str = include_str!("../assets/sail-lsp-completions.js");
const SAIL_LSP_DIAGNOSTICS_JS: &str = include_str!("../assets/sail-lsp-diagnostics.js");
const LIVE_RELOAD_JS: &str = include_str!("../assets/live-reload.js");
const ERROR_OVERLAY_JS: &str = include_str!("../assets/error-overlay.js");
const SAIL_STUDIO_BRIDGE_JS: &str = include_str!("../assets/sail-studio-bridge.js");

/// Public helper (used by route.rs). The in-process bundler writes route bundles
/// to `<root>/.sail/routes/<name>/route.js` (the per-route generate dir the bundler
/// owns). This reads that bundle when it exists so a controller route is served
/// from its compiled, inlined-controller form — the on-demand build path in
/// `route.rs::compile_route_js` only kicks in when no bundle is on disk yet.
///
/// Mount-scoped: the roots come from the per-request [`Resolution`] (per-app
/// route outputs first, then `<project_root>/dist`), never a process global —
/// so one process serving many apps reads each app's bundles from ITS own root.
pub fn resolve_route_js(
    resolution: Option<&crate::resolver::Resolution>,
    name: &str,
) -> Option<String> {
    let resolution = resolution?;
    for route_root in resolution.route_bundle_roots() {
        if let Some(js) = read_route_bundle(&route_root, name) {
            return Some(js);
        }
    }
    None
}

fn read_route_bundle(root: &std::path::Path, name: &str) -> Option<String> {
    let bundle = root.join(format!("routes/{name}/route.js"));
    std::fs::read(&bundle)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
}

//
// A `AppServeConfig` (from `apps.jsonc` or built-in defaults) describes one
// HTML shell: which built artifact to load (`source`), and which
// `window.<key>=<value>` globals to inject (`inject_globals`). The app's
// `name` is its URL namespace — assets are written by the bundler with
// the final served path. This single function handles every scaffold
// route (shell `GET /`, space `GET /app/{id}`, etc.).
//
// `params` are route-param substitutions — e.g. `&[("id", "my-space")]` lets
// the config value `"$id"` resolve to the actual route param. Any `$key` token
// in an `inject_globals` value is replaced with the matching param.
/// Serve an HTML scaffold defined by a `AppServeConfig`.
///
/// - Reads the built index.html from `<deck_out>/<source>/index.html`.
/// - Injects one `<script>window.<key>=<value>;</script>` per
///   `inject_globals` entry, substituting `$<param>` tokens from `params`.
/// - Injects the daemon-mode meta tag via `inject_mode_meta`.
///
/// Reads the built `<source>/index.html` (host-owned; default deploy is `dist/`); in dev (no build artifact yet)
/// falls back to the authored source `<source>/index.html`. Returns a styled
/// 404 only when neither exists.
/// First-paint HTML for the `routes/` page matching `request_path`, via the
/// ship-js SSR seam. Mirrors the named-route path (`route_payload`) but with the
/// file-routed layout: split the URL into name + sub-path, resolve the matching
/// `routes/<…>/page.sail` (+ captured `[param]` params), and render through the
/// CARRIED `renderer`. Returns `None` (→ empty `#root`, client mounts CSR) when
/// there's no project root, no matching page, no renderer carried, or the render
/// fails (the failure is logged loud inside `synthetic_ssr`).
fn routes_first_paint(
    request_path: &str,
    resolution: Option<&crate::resolver::Resolution>,
    renderer: Option<&dyn crate::synthetic_ssr::SyntheticSsr>,
) -> Option<String> {
    // No renderer carried for this mount ⇒ CSR (never a process-global engine).
    let renderer = renderer?;
    // Mount-scoped root: resolve against the per-request Resolution — one process
    // serving many apps never first-paints app B's route against app A's root
    // (`inv-deck-mount-scoped-resolution`).
    let project_root = crate::resolver::resolve_project_root_with(resolution)?;
    // `/greet/zelda` → name=greet, sub=zelda; `/counter` → name=counter, sub=None;
    // `/` → name="" (the index `routes/page.sail`).
    let trimmed = request_path.trim_start_matches('/');
    let (name, sub) = match trimmed.split_once('/') {
        Some((n, rest)) if !rest.is_empty() => (n, Some(rest)),
        _ => (trimmed, None),
    };
    // The index route is `routes/page.sail` (empty name). `resolve()` keys off a
    // non-empty `<name>` dir, so resolve the index leaf directly.
    if name.is_empty() {
        let source = project_root.join("routes").join("page.sail");
        if !source.is_file() {
            return None;
        }
        let req = crate::synthetic_ssr::SsrRequest {
            route_name: "",
            sub_path: None,
            source_path: &source,
            pathname: request_path,
            params: serde_json::json!({}),
            query: serde_json::json!({}),
        };
        return crate::synthetic_ssr::first_paint(renderer, &req);
    }
    let layout = deck_host::resolver::RouteLayout {
        route_dirs: vec!["routes".to_string()],
        route_leaf: "page.sail".to_string(),
        first_party_root: None,
    };
    let resolved = deck_host::resolver::resolve(&project_root, &layout, name, sub).ok()?;
    let source_path = resolved.source_path.as_deref()?;
    let params = serde_json::to_value(&resolved.params).unwrap_or(serde_json::Value::Null);
    let req = crate::synthetic_ssr::SsrRequest {
        route_name: name,
        sub_path: sub,
        source_path,
        pathname: request_path,
        params,
        query: serde_json::json!({}),
    };
    crate::synthetic_ssr::first_paint(renderer, &req)
}

pub fn serve_scaffold(
    mode: DaemonMode,
    scaffold: &AppServeConfig,
    params: &[(&str, &str)],
    // The request pathname, when the caller is a route handler (serve_index /
    // spa_* / spa_fallback). Drives `render: ssr` first-paint resolution. `None`
    // (e.g. the multi-app host path, tests) ⇒ no SSR resolution → CSR shell.
    request_path: Option<&str>,
    // The per-request mount-scoped resolution context. Threaded so asset + SSR
    // resolution runs against THIS app's declared root — never a process global.
    // `None` ⇒ no root ⇒ an honest 404.
    resolution: Option<&crate::resolver::Resolution>,
    // The per-mount SSR renderer (`render: ssr` only). `None` ⇒ CSR shell. A
    // per-request handle, never a process-global engine install.
    renderer: Option<&dyn crate::synthetic_ssr::SyntheticSsr>,
) -> DeckResponse {
    // Mount-scoped root: THIS request's project (the host's per-request
    // Resolution). No Resolution ⇒ an honest 404, never a shared global.
    let Some(root) = crate::resolver::resolve_project_root_with(resolution) else {
        return DeckResponse::not_found(
            format!(
                "scaffold '{}': no project root (host must carry a Resolution)",
                scaffold.name
            ),
            mode,
        );
    };
    // index.html IS text — decode the binary-safe read (a scaffold's html
    // that isn't UTF-8 is honestly broken, not a font).
    let Some(html) = read_source_asset(&root, &format!("{}/index.html", scaffold.source))
        .and_then(|b| String::from_utf8(b).ok())
    else {
        return DeckResponse::not_found(
            format!(
                "scaffold '{}' not built (missing {}/index.html)",
                scaffold.name, scaffold.source
            ),
            mode,
        );
    };

    // Build the globals injection block.
    let inject = if scaffold.inject_globals.is_empty() {
        String::new()
    } else {
        let mut parts = String::from("<script>");
        for (key, raw_value) in &scaffold.inject_globals {
            // Validate that the key is a safe JS identifier before embedding
            // it verbatim in the emitted script. An unchecked key like
            // `x;alert(1)//` would inject arbitrary script on every page load.
            if !is_valid_js_identifier(key) {
                tracing::warn!(
                    scaffold = %scaffold.name,
                    key = %key,
                    "inject_globals key is not a valid JS identifier — skipping"
                );
                continue;
            }
            // Substitute $param tokens.
            let resolved = substitute_params(raw_value, params);
            // JSON-encode the value so it is a safe JS string literal.
            let js_value = serde_json::to_string(&resolved).unwrap_or_else(|_| "\"\"".to_string());
            parts.push_str(&format!("window.{key}={js_value};"));
        }
        parts.push_str("</script>");
        parts
    };

    // Inject globals before `</head>` so the JS variables are available
    // as early as possible. Fall back to `<head>` insertion when
    // `</head>` is absent (malformed scaffold). The asset hrefs in
    // the HTML are written by the bundler with the final served
    // path — no rewrite here.
    let html = if !inject.is_empty() {
        if html.contains("</head>") {
            html.replace("</head>", &format!("{inject}</head>"))
        } else {
            html.replacen("<head>", &format!("<head>{inject}"), 1)
        }
    } else {
        html
    };

    let html = crate::mode_html::inject_mode_meta(&html, mode);
    // Surface the app's render mode (csr|ssr) so it's legible client-side
    // rather than implicit (empty `#root` = csr was the old tell).
    let html = crate::mode_html::inject_render_meta(&html, scaffold.render);
    // Multi-app mount (e.g. Studio at `/_studio`): CSR router strips this base.
    let html = crate::mode_html::inject_app_route_meta(&html, &scaffold.route);
    // Portable design studio: stamp active project + themes dir (workspace-rel).
    // Overlay Studio (`ops_project`) targets the *primary* app, not apps/studio.
    let ops_project_root = scaffold
        .ops_project
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .and_then(|rel| {
            root.parent() // apps/
                .and_then(|p| p.parent()) // workspace
                .map(|ws| ws.join(rel))
                .filter(|p| p.is_dir())
        });
    let meta_root = ops_project_root.as_deref().unwrap_or(&root);
    let html = crate::mode_html::inject_project_themes_meta(&html, meta_root);
    // Product theme picker: effective emit set (project ∪ theme.enabled).
    let html = crate::mode_html::inject_effective_themes_meta(&html, meta_root);
    // Boot defaults (theme.default / defaultMode) for pre-paint + runtime fallback.
    let html = crate::mode_html::inject_theme_defaults_meta(&html, meta_root);
    // SSR (`render: ssr`): render the matched `routes/` page server-side and
    // inject its first-paint HTML into the (otherwise empty) mount root, so the
    // client HYDRATES it instead of building from scratch. The mount-root id is
    // the bundler's `templates::ROOT_ID` contract (`root`); deck-core can't
    // depend on deck-bundler (cycle), so the id is pinned here. No registered
    // engine / no route match / a render error ⇒ `routes_first_paint` is None
    // and we serve the empty shell (CSR) exactly as before.
    let html = match (scaffold.render, request_path) {
        (RenderMode::Ssr, Some(path)) => match routes_first_paint(path, resolution, renderer) {
            Some(ssr) => html.replacen(
                "<div id=\"root\"></div>",
                &format!("<div id=\"root\">{ssr}</div>"),
                1,
            ),
            None => html,
        },
        _ => html,
    };
    // Dev-only: attach the live-reload client. deck dev rebuilds the SPA on an
    // edit and emits `module-changed` (target `app`); this client re-imports
    // the hashed entry → `__deckHmr.swapModules` (app-granular remount: keep
    // signal values, swap CSS). No opts still remounts when `__deckHmr` is
    // present — it is not a full-document-only client. Served at
    // `/_deck/live-reload.js`; it self-
    // gates on the `deck-mode=development` meta injected above, so this is inert
    // in production even if the tag is present.
    let html = if matches!(mode, DaemonMode::Development) {
        let tag = "<script type=\"module\">import { attachLiveReload } from '/_deck/live-reload.js'; attachLiveReload()</script>";
        if html.contains("</body>") {
            html.replace("</body>", &format!("  {tag}\n</body>"))
        } else {
            format!("{html}{tag}")
        }
    } else {
        html
    };
    // Dev-only: attach the runtime error overlay. A `sail:error` event
    // (dispatched by packages/sail/runtime when an effect or setup throw
    // reaches no error boundary) previously had no visible surface —
    // console.error only, invisible unless devtools was already open. This
    // tag is never written in Production, so the overlay ships zero bytes
    // there; the client script also self-gates on the `deck-mode` meta
    // above, matching the live-reload tag's defense-in-depth pattern.
    let html = if matches!(mode, DaemonMode::Development) {
        let tag = "<script type=\"module\">import { attachErrorOverlay } from '/_deck/error-overlay.js'; attachErrorOverlay()</script>";
        if html.contains("</body>") {
            html.replace("</body>", &format!("  {tag}\n</body>"))
        } else {
            format!("{html}{tag}")
        }
    } else {
        html
    };
    // Live CSP honors package `network` (EXT-1 E6). Resolve via the SAME
    // walk-up the compiler lint uses (`Policy::network_for`) so connect-src
    // and the static allowlist never drift. Empty list (no config / no
    // grant) ⇒ byte-identical to the historical `'self'` headers.
    // Synthetic path under the project root: only the walk-up parent chain
    // matters; the leaf need not exist on disk.
    let network = sail_compiler::check::policy::Policy::network_for(&root.join("page.sail"));
    DeckResponse::html_route_for(html, mode, &network)
}

/// Serve a static asset from `<deck_out>/<source>/assets/<file>`.
///
/// The `file` parameter comes from the axum path extractor and is
/// percent-decoded but NOT stripped of `..` segments. Validate it before
/// constructing the path to prevent directory traversal: a request like
/// `/app/_assets/../../../etc/passwd` would otherwise escape the assets dir.
pub fn serve_scaffold_asset(
    mode: DaemonMode,
    scaffold: &AppServeConfig,
    file: &str,
    resolution: Option<&crate::resolver::Resolution>,
) -> DeckResponse {
    // Reject path-traversal patterns. Only simple filenames (no slashes, no
    // leading dots, no ".." components) are valid asset names. The chunk
    // route's same guard was here formerly (`serve_sail_chunk`); it lives
    // here now since the chunk route is gone.
    if file.contains("..") || file.contains('/') || file.contains('\\') || file.starts_with('.') {
        return DeckResponse::not_found(format!("invalid asset path: {file}"), mode);
    }
    // Mount-scoped root: THIS request's project. No Resolution ⇒ honest 404.
    let Some(root) = crate::resolver::resolve_project_root_with(resolution) else {
        return DeckResponse::not_found(
            format!("scaffold '{}' asset: no project root", scaffold.name),
            mode,
        );
    };
    // Per-app build output: `<project_root>/<source>/assets/<file>` (e.g.
    // `apps/hub/dist/assets/<file>`). The host owns the output location via
    // `scaffold.source`.
    let path = format!("{}/assets/{file}", scaffold.source);
    match read_source_asset(&root, &path) {
        Some(body) if file.ends_with(".css") => DeckResponse::css_route(body, mode),
        Some(body) if file.ends_with(".woff2") => DeckResponse::font_asset(body, mode),
        Some(body) => DeckResponse::js_asset(body, mode),
        None => DeckResponse::not_found(
            format!("scaffold '{}' asset not found: {file}", scaffold.name),
            mode,
        ),
    }
}

/// `GET /<file>` — serve a single-segment static asset from the host's declared
/// public directory (`DeckConfig.public_dir`), resolved under the per-request
/// [`Resolution`]'s project root (never a Ship path). Single-segment +
/// traversal-guarded (the route is `/{file}`, but reject `..`/slashes/leading-dot
/// belt-and-suspenders); content-type by extension; immutable long cache in
/// Production. 404 when no public dir is configured or the file is absent.
pub fn serve_public_asset(
    mode: DaemonMode,
    public_dir: &str,
    file: &str,
    resolution: Option<&crate::resolver::Resolution>,
) -> DeckResponse {
    if file.is_empty()
        || file.contains("..")
        || file.contains('/')
        || file.contains('\\')
        || file.starts_with('.')
    {
        return DeckResponse::not_found(format!("invalid public asset path: {file}"), mode);
    }
    // Mount-scoped root: THIS request's project (never a process global).
    let Some(root) = crate::resolver::resolve_project_root_with(resolution) else {
        return DeckResponse::not_found("no project root for public assets", mode);
    };
    let path = root.join(public_dir).join(file);
    let Ok(body) = std::fs::read(&path) else {
        return DeckResponse::not_found(format!("public asset not found: {file}"), mode);
    };
    DeckResponse::new(
        200,
        public_asset_content_type(file),
        CachePolicy::Asset,
        mode,
        body,
    )
}

/// Content-type for a public asset by file extension; unknown extensions get
/// `application/octet-stream`.
fn public_asset_content_type(file: &str) -> &'static str {
    match file.rsplit('.').next() {
        Some("svg") => "image/svg+xml; charset=utf-8",
        Some("ico") => "image/x-icon",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("css") => "text/css; charset=utf-8",
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// Validate that `key` is a safe JS identifier: `[A-Za-z_][A-Za-z0-9_]*`.
///
/// This is called before using a key verbatim in `window.{key}=...` in the
/// emitted `<script>` block. An unvalidated key like `x;alert(1)//` would
/// inject arbitrary script on every page load.
fn is_valid_js_identifier(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }
    let mut chars = key.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Substitute `$<key>` tokens in `value` with matching entries from `params`.
fn substitute_params(value: &str, params: &[(&str, &str)]) -> String {
    let mut out = value.to_string();
    for (key, replacement) in params {
        out = out.replace(&format!("${key}"), replacement);
    }
    out
}
// Framework singletons: served from the embedded `include_str!` constant
// (baked into the binary at compile time by build.rs staging
// `.sail/framework/` → `assets/`). The runtime `.deck/framework/` read
// was the framework's dev-override — purged (deck-purge-root). The
// bake pipeline (build.rs) stages `.sail/framework/` → `assets/` for
// compile-time inclusion, but the runtime serves nothing from
// the generate tree directly.

pub fn serve_sail_runtime(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SUI_RUNTIME_JS.to_string(), mode)
}

pub fn serve_sail_dom(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SUI_DOM_JS.to_string(), mode)
}

pub fn serve_sail_sdk_client(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SUI_SDK_CLIENT_JS.to_string(), mode)
}

/// `GET /_deck/compiler-wasm.js` — the `@ship/sail-compiler-wasm` singleton
/// (wasm-bindgen glue for the in-browser SAIL compiler). The importmap binds
/// every bare import of this package to this URL; the companion `.wasm` binary
/// is served at `/sail_compiler_bg.wasm` by the host app's `public/` assets.
pub fn serve_sail_compiler_wasm(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SAIL_COMPILER_WASM_JS.to_string(), mode)
}

/// `GET /_deck/lsp-completions.js` — the `@ship/sail-lsp-browser/completions`
/// singleton used by the code editor for scope-aware `.sail` completions.
pub fn serve_sail_lsp_completions(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SAIL_LSP_COMPLETIONS_JS.to_string(), mode)
}

/// `GET /_deck/lsp-diagnostics.js` — the `@ship/sail-lsp-browser/diagnostics`
/// singleton used by the code editor for streaming `.sail` diagnostics.
pub fn serve_sail_lsp_diagnostics(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SAIL_LSP_DIAGNOSTICS_JS.to_string(), mode)
}

/// Parse the `/*! sail-framework-stamp: sha=<hash> */` leading comment
/// `deck-cli build_assets` writes into a baked bundle (deck-deploy-gap). The
/// writer half lives in `deck_bundler::stamp::format_stamp_header` — keep the
/// format in lockstep. `None` for a bake that predates stamping (an older
/// commit's `.sail/framework/*.js`, still staged into `assets/` verbatim) —
/// callers treat that as "can't verify," not "verified fresh."
pub fn parse_baked_stamp(js: &str) -> Option<&str> {
    js.lines()
        .next()?
        .strip_prefix("/*! sail-framework-stamp: sha=")?
        .strip_suffix(" */")
}

/// `(bundle out_file, baked stamp hash)` for every framework singleton this
/// binary bakes. Read at `deck dev` startup and compared against
/// `deck_bundler::stamp::current_framework_stamps` — deck-core can't depend
/// on deck-bundler (it's the other way around), so the comparison itself
/// lives in deck-cli; this just exposes the baked half.
pub fn baked_framework_stamps() -> Vec<(&'static str, Option<&'static str>)> {
    vec![
        ("sail-runtime.js", parse_baked_stamp(SUI_RUNTIME_JS)),
        ("sail-dom.js", parse_baked_stamp(SUI_DOM_JS)),
        ("sail-sdk-client.js", parse_baked_stamp(SUI_SDK_CLIENT_JS)),
        ("deck-route.js", parse_baked_stamp(DECK_ROUTE_JS)),
        (
            "sail-compiler-wasm.js",
            parse_baked_stamp(SAIL_COMPILER_WASM_JS),
        ),
        (
            "sail-lsp-completions.js",
            parse_baked_stamp(SAIL_LSP_COMPLETIONS_JS),
        ),
        (
            "sail-lsp-diagnostics.js",
            parse_baked_stamp(SAIL_LSP_DIAGNOSTICS_JS),
        ),
        (
            "sail-studio-bridge.js",
            parse_baked_stamp(SAIL_STUDIO_BRIDGE_JS),
        ),
    ]
}

/// `GET /_deck/route.js` — the `@ship/deck/router` framework singleton.
/// The bundler-generated entry imports it for `defineRoutes`, `route`,
/// `navigate`, `setEditMode`. The bundle is the package compiled
/// output (the source lives at `packages/deck/router/` and is shipped
/// via `include_str!`).
pub fn serve_deck_route(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(DECK_ROUTE_JS.to_string(), mode)
}
/// `GET /_deck/icons/{role}.svg` — used glyph bytes from the app out graph.
/// Deck does not embed a pack. Missing bake → 404 (runtime slotted fallback).
///
/// `registry.json` lists Sail roles (same registry as MCP / LSP), not a pack.
pub fn serve_sail_icon(mode: DaemonMode, name: &str) -> DeckResponse {
    serve_sail_icon_from(mode, name, None)
}

/// Same as [`serve_sail_icon`], probing `{out}/_deck/icons/{role}.svg` first.
pub fn serve_sail_icon_from(
    mode: DaemonMode,
    name: &str,
    resolution: Option<&crate::resolver::Resolution>,
) -> DeckResponse {
    if name == "registry" || name == "registry.json" {
        return serve_icon_registry(mode);
    }
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return DeckResponse::not_found("invalid icon name", mode);
    }
    if let Some(svg) = read_baked_icon(resolution, name) {
        return DeckResponse::new(
            200,
            "image/svg+xml; charset=utf-8",
            crate::response::CachePolicy::Asset,
            mode,
            svg,
        );
    }
    // Known Sail roles (core ∪ project sail.config) resolve even when the
    // bake missed them — dynamic `name={expr}` and app-local `src/` icons
    // are not always in the used-glyph set. Unknown lucide ids stay 404.
    if let Some(svg) = resolve_role_glyph(resolution, name) {
        return DeckResponse::new(
            200,
            "image/svg+xml; charset=utf-8",
            crate::response::CachePolicy::Asset,
            mode,
            svg,
        );
    }
    DeckResponse::not_found(format!("icon not in app output: {name}"), mode)
}

fn resolve_role_glyph(
    resolution: Option<&crate::resolver::Resolution>,
    name: &str,
) -> Option<Vec<u8>> {
    let root = resolution?.project_root()?;
    let registry = sail_compiler::RoleRegistry::load_from_project(root);
    let binding = registry.resolve(name)?;
    let svg = sail_compiler::load_glyph_svg(&binding.pack, &binding.glyph, &[root])?;
    Some(svg.into_bytes())
}

fn read_baked_icon(
    resolution: Option<&crate::resolver::Resolution>,
    name: &str,
) -> Option<Vec<u8>> {
    let rel = format!("_deck/icons/{name}.svg");
    if let Some(res) = resolution {
        if let Some(root) = res.project_root() {
            if let Some(bytes) = read_source_asset(&deck_config::deck_out_path(root), &rel) {
                return Some(bytes);
            }
        }
        for route_root in res.route_bundle_roots() {
            if let Some(bytes) = read_source_asset(&route_root, &rel) {
                return Some(bytes);
            }
        }
    }
    None
}

/// `GET /_deck/icons/registry.json` — project role ids (Sail SSOT).
pub fn serve_icon_registry(mode: DaemonMode) -> DeckResponse {
    let names = sail_compiler::RoleRegistry::core().role_ids();
    let body = serde_json::to_vec(&names).unwrap_or_else(|_| b"[]".to_vec());
    DeckResponse::new(
        200,
        "application/json; charset=utf-8",
        crate::response::CachePolicy::Asset,
        mode,
        body,
    )
}

/// Framework woff2 files baked from `packages/sail/tokens/src/fonts/`.
/// Keep in lockstep with `FONTS` in `packages/sail/tokens/src/fonts.ts` —
/// the gen-tut-preview-theme rewrite and the drift lock both assume this
/// basename set is the allowlist for `GET /_deck/fonts/{file}`.
const FRAMEWORK_FONTS: &[(&str, &[u8])] = &[
    (
        "dm-sans-latin-wght-normal.woff2",
        include_bytes!(
            "../../../../packages/sail/tokens/src/fonts/dm-sans-latin-wght-normal.woff2"
        ),
    ),
    (
        "syne-latin-wght-normal.woff2",
        include_bytes!("../../../../packages/sail/tokens/src/fonts/syne-latin-wght-normal.woff2"),
    ),
    (
        "jetbrains-mono-latin-wght-normal.woff2",
        include_bytes!(
            "../../../../packages/sail/tokens/src/fonts/jetbrains-mono-latin-wght-normal.woff2"
        ),
    ),
    (
        "press-start-2p-latin-normal.woff2",
        include_bytes!(
            "../../../../packages/sail/tokens/src/fonts/press-start-2p-latin-normal.woff2"
        ),
    ),
];

/// `GET /_deck/fonts/{file}` — one framework woff2 from the token package.
///
/// The bundled path rewrites `url('./fonts/X.woff2')` to hashed assets via
/// `ShipAssetCssPlugin`. Non-bundled consumers (tutorial preview CSS dumb-
/// concatenated into a public sheet) rewrite to this stable route instead,
/// so the app never re-hosts framework font bytes under `public/fonts`.
///
/// Allowlist-only: basename must end in `.woff2`, pass a simple shape check
/// (no `..` / slashes), and match a row baked from the FONTS manifest. Unknown
/// or traversal inputs 404.
pub fn serve_framework_font(mode: DaemonMode, file: &str) -> DeckResponse {
    if !is_safe_framework_font_name(file) {
        return DeckResponse::not_found(format!("invalid framework font path: {file}"), mode);
    }
    match FRAMEWORK_FONTS.iter().find(|(name, _)| *name == file) {
        Some((_, bytes)) => DeckResponse::font_asset(*bytes, mode),
        None => DeckResponse::not_found(format!("framework font not found: {file}"), mode),
    }
}

/// Basename allow shape for `/_deck/fonts/{file}`: single segment, `.woff2`
/// suffix, kebab/alnum characters only. Rejects traversal and non-font names
/// before the baked allowlist is consulted.
fn is_safe_framework_font_name(file: &str) -> bool {
    if file.is_empty()
        || file.len() > 128
        || !file.ends_with(".woff2")
        || file.contains("..")
        || file.contains('/')
        || file.contains('\\')
        || file.starts_with('.')
    {
        return false;
    }
    // Stem: ASCII lowercase / digit / hyphen only (matches FONTS basenames).
    let stem = &file[..file.len() - ".woff2".len()];
    !stem.is_empty()
        && stem
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// `GET /_deck/live-reload.js` — the one HMR client, shared by the route
/// host and the shell. Served in every mode (it self-gates on the
/// `deck-mode` meta tag); only subscribes when the daemon is in dev.
pub fn serve_sail_live_reload(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("live-reload.js").unwrap_or_else(|| LIVE_RELOAD_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

/// `GET /_deck/error-overlay.js` — the dev-mode runtime error overlay
/// client. Served in every mode like `live-reload.js` (it self-gates on the
/// `deck-mode` meta tag); the injecting `<script>` tag in `serve_scaffold`
/// is what actually keeps it out of production pages.
pub fn serve_sail_error_overlay(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("error-overlay.js").unwrap_or_else(|| ERROR_OVERLAY_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

/// `GET /_deck/sail-studio-bridge.js` — Sail Studio bridge under `sail dev`.
/// Baked from `@ship/sail-studio-bridge`. Imported by live-reload; self-gates
/// on deck-mode=development. Hotkeys + FAB + selection ping only (no ops URLs).
pub fn serve_sail_studio_bridge(mode: DaemonMode) -> DeckResponse {
    let js =
        read_asset("sail-studio-bridge.js").unwrap_or_else(|| SAIL_STUDIO_BRIDGE_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

/// `GET /_deck/{file}` — was a shared chunk emitted by the multi-entry route
/// build (#267), served content-addressed from the `.deck/` root. PURGED
/// (deck-purge-root): the framework no longer reads a monorepo-root
/// `.deck/`, so the runtime serves no shared chunks at this path. The
/// route is gone; route bundles that used to import `../../<chunk>.js`
/// now fail to load — route chunk serving is a follow-up.
// (formerly `pub fn serve_sail_chunk(mode, file) -> DeckResponse`) removed.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_js_returns_baked_fallback() {
        let resp = serve_sail_runtime(DaemonMode::Development);
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn parse_baked_stamp_reads_the_leading_comment() {
        let js = "/*! sail-framework-stamp: sha=deadbeef00000000 */\nexport const x = 1;\n";
        assert_eq!(parse_baked_stamp(js), Some("deadbeef00000000"));
    }

    #[test]
    fn parse_baked_stamp_none_when_absent() {
        // A bake that predates stamping (or a non-framework asset) has no
        // header — "can't verify," not a fabricated hash.
        assert_eq!(parse_baked_stamp("export const x = 1;\n"), None);
        assert_eq!(parse_baked_stamp(""), None);
    }

    #[test]
    fn baked_framework_stamps_covers_every_singleton() {
        let stamps = baked_framework_stamps();
        let names: Vec<_> = stamps.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            names,
            vec![
                "sail-runtime.js",
                "sail-dom.js",
                "sail-sdk-client.js",
                "deck-route.js",
                "sail-compiler-wasm.js",
                "sail-lsp-completions.js",
                "sail-lsp-diagnostics.js",
                "sail-studio-bridge.js",
            ]
        );
    }

    #[test]
    fn icon_baked_name_returns_svg() {
        let tmp = tempfile::tempdir().unwrap();
        let icons = deck_config::deck_out_path(tmp.path()).join("_deck/icons");
        std::fs::create_dir_all(&icons).unwrap();
        std::fs::write(
            icons.join("search.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
        )
        .unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(tmp.path());
        let resp = serve_sail_icon_from(DaemonMode::Development, "search", Some(&resolution));
        assert_eq!(resp.status, 200, "baked search");
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.starts_with("<svg"), "{body}");
    }

    #[test]
    fn icon_unbaked_returns_404() {
        let resp = serve_sail_icon(DaemonMode::Development, "search");
        assert_eq!(resp.status, 404, "no embed, no bake");
    }

    #[test]
    fn icon_unbaked_core_role_resolves_from_project_pack() {
        let tmp = tempfile::tempdir().unwrap();
        let pack = tmp.path().join("node_modules/@iconify-json/lucide");
        std::fs::create_dir_all(&pack).unwrap();
        std::fs::write(
            pack.join("icons.json"),
            r#"{"prefix":"lucide","width":24,"height":24,"icons":{"search":{"body":"<circle cx=\"11\" cy=\"11\" r=\"8\"/>"}}}"#,
        )
        .unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(tmp.path());
        let resp = serve_sail_icon_from(DaemonMode::Development, "search", Some(&resolution));
        assert_eq!(resp.status, 200, "core role search from pack");
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("circle"), "{body}");
        let unknown = serve_sail_icon_from(DaemonMode::Development, "files", Some(&resolution));
        assert_eq!(unknown.status, 404, "non-role lucide id stays 404");
    }

    #[test]
    fn icon_registry_json_lists_roles_not_full_pack() {
        let resp = serve_sail_icon(DaemonMode::Development, "registry.json");
        assert_eq!(resp.status, 200);
        assert!(
            resp.content_type.contains("json"),
            "ct={}",
            resp.content_type
        );
        let names: Vec<String> = serde_json::from_slice(&resp.body).expect("json names");
        assert!(names.iter().any(|n| n == "search"), "{names:?}");
        assert!(names.iter().any(|n| n == "settings"), "{names:?}");
        assert!(
            names.len() < 200,
            "roles are a closed set, not a full pack: {}",
            names.len()
        );
        assert!(names.windows(2).all(|w| w[0] <= w[1]), "sorted");
    }

    #[test]
    fn icon_unknown_returns_404() {
        let resp = serve_sail_icon(DaemonMode::Development, "definitely-not-a-real-icon");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn icon_path_traversal_rejected() {
        let resp = serve_sail_icon(DaemonMode::Development, "../etc/passwd");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn icon_empty_name_rejected() {
        let resp = serve_sail_icon(DaemonMode::Development, "");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn framework_font_known_file_returns_woff2() {
        let resp = serve_framework_font(DaemonMode::Development, "dm-sans-latin-wght-normal.woff2");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.content_type, "font/woff2");
        assert_eq!(resp.cache_control, "no-cache");
        // wOFF2 files start with the ASCII tag "wOF2".
        assert!(
            resp.body.starts_with(b"wOF2"),
            "expected woff2 magic, got {} bytes",
            resp.body.len()
        );
        assert!(!resp.body.is_empty());
    }

    #[test]
    fn framework_font_production_is_immutable() {
        let resp = serve_framework_font(DaemonMode::Production, "syne-latin-wght-normal.woff2");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.content_type, "font/woff2");
        assert_eq!(resp.cache_control, "public, immutable, max-age=31536000");
        assert!(resp.etag.is_some(), "prod assets carry an ETag");
    }

    #[test]
    fn framework_font_unknown_returns_404() {
        let resp = serve_framework_font(DaemonMode::Development, "not-a-real-face.woff2");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn framework_font_path_traversal_rejected() {
        for bad in [
            "../etc/passwd",
            "..%2Fetc%2Fpasswd.woff2",
            "/etc/passwd.woff2",
            "fonts/../../../etc/passwd.woff2",
            ".hidden.woff2",
            "nope.ttf",
            "",
        ] {
            let resp = serve_framework_font(DaemonMode::Development, bad);
            assert_eq!(resp.status, 404, "expected 404 for {bad:?}");
        }
    }

    #[test]
    fn framework_font_allowlist_matches_baked_set() {
        // Every baked row must serve; keeps the const table honest if a
        // basename is mistyped relative to the include_bytes! path.
        for (name, bytes) in FRAMEWORK_FONTS {
            let resp = serve_framework_font(DaemonMode::Development, name);
            assert_eq!(resp.status, 200, "{name} must serve");
            assert_eq!(resp.body.as_slice(), *bytes);
        }
        assert_eq!(
            FRAMEWORK_FONTS.len(),
            4,
            "FONTS manifest currently has 4 faces"
        );
    }

    // ── serve_scaffold ────────────────────────────────────────────────────────

    /// Build a minimal AppServeConfig in-memory for tests.
    fn make_scaffold(name: &str, inject_globals: Vec<(&str, &str)>) -> AppServeConfig {
        use std::collections::BTreeMap;
        let mut globals = BTreeMap::new();
        for (k, v) in inject_globals {
            globals.insert(k.to_string(), v.to_string());
        }
        AppServeConfig {
            name: name.to_string(),
            route: "/".to_string(),
            source: "nonexistent-scaffold".to_string(),
            inject_globals: globals,
            render: RenderMode::Csr,
            entry: None,
            ops_project: None,
        }
    }

    #[test]
    fn serve_scaffold_missing_source_returns_404() {
        let scaffold = make_scaffold("test", vec![]);
        // A real root is carried, but the scaffold source doesn't exist → 404.
        let dir = tempfile::tempdir().unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(dir.path());
        let resp = serve_scaffold(
            DaemonMode::Development,
            &scaffold,
            &[],
            None,
            Some(&resolution),
            None,
        );
        assert_eq!(resp.status, 404);
    }

    /// `inv-deck-mount-scoped-resolution` end-to-end on the serve path: TWO
    /// projects in ONE process, each carrying its OWN `Resolution`, serve public
    /// assets from THEIR OWN root — never each other's. This is the isolation
    /// the removed `PROJECT_ROOT_OVERRIDE` global could not give: there is no
    /// shared state, so project B can never first-paint against project A's root.
    #[test]
    fn serve_path_is_mount_scoped_two_projects_isolated() {
        use crate::resolver::Resolution;
        use std::io::Write;
        // Project A: public/logo.svg = "AAA".
        let a = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(a.path().join("public")).unwrap();
        std::fs::File::create(a.path().join("public/logo.svg"))
            .unwrap()
            .write_all(b"AAA")
            .unwrap();
        // Project B: public/logo.svg = "BBB".
        let b = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(b.path().join("public")).unwrap();
        std::fs::File::create(b.path().join("public/logo.svg"))
            .unwrap()
            .write_all(b"BBB")
            .unwrap();

        let res_a = Resolution::new().with_root(a.path());
        let res_b = Resolution::new().with_root(b.path());

        // Same relative request, two mounts — each resolves against its own root.
        let ra = serve_public_asset(DaemonMode::Development, "public", "logo.svg", Some(&res_a));
        let rb = serve_public_asset(DaemonMode::Development, "public", "logo.svg", Some(&res_b));
        assert_eq!(ra.body, b"AAA", "project A serves ITS OWN root, not B's");
        assert_eq!(rb.body, b"BBB", "project B serves ITS OWN root, not A's");
        // And with NO Resolution there is no root to fall back to — honest 404,
        // never a leaked global from another mount.
        let none = serve_public_asset(DaemonMode::Development, "public", "logo.svg", None);
        assert_eq!(
            none.status, 404,
            "no Resolution ⇒ 404, never a shared global"
        );
    }
    #[test]
    fn substitute_params_replaces_dollar_token() {
        assert_eq!(substitute_params("$id", &[("id", "my-space")]), "my-space");
        assert_eq!(
            substitute_params("prefix-$id-suffix", &[("id", "x")]),
            "prefix-x-suffix"
        );
        assert_eq!(substitute_params("no-token", &[("id", "x")]), "no-token");
        // Unknown token stays as-is (no panic, no empty string).
        assert_eq!(substitute_params("$other", &[("id", "x")]), "$other");
    }

    #[test]
    fn substitute_params_multiple_tokens() {
        let result = substitute_params("$a/$b", &[("a", "hello"), ("b", "world")]);
        assert_eq!(result, "hello/world");
    }
    /// serve_scaffold with a real (tmpdir) HTML source rewrites asset hrefs
    /// and injects globals. The project root is carried mount-scoped via a
    /// per-request `Resolution` — no process global.
    #[test]
    fn serve_scaffold_serves_html_and_injects_globals() {
        let dir = tempfile::tempdir().unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(dir.path());

        // Per-app build output: <project_root>/<source>/index.html
        // (post-cutover; the retired `<root>/.deck/<source>/index.html`
        // was purged with the framework's `.deck/` runtime read).
        let scaffold_dir = dir.path().join("my-scaffold");
        std::fs::create_dir_all(&scaffold_dir).unwrap();
        let html = r#"<!doctype html><html><head></head><body><script src="/app/main.js"></script></body></html>"#;
        std::fs::write(scaffold_dir.join("index.html"), html).unwrap();
        let mut globals = std::collections::BTreeMap::new();
        globals.insert("DECK_APP_ID".to_string(), "$id".to_string());
        let scaffold = AppServeConfig {
            name: "space".to_string(),
            route: "/app/{id}".to_string(),
            source: "my-scaffold".to_string(),
            inject_globals: globals,
            render: RenderMode::Csr,
            entry: None,
            ops_project: None,
        };
        let resp = serve_scaffold(
            DaemonMode::Development,
            &scaffold,
            &[("id", "my-space")],
            None,
            Some(&resolution),
            None,
        );

        assert_eq!(resp.status, 200);
        // The asset href is preserved (bundler wrote the final URL).
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(
            body.contains("/app/main.js"),
            "expected /app/main.js in: {body}"
        );
        // The inject_globals block lands before </head>.
        assert!(
            body.contains("window.DECK_APP_ID=\"my-space\""),
            "expected injected global in: {body}"
        );
    }

    /// serve_scaffold surfaces the app's render mode as a `deck-render` meta
    /// tag so CSR-vs-SSR is observable client-side. An `ssr` app emits
    /// `content="ssr"`; the default `csr` app emits `content="csr"`.
    #[test]
    fn serve_scaffold_injects_render_meta() {
        let dir = tempfile::tempdir().unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(dir.path());
        let scaffold_dir = dir.path().join("ssr-scaffold");
        std::fs::create_dir_all(&scaffold_dir).unwrap();
        std::fs::write(
            scaffold_dir.join("index.html"),
            r#"<!doctype html><html><head></head><body></body></html>"#,
        )
        .unwrap();
        let scaffold = AppServeConfig {
            name: "ssr-app".to_string(),
            route: "/".to_string(),
            source: "ssr-scaffold".to_string(),
            inject_globals: std::collections::BTreeMap::new(),
            render: RenderMode::Ssr,
            entry: None,
            ops_project: None,
        };
        let resp = serve_scaffold(
            DaemonMode::Development,
            &scaffold,
            &[],
            None,
            Some(&resolution),
            None,
        );
        assert_eq!(resp.status, 200);
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(
            body.contains("name=\"deck-render\" content=\"ssr\""),
            "render mode must be observable as deck-render meta; got: {body}"
        );
        // The "no ship-* branding" regression is guarded at the SOURCE level by
        // boundary.rs::deck_crates_name_no_ship_app (the forbidden set includes
        // `ship-mode`/`ship-render`/`ship-keepalive`) — a stronger guard than a
        // runtime output check, since it catches the emitter at authoring time.
    }

    /// serve_scaffold injects the dev-mode error-overlay script tag in
    /// Development and NEVER in Production — zero overlay bytes in prod,
    /// pinned at the injection site itself.

    #[test]
    fn serve_scaffold_injects_sail_project_themes_meta() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        // Monorepo-shaped path: .../apps/my-app so sail_project_rel_from_root works.
        let app = dir.path().join("apps").join("my-app");
        let out = app.join("dist");
        std::fs::create_dir_all(&out).unwrap();
        std::fs::File::create(out.join("index.html"))
            .unwrap()
            .write_all(b"<head><title>t</title></head><body><div id=\"root\"></div></body>")
            .unwrap();
        let scaffold = AppServeConfig {
            name: "my-app".into(),
            route: "/".into(),
            source: "dist".into(),
            inject_globals: Default::default(),
            render: RenderMode::Csr,
            entry: None,
            ops_project: None,
        };
        let resolution = crate::resolver::Resolution::new().with_root(&app);
        let resp = serve_scaffold(
            DaemonMode::Development,
            &scaffold,
            &[],
            None,
            Some(&resolution),
            None,
        );
        assert_eq!(resp.status, 200);
        let html = String::from_utf8(resp.body).unwrap();
        assert!(
            html.contains("name=\"sail-project\" content=\"apps/my-app\""),
            "missing sail-project meta: {html}"
        );
        assert!(
            html.contains("name=\"sail-project-themes\" content=\"apps/my-app/themes\""),
            "missing sail-project-themes meta: {html}"
        );
    }

    #[test]
    fn serve_scaffold_overlay_in_dev_no_overlay_in_prod() {
        let dir = tempfile::tempdir().unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(dir.path());
        let scaffold_dir = dir.path().join("overlay-scaffold");
        std::fs::create_dir_all(&scaffold_dir).unwrap();
        std::fs::write(
            scaffold_dir.join("index.html"),
            r#"<!doctype html><html><head></head><body></body></html>"#,
        )
        .unwrap();
        let scaffold = AppServeConfig {
            name: "overlay-app".to_string(),
            route: "/".to_string(),
            source: "overlay-scaffold".to_string(),
            inject_globals: std::collections::BTreeMap::new(),
            render: RenderMode::Csr,
            entry: None,
            ops_project: None,
        };

        let dev = serve_scaffold(
            DaemonMode::Development,
            &scaffold,
            &[],
            None,
            Some(&resolution),
            None,
        );
        let dev_body = std::str::from_utf8(&dev.body).unwrap();
        assert!(
            dev_body.contains("/_deck/error-overlay.js"),
            "dev-mode HTML must include the error-overlay script; got: {dev_body}"
        );

        let prod = serve_scaffold(
            DaemonMode::Production,
            &scaffold,
            &[],
            None,
            Some(&resolution),
            None,
        );
        let prod_body = std::str::from_utf8(&prod.body).unwrap();
        assert!(
            !prod_body.contains("error-overlay"),
            "prod-mode HTML must carry ZERO overlay bytes; got: {prod_body}"
        );
    }

    /// serve_scaffold falls back to the authored SOURCE `<source>/index.html`
    /// when there is no built `<source>/` artifact — the dev/source path
    /// (mirrors route.rs). Gates the fallback so the demo (and any host) never
    /// needs to commit a `dist/` artifact to serve a scaffold.
    #[test]
    fn serve_scaffold_falls_back_to_source_when_unbuilt() {
        let dir = tempfile::tempdir().unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(dir.path());
        // SOURCE location: <root>/my-scaffold/index.html — NOT under .deck/.
        let src_dir = dir.path().join("my-scaffold");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("index.html"),
            r#"<!doctype html><html><head></head><body><div id="sail-root"></div></body></html>"#,
        )
        .unwrap();
        let scaffold = AppServeConfig {
            name: "space".to_string(),
            route: "/app/{id}".to_string(),
            source: "my-scaffold".to_string(),
            inject_globals: std::collections::BTreeMap::new(),
            render: RenderMode::Csr,
            entry: None,
            ops_project: None,
        };
        let resp = serve_scaffold(
            DaemonMode::Development,
            &scaffold,
            &[("id", "x")],
            None,
            Some(&resolution),
            None,
        );
        // Status 200 is the only contract: the source-fallback path
        // served the unbuilt scaffold. The body shape is incidental.
        assert_eq!(
            resp.status, 200,
            "source-fallback must serve the unbuilt scaffold"
        );
    }
    #[test]
    fn scaffold_asset_dotdot_rejected() {
        let scaffold = make_scaffold("test", vec![]);
        let resp = serve_scaffold_asset(
            DaemonMode::Development,
            &scaffold,
            "../../../etc/passwd",
            None,
        );
        assert_eq!(resp.status, 404, "'..' traversal must be rejected");
    }

    #[test]
    fn scaffold_asset_slash_in_name_rejected() {
        let scaffold = make_scaffold("test", vec![]);
        let resp =
            serve_scaffold_asset(DaemonMode::Development, &scaffold, "sub/dir/file.js", None);
        assert_eq!(resp.status, 404, "slash in filename must be rejected");
    }

    #[test]
    fn scaffold_asset_leading_dot_rejected() {
        let scaffold = make_scaffold("test", vec![]);
        let resp = serve_scaffold_asset(DaemonMode::Development, &scaffold, ".hidden", None);
        assert_eq!(resp.status, 404, "leading dot must be rejected");
    }

    // ── is_valid_js_identifier ────────────────────────────────────────────────

    #[test]
    fn valid_js_identifiers_accepted() {
        assert!(is_valid_js_identifier("DECK_APP_ID"));
        assert!(is_valid_js_identifier("myVar"));
        assert!(is_valid_js_identifier("_private"));
        assert!(is_valid_js_identifier("x"));
        assert!(is_valid_js_identifier("abc123"));
    }

    #[test]
    fn invalid_js_identifiers_rejected() {
        assert!(!is_valid_js_identifier(""));
        assert!(!is_valid_js_identifier("0starts_with_digit"));
        assert!(!is_valid_js_identifier("has-hyphen"));
        assert!(!is_valid_js_identifier("has space"));
        assert!(!is_valid_js_identifier("x;alert(1)//"));
        assert!(!is_valid_js_identifier("$dollar"));
        assert!(!is_valid_js_identifier("has.dot"));
    }

    /// A scaffold with an invalid inject_globals key must not emit that key in
    /// the <script> block. The request that used to allow script injection now
    /// skips the bad key and either produces an empty inject block or only
    /// includes valid keys.
    #[test]
    fn inject_globals_invalid_key_skipped_not_injected() {
        let dir = tempfile::tempdir().unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(dir.path());
        // Per-app build output: <project_root>/safe-scaffold/index.html
        // (the retired `<root>/.deck/safe-scaffold/` was purged).
        let safe_scaffold = dir.path().join("safe-scaffold");
        std::fs::create_dir_all(&safe_scaffold).unwrap();
        let html = r#"<!doctype html><html><head></head><body></body></html>"#;
        std::fs::write(safe_scaffold.join("index.html"), html).unwrap();
        let mut globals = std::collections::BTreeMap::new();
        // Invalid key: would inject `alert(1)` if not validated.
        globals.insert("x;alert(1)//".to_string(), "injected".to_string());
        // Valid key alongside it — should still appear.
        globals.insert("SAFE_KEY".to_string(), "safe_value".to_string());
        let scaffold = AppServeConfig {
            name: "test".to_string(),
            route: "/".to_string(),
            source: "safe-scaffold".to_string(),
            inject_globals: globals,
            render: RenderMode::Csr,
            entry: None,
            ops_project: None,
        };
        let resp = serve_scaffold(
            DaemonMode::Development,
            &scaffold,
            &[],
            None,
            Some(&resolution),
            None,
        );

        assert_eq!(resp.status, 200);
        let body = std::str::from_utf8(&resp.body).unwrap();
        // The malicious key must NOT appear in the output.
        assert!(
            !body.contains("alert(1)"),
            "invalid inject_globals key must be stripped; got: {body}"
        );
        // The valid key must still be injected.
        assert!(
            body.contains("window.SAFE_KEY"),
            "valid inject_globals key must be injected; got: {body}"
        );
    }

    #[test]
    fn public_asset_serves_from_dir_and_rejects_traversal() {
        use std::io::Write;
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("public")).unwrap();
        std::fs::File::create(tmp.path().join("public/logo.svg"))
            .unwrap()
            .write_all(b"<svg/>")
            .unwrap();
        let resolution = crate::resolver::Resolution::new().with_root(tmp.path());

        let ok = serve_public_asset(
            DaemonMode::Development,
            "public",
            "logo.svg",
            Some(&resolution),
        );
        assert_eq!(ok.status, 200, "present public asset serves 200");
        assert!(
            ok.content_type.starts_with("image/svg+xml"),
            "svg content-type by extension"
        );
        assert_eq!(ok.body, b"<svg/>", "serves the file bytes");

        assert_eq!(
            serve_public_asset(
                DaemonMode::Development,
                "public",
                "../secret",
                Some(&resolution)
            )
            .status,
            404,
            "path traversal must 404"
        );
        assert_eq!(
            serve_public_asset(
                DaemonMode::Development,
                "public",
                "nope.svg",
                Some(&resolution)
            )
            .status,
            404,
            "missing file must 404"
        );
    }
}
