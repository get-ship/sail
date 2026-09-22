# IMPORTS.md — fixture → runtime specifier → source mapping

Branch: `job/bakeoff-compile-away` · pinned head `1991ce2b4`
Emitter: `crates/sail/compiler/src/emit/synthetic/render.rs` (`emit_render`, lines 182–205)
— per-feature named imports on three specifiers: `/internal` (ABI),
`/actions` (use: actions), `/motion` (scrub-lane values).

Both fixtures import ONLY from `@ship/sail-runtime/internal`. Neither uses an
action (`/actions`) nor motion (`/motion`), so those specifiers are never
emitted — the corresponding modules cannot enter the graph.

## counter.js

| Specifier | Named imports | Runtime src file(s) |
|---|---|---|
| `@ship/sail-runtime/internal` | `__tpl` | `src/tpl.ts` |
|  | `bindText` | `src/bind.ts` |
|  | `component` | `src/component.ts` |
|  | `delegateEvents` | `src/delegate.ts` |
|  | `markHydrateText` | `src/hydrate-mark.ts` |
|  | `on` | `src/on.ts` |
|  | `prop` | `src/component.ts` (re-exported from root barrel; factory lives beside `component`) — see note |
|  | `signal` | `src/signal.ts` |

Note on `prop`: the emitter treats it as an ABI name, but its value object is
defined in `src/component-factory.ts` and re-exported through
`src/component.ts` (`export { component, prop, defineComponent }`) and then via
the root barrel `src/index.ts` (`export { component, prop, … }`), which
`internal.ts` star-re-exports. Importing `prop` therefore pulls
`component-factory.ts` into the graph.

Local controller import: `./counter.sail.controller.js` →
`controller()` from `src/controller-resolver.ts` + `signal` from `src/signal.ts`.

## list.js

| Specifier | Named imports | Runtime src file(s) |
|---|---|---|
| `@ship/sail-runtime/internal` | `__tpl` | `src/tpl.ts` |
|  | `bindFor` | `src/for.ts` |
|  | `bindText` | `src/bind.ts` |
|  | `component` | `src/component.ts` |
|  | `delegateEvents` | `src/delegate.ts` |
|  | `markHydrateText` | `src/hydrate-mark.ts` |
|  | `on` | `src/on.ts` |
|  | `prop` | `src/component.ts` / `src/component-factory.ts` (see counter note) |
|  | `signal` | `src/signal.ts` |

The only delta vs counter is `bindFor` (`src/for.ts`, +2.7 KB minified) plus
its edge to `src/transition-runner.ts` (enter/leave hooks, 54 B).

## Transitive closure actually pulled in (per esbuild metafile)

`on.ts`, `bind.ts`, `for.ts` reach `signal.ts` → `effect-scope.ts` →
`error-boundary.ts` / `error-overlay.ts`. `component.ts` reaches
`element-scope.ts`, `host-api.ts`, `compose.ts`, `controller-resolver.ts`,
`navigate.ts`, `host-route.ts`, and — through `component-factory.ts` —
`component-form.ts`, `slot-project.ts`, `styles/contracts-seam.ts`,
`component-decoders.ts`, `decode.ts`. The root barrel `index.ts` (via
`internal.ts`'s `export * from './index'`) drags in `theme.ts`,
`theme-session.ts`, `theme-catalog.ts` → `@ship/tokens/themes`
(see RESULTS.md for why that survives tree-shaking).
