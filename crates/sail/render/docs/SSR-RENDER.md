# SSR-RENDER — compile → controller initial state → takumi

**Status:** shipped slice (static projection). Full ship-js SSR remains the production serve path.

## Goal

Agents and eval grade **static** screenshots via `SailRender/Screenshot` / `sail render`.
Controller-driven pages must not paint as **empty shells** when the first-paint
state is knowable without a browser.

Target pipeline:

```text
.sail  →  compile / lower  →  SSR with controller initial state  →  takumi
```

## How controller initial state is obtained

| Source | When | What we get |
|--------|------|-------------|
| **SSR defaults (static projection)** | Always-on in `sail-render` | Parse the inline `<controller>` (or sibling controller when wired later), take `signal(literal)` / `signal([...])` / `signal({...})` inits for returned scope members (S-tier literals). Project those values into the template. |
| **Explicit initial** | Same as defaults | Author-written `signal(0)`, `signal("Hi")`, `signal([{…}])` **are** the explicit first-paint contract. |
| **Eval of setup (deck-ssr / ship-js)** | Production `render: ssr` serve; optional future opt-in for Screenshot when a manifest + V8 engine are present | Full `controller(setup)` run on the server DOM: queries, derived, control flow, host `route` / data. |

This slice implements **row 1–2**. Row 3 already exists in `deck-ssr` for the HTTP serve path; Screenshot does **not** pull V8 into the default `sail` binary (workspace default-members exclude `deck-ssr`).

### What counts as a projectable init

- Numbers, strings, booleans, `null` / `undefined`
- Array / object literals composed of the above (no function calls, no `new`, no spreads of non-literals)
- `signal<T>(init)` generics are stripped before parsing the init

**Not projectable (remain dynamics / empty or unevaluated branches):**

- `signal(window…)` / timers / storage / `matchMedia` (tier **C**)
- `derived(() => …)` (even pure) — needs real eval; left for deck-ssr
- `query()` / client data — needs host DataPort + ship-js settle
- Event-handler-only state (`mount` mutations) — not first paint

## Seam in SailRender / Screenshot

```text
render(source)
  │
  ├─ build_inspect_tree(source)          # author identity (pre-projection)
  │
  ├─ ssr_project::project(source)        # NEW seam
  │     extract <controller>
  │     parse returned signal() inits
  │     expand <if>/<for> when cond/of is known
  │     substitute {name()} text when known
  │     residual exprs → dynamics list
  │
  ├─ preprocess(projected)               # sx → class, layout → div, skins CSS
  ├─ strip remaining {expr} leaks        # honesty for unknown dynamics
  ├─ taffy layout + paint styles
  └─ takumi raster
```

Call sites (unchanged API):

- `sail_render::render_source` / `render_file`
- `sail op SailRender/Screenshot`
- `sail render <file>`

Optional future hook (not required for this slice): pass through pre-SSR'd HTML
from `deck_ssr` / `SyntheticSsr::first_paint` into the paint half only
(CSS still from preprocess). That reuses the production engine without making
V8 a default-build dependency of Screenshot.

## Empty-page failure class

**Symptom:** PNG / regions show chrome (page bg, empty columns) but **no
controller-owned copy or list rows**. Agents grade "blank content" even though
CSR would show data after hydrate.

**Canonical repro (fixture):** `tests/fixtures/controller-initial.sail` —
`signal("…")` title + `signal([{label}])` list. Before this work the projected
HTML had empty `<h1>` and no rows; after, title text and row labels are present
in `PipelineOutput.html` and in the raster.

**Regression detection:**

1. Unit/golden: fixture HTML **must contain** the projected title and at least
   one list label substring; dynamics must **not** list the projected signal
   reads as unresolved `dynamic-expr:{title()}` (they may still list true
   client-only exprs).
2. Paint smoke: non-trivial non-background pixel count on the controller fixture
   (same spirit as existing `golden_legacy_fixture_still_renders`).
3. PARITY paint/geometry suites stay on **static-only** fixtures until a
   follow-up deliberately adds controller fixtures under the SSR-defaults
   contract (see `tests/parity/README.md`).

## Residual limits

- No full CSR fidelity: no timers, no `query()`, no pure-`derived` evaluation.
- Nested components / custom elements still paint as placeholders when unknown
  to the static layout tree.
- `deck-ssr` isomorphism (hydration markers, form PE actions) is **not**
  reproduced; static projection is paint-honest for S-tier initials only.
- **Unevaluable `<if>` / `<for>`:** both branches / the loop body are **dropped**
  from the paint tree (empty hole + `dynamics` entry for the cond/`of` expr).
  We deliberately do **not** leave raw markup that would rasterize then+else
  as sibling static content. Agents see a missing branch, not double content.
- Existing RENDER-1 goldens are static templates — projection is a no-op for
  them when there is no controller / no projectable control-flow or expr.

## Related

- PARITY-1 §3 (helm mission + `tests/parity/README.md`)
- `deck-ssr` / `deck_core::synthetic_ssr` — production first-paint engine
- RENDER-1 — L1 paint fidelity (chrome), orthogonal to controller content
