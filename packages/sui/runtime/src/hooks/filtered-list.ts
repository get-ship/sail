// useFilteredList — filter a list of items by a query string.
//
// Bundles `state.query`, `derived.filtered`, `on.setQuery` / `on.clear`
// as a coherent slice. Composed into a controller via the `use:`
// section, namespaced by key.
//
// Source can be either a plain `Signal<T[]>` (e.g. a derived array)
// or a `LiveSignal<{ items: T[] }>` (a `<data live>` declaration whose
// payload follows the standard envelope). The hook detects which at
// read time and unwraps `.value.items` for the live case.

import type { ControllerHook } from '../controller'
import type { LiveSignal } from '../live'

/**
 * A source the filter reads from. Three accepted shapes — anything
 * that returns a `T[]` (or a live envelope around one) when called:
 *
 * - `() => T[]` — a plain reader. Covers `Signal<T[]>` (callable +
 *   `.set`), `derived(() => T[])`, and any other tracked-callable.
 * - `() => LiveState<{ items: T[] }>` — also covered: the function
 *   returns an envelope, the hook discriminates on `.kind`.
 * - `LiveSignal<{ items: T[] }>` — a `<data live />` declaration.
 *   Equivalent to the second form, named for clarity.
 *
 * The hook never writes to the source, so a writable Signal isn't
 * required — any reader works.
 */
export type FilteredListSource<T> =
  | (() => T[])
  | (() => { kind: 'loading' | 'error' | 'ok'; value?: { items: T[] } })
  | LiveSignal<{ items: T[] }>

/**
 * Read items out of a `FilteredListSource<T>`. Empty array when the
 * underlying live envelope is `loading` or `error`.
 */
function readItems<T>(source: FilteredListSource<T>): T[] {
  const v = source()
  // Plain array → return as-is.
  if (Array.isArray(v)) return v
  // Envelope → only 'ok' has items.
  if (v && typeof v === 'object' && 'kind' in v) {
    const env = v as { kind: string; value?: { items?: T[] } }
    if (env.kind === 'ok' && env.value && Array.isArray(env.value.items)) {
      return env.value.items
    }
    return []
  }
  return []
}

/**
 * Filter a reactive list by a query string.
 *
 * Returns a controller hook with:
 *
 * - `state.query` — the current query (mutable signal)
 * - `derived.filtered` — `T[]`, items where `predicate(item, query)` is true
 * - `on.setQuery(q)` — replace the query
 * - `on.clear()` — empty the query
 *
 * @param source A signal or live signal of items.
 * @param predicate `(item, query) => boolean`. Called with the current
 *   query string for each item; return true to include.
 *
 * @example
 * ```ts
 * // in a controller
 * use: ({ agents }) => ({
 *   list: useFilteredList(agents, (a, q) =>
 *     a.name.toLowerCase().includes(q.toLowerCase())),
 * })
 * ```
 *
 * Template:
 * ```html
 * <input bind:value={list.query} />
 * {#for a of list.filtered() key={a.id}}
 *   <ship-card>{a.name}</ship-card>
 * {/for}
 * ```
 */
export function useFilteredList<T>(
  source: FilteredListSource<T>,
  predicate: (item: T, query: string) => boolean,
): ControllerHook<{ query: string }> {
  return {
    state: { query: '' },
    derived: {
      filtered: ({ query }) => {
        const q = (query as () => string)()
        return readItems(source).filter((i) => predicate(i, q))
      },
    },
    on: {
      setQuery: (s: { query: string }, q: string) => {
        s.query = q
      },
      clear: (s: { query: string }) => {
        s.query = ''
      },
    },
  }
}
