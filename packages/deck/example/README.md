# Mise — the deck example app

**Mise** is a small, fully **ship-free** recipe box that exercises the
[`deck`](../) web framework end-to-end. It is the canonical deck example: a
standalone `routes/` SPA that depends only on framework packages
(`@ship/sui-runtime`, `@ship/sui-dom`, `@ship/tokens`, `@ship/sui-primitives`) —
no ship product, no daemon, no SDK client.

> `@ship/*` here is the org npm **scope** for the framework packages. The app
> names nothing of the ship product (no `ship.*` clients, no `.ship/` dirs, no
> `@ship/sdk-client`, no daemon assumption, no host routes).

## What it demonstrates

| deck feature | where |
| --- | --- |
| **Index + nested routes** | `routes/page.sui` (`/`), `routes/recipes/page.sui` (`/recipes`), `routes/kitchen/page.sui` (`/kitchen`) |
| **Dynamic `[param]` route** | `routes/recipes/[id]/page.sui` — reads the param as a **signal**: `route.params.id()` (note the `()`) |
| **Layout chain** | `routes/_layout.sui` mounts the `<sui-app-shell>` chrome (header, nav, theme toggle); each route renders into its `<slot name="route">` |
| **Reactivity / controllers** | `signal(...)` controllers: a counter, a toggle, and a form (`routes/kitchen/`) |
| **Client-side data client (clients seam)** | `deck.jsonc#entry` wires `src/clients.ts`'s `data` export; views read it as `ctx.clients.data`. This is an in-memory **client signal**, not server live `<data source>` — standalone deck has no live-data transport (that's a host capability / #32). |
| **Form / mutation** | the kitchen "Add a recipe" form appends to the shared client signal, then navigates to the new recipe |
| **Component packs** | real `@ship/sui-primitives` elements — `<sui-badge>`, `<sui-switch>` — plus the local `<sui-app-shell>` chrome |
| **Theming** | `@ship/tokens` light/dark; the header toggle flips the `.dark` class on `<html>` |

## Run it

deck resolves the framework internally — a standalone deck app needs only
`package.json` + `routes/`, so **no workspace install or `build-assets` step** is
required to build.

```bash
# from the repo root

# build a static bundle to packages/deck/example/.deck/
cargo run -p deck-cli -- build --project packages/deck/example

# run the dev server (CSR SPA + live-reload) on http://127.0.0.1:7880
cargo run -p deck-cli -- dev --project packages/deck/example --addr 127.0.0.1:7880
```

### Browser tests

A small Playwright suite drives the **real** `deck dev` CLI and asserts the
runtime behavior (home renders inside the chrome, the `[id]` param echoes, the
counter increments, the form mutates client state + navigates):

```bash
cd packages/deck/example
bunx playwright test        # headless; boots `deck dev` via the webServer hook
```

The dev `routes/` SPA path is **CSR**: the served HTML is a shell
(`<meta name="deck-render" content="csr">` + an empty `#root` + the
importmap), and route content appears after the client bundle mounts. The dev
server streams live-reload SSE that never idles, so tests wait on `load` +
locators, never `networkidle`.

## Layout

```
packages/deck/example/
├── deck.jsonc                 # host entry bootstrap → clients seam (data)
├── package.json               # framework deps only (workspace:*)
├── src/
│   ├── clients.ts             # in-memory `data` client (reactive signal of recipes)
│   └── sui-app-shell/         # the layout chrome component (+ theme-toggle controller)
├── routes/
│   ├── _layout.sui            # root layout → <sui-app-shell> with a route outlet
│   ├── page.sui  + style.css  # home
│   ├── recipes/               # list (reads ctx.clients.data)
│   │   ├── page.sui + controller.ts + style.css
│   │   └── [id]/              # dynamic detail (route.params.id())
│   └── kitchen/               # counter + toggle + add-recipe form (mutation)
└── tests/app.spec.ts          # Playwright runtime checks
```

Generated files (`.deck/`, `routes.gen.ts`, `entry.gen.ts`, `components.gen.ts`)
are produced by `deck build`/`dev` and are git-ignored.
