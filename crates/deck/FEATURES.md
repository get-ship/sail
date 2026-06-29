# Deck — Feature Inventory

Deck is the host / meta-framework for SUI. The relationship is
`SUI : Deck : Ship :: React : Next : app` — SUI is the component language, Deck
is the framework that *hosts* SUI views and bundles their assets, and a Ship app
(the hub, a space) is built *with* Deck. Deck owns *what* gets served and *how*
assets are built; it never owns app concepts.

This document is the canonical list of Deck features and where each is
implemented. It is gated against drift by
`crates/deck/deck-core/tests/features_doc.rs`: every `Implementation` reference
below is verified to point at a file that exists and (for `::symbol` refs) a
symbol token that is actually present. A moved or renamed symbol breaks that
test — so this inventory cannot silently rot.

## The two seams (both CLEAN, both test-enforced)

Deck sits between SUI below and the server/app above. Both seams are clean and
each is guarded by a dedicated gate test:

- **deck / sui seam** — SUI emits *structure*, never a serve location. Compiled
  output carries DOM wiring + bare-specifier imports only; it never bakes a
  `/_deck/...` URL or an importmap (serving is Deck's concern; the compiler knows
  no URLs). Gate: `crates/sui/compiler/tests/boundary_invariants.rs`
  (`view_output_emits_no_behavior_or_urls`, `form_mutation_ssr_contains_no_sui_urls`,
  `live_data_ssr_contains_no_sui_urls`). Deck injects the `/_deck/...` URLs
  host-side (form actions, the live-data `__subscribe__` URL).
- **deck / server seam** — Deck's host + bundler crates (`deck-core`,
  `deck-axum`, `deck-host`, `deck-bundler`, `deck-config`) name no Ship app
  concept. They serve and build for *any* host: roots and config are passed in,
  never assumed. Gate: `crates/deck/deck-core/tests/boundary.rs`
  (`deck_crates_name_no_ship_app`).

## Crate map

- `deck-core` — transport-agnostic hosting logic. Produces a `DeckResponse`
  per request; never touches HTTP.
- `deck-axum` — the axum adapter: thin shims + `router()` + the HMR stream.
- `deck-host` — pure view resolution + render helpers (the SUI-facing read side).
- `deck-bundler` — in-process Rolldown bundler (compile, per-view resolve,
  shared-chunk dedup, app/view builds, asset-graph, watch/HMR, catalog,
  route codegen).
- `deck-config` — the optional `deck.jsonc` view-discovery + watch override schema.
- `deck-cli` — the `deck` framework CLI (build / build-assets / routes / scaffold).
- `deck` — thin re-export facade over the above (what shipd imports).

## Executable reference

`crates/deck/deck-conformance` is the ship-free worked example. It is a zero-config Deck
host (no Ship app, no `.ship` dir) that exercises every feature below — view
host, payload, scaffolds, framework bundles, icons, live-data SSE, form
mutation, HMR — so the contract is demonstrated end-to-end without any Ship
coupling.

---

## Serving (`deck-core` → `DeckResponse`; `deck-axum` → axum `Response`)

Each handler logic lives in `deck-core` and returns a `DeckResponse`; the matching
`deck-axum` shim extracts request params and maps it onto an axum `Response`.

| Feature | Route | Implementation | Description |
| --- | --- | --- | --- |
| View host shell | `GET /_deck/view/{name}[/{*sub}]` | `crates/deck/deck-core/src/view_host.rs::serve_view_host` | Serves the HTML shell with SSR content spliced in; live-emits SSR on cold miss, mtime-caches in prod, renders the styled 404 for an unresolvable name. |
| View payload (JSON) | `GET /_deck/views/{name}` | `crates/deck/deck-core/src/view_payload.rs::serve_view_payload` | Returns the `ViewPayload` JSON (html, css, scope, moduleUrl, params) the runtime mounts. |
| View payload (sub-route) | `GET /_deck/views/{name}/{*sub}` | `crates/deck/deck-core/src/view_payload.rs::serve_view_payload_subpath` | Same payload for a nested/parameterized route; folds the sub into the subscribe URL. |
| Compiled view module | `GET /_deck/views/{name}/view.js` | `crates/deck/deck-core/src/view.rs::serve_sui_view_setup` | Serves the per-view ESM bundle; compiles on demand when no `.deck` artifact exists. |
| sui-runtime bundle | `GET /_deck/runtime.js` | `crates/deck/deck-core/src/assets.rs::serve_sui_runtime` | Deck's own baked runtime singleton. |
| sui-dom bundle | `GET /_deck/dom.js` | `crates/deck/deck-core/src/assets.rs::serve_sui_dom` | Deck's own baked DOM module. |
| sdk-client bundle | `GET /_deck/sdk-client.js` | `crates/deck/deck-core/src/assets.rs::serve_sui_sdk_client` | Deck's own baked SDK client. |
| Per-icon SVG | `GET /_deck/icons/{name}.svg` | `crates/deck/deck-core/src/assets.rs::serve_sui_icon` | One lucide icon per request, alias-resolved server-side, from the embedded set. |
| Shared chunk | `GET /_deck/{file}` | `crates/deck/deck-core/src/assets.rs::serve_sui_chunk` | Content-addressed shared chunk from the multi-entry view build, served from `.deck/`. |
| Scaffold shell | `GET /` and `GET /space/{id}` | `crates/deck/deck-core/src/assets.rs::serve_scaffold` | Serves an HTML scaffold (shell/space), rewriting asset hrefs and injecting validated globals. |
| Scaffold asset | `GET /{shell,space}/_assets/{file}` | `crates/deck/deck-core/src/assets.rs::serve_scaffold_asset` | Serves a scaffold's hashed static asset with path-traversal rejection. |
| Meta / drift stamp | `GET /_deck/meta` | `crates/deck/deck-core/src/meta.rs::serve_meta` | Compiler + daemon build identity for the shell's version-drift check. |
| Workspace catalog | (purged — `GET /_deck/catalog/workspace`) | (purged — `deck-core/src/packs.rs`) | The runtime catalog serving was the framework's hardcoded `.deck/catalogs/` read. Purged. |
| Live-reload client | `GET /_deck/live-reload.js` | `crates/deck/deck-core/src/assets.rs::serve_sui_live_reload` | The single HMR client; self-gates on the `ship-mode` meta so it only subscribes in dev. |
| Styled 404 page | (host 404 body) | `crates/deck/deck-core/src/error_page.rs::view_not_found` | Rendered `text/html` not-found page (with servable-view list), not raw JSON. |
| View resolution | (internal) | `crates/deck/deck-core/src/resolver.rs::resolve` | Host-agnostic project-root + first-party resolution feeding every view handler. |
| Asset-graph read | (purged — `deck-core/src/asset_graph.rs`) | (purged — `deck-purge-root`) | The runtime read of `.deck/asset-graph.json` was the framework's hardcoded `.deck/` reach. Purged. |
| Module URL builder | (internal) | `crates/deck/deck-core/src/routes.rs::view_module_url` | One source of truth for the view-module `/_deck/...` URL. |
| Mutation URL builder | (internal) | `crates/deck/deck-core/src/routes.rs::mutation_action` | Builds the `POST /_deck/mutation/{name}` form-action URL. |
| Subscribe URL builder | (internal) | `crates/deck/deck-core/src/routes.rs::subscribe_url` | Builds the `/_deck/views/{name}/sse` live-data URL. |
| Form action injection | (host-side SSR) | `crates/deck/deck-core/src/form_inject.rs::inject_form_actions` | Augments `<form mutation="bN">` SSR with the daemon-relative action URL (idempotent). |
| Live-data SSE envelopes | (SSE wire) | `crates/deck/deck-core/src/live_wire.rs::format_patch_signals` | SSE frame encoders (`patch-signals`/`patch-elements`/`patch`/keepalive) for the live-data protocol. |
| DeckResponse model | (transport port) | `crates/deck/deck-core/src/response.rs::DeckResponse` | The framework-agnostic response (status/headers/body) every handler returns. |
| Daemon mode | (internal) | `crates/deck/deck-core/src/lib.rs::DaemonMode` | Dev/Prod switch driving cache headers, HMR gating, and SSR caching. |
| Mode meta injection | (HTML rewrite) | `crates/deck/deck-core/src/mode_html.rs::inject_mode_meta` | Rewrites `<meta name="ship-mode">` so the live-reload client self-gates. |
| Render meta injection | (HTML rewrite) | `crates/deck/deck-core/src/mode_html.rs::inject_render_meta` | Rewrites `<meta name="ship-render">` so the app's csr/ssr mode is observable client-side. |
| App-registration port | (host seam) | `crates/deck/deck-core/src/app_port.rs::AppRegistry` | Sibling to `DataPort`: the host enumerates the apps it serves (each a per-app `deck.jsonc`-loaded `AppServeConfig`); deck names no path. |
| axum router | (mount unit) | `crates/deck/deck-axum/src/lib.rs::router` | Registers Deck's whole hosting surface as one mountable axum `Router`. |
| DeckResponse → axum | (adapter) | `crates/deck/deck-axum/src/lib.rs::into_axum` | Maps a `DeckResponse` 1:1 onto an axum `Response`. |
| HMR stream | `GET /_deck/dev/hmr` | `crates/deck/deck-axum/src/handlers.rs::serve_bundle_hmr` | SSE firehose of bundler events for the live-reload client; cancel-token terminated for graceful drain. |

## Bundling (`deck-bundler`)

In-process Rolldown — no child process. The bundler compiles `.sui` via the
linked `sui-compiler` crate, resolves each view's components, dedups shared
chunks, builds apps and views, writes the asset graph, and watches for HMR.

| Feature | Output | Implementation | Description |
| --- | --- | --- | --- |
| Bundler handle | (in-process) | `crates/deck/deck-bundler/src/lib.rs::Bundler` | Clone-safe handle: locate compiler once, build / build_all / watch, single-flight coalescing. |
| Build all targets | `.deck/` + `crates/deck/deck-core/assets/` | `crates/deck/deck-bundler/src/lib.rs::build_all` | Bakes every target (views multi-entry), refreshes the catalog, writes the asset graph; honest per-target failure. |
| In-process compile | (JS string) | `crates/deck/deck-bundler/src/compile.rs::compile_sui` | Compiles one `.sui` to JS via `sui_compiler::compile`, auto-wiring a paired controller. |
| Rolldown driver | per-target `.js` | `crates/deck/deck-bundler/src/rolldown_bundle.rs::bundle_target` | Drives Rolldown for one target with the SUI plugin; trims arenas after each build. |
| SUI resolve/load plugin | (Rolldown plugin) | `crates/deck/deck-bundler/src/sui_plugin.rs` | `resolve_id`/`load` hooks: compile `.sui`, externalize singletons, re-anchor `@ship/<pkg>`, resolve `@sui-tag/<tag>`. |
| Per-view `@sui-tag` resolver | (view bundle) | `crates/deck/deck-bundler/src/sui_plugin.rs` | `<tag>` → source `.sui` index so a view bundle carries exactly the components it uses. |
| Shared-chunk dedup | `.deck/` shared chunks | `crates/deck/deck-bundler/src/views_bundle.rs::bundle_views` | Multi-entry view build: a component used by ≥2 views lands in one shared chunk. |
| App build | static dir | `crates/deck/deck-bundler/src/app.rs::bundle_app` | Bundles a SUI SPA (hub/space), externalizing the document importmap; compiles its `.sui` in-process. |
| Asset-graph write | (purged — `deck-bundler/src/asset_graph.rs`) | (purged — `deck-purge-root`) | The framework no longer writes a monorepo-root `.deck/asset-graph.json`. |
| Watch / HMR | (broadcast events) | `crates/deck/deck-bundler/src/watch.rs` | Pruned non-recursive FS watch, bounded drop-oldest ring, debounced rebuild, emits `BundlerEvent`s. |
| Catalog aggregation | (purged — `.deck/catalogs/`) | (purged — `deck-purge-root`) | The framework no longer writes a monorepo-root `.deck/catalogs/workspace.json`. The catalog walker (`crates/deck/deck-bundler/src/catalog/mod.rs::build_catalog`) is available in-memory for hosts. |
| Tag → source index | (per-view resolver feed) | `crates/deck/deck-bundler/src/catalog/mod.rs::tag_source_index` | Maps every component tag to its source `.sui` from the same CEM walk. |
| Workspace discovery | (catalog input) | `crates/deck/deck-bundler/src/catalog/workspace.rs::discover_workspace_packages` | Expands `package.json` workspace globs into the package set the catalog walks. |
| Routes codegen | `routes.gen.ts` (per root) | `crates/deck/deck-bundler/src/routes.rs::generate_routes` | Walks every `[param]` dir under each app's `routes/` (shell + spaces) and emits a typed `RouteShape[]` the runtime registers via `defineRoutes` (write-if-changed). Multi-root via `RouteGenSpec` (`dir` + `out` + `label`). |
| Targets / layout | (target set) | `crates/deck/deck-bundler/src/targets.rs::BundleLayout` | Framework singletons + discovered consumer bundles + discovered views, and where each output lands. |
| Consumer bundle discovery | (per-package self-decl) | `crates/deck/deck-bundler/src/catalog/workspace.rs::discover_bundle_declarations` | Walks the workspace for each package's `package.json` `deck.bundles[]` (name/entry/externals/lazy) — the dissolved central `targets` list. |
| Component-root map | (`@ship/<name>` resolution) | `crates/deck/deck-bundler/src/catalog/workspace.rs::component_package_roots` | npm-name→package-root derived from the workspace map; replaces the hand-maintained `componentSourceRoots` search order. |

## Config (`deck-config`)

| Feature | File | Implementation | Description |
| --- | --- | --- | --- |
| Manifest schema | `deck.jsonc` (optional) | `crates/deck/deck-config/src/lib.rs::DeckConfig` | Optional host override for the VIEW-discovery + watch layout: source roots, watch dirs, packs, view dirs, view leaf. Consumer bundles + `@ship/<name>` resolution are NOT here (self-declared + discovered). |
| Config load | (startup) | `crates/deck/deck-config/src/lib.rs` | Reads `<root>/deck.jsonc` when present, strips JSONC comments, falls back to defaults (never panics). |
| Zero-config defaults | (built-in) | `crates/deck/deck-config/src/lib.rs::defaults` | The defaults a host with no `deck.jsonc` (deck-conformance, and the Ship repo itself) runs on. |
| Bundle self-declaration | `package.json` `deck.bundles[]` | `crates/deck/deck-bundler/src/catalog/workspace.rs::BundleDecl` | One consumer bundle a package declares for itself (name, package-relative entry, externals, lazy). Discovered by the bundler — no central registry. |

## Host contract (`deck-host`, `deck-cli`)

The pure SUI-facing read side, plus the framework CLI.

| Feature | Surface | Implementation | Description |
| --- | --- | --- | --- |
| View resolver | (on-disk lookup) | `crates/deck/deck-host/src/resolver.rs::resolve` | Resolves a view name + sub-path across first-party / `views/` / `.ship/views/` roots with `[param]` dirs. |
| Resolved view | (resolution result) | `crates/deck/deck-host/src/resolver.rs::ResolvedView` | Source + params + canonical source path returned by the resolver. |
| Render + mtime cache | (SSR) | `crates/deck/deck-core/src/view_payload.rs::serve_view_payload` | Compiles + renders a resolved view, mtime-keyed (`INIT_CACHE`) so a re-emitted artifact is never stale. |
| View payload shape | (JSON contract) | `crates/deck/deck-host/src/view_payload.rs::ViewPayload` | The serialized payload contract (html/css/scope/moduleUrl/params) shared with the runtime. |
| Render helper | (source read) | `crates/deck/deck-host/src/render_helpers.rs::render_view_source` | Reads a view's `page.sui` source from the project root. |
| CLI: build app | `deck build` | `crates/deck/deck-cli/src/main.rs` | Builds a SUI SPA app to a static output dir. |
| CLI: bake assets | `deck build-assets` | `crates/deck/deck-cli/src/main.rs` | Bakes framework + consumer bundles via the in-process bundler, with non-empty guards. |
| CLI: routes | `deck routes` | `crates/deck/deck-cli/src/main.rs` | Regenerates the typed dynamic-route table standalone. `--root=<path>` is required and repeatable; the CLI walks `<root>/routes/` and writes `<root>/routes.gen.ts` with a label auto-derived from the path's basename (no app-name coupling, no monorepo-defaults heuristic). |
| CLI: scaffold view | `deck scaffold view` | `crates/deck/deck-cli/src/scaffold.rs::run_view` | Generates a new view (page.sui + controller.ts + style.css + manifest.jsonc). |
| CLI: scaffold component | `deck scaffold component` | `crates/deck/deck-cli/src/scaffold.rs::run_component` | Generates a new `sui-*` primitive under the primitives package. |
| CLI: scaffold pack | `deck scaffold pack` | `crates/deck/deck-cli/src/scaffold.rs::run_pack` | Generates a new first-party `@ship/<name>` component pack. |

## Security

| Feature | Surface | Implementation | Description |
| --- | --- | --- | --- |
| HTML security headers | HTML responses | `crates/deck/deck-core/src/response.rs::html_view` | X-Frame-Options, X-Content-Type-Options nosniff, and a self-only CSP on every HTML body. |
| View-name validation | view routes | `crates/deck/deck-host/src/render_helpers.rs::is_valid_view_name` | Rejects traversal / dotted / oversize / bad-char names before any disk access. |
| Scaffold asset traversal guard | scaffold assets | `crates/deck/deck-core/src/assets.rs::serve_scaffold_asset` | Rejects `..`, slashes, and leading dots in asset filenames. |
| inject_globals key validation | scaffold HTML | `crates/deck/deck-core/src/assets.rs::serve_scaffold` | Skips any `inject_globals` key that is not a safe JS identifier (no script injection). |
| Symlink containment | view resolution | `crates/deck/deck-host/src/resolver.rs::read_view_artifact` | Canonicalizes + size-caps view artifacts; the resolver rejects leaves that escape their bucket root. |
| Source-size cap | view source | `crates/deck/deck-host/src/resolver.rs::MAX_VIEW_SOURCE_BYTES` | 4 MiB ceiling (stat-checked before read) so one request can't pin a worker on a giant parse. |
| Prod error redaction | errors / 404s | `crates/deck/deck-core/src/response.rs::not_found` | Suppresses internal detail (paths, build hints) in production; surfaces it only in dev. |
| Strong ETag (prod) | cacheable bodies | `crates/deck/deck-core/src/response.rs::compute_etag` | Content-derived ETag for prod cacheable responses. |
</content>
</invoke>
