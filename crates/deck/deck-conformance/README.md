# deck-conformance — deck, standing on its own

A **ship-free** deck app: `deck::router()` mounted on axum, serving a plain
directory of `.sail` routes with **no daemon, no ship domains, no
`@ship/primitives`**. It is two things at once:

1. **An instructional example** — the smallest real "how do I serve a deck
   app?" Effectively the `deck serve` the CLI doesn't have yet.
2. **The deck reliability harness** — each slice adds one deck feature and a
   test that doubles as its documentation. Because there's no ship here, a
   failing test means *deck* broke, not ship's wiring. Because the components
   are trivial and hand-written, the design-system primitive redesign can't
   perturb these tests.

```
cargo run  -p deck-conformance      # serve on http://127.0.0.1:7878 (try /_deck/route/hello)
cargo test -p deck-conformance      # run the harness
```

## The host contract (how a non-ship app drives deck)

Just two env vars + mounting the router:

| Var | Meaning here |
|---|---|
| `SHIP_DEV_PROJECT_DIR` | App root holding `views/<name>/page.sail`. Set to `app/`. |
| `SHIP_FIRST_PARTY_VIEWS` | **Unset** — a non-ship host has no first-party route bucket. |

```rust
unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", app_dir()); }
let app = deck::router().layer(Extension(DaemonMode::Development));
axum::serve(listener, app).await?;
```

## The three rendering modes (the matrix we're testing)

| Mode | What deck does | Slice |
|---|---|---|
| **SSR + hydrate** | Per-view: compile `.sail` → SSR HTML snapshot spliced into the host shell; client fetches `/_deck/views/{name}/view.js` and `mount()`s. Visit `/_deck/view/{name}` to load. | 1–3 |
| **SPA** | `deck build` bundles one app entry (split, hashed); framework external via importmap; client mounts everything. | 4 |
| **Static** | `deck build --bundle_framework` — SPA + materialized `/_deck/*`, deployable with no daemon. | 4 |

Orthogonal: `DaemonMode::Development` (no-cache + reload) vs `Production`
(ETag + immutable cache).

## Slices (feature by feature)

- [x] **1 — SSR baseline + assets.** Controller-less route, on-the-fly compile,
  no build step. `tests/ssr_baseline.rs` (3 tests).
- [x] **2 — Controller → bundling.** `routes/counter` (signal + controller)
  forces an on-demand Rolldown route bundle in `<app>/.deck/`. Integration:
  `tests/bundling.rs` (2 tests). End-to-end (real browser): `e2e/counter.sh` —
  SSR → hydrate(0) → click → 1, proving the whole loop with no daemon/ship.
- [x] **3 — Reload.** `deck_dev_server` runs the bundler's FS watcher and wires
  the `/_deck/dev/hmr` SSE (bridging `subscribe_events()` → the live-reload
  client; the route `router()` doesn't provide). `tests/reload.rs`: the **loop
  net** — one edit fires *bounded* rebuilds then *settles* (guards the watch.rs
  read/atime self-trigger fix) — plus the HMR endpoint is served as an SSE
  stream. The e2e's `/_deck/dev/hmr` 404 is gone.
- [x] **4 — SPA + static.** `spa/` is a client-mounted entry (`index.html` +
  `src/main.ts` + `src/chrome/layout.css`). `tests/build.rs` asserts `deck
  build`'s dist output — hashed entry chunk + hashed CSS + rewritten HTML, with
  the runtime left external via the importmap. `e2e/static.sh` proves the
  **static deploy**: `deck build` → serve `dist/` with a dumb static server
  (`e2e/static-server.py`) → the counter renders + increments in a browser with
  **no daemon, no deck, no ship**.

Run the e2e flows (need `cargo build -p deck-conformance` / `-p deck-cli`; evidence
lands in `.artifacts/web-qa/deck-conformance/`):
- `crates/deck/deck-conformance/e2e/counter.sh` — SSR + hydration loop (Slice 2).
- `crates/deck/deck-conformance/e2e/static.sh` — static deploy, no daemon (Slice 4).

## The ship-coupling line

`deck::router()` deliberately **excludes** `/api/_meta`, the HMR-state
endpoint, and the live-data SSE/data/mutation endpoints — those need
host-supplied extensions or router state. So this pure-deck app gets routing +
SSR + assets + bundling + reload, but **live data is the host's job**. A later
slice stands up a tiny non-ship data port (or a test axum/hono server) to
exercise a live query() subscription without pulling in ship.

## Findings (deck gaps surfaced while building this)

Fixed (each backed by a harness test):

- **✅ env vars no longer name "ship" (Fix C).** deck-core now reads the
  canonical `DECK_PROJECT_DIR` / `DECK_FIRST_PARTY_VIEWS`, falling back to the
  legacy `SHIP_*` so unmigrated hosts (shipd) keep working. `deck_app` sets
  `DECK_PROJECT_DIR`. *Remaining:* config is still a process-global env, not an
  explicit parameter — and the `SHIP_*` fallback should be deleted once shipd
  migrates.
- **✅ HMR SSE bridge lives in deck-axum (Fix A).** `deck_axum::serve_bundle_hmr`
  subscribes to the bundler's event firehose directly and maps it to the SSE
  shape the live-reload client expects — the one HMR handler every host mounts.
  deck-conformance uses it (no hand-rolled variant). *Remaining:* shipd should adopt it
  and delete its global-channel forwarder (`sui_hmr.rs` + `publish_bundle_event`)
  — deferred to avoid growing the `feat/window-actors` merge surface pre-merge.
- **✅ `--bundle_framework` degrades to singletons for island-free apps (Fix B).**
  The self-contained build now skips ship-static island targets whose entry is
  absent and ships just the four framework singletons instead of bailing on a
  missing island dir. *Remaining:* the `deck build` **CLI** still resolves
  `repo_root` via `find_repo_root` (the Cargo workspace root), so an *in-tree*
  app pulls the workspace's islands — framework origin should be decoupled from
  the workspace root (then `e2e/static.sh` could drop its singleton-staging).

Still open:

- ~~**No `deck serve` / `deck dev`.**~~ **Shipped:** `deck dev` is now a real
  CLI subcommand (standalone dev server — CSR/SSR routing, HMR, no live data).
  `lib.rs::deck_dev_server` here remains the wired-demo variant (with the
  conformance `DemoStore`) the standalone CLI deliberately does not bundle.
- **`Bundler` construction couples to `ship_substrate::ShipHome`.** Its
  constructor takes `&ShipHome` (used only to locate `sail-compiler` + a staging
  dir). A standalone deck should accept a plain compiler path + staging dir.
- **Route-host shell references ship surfaces.** It links
  `/api/assets/route-tokens.css` (and the browser requests `/favicon.ico`); a
  non-ship host 404s both — the only console errors left. deck should serve a
  default/empty token sheet, or not emit the link when no token source is set.
- **Bundler out-dir must equal the resolver's project root.** `resolve_route_js`
  reads `<DECK_PROJECT_DIR>/.deck`; the bundler writes `<repo_root>/.deck`.
  `deck_app` sets `repo_root = app_dir` so they agree — but nothing enforces it,
  so a host that sets them apart silently fails to find built bundles.
