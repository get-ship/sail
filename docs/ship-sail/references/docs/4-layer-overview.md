---
group: Views
order: 2
title: SAIL — Four-Layer Overview
description: The compile/runtime/host/chrome layering of a SAIL view.
---

# SAIL — Four-Layer Overview

SAIL splits cleanly into four layers. Each owns one concern and exposes a narrow contract to the next.

```
page.sail (with inline <controller>)     (authoring — what humans write)
    ↓ crates/sail/compiler/ (AOT compile — emits ESM + .d.ts + .ssr.html)
.js + .sail.d.ts + .ssr.html  (compiler output — gitignored under dist/)
    ↓ packages/sail/runtime/  (reactive primitives + DOM bindings + attach())
runtime in the browser       (signals, control-flow, transitions, error-boundary)
    ↓ crates/deck/deck-core (daemon view-host — resolver + assets + SSR + SSE)
HTTP /_deck/* + /view-host    (apps/shipd routes wire it together)
```

## Layer 1 — Compiler (`crates/sail/compiler/`)

Pure transformation. No filesystem walk, no daemon, no network. Library entry: `crates/sail/compiler/src/lib.rs`.

```rust
pub fn compile(source: &str, opts: CompileOpts) -> Result<CompileOutput, CompileError>
```

`CompileOpts` knobs: `setup_url` (virtual `<file>.sail.controller.ts` import for the source's inline `<controller>` block — nothing on disk; a template with no block gets no setup import), `mode` (`Component | Auto`), `source_path` (diagnostic spans), `component_meta` (tag name + shadow flag), `styles` (component-only external CSS), `emit_dts`, `emit_component_imports`.

`CompileOutput` carries:

| Field | What it is |
|---|---|
| `js` | ESM module that imports from `@ship/sail-runtime`. Constructs the render tree directly — no parser ships to the client. |
| `css` | Extracted / passed-through CSS (component `<style>` blocks, `styles` opt), present when the source carries styles. |
| `cem` | Custom Elements Manifest (components only) — fuel for editor tooling. |
| `schema` | Prop schema JSON — read by `decode.ts` to validate runtime prop values. |
| `dts` | Typed `Scope` for the inline `<controller>` block. Includes both `<data>` names and the controller function's returned keys. |
| `meta` | `.sail.meta.json` — binding kinds / exprs / deps / renderKey. Developer-tooling fuel; the runtime never loads it. |

Pipeline (`src/lib.rs:185` `compile()`): `parse::parse` → `check::run` → `bindings::build` → `lower::lower` → `emit::js::emit_module` (+ `emit::cem`, `emit::schema`, `emit::dts`, `emit::meta`). SSR is no longer a compiler concern — the host (ship-js) renders the module server-side. Diagnostics carry byte spans (`src/diagnostics.rs`).

Binary: `crates/sail/compiler/src/bin/sail-compiler.rs` (the `sail-compiler` Cargo bin). Used by the deck bundler and editor extensions for one-shot compiles.

## Layer 2 — Runtime (`packages/sail/runtime/`)

Public API barrel: `packages/sail/runtime/src/index.ts`. Anything not re-exported there is internal.

| Surface | Symbols |
|---|---|
| Reactivity | `signal`, `derived`, `effect`, `batch`, `untrack`, `createScope`, `createChildScope`, `onCleanup` |
| DOM bindings | `bindText`, `bindShow`, `bindAttr`, `bindClass`, `bindProp`, `bindValue`, `bindChecked`, `bindHtml`, `bindRawHtml` |
| Events | `on`, `matchesKey` |
| Control flow | `bindFor`, `bindForSignal`, `bindIf` |
| Projection | `projectSlots`, `resolveSlots`, `snapshotChildren`, `slotFilled`, `bindSlotEmpty` (light-DOM slot redistribution + empty-reflection; `projectSlots` is the runtime path the component factory uses at connect) |
| Components | `defineComponent`, `component`, `prop`, `camelOf` |
| Prop decoders | `decodeString`, `decodeNumber`, `decodeBoolean`, `decodeJson`, `decodeEnum` |
| Data | `data`, `query`, `unwrap`, `unwrapOr`, `wireFormMutation` |
| State helpers | `state`, `debounce`, `useFilteredList`, `useSelection`, `useForm` |
| Shared context | `shared`, `isShared`, `provide`, `useShared`, `pickShared` |
| Host API | `emit`, `expose`, `attachHostApi`, `bindNativeEvent`, `slotted`, `bindHostAttr` |
| Transitions | `registerTransition`, `runIn`, `runOut`, `runEnter`, `runLeave`, `cancelTransitions`, `fade`, `slide`, `scale` |
| Morph / boundary | `morph`, `attachBoundary`, `detachBoundary`, `findNearestBoundary`, `runProtected` |
| While-states | `bindWhileLoading`, `bindWhileError` |
| Controller | `controller`, `resolveController`, `LIFECYCLE_KEY` |
| Template hooks | `registerTemplateHook`, `getTemplateHook`, `clearRegistry`, `listRegisteredHooks` |
| Mount | `mount(target, payload, opts) -> MountHandle` |
| Wire-contract types | `ManifestRail`, `ViewListEntry`, `ViewManifest`, … (daemon ↔ runtime contract; committed at `packages/sail/runtime/src/generated.d.ts` — a hand-maintained mirror of the Rust wire types, no `gen-types` task exists in this repo) |

`CompileOpts` knobs: `setup_url` (virtual `<file>.sail.controller.ts` import for the source's inline `<controller>` block — nothing on disk; a template with no block gets no setup import), `mode` (`Component | Auto`), `source_path` (diagnostic spans), `component_meta` (tag name; shadow is no longer a per-component opt — it's declared in the anatomy manifest and only paid for by components that wrap foreign content), `styles` (component-only external CSS), `emit_dts`, `emit_component_imports`.

`CompileOutput` carries:

| Field | What it is |
|---|---|
| `js` | ESM module that imports from `@ship/sail-runtime`. Constructs the render tree directly — no parser ships to the client. |
| `css` | Extracted / passed-through CSS (component `<style>` blocks, `styles` opt), present when the source carries styles. |
| `cem` | Custom Elements Manifest (components only) — fuel for editor tooling. |
| `schema` | Prop schema JSON — read by `decode.ts` to validate runtime prop values. |
| `dts` | Typed `Scope` for the inline `<controller>` block. Includes both `<data>` names and the controller function's returned keys. |
| `meta` | `.sail.meta.json` — binding kinds / exprs / deps / renderKey. Developer-tooling fuel; the runtime never loads it. |

The dev-only leak detector lives at the subpath `@ship/sail-runtime/dev` so it doesn't ship in production bundles.

`<sail-if>`, `<sail-for>`, `<sail-match>` custom elements register on module load (`import './boundary-elements'` at the bottom of `index.ts`).

## Layer 3 — Daemon view-host (`crates/deck/deck-core/`, `crates/deck/deck-axum/`, `apps/shipd/`)

The view-host is split into a transport-agnostic core and an HTTP adapter. Core layout (`crates/deck/deck-core/src/`):

| File | Owns |
|---|---|
| `resolver.rs` | Walks `views/` + `.ship/views/` + project mirror trees, picks the manifest for a route, resolves `[param]` segments. |
| `view.rs` | Serves `.sail` source + compiled `.js` + scoped `.css` (`serve_sui_view_template`, `serve_sui_view_setup`, `serve_sui_view_css`). |
| `view_payload.rs` | Static SSR HTML payload and the `ViewPayload` type (`serve_view_payload`, `serve_view_payload_subpath`). |
| `view_host.rs` | The view-host HTML shell that hydrates the payload (`serve_view_host`). |
| `meta.rs` | Drift-detection metadata (`GET /api/_meta`). |

The HTTP adapter (`crates/deck/deck-axum/src/handlers.rs`) wires the core functions to Axum routes. Ship wiring lives in `apps/shipd/src/views/sui_*.rs`.

The core crate exposes `DaemonMode::{Development, Production}` — drives cache headers, HMR availability, and the SSR cache.

Daemon-side route table (`apps/shipd/src/router.rs`) mounts the surface:

| Route | Handler |
|---|---|
| `GET /_deck/runtime.js` / `dom.js` / `sdk-client.js` / `components.js` | `crates/deck/deck-core/src/assets.rs` |
| `GET /_deck/icons/{name}.svg` | `crates/deck/deck-core/src/assets.rs` (per-icon lucide; daemon embeds the set) |
| `GET /_deck/sail-components/{name}.js` | separate-bundle component |
| `GET /_deck/views/{name}/view.sail` / `view.js` / `view.css` | `crates/deck/deck-core/src/view.rs` (route segment stems are `view.*` regardless of on-disk `page.sail`/`style.css`) |
| `GET /_deck/views/{name}` | `crates/ship-views/src/view_payload.rs` (extracted out of deck into the `ship-views` crate) |
| `GET /_deck/views/{name}/sse` | `apps/shipd/src/views/sui_patch_channel.rs` |
| `GET /_deck/hmr` / `hmr-client.js` / `dev/hmr` | `apps/shipd/src/views/sui_hmr.rs` |
| `GET /_deck/data` | `apps/shipd/src/views/sui_data.rs` (the `<data live>` SSE channel) |
| `POST /_deck/mutation/{name}` | `apps/shipd/src/views/sui_mutation.rs` |
| `POST /_deck/views/{name}/patch-elements` | `apps/shipd/src/views/sui_patches.rs` |
| `GET /_deck/source/{tag}` | `apps/shipd/src/views/sui_source.rs` |

The router wiring is in `apps/shipd/src/router.rs`. Signal-backed sources (tighter wake-up than the default `config` kernel namespace) live under `apps/shipd/src/views/sui_data_resolvers/` and `apps/shipd/src/views/git_signal.rs`; `sui_sse.rs::is_signal_backed_expr` picks them up by registry name.

## Layer 4 — Authoring (`views/<name>/`, `packages/components/ui-components/src/<tag>/`)

Directory shapes:

```
views/<name>/                    # first-party views
  page.sail                       # required — controller, if any, is an inline <controller> block inside
  style.css                      # optional
  manifest.jsonc                 # required
  dist/                          # compiler output (gitignored)
  _components/<view>-<tag>/      # view-private widgets
    <view>-<tag>.sail

.ship/views/<name>/              # project-local extension views; same shape

packages/components/ui-components/src/<tag>/   # first-party components
  <tag>.sail                      # inline <controller> block, if any, lives here
  <tag>.css                      # optional

packages/components/ui-components/src/<tag>/   # separate-bundle (lazy) components also live here;
                                    # they opt in via `<bundle separate />` in their .sail
```

Five rules carried over from the `ship-views` skill:

1. **`page.sail` is the route entry.** Filename stem says what the file IS — a page, not a reusable widget.
2. **The controller is inline, no sibling file.** A `<controller>` block lives inside `page.sail` itself; a paired `<stem>.controller.ts` sibling file is a compile error.
3. **`style.css` for page-scoped styles.** Use `<style>` blocks inside `.sail` for component-scoped styles.
4. **`manifest.jsonc` is required for views.** JSONC, not YAML.
5. **`_components/` for view-private widgets.** Tag MUST prefix with the view name (`home-hero` for `views/home/`) so two views can each ship their own widget without colliding on `customElements.define`.

Every first-party view is on this shape — `views/agents/`, `views/git/`, `views/home/`, etc. all ship `page.sail` + `style.css` + `manifest.jsonc`. The router (`@ship/sail-runtime/route + defineRoutes() in apps/shell/src/main.ts`) walks only manifest-bearing directories.

## How the layers compose

A request for `/agents` resolves end-to-end:

1. Browser hits `GET /agents` → daemon serves the view-host HTML shell (`serve_view_host`).
2. Shell loads `/_deck/views/agents` → SSR payload (compiled HTML + manifest).
3. Shell loads `/_deck/views/agents/view.js` → compiled ESM module.
4. Module imports from `/_deck/runtime.js` (the SAIL runtime barrel) and `/_deck/sdk-client.js`.
5. `attach()` walks the SSR DOM, binds reactive identifiers from the module's controller, opens `<data live>` SSE channels to `/_deck/data?expr=…`.
6. Edits in dev mode emit `module-changed` on `/_deck/dev/hmr`; `apps/shell/src/hmr.ts` cache-busts the rebuilt `view.js` and re-mounts in place.
