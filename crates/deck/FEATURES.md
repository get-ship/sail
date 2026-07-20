# Deck — Feature Inventory

Deck is the host / meta-framework for SAIL. The relationship is
`SAIL : Deck : Ship :: React : Next : app` — SAIL is the component language, Deck
is the framework that *hosts* SAIL routes and bundles their assets, and a Ship app
(the hub, a space) is built *with* Deck. Deck owns *what* gets served and *how*
assets are built; it never owns app concepts.

This document is the canonical list of Deck features and where each is
implemented. It is gated against drift by
`crates/deck/deck-core/tests/features_doc.rs`: every `Implementation` reference
below is verified to point at a file that exists and (for `::symbol` refs) a
symbol token that is actually present. A moved or renamed symbol breaks that
test — so this inventory cannot silently rot.

## The two seams (both CLEAN, both test-enforced)

Deck sits between SAIL below and the server/app above. Both seams are clean and
each is guarded by a dedicated gate test:

- **deck / sail seam** — SAIL emits *structure*, never a serve location. Compiled
  output carries DOM wiring + bare-specifier imports only; it never bakes a
  `/_deck/...` URL or an importmap (serving is Deck's concern; the compiler knows
  no URLs). Gate: `crates/sail/compiler/tests/boundary_invariants.rs`
  (`route_output_emits_no_behavior_or_urls`, `form_mutation_ssr_contains_no_sui_urls`,
  `live_data_ssr_contains_no_sui_urls`). Deck injects the `/_deck/...` URLs
  host-side (form actions, the live-data `__subscribe__` URL).
- **deck / server seam** — Deck's host + bundler crates (`deck-core`,
  `deck-axum`, `deck-host`, `deck-bundler`, `deck-config`) name no Ship app
  concept. They serve and build for *any* host: roots and config are passed in,
  never assumed. Gate (the conformance layer, OUTSIDE deck's shipped crates):
  `crates/deck/deck-conformance/tests/boundary.rs`
  (`deck_crates_name_no_ship_app`) — a backstop. The real proof is behavioral:
  deck-cli's cli_smoke standalone build test compiles + emits a zero-ship-product
  app.

## Crate map

- `deck-core` — transport-agnostic hosting logic. Produces a `DeckResponse`
  per request; never touches HTTP.
- `deck-axum` — the axum adapter: thin shims + `router()` + the HMR stream.
- `deck-host` — pure route resolution + render helpers (the SAIL-facing read side).
- `deck-bundler` — in-process Rolldown bundler (compile, per-route resolve,
  shared-chunk dedup, app/route builds, asset-graph, watch/HMR, catalog,
  route codegen).
- `deck-config` — the optional `deck.jsonc` route-discovery + watch override schema.
- `deck-cli` — the `deck` framework CLI (build / build-assets / routes / scaffold).
- `deck` — thin re-export facade over the above (what shipd imports).

## Executable reference

`crates/deck/deck-conformance` is the ship-free worked example. It is a zero-config Deck
host (no Ship app, no `.ship` dir) that exercises every feature below — route
host, payload, scaffolds, framework bundles, icons, live-data SSE, form
mutation, HMR — so the contract is demonstrated end-to-end without any Ship
coupling.

---

## Serving (`deck-core` → `DeckResponse`; `deck-axum` → axum `Response`)

Each handler logic lives in `deck-core` and returns a `DeckResponse`; the matching
`deck-axum` shim extracts request params and maps it onto an axum `Response`.

| Feature | Route | Implementation | Description |
| --- | --- | --- | --- |
| Compiled route module | `GET /_deck/views/{name}/view.js` | `crates/deck/deck-core/src/route.rs::serve_sui_route_setup` | Serves the per-route ESM bundle; compiles on demand when no `.deck` artifact exists. (The `/_deck/views/*` serving URL is the ship-views wire path — see the ship-ui handoff.) |
| sail-runtime bundle | `GET /_deck/runtime.js` | `crates/deck/deck-core/src/assets.rs::serve_sui_runtime` | Deck's own baked runtime singleton. |
| sail-dom bundle | `GET /_deck/dom.js` | `crates/deck/deck-core/src/assets.rs::serve_sui_dom` | Deck's own baked DOM module. |
| sdk-client bundle | `GET /_deck/sdk-client.js` | `crates/deck/deck-core/src/assets.rs::serve_sui_sdk_client` | Deck's own baked SDK client. |
| Per-icon SVG | `GET /_deck/icons/{name}.svg` | `crates/deck/deck-core/src/assets.rs::serve_sui_icon` | One lucide icon per request, alias-resolved server-side, from the embedded set. |
| Framework font | `GET /_deck/fonts/{file}` | `crates/deck/deck-core/src/assets.rs::serve_framework_font` | One token-package woff2 per request, allowlist-only (FONTS basenames), no path traversal. Non-bundled consumers (e.g. tutorial preview CSS) rewrite relative `url('./fonts/…')` here instead of re-hosting under app `public/fonts`. |
| Shared chunk | `GET /_deck/{file}` | `crates/deck/deck-core/src/assets.rs::serve_sui_chunk` | Content-addressed shared chunk from the multi-entry route build, served from `.deck/`. |
| Scaffold shell | `GET /` and `GET /space/{id}` | `crates/deck/deck-core/src/assets.rs::serve_scaffold` | Serves an HTML scaffold (shell/space), rewriting asset hrefs and injecting validated globals. |
| Scaffold asset | `GET /{shell,space}/_assets/{file}` | `crates/deck/deck-core/src/assets.rs::serve_scaffold_asset` | Serves a scaffold's hashed static asset with path-traversal rejection. |
| Meta / drift stamp | `GET /_deck/meta` | `crates/deck/deck-core/src/meta.rs::serve_meta` | Compiler + daemon build identity for the shell's version-drift check. |
| Workspace catalog | (purged — `GET /_deck/catalog/workspace`) | (purged — `deck-core/src/packs.rs`) | The runtime catalog serving was the framework's hardcoded `.deck/catalogs/` read. Purged. |
| Live-reload client | `GET /_deck/live-reload.js` | `crates/deck/deck-core/src/assets.rs::serve_sui_live_reload` | The single HMR client; self-gates on the `ship-mode` meta so it only subscribes in dev. |
| Styled 404 page | (host 404 body) | `crates/deck/deck-core/src/error_page.rs::route_not_found` | Rendered `text/html` not-found page (with servable-route list), not raw JSON. |
| Route resolution | (internal) | `crates/deck/deck-core/src/resolver.rs::resolve` | Host-agnostic project-root + first-party resolution feeding every route handler. |
| Asset-graph read | (purged — `deck-core/src/asset_graph.rs`) | (purged — `deck-purge-root`) | The runtime read of `.deck/asset-graph.json` was the framework's hardcoded `.deck/` reach. Purged. |
| Module URL builder | (internal) | `crates/deck/deck-core/src/routes.rs::route_module_url` | One source of truth for the route-module `/_deck/...` URL. |
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

In-process Rolldown — no child process. The bundler compiles `.sail` via the
linked `sail-compiler` crate, resolves each route's components, dedups shared
chunks, builds apps and routes, writes the asset graph, and watches for HMR.

| Feature | Output | Implementation | Description |
| --- | --- | --- | --- |
| Bundler handle | (in-process) | `crates/deck/deck-bundler/src/lib.rs::Bundler` | Clone-safe handle: locate compiler once, build / build_all / watch, single-flight coalescing. |
| Build all targets | `.deck/` + `crates/deck/deck-core/assets/` | `crates/deck/deck-bundler/src/lib.rs::build_all` | Bakes every target (routes multi-entry), refreshes the catalog, writes the asset graph; honest per-target failure. |
| In-process compile | (JS string) | `crates/deck/deck-bundler/src/compile.rs::compile_sui` | Compiles one `.sail` to JS via `sail_compiler::compile`, auto-wiring a paired controller. |
| Rolldown driver | per-target `.js` | `crates/deck/deck-bundler/src/rolldown_bundle.rs::bundle_target` | Drives Rolldown for one target with the SAIL plugin; trims arenas after each build. |
| SAIL resolve/load plugin | (Rolldown plugin) | `crates/deck/deck-bundler/src/sail_plugin.rs` | `resolve_id`/`load` hooks: compile `.sail`, externalize singletons, re-anchor `@ship/<pkg>`, resolve `@sail-tag/<tag>`. |
| Per-route `@sail-tag` resolver | (route bundle) | `crates/deck/deck-bundler/src/sail_plugin.rs` | `<tag>` → source `.sail` index so a route bundle carries exactly the components it uses. |
| Shared-chunk dedup | `.deck/` shared chunks | `crates/deck/deck-bundler/src/routes_bundle.rs::bundle_routes` | Multi-entry route build: a component used by ≥2 routes lands in one shared chunk. |
| App build | static dir | `crates/deck/deck-bundler/src/app.rs::bundle_app` | Bundles a SAIL SPA (hub/space), externalizing the document importmap; compiles its `.sail` in-process. |
| Asset-graph write | (purged — `deck-bundler/src/asset_graph.rs`) | (purged — `deck-purge-root`) | The framework no longer writes a monorepo-root `.deck/asset-graph.json`. |
| Watch / HMR | (broadcast events) | `crates/deck/deck-bundler/src/watch.rs` | Pruned non-recursive FS watch, bounded drop-oldest ring, debounced rebuild, emits `BundlerEvent`s. |
| Catalog aggregation | (purged — `.deck/catalogs/`) | (purged — `deck-purge-root`) | The framework no longer writes a monorepo-root `.deck/catalogs/workspace.json`. The catalog walker (`crates/deck/deck-bundler/src/catalog/mod.rs::build_catalog`) is available in-memory for hosts. |
| Tag → source index | (per-route resolver feed) | `crates/deck/deck-bundler/src/catalog/mod.rs::tag_source_index` | Maps every component tag to its source `.sail` from the same CEM walk. |
| Workspace discovery | (catalog input) | `crates/deck/deck-bundler/src/catalog/workspace.rs::discover_workspace_packages` | Expands `package.json` workspace globs into the package set the catalog walks. |
| Routes codegen | `routes.gen.ts` (per root) | `crates/deck/deck-bundler/src/routes.rs::generate_routes` | Walks every `[param]` dir under each app's `routes/` (shell + spaces) and emits a typed `RouteShape[]` the runtime registers via `defineRoutes` (write-if-changed). Multi-root via `RouteGenSpec` (`dir` + `out` + `label`). |
| Targets / layout | (target set) | `crates/deck/deck-bundler/src/targets.rs::BundleLayout` | Framework singletons + discovered consumer bundles + discovered routes, and where each output lands. |
| Consumer bundle discovery | (per-package self-decl) | `crates/deck/deck-bundler/src/catalog/workspace.rs::discover_bundle_declarations` | Walks the workspace for each package's `package.json` `deck.bundles[]` (name/entry/externals/lazy) — the dissolved central `targets` list. |
| Component-root map | (`@ship/<name>` resolution) | `crates/deck/deck-bundler/src/catalog/workspace.rs::component_package_roots` | npm-name→package-root derived from the workspace map; replaces the hand-maintained `componentSourceRoots` search order. |

## Config (`deck-config`)

| Feature | File | Implementation | Description |
| --- | --- | --- | --- |
| Manifest schema | `deck.jsonc` (optional) | `crates/deck/deck-config/src/lib.rs::DeckConfig` | Optional host override for the ROUTE-discovery + watch layout: source roots, watch dirs, packs, route dirs, route leaf. Consumer bundles + `@ship/<name>` resolution are NOT here (self-declared + discovered). |
| Config load | (startup) | `crates/deck/deck-config/src/lib.rs` | Reads `<root>/deck.jsonc` when present, strips JSONC comments, falls back to defaults (never panics). |
| Zero-config defaults | (built-in) | `crates/deck/deck-config/src/lib.rs::defaults` | The defaults a host with no `deck.jsonc` (deck-conformance, and the Ship repo itself) runs on. |
| Bundle self-declaration | `package.json` `deck.bundles[]` | `crates/deck/deck-bundler/src/catalog/workspace.rs::BundleDecl` | One consumer bundle a package declares for itself (name, package-relative entry, externals, lazy). Discovered by the bundler — no central registry. |

## Host contract (`deck-host`, `deck-cli`)

The pure SAIL-facing read side, plus the framework CLI.

| Feature | Surface | Implementation | Description |
| --- | --- | --- | --- |
| Route resolver | (on-disk lookup) | `crates/deck/deck-host/src/resolver.rs::resolve` | Resolves a route name + sub-path across first-party / `views/` / `.ship/views/` roots with `[param]` dirs. |
| Resolved route | (resolution result) | `crates/deck/deck-host/src/resolver.rs::ResolvedRoute` | Source + params + canonical source path returned by the resolver. |
| Route payload shape | (JSON contract) | `crates/deck/deck-host/src/route_payload.rs::RoutePayload` | The serialized payload contract (html/css/scope/moduleUrl/params) shared with the runtime. |
| Render helper | (source read) | `crates/deck/deck-host/src/render_helpers.rs::render_route_source` | Reads a route's `page.sail` source from the project root. |
| CLI: build app | `deck build` | `crates/deck/deck-cli/src/main.rs` | Builds a SAIL SPA app to a static output dir. |
| CLI: bake assets | `deck build-assets` | `crates/deck/deck-cli/src/main.rs` | Bakes framework + consumer bundles via the in-process bundler, with non-empty guards. |
| CLI: routes | `deck routes` | `crates/deck/deck-cli/src/main.rs` | Regenerates the typed dynamic-route table standalone. `--root=<path>` is required and repeatable; the CLI walks `<root>/routes/` and writes `<root>/routes.gen.ts` with a label auto-derived from the path's basename (no app-name coupling, no monorepo-defaults heuristic). |
| CLI: scaffold route | `deck scaffold route` | `crates/deck/deck-cli/src/scaffold.rs::run_route` | Generates a new route (page.sail + manifest.jsonc; state goes in an inline `<controller>`). |
| CLI: scaffold component | `deck scaffold component` | `crates/deck/deck-cli/src/scaffold.rs::run_component` | Generates a new `sail-*` primitive under the primitives package. |
| CLI: scaffold pack | `deck scaffold pack` | `crates/deck/deck-cli/src/scaffold.rs::run_pack` | Generates a new first-party `@ship/<name>` component pack. |

## Security

| Feature | Surface | Implementation | Description |
| --- | --- | --- | --- |
| HTML security headers | HTML responses | `crates/deck/deck-core/src/response.rs::html_route` | X-Frame-Options, X-Content-Type-Options nosniff, and a self-only CSP on every HTML body. |
| Route-name validation | route routes | `crates/deck/deck-host/src/render_helpers.rs::is_valid_route_name` | Rejects traversal / dotted / oversize / bad-char names before any disk access. |
| Scaffold asset traversal guard | scaffold assets | `crates/deck/deck-core/src/assets.rs::serve_scaffold_asset` | Rejects `..`, slashes, and leading dots in asset filenames. |
| inject_globals key validation | scaffold HTML | `crates/deck/deck-core/src/assets.rs::serve_scaffold` | Skips any `inject_globals` key that is not a safe JS identifier (no script injection). |
| Symlink containment | route resolution | `crates/deck/deck-host/src/resolver.rs::read_route_artifact` | Canonicalizes + size-caps route artifacts; the resolver rejects leaves that escape their bucket root. |
| Source-size cap | route source | `crates/deck/deck-host/src/resolver.rs::MAX_ROUTE_SOURCE_BYTES` | 4 MiB ceiling (stat-checked before read) so one request can't pin a worker on a giant parse. |
| Prod error redaction | errors / 404s | `crates/deck/deck-core/src/response.rs::not_found` | Suppresses internal detail (paths, build hints) in production; surfaces it only in dev. |
| Strong ETag (prod) | cacheable bodies | `crates/deck/deck-core/src/response.rs::compute_etag` | Content-derived ETag for prod cacheable responses. |
</content>
</invoke>
