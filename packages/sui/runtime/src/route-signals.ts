// route-signals.ts — SUI-local structural route type.
//
// SUI views declare their `route` scope via the app manifest (e.g.
// `route: { from: '@ship/deck-router', import: 'route', construct: true }`).
// The runtime doesn't need to know the precise deck/route implementation
// — it only needs a structural shape the render can read against. This
// is the SUI side of the contract: a view reads `scope.route.params.id()`
// and gets a `Signal<string | undefined>` back.
//
// Slice 2 — this type replaces the `import type { RouteSignals } from
// '@ship/deck-router'` in `view.ts` / `controller.ts` / `mount.ts` /
// `controller-resolver-types.ts`. The deck-side `RouteSignals` is
// structurally compatible (it imports the SAME `Signal` from
// `@ship/sui-runtime`); runtime values from the deck router satisfy
// this type at the assignment site (the deck `route` factory returns
// a structurally-compatible object).

import type { Signal } from './signal'

/**
 * The reactive URL handle exposed to view scope as `route`. SUI's
 * structural view of the deck-side `RouteSignals` — every property is
 * a `Signal` (already a SUI primitive) so the view's typed scope can
 * read `scope.route.params.<name>()` and subscribe to changes.
 */
export interface RouteSignals {
  /** Path params (e.g. `/agents/:id` → `params.id`). Lazy proxy. */
  params: Record<string, Signal<string | undefined>>
  /** Query params (e.g. `?q=foo` → `query.q`). Lazy proxy. */
  query: Record<string, Signal<string | undefined>>
  /** Pathname. Updates on every navigation. */
  pathname: Signal<string>
  /** Hash fragment, including the leading `#`. */
  hash: Signal<string>
}
