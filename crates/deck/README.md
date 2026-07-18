# deck

deck is Ship's **host / meta-framework** — the layer that mounts SAIL views,
does SSR + routing, serves every asset, and supplies views with a data-source
client. It is to SAIL what Next is to React.

```
SAIL  :  Deck  :  Ship      ::      React  :  Next  :  your app
```

Dependencies point one way only: **deck → sail**, **ship → deck**. The SAIL
compiler depends on nothing downstream; deck names no Ship domain or daemon URL;
Ship (the `shipd` daemon) is the app that mounts deck and implements the one
data seam deck asks for. A boundary lint (`scripts/lint/*`,
`deck-core/tests/boundary.rs`) enforces that deck's code names zero Ship
concepts.

## What deck owns

**All frontend serving.** HTML scaffolds, framework bundles
(runtime/dom/sdk-client), per-icon lucide, design tokens + theme CSS, per-view
compiled JS + SSR payload, the view-host, space/shell scaffolds, the SPA
navigation handler, and the dev HMR firehose. If a byte reaches the browser, a
deck handler served it. `shipd` serves no frontend — it is RPC + daemon only.

## Rendering & build modes

deck runs **standalone** (`deck dev` / `deck build` — no shipd) OR **mounted in a
host** (shipd merges `deck::router()`). The frontend has three modes, each proven
end-to-end by the browser harness (`packages/deck/e2e`):

| Mode | Command | What the browser gets |
|---|---|---|
| **Dev — CSR SPA** | `deck dev` (default) | An empty `#root` + importmap shell; the client entry mounts the file-routed `routes/` SPA and renders. `<a>` nav is client-side (no document reload). |
| **Dev — SSR** | `deck dev`, `render: "ssr"` in `deck.jsonc` | The matched `routes/<…>/page.sail` is server-rendered (via ship-js) INTO `#root`; the client hydrates it. |
| **Prod build** | `deck build --bundle-framework` | A self-contained static dir: `index.html` (importmap) + `_deck/{runtime,dom,route}.js` (framework singletons, materialized) + `assets/page-<hash>.js` (one **code-split, lazily-loaded** chunk per route). |

**Routing.** File-routed: `routes/<seg>/page.sail` (state in the page's inline `<controller>` block); a
`[param]` dir segment is a dynamic route (`routes.gen.ts` emits a typed `:param`
pattern). The bundler generates the entry; you write pages.

**HMR (dev only).** Editing a route's `.sail` soft-reloads it with **state
preserved** (no full document reload); a `<sail-*>` component edit **hot-swaps**
live instances in place.

**Serve-time vs baked.** `deck dev` injects the `deck-render`/`deck-mode` metas
and the live-reload client at serve time; a `deck build` artifact carries **none**
of that — a shipped build is clean.

**Static hosting.** By default a build references app assets under the per-app
serve namespace (`/<name>/<file>`, which a host maps to `assets/`). For a build a
dumb static host (Netlify/S3/nginx) can serve directly, pass **`--base assets`** —
asset URLs become `/assets/<file>`, matching the on-disk layout. (`deck build
--bundle-framework --base assets`.)

## Crate map

| Crate | Role |
|---|---|
| `deck-core` | Transport-agnostic hosting logic. Produces a `DeckResponse` (status + bytes + headers); never speaks HTTP. Owns view resolution, SSR payload assembly, token/theme CSS, the setup flow, and the `DataPort` trait. |
| `deck-axum` | The axum adapter. Thin `serve_*` shims map `DeckResponse` → `axum::Response`. `router()` registers deck's whole serving surface as one mountable unit; `multiplex_router()` is the live-data transport — **host-wired, not mounted by `deck dev`** (see the data seam below). |
| `deck-host` | Pure host helpers shared across adapters: `render_helpers`, `resolver` (project-root + view resolution), `route_payload`. |
| `deck-bundler` | The one bundler (#163). In-process Rolldown; a Rust plugin compiles `.sail` via the linked `sail-compiler`. Builds framework singletons, packs, views, and app scaffolds. |
| `deck-config` | Parses `deck.jsonc` into `DeckConfig` — the host's single configuration surface (below). |
| `deck-cli` | The `deck` binary: `dev`, `build`, `build-assets`, `routes`, `emit`, `scaffold {view,component,pack}`. The `next`/`vite` analog. |
| `deck` | Thin re-export facade so a host writes `deck::*` for everything above. |

## The data seam — `DataPort`

deck mounts views but owns no data. The host implements one trait
(`deck_core::DataPort`) and injects it:

```rust
pub trait DataPort {
    /// Resolve a live-data subscription's current value. `None` → the host has
    /// no resolver for `(domain, op)`; deck surfaces an explicit error, never a
    /// silent null.
    fn resolve(&self, domain: &str, op: &str, args: &Value) -> Option<Value>;

    /// Whether the host is provisioned. deck's `/` and `/setup` handlers branch
    /// on this instead of importing any Ship type. Defaults to `false`.
    fn is_provisioned(&self) -> bool { false }
}
```

The reference implementor is `shipd`'s `ShipDataPort` (in the `shipd` repo —
deck names it but does not contain it). A host merges `deck::multiplex_router()`
for the live-data transport (`/_deck/mux/*`) and layers its `DataPort` as an
`Extension`. This is the sole coupling point between deck and the app — deck
stays host-generic; the app keeps its domains.

**Standalone has no live data (yet).** `deck dev` / `deck build` serve routing,
assets, SSR and HMR, but do **not** mount the multiplex or any real `DataPort`,
so a `<data source="ship.x.y()" live />` binding is **not served** standalone —
live data is a host capability today. Making it work without a host (JS server
functions backing `DataPort::resolve`) is the server-functions arc (#32),
targeted for a later release. Standalone views should render without server
`<data>`, or run under a host (shipd) that wires the seam.

## Configuration surface — `deck.jsonc` / `DeckConfig`

Everything Ship-specific about the layout lives in `deck.jsonc`, not in deck's
code. Key fields (`deck-config/src/lib.rs`):

- `targets[]` — consumer bundle targets (`name`, `entry`, `externals`, `kind`).
- `sourceRoots` / `componentSourceRoots` — where `.sail` and `@ship/<pack>`
  components resolve from. The host declares its monorepo layout here; deck's
  resolver names none of it.
- `packs[]` — the per-tag component resolver's roots.
- `viewDirs` / `viewLeaf` — where views live and the entry filename.
- `watch[]` — dev-watch paths.

The Ship app list (hub, spaces) lives in `apps.jsonc` at the workspace
root, owned by shipd (L2). Deck knows nothing about which apps exist
or how they are routed.
A zero-config host gets sensible generic defaults (e.g. `sourceRoots: ["packages"]`).

## Mounting deck in a host (what shipd does)

```rust
let root = Router::new()
    .merge(public_routes)        // auth bootstrap, health
    .merge(deck::router())       // ALL serving — no fallback of its own
    .merge(authenticated_routes) // Connect-ops RPC (its fallback_service dispatches /ship.ops.v1.*)
    .layer(Extension(daemon_mode))
    .layer(Extension(data_port)) // the DataPort impl deck's setup/live handlers read
    .layer(Extension(deck_cfg));
```

Two composition rules the host must honor:

1. **deck's `router()` sets no `.fallback`.** It merges alongside the Connect
   router, whose entire RPC dispatch *is* a `fallback_service`. Two merged
   fallbacks panic axum, and a host `.fallback(spa)` would shadow the RPC
   dispatch into a dead data plane. deck exports `spa_fallback`; the host
   composes navigation-vs-RPC itself (shipd uses a pre-routing nav shim that
   serves the shell for client-route GETs and lets everything else reach the
   Connect fallback).
2. **The host injects `DataPort`** as an `Extension` so deck's `/` `/setup` and
   live-data handlers can read provisioning + resolve subscriptions without
   importing the app.

## CLI

deck dev --project ./my-app          # standalone dev server + HMR on 127.0.0.1:7878 — NO shipd
deck build --project apps/hub --bundle-framework
deck build-assets          # bake framework + consumer bundles (in-process Rolldown)
deck routes --root apps/shell --root apps/spaces
# walks <root>/routes/ → emits <root>/routes.gen.ts (label = basename).
# Repeatable; required at least once.
deck scaffold view <name>  # new view (page.sail + manifest.jsonc)
deck scaffold component <tag>
deck scaffold pack <name>
deck emit --target <name> --out <dir>   # copy baked targets (+ chunk closure) out
```

### `deck emit`

```sh
deck emit --target sail-runtime --target sail-marketing --out dist/_assets
```

Copy already-baked target bundles into a directory for a consumer that loads
them without the daemon (e.g. a static site build). Run it **after**
`deck build-assets` — emit copies bake output, it does not bake.

- `--target <name>` (repeatable, required) — a framework singleton
  (`sail-runtime`, `sail-dom`, `sail-sdk-client`) or a consumer target declared in
  `deck.jsonc` (`sail-marketing`, `sail-arcade`, `ship-code-editor`, …).
- `--out <dir>` — destination directory, created if absent.

deck resolves each target's output path via the same `BundleLayout` the bundler
writes through, copies the target file, then follows its relative
`import("./chunk.js")` graph and copies the whole content-hashed lazy-chunk
**closure** — so the consumer never hardcodes deck's internal output layout
(`crates/deck/deck-bundler/src/emit.rs`). The canonical consumer is
`apps/site/scripts/copy-sail-assets.ts`, which names the six targets the static
Astro build needs and shells out to `cargo run -p deck-cli -- emit`.

## See also

- `packages/deck/docs` — the deck **documentation site**, built with deck and
  served by deck. It is the canonical standalone example (a ship-free, file-routed
  app exercising routing, dynamic `[param]` routes, layouts, controllers/reactivity,
  the data seam, HMR and the static build) AND the native-path release gate
  (`bunx playwright test` drives the real CLI; 6 specs).
- `.claude/skills/sail-authoring` — authoring `.sail` views + components.
- `docs/ship-adrs/` — the architectural decisions deck implements.
