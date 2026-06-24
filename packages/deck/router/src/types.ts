// Public route types — split out of `index.ts` so they can be imported
// without pulling the browser router (`mount-router` → the runtime's
// `mount()` → `./generated`). The server entry (`./server`) and any
// type-only consumer import from here; `index.ts` re-exports them so the
// public `@ship/deck-router` surface is unchanged.

import type { Signal } from '@ship/sui-runtime'

/**
 * The reactive URL handle exposed to view scope as `route`.
 *
 * Each `params.<name>` and `query.<name>` is a `Signal<string | undefined>`
 * — call it to read the current value (and subscribe the surrounding
 * reactive scope). Writing to either record throws; mutate via
 * `navigate(path)` instead.
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

/**
 * A registered route pattern (e.g. `/agents/:id`). Pattern segments
 * starting with `:` are captured as named params on `route.params`.
 */
export interface RouteShape {
  /** Path pattern, e.g. `'/agents/:id'`. */
  pattern: string
  /** Optional name for debugging. */
  name?: string
}
