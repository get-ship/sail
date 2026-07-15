use crate::{CachePolicy, DaemonMode, DeckResponse};
use deck_config::AppServeConfig;
use deck_config::RenderMode;

const ASSETS_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// deck's OWN baked product (sui-runtime/dom/sdk-client + primitives umbrella +
/// lucide). These ship inside the deck-core crate, like Next shipping its own
/// runtime. App + user content is NOT here — see `read_source_asset`.
///
/// Used for committed assets (lucide-icons.json, live-reload.js) that remain in
/// the crate's `assets/` directory. Framework singletons come from the
/// `include_str!` constants below — the `.deck/framework/` dev-override was
/// purged (see deck-purge-root); the framework no longer reads a monorepo-root
/// `.deck/framework/`.
fn read_asset(rel: &str) -> Option<String> {
    let path = std::path::Path::new(ASSETS_DIR).join("assets").join(rel);
    std::fs::read_to_string(path).ok()
}

/// App + user build output, read from the host-supplied output tree. deck
/// bakes none of this — view + chrome bundles are the project's content,
/// served from the path the host passes in via `scaffold.source` (per-app)
/// or the asset graph (workspace-level). The legacy `<root>/.deck/<rel>`
/// read was the framework's hardcoded `.deck/` reach — PURGED
/// (deck-purge-root). The framework no longer reads a monorepo-root
/// `.deck/`; the host owns the output location.
// (formerly `read_deck_asset(rel)` — `<root>/.deck/<rel>`) removed.

/// Project SOURCE (not built output), read from the project root directly.
/// The dev/source fallback for scaffolds: when a scaffold has no built
/// artifact under `<project_root>/<source>/`, serve the authored source
/// at the same path — the same built-or-source pattern view.rs uses.
fn read_source_asset(rel: &str) -> Option<String> {
    let root = crate::resolver::resolve_project_root()?;
    std::fs::read_to_string(root.join(rel)).ok()
}

const SUI_RUNTIME_JS: &str = include_str!("../assets/sui-runtime.js");
const SUI_DOM_JS: &str = include_str!("../assets/sui-dom.js");
const SUI_SDK_CLIENT_JS: &str = include_str!("../assets/sui-sdk-client.js");
// `deck/route` is the bundler-generated entry's `@ship/deck/router`
// import. The runtime exports `defineRoutes`, `route`, `navigate`,
// `setEditMode` — every app's entry needs this singleton, so the
// framework ships the bundle as an `include_str!` constant.
const DECK_ROUTE_JS: &str = include_str!("../assets/deck-route.js");
const SUI_LUCIDE_ICONS_JSON: &str = include_str!("../assets/lucide-icons.json");
const LIVE_RELOAD_JS: &str = include_str!("../assets/live-reload.js");
// Aliases the runtime used to resolve client-side. Promoted to the daemon so a
// `<sui-icon name="git">` request finds `git-branch` server-side with no client
// indirection. Keep in lockstep with `packages/components/sui/primitives/src/sui-icon/registry.ts`.
const ICON_ALIASES: &[(&str, &str)] = &[
    ("git", "git-branch"),
    ("refresh", "refresh-cw"),
    ("stop", "stop-circle"),
    ("warning", "triangle-alert"),
];

/// Parse the embedded `lucide-icons.json` into a `name -> inner-svg-fragment`
/// map once at first access. `LazyLock` keeps the deserialized map
/// alongside its initializer — no separate `get_or_init` accessor needed
/// and no missing-init path.
static ICON_MAP: std::sync::LazyLock<std::collections::HashMap<String, String>> =
    std::sync::LazyLock::new(|| {
        serde_json::from_str(SUI_LUCIDE_ICONS_JSON)
            .expect("deck-core lucide-icons.json failed to deserialize at startup")
    });

fn alias_for(name: &str) -> Option<&'static str> {
    ICON_ALIASES.iter().find_map(|(k, v)| (*k == name).then_some(*v))
}
// Public helper (used by view.rs). The in-process bundler writes view bundles
// to `<project_root>/.deck/views/<name>/view.js` (the per-view output dir the
// bundler owns). This helper reads that bundle when it exists so a controller
// view is served from its compiled, inlined-controller form — the on-demand
// build path in `view.rs::compile_view_js` only kicks in when no bundle is on
// disk yet. The host calls `set_project_root(app_dir)` so the path resolves to
// the right per-app workspace (e.g. conformance's `app/` dir, shipd's repo
// root).
pub fn resolve_view_js(name: &str) -> Option<String> {
    // Per-app view outputs first (slice-2 component 1): a host with a
    // non-conventional layout (e.g. `apps/hub/.deck/views/<name>/view.js`)
    // registers its apps via `register_view_output` so the runtime finds
    // the bundle without depending on the workspace-root `.deck/`.
    for (_app, root) in crate::resolver::current_view_outputs() {
        if let Some(js) = read_view_bundle(&root, name) {
            return Some(js);
        }
    }
    // Fallback: the bundler's default workspace-root output.
    for view_root in candidate_view_roots() {
        if let Some(js) = read_view_bundle(&view_root, name) {
            return Some(js);
        }
    }
    None
}

fn read_view_bundle(root: &std::path::Path, name: &str) -> Option<String> {
    let bundle = root.join(format!("views/{name}/view.js"));
    std::fs::read(&bundle).ok().and_then(|b| String::from_utf8(b).ok())
}

/// Iterate view-bundle roots to probe for `<root>/views/<name>/view.js`.
/// Order: per-app view-output dirs first (host-supplied via
/// `AppServeConfig.viewOutput`), then the bundler's default workspace root
/// (`<project_root>/.deck/views`). Per-app wins so apps with a non-conventional
/// layout (e.g. `apps/hub/.deck/views/<name>/view.js`) serve from their own
/// dir without the host having to set a global project root.
fn candidate_view_roots() -> Vec<std::path::PathBuf> {
    use crate::resolver::resolve_project_root;
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Some(pr) = resolve_project_root() {
        // Same single join the bundler writes through (`BundleLayout::deck_out`)
        // — reader and writer resolve `<root>/.deck` from the one constant.
        roots.push(deck_config::deck_out_path(&pr));
    }
    roots
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
/// Reads the built `.deck/<source>/index.html`; in dev (no build artifact yet)
/// falls back to the authored source `<source>/index.html`. Returns a styled
/// 404 only when neither exists.
/// First-paint HTML for the `routes/` page matching `request_path`, via the
/// ship-js SSR seam. Mirrors the named-`view` path (`view_payload`) but with the
/// file-routed layout: split the URL into name + sub-path, resolve the matching
/// `routes/<…>/page.sui` (+ captured `[param]` params), and render through
/// `synthetic_ssr`. Returns `None` (→ empty `#root`, client mounts CSR)
/// when there's no project root, no matching page, no registered renderer, or
/// the render fails (the failure is logged loud inside `synthetic_ssr`).
fn routes_first_paint(
    request_path: &str,
    resolution: Option<&crate::resolver::Resolution>,
) -> Option<String> {
    // `Resolution` Step 2: resolve against the per-request mount-scoped root
    // (the host's app), falling back to the process global only when no
    // Resolution is carried — so one process serving many apps never first-
    // paints app B's route against app A's root (`inv-deck-single-root-scoped`).
    let project_root = crate::resolver::resolve_project_root_with(resolution)?;
    // `/greet/zelda` → name=greet, sub=zelda; `/counter` → name=counter, sub=None;
    // `/` → name="" (the index `routes/page.sui`).
    let trimmed = request_path.trim_start_matches('/');
    let (name, sub) = match trimmed.split_once('/') {
        Some((n, rest)) if !rest.is_empty() => (n, Some(rest)),
        _ => (trimmed, None),
    };
    // The index route is `routes/page.sui` (empty name). `resolve()` keys off a
    // non-empty `<name>` dir, so resolve the index leaf directly.
    if name.is_empty() {
        let source = project_root.join("routes").join("page.sui");
        if !source.is_file() {
            return None;
        }
        let req = crate::synthetic_ssr::SsrRequest {
            view_name: "",
            sub_path: None,
            source_path: &source,
            pathname: request_path,
            params: serde_json::json!({}),
            query: serde_json::json!({}),
        };
        return crate::synthetic_ssr::first_paint(&req);
    }
    let layout = deck_host::resolver::ViewLayout {
        view_dirs: vec!["routes".to_string()],
        view_leaf: "page.sui".to_string(),
        first_party_root: None,
    };
    let resolved = deck_host::resolver::resolve(&project_root, &layout, name, sub).ok()?;
    let source_path = resolved.source_path.as_deref()?;
    let params = serde_json::to_value(&resolved.params).unwrap_or(serde_json::Value::Null);
    let req = crate::synthetic_ssr::SsrRequest {
        view_name: name,
        sub_path: sub,
        source_path,
        pathname: request_path,
        params,
        query: serde_json::json!({}),
    };
    crate::synthetic_ssr::first_paint(&req)
}

pub fn serve_scaffold(
    mode: DaemonMode,
    scaffold: &AppServeConfig,
    params: &[(&str, &str)],
    // The request pathname, when the caller is a route handler (serve_index /
    // spa_* / spa_fallback). Drives `render: ssr` first-paint resolution. `None`
    // (e.g. the multi-app host path, tests) ⇒ no SSR resolution → CSR shell.
    request_path: Option<&str>,
    // The per-request mount-scoped resolution context (`Resolution` Step 2).
    // Threaded so the SSR first-paint resolves against THIS app's declared root
    // rather than a shared process global — the host (shipd) carries one per app
    // so a single process serves many projects without cross-talk. `None` ⇒ the
    // global resolver (the single-root `deck dev` default + cwd floor).
    resolution: Option<&crate::resolver::Resolution>,
) -> DeckResponse {
    let Some(html) = read_source_asset(&format!("{}/index.html", scaffold.source))
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
            let js_value =
                serde_json::to_string(&resolved).unwrap_or_else(|_| "\"\"".to_string());
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
    // SSR (`render: ssr`): render the matched `routes/` page server-side and
    // inject its first-paint HTML into the (otherwise empty) mount root, so the
    // client HYDRATES it instead of building from scratch. The mount-root id is
    // the bundler's `templates::ROOT_ID` contract (`root`); deck-core can't
    // depend on deck-bundler (cycle), so the id is pinned here. No registered
    // engine / no route match / a render error ⇒ `routes_first_paint` is None
    // and we serve the empty shell (CSR) exactly as before.
    let html = match (scaffold.render, request_path) {
        (RenderMode::Ssr, Some(path)) => match routes_first_paint(path, resolution) {
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
    // edit and emits `module-changed`; this client (no opts → full-document
    // reload) drives the refresh. Served at `/_deck/live-reload.js`; it self-
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
    DeckResponse::html_view(html, mode)
}

/// Serve a static asset from `<deck_out>/<source>/assets/<file>`.
///
/// The `file` parameter comes from the axum path extractor and is
/// percent-decoded but NOT stripped of `..` segments. Validate it before
/// constructing the path to prevent directory traversal: a request like
/// `/app/_assets/../../../etc/passwd` would otherwise escape the assets dir.
pub fn serve_scaffold_asset(mode: DaemonMode, scaffold: &AppServeConfig, file: &str) -> DeckResponse {
    // Reject path-traversal patterns. Only simple filenames (no slashes, no
    // leading dots, no ".." components) are valid asset names. The chunk
    // route's same guard was here formerly (`serve_sui_chunk`); it lives
    // here now since the chunk route is gone.
    if file.contains("..") || file.contains('/') || file.contains('\\') || file.starts_with('.') {
        return DeckResponse::not_found(
            format!("invalid asset path: {file}"),
            mode,
        );
    }
    // Per-app build output: `<project_root>/<source>/assets/<file>` (e.g.
    // `apps/hub/.deck/assets/<file>`). The framework's old
    // `<deck_out>/<source>/assets/<file>` fallback was the legacy
    // `deck.jsonc`-driven scaffold shape — purged (deck-purge-root);
    // the host owns the output location via `scaffold.source`.
    let path = format!("{}/assets/{file}", scaffold.source);
    match read_source_asset(&path) {
        Some(body) if file.ends_with(".css") => DeckResponse::css_view(body, mode),
        Some(body) => DeckResponse::js_asset(body, mode),
        None => DeckResponse::not_found(
            format!("scaffold '{}' asset not found: {file}", scaffold.name),
            mode,
        ),
    }
}

/// `GET /<file>` — serve a single-segment static asset from the host's declared
/// public directory (`DeckConfig.public_dir`), resolved under deck's OWN project
/// root (`resolver::resolve_project_root`, never a Ship path). Single-segment +
/// traversal-guarded (the route is `/{file}`, but reject `..`/slashes/leading-dot
/// belt-and-suspenders); content-type by extension; immutable long cache in
/// Production. 404 when no public dir is configured or the file is absent.
pub fn serve_public_asset(mode: DaemonMode, public_dir: &str, file: &str) -> DeckResponse {
    if file.is_empty()
        || file.contains("..")
        || file.contains('/')
        || file.contains('\\')
        || file.starts_with('.')
    {
        return DeckResponse::not_found(format!("invalid public asset path: {file}"), mode);
    }
    let Some(root) = crate::resolver::resolve_project_root() else {
        return DeckResponse::not_found("no project root for public assets", mode);
    };
    let path = root.join(public_dir).join(file);
    let Ok(body) = std::fs::read(&path) else {
        return DeckResponse::not_found(format!("public asset not found: {file}"), mode);
    };
    DeckResponse::new(200, public_asset_content_type(file), CachePolicy::Asset, mode, body)
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
// `.deck/framework/` → `assets/`). The runtime `.deck/framework/` read
// was the framework's dev-override — purged (deck-purge-root). The
// bake pipeline (build.rs) still stages `.deck/framework/` → `assets/` for
// compile-time inclusion, but the runtime serves nothing from
// `.deck/framework/` directly.

pub fn serve_sui_runtime(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SUI_RUNTIME_JS.to_string(), mode)
}

pub fn serve_sui_dom(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SUI_DOM_JS.to_string(), mode)
}

pub fn serve_sui_sdk_client(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(SUI_SDK_CLIENT_JS.to_string(), mode)
}

/// `GET /_deck/route.js` — the `@ship/deck/router` framework singleton.
/// The bundler-generated entry imports it for `defineRoutes`, `route`,
/// `navigate`, `setEditMode`. The bundle is the package compiled
/// output (the source lives at `packages/deck/router/` and is shipped
/// via `include_str!`).
pub fn serve_deck_route(mode: DaemonMode) -> DeckResponse {
    DeckResponse::js_asset(DECK_ROUTE_JS.to_string(), mode)
}
/// the retired whole-blob `/_deck/lucide-icons.json` endpoint (~410 KB fetched
/// once on first mount). Per-icon serving hands the browser HTTP cache the
/// granularity that matches usage: each `<sui-icon>` triggers one tiny GET
/// (typically <1 KB), 304-cached on second mount.
///
/// The response is a full standalone `<svg>` with lucide's canonical viewBox
/// and `stroke="currentColor"` so the icon picks up the host's `color` CSS.
/// Aliases (`git` → `git-branch` etc.) are resolved server-side so the
/// client can stay alias-unaware.
pub fn serve_sui_icon(mode: DaemonMode, name: &str) -> DeckResponse {
    // Reject path-traversal + non-lucide name shapes early — names are
    // kebab-case ASCII, identical to the file stems lucide-static ships.
    if name.is_empty()
        || name.len() > 64
        || !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return DeckResponse::not_found("invalid icon name", mode);
    }
    let resolved = if ICON_MAP.contains_key(name) {
        name
    } else if let Some(via) = alias_for(name) {
        via
    } else {
        return DeckResponse::not_found(format!("icon not in lucide set: {name}"), mode);
    };
    let inner = ICON_MAP.get(resolved).expect("resolved name in map");
    // Canonical lucide attrs — kept in lockstep with `makeSvg()` in
    // sui-icon.controller.ts so client (CDN-src mode) + daemon (named mode)
    // render identically.
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" \
         stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" \
         stroke-linejoin=\"round\">{inner}</svg>"
    );
    DeckResponse::new(
        200,
        "image/svg+xml; charset=utf-8",
        crate::response::CachePolicy::Asset,
        mode,
        svg.into_bytes(),
    )
}

/// `GET /_deck/live-reload.js` — the one HMR client, shared by the view
/// host and the shell. Served in every mode (it self-gates on the
/// `deck-mode` meta tag); only subscribes when the daemon is in dev.
pub fn serve_sui_live_reload(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("live-reload.js").unwrap_or_else(|| LIVE_RELOAD_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

/// `GET /_deck/{file}` — was a shared chunk emitted by the multi-entry view
/// build (#267), served content-addressed from the `.deck/` root. PURGED
/// (deck-purge-root): the framework no longer reads a monorepo-root
/// `.deck/`, so the runtime serves no shared chunks at this path. The
/// route is gone; view bundles that used to import `../../<chunk>.js`
/// now fail to load — view chunk serving is a follow-up.
// (formerly `pub fn serve_sui_chunk(mode, file) -> DeckResponse`) removed.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_js_returns_baked_fallback() {
        let resp = serve_sui_runtime(DaemonMode::Development);
        assert_eq!(resp.status, 200);
    }

    /// inv-deck-no-node-runtime (bake freshness): framework singletons are baked
    /// via `include_str!`, so the bake MUST be the CURRENT build — never stale.
    /// Guards the de-view + engine-collapse: the runtime vends `component()` /
    /// `mountComponent()` and NO `view()` export. A stale bake (e.g. the
    /// partially-rebaked `view`-exporting runtime seen mid-migration) trips this.
    /// Fix: `cargo run -p deck-cli -- build-assets`.
    #[test]
    fn baked_runtime_is_the_current_engine_not_a_stale_bake() {
        assert!(
            SUI_RUNTIME_JS.contains("as mountComponent,"),
            "baked sui-runtime.js missing the `mountComponent` export — stale bake"
        );
        assert!(
            SUI_RUNTIME_JS.contains("as component,"),
            "baked sui-runtime.js missing the `component` export — stale bake"
        );
        assert!(
            SUI_RUNTIME_JS.contains("as route,") || SUI_RUNTIME_JS.contains("as route}"),
            "baked sui-runtime.js missing the `route` export — `@ship/sui-runtime/route` \
             maps to this same bundle in production; a stale bake or a missing index.ts \
             re-export breaks every compiled view's route import"
        );
        // The removed `view()` export must be gone. Match the export list (`as
        // view`), not warning strings that merely mention the word "view".
        assert!(
            !SUI_RUNTIME_JS.contains("as view,") && !SUI_RUNTIME_JS.contains("as view}"),
            "baked sui-runtime.js still EXPORTS `view` (removed in de-view) — stale \
             bake; run `cargo run -p deck-cli -- build-assets`"
        );
    }

    #[test]
    fn icon_known_name_returns_svg() {
        let resp = serve_sui_icon(DaemonMode::Development, "search");
        assert_eq!(resp.status, 200, "lucide ships `search`");
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.starts_with("<svg"));
        assert!(body.contains("viewBox=\"0 0 24 24\""));
        assert!(body.contains("stroke=\"currentColor\""));
    }

    #[test]
    fn icon_alias_resolves_server_side() {
        let direct = serve_sui_icon(DaemonMode::Development, "git-branch");
        let aliased = serve_sui_icon(DaemonMode::Development, "git");
        assert_eq!(direct.status, 200);
        assert_eq!(aliased.status, 200);
        assert_eq!(direct.body, aliased.body, "alias must serve identical bytes");
    }

    #[test]
    fn icon_unknown_returns_404() {
        let resp = serve_sui_icon(DaemonMode::Development, "definitely-not-a-real-icon");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn icon_path_traversal_rejected() {
        let resp = serve_sui_icon(DaemonMode::Development, "../etc/passwd");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn icon_empty_name_rejected() {
        let resp = serve_sui_icon(DaemonMode::Development, "");
        assert_eq!(resp.status, 404);
    }

    // ── serve_scaffold ────────────────────────────────────────────────────────

    /// Build a minimal AppServeConfig in-memory for tests.
    fn make_scaffold(
        name: &str,
        inject_globals: Vec<(&str, &str)>,
    ) -> AppServeConfig {
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
        }
    }

    #[test]
    fn serve_scaffold_missing_source_returns_404() {
        let scaffold = make_scaffold("test", vec![]);
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[], None, None);
        assert_eq!(resp.status, 404);
    }
    #[test]
    fn substitute_params_replaces_dollar_token() {
        assert_eq!(substitute_params("$id", &[("id", "my-space")]), "my-space");
        assert_eq!(substitute_params("prefix-$id-suffix", &[("id", "x")]), "prefix-x-suffix");
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
    /// and injects globals. The resolver's project root is pinned to the
    /// tmpdir via the test-only `ProjectRootGuard` so the resolver
    /// deterministically sees our fixture.
    #[test]
    fn serve_scaffold_serves_html_and_injects_globals() {
        use crate::test_lock::ProjectRootGuard;
        let dir = tempfile::tempdir().unwrap();
        let _root = ProjectRootGuard::new(dir.path());

        // Per-app build output: <project_root>/<source>/index.html
        // (post-cutover; the legacy `<root>/.deck/<source>/index.html`
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
        };
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[("id", "my-space")], None, None);

        assert_eq!(resp.status, 200);
        // The asset href is preserved (bundler wrote the final URL).
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("/app/main.js"), "expected /app/main.js in: {body}");
        // The inject_globals block lands before </head>.
        assert!(body.contains("window.DECK_APP_ID=\"my-space\""), "expected injected global in: {body}");
    }

    /// serve_scaffold surfaces the app's render mode as a `deck-render` meta
    /// tag so CSR-vs-SSR is observable client-side. An `ssr` app emits
    /// `content="ssr"`; the default `csr` app emits `content="csr"`.
    #[test]
    fn serve_scaffold_injects_render_meta() {
        use crate::test_lock::ProjectRootGuard;
        let dir = tempfile::tempdir().unwrap();
        let _root = ProjectRootGuard::new(dir.path());
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
        };
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[], None, None);
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

    /// serve_scaffold falls back to the authored SOURCE `<source>/index.html`
    /// when there is no built `.deck/<source>/` artifact — the dev/source path
    /// (mirrors view.rs). Gates the fallback so the demo (and any host) never
    /// needs to commit a `.deck/` artifact to serve a scaffold.
    #[test]
    fn serve_scaffold_falls_back_to_source_when_unbuilt() {
        use crate::test_lock::ProjectRootGuard;
        let dir = tempfile::tempdir().unwrap();
        let _root = ProjectRootGuard::new(dir.path());
        // SOURCE location: <root>/my-scaffold/index.html — NOT under .deck/.
        let src_dir = dir.path().join("my-scaffold");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("index.html"),
            r#"<!doctype html><html><head></head><body><div id="sui-root"></div></body></html>"#,
        )
        .unwrap();
        let scaffold = AppServeConfig {
            name: "space".to_string(),
            route: "/app/{id}".to_string(),
            source: "my-scaffold".to_string(),
            inject_globals: std::collections::BTreeMap::new(),
            render: RenderMode::Csr,
            entry: None,
        };
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[("id", "x")], None, None);
        // Status 200 is the only contract: the source-fallback path
        // served the unbuilt scaffold. The body shape is incidental.
        assert_eq!(resp.status, 200, "source-fallback must serve the unbuilt scaffold");
    }
    #[test]
    fn scaffold_asset_dotdot_rejected() {
        let scaffold = make_scaffold("test", vec![]);
        let resp = serve_scaffold_asset(DaemonMode::Development, &scaffold, "../../../etc/passwd");
        assert_eq!(resp.status, 404, "'..' traversal must be rejected");
    }

    #[test]
    fn scaffold_asset_slash_in_name_rejected() {
        let scaffold = make_scaffold("test", vec![]);
        let resp = serve_scaffold_asset(DaemonMode::Development, &scaffold, "sub/dir/file.js");
        assert_eq!(resp.status, 404, "slash in filename must be rejected");
    }

    #[test]
    fn scaffold_asset_leading_dot_rejected() {
        let scaffold = make_scaffold("test", vec![]);
        let resp = serve_scaffold_asset(DaemonMode::Development, &scaffold, ".hidden");
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
        use crate::test_lock::ProjectRootGuard;
        let dir = tempfile::tempdir().unwrap();
        let _root = ProjectRootGuard::new(dir.path());
        // Per-app build output: <project_root>/safe-scaffold/index.html
        // (the legacy `<root>/.deck/safe-scaffold/` was purged).
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
        };
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[], None, None);

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
        use crate::test_lock::ProjectRootGuard;
        use std::io::Write;
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("public")).unwrap();
        std::fs::File::create(tmp.path().join("public/logo.svg"))
            .unwrap()
            .write_all(b"<svg/>")
            .unwrap();
        let _root = ProjectRootGuard::new(tmp.path());

        let ok = serve_public_asset(DaemonMode::Development, "public", "logo.svg");
        assert_eq!(ok.status, 200, "present public asset serves 200");
        assert!(ok.content_type.starts_with("image/svg+xml"), "svg content-type by extension");
        assert_eq!(ok.body, b"<svg/>", "serves the file bytes");

        assert_eq!(
            serve_public_asset(DaemonMode::Development, "public", "../secret").status,
            404,
            "path traversal must 404"
        );
        assert_eq!(
            serve_public_asset(DaemonMode::Development, "public", "nope.svg").status,
            404,
            "missing file must 404"
        );
    }
}
