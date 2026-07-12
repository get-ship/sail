import { signal, derived } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'

/**
 * A list source: a plain array, a `Signal<T[]>`, a `derived(() => T[])`, or any
 * tracked callable returning one — including a live envelope around one, which is
 * what `<data>` hands you.
 */
export type ListSource<T> = T[] | (() => unknown)

/** Unwrap whatever the source is into a plain array, tracking it if it's reactive. */
function readItems<T>(source: ListSource<T>): T[] {
  if (Array.isArray(source)) return source
  const v = (source as () => unknown)()
  if (Array.isArray(v)) return v
  // A live envelope (`<data>`): { kind: 'ok', value: { items } }.
  if (v && typeof v === 'object') {
    const env = v as { kind?: string; value?: { items?: T[] } }
    if (env.kind === 'ok' && Array.isArray(env.value?.items)) return env.value.items
  }
  return []
}

/**
 * `filteredList(source, matches)` — a query signal over a list.
 *
 * Returns `{ query, filtered, setQuery, clear }`. `filtered` is a `derived`, so it
 * recomputes only when the query or the source actually change, and any template
 * reading it re-renders on its own.
 *
 * ```ts
 * // in the <controller> block
 * export const list = filteredList(rows, (row, q) => row.name.toLowerCase().includes(q))
 * ```
 * ```html
 * <input on:input={(e) => list.setQuery(e.target.value)}>
 * {#each list.filtered() as row}<row>{row.name}</row>{/each}
 * ```
 *
 * Note what this is NOT: there is no registration, no `use:` section, no namespace.
 * The returned signals are ordinary scope members and the template binds to them
 * like anything else you declared. That is the entire integration story.
 */
export function filteredList<T>(
  source: ListSource<T>,
  matches: (item: T, query: string) => boolean = defaultMatch,
): {
  query: Signal<string>
  filtered: () => T[]
  setQuery: (q: string) => void
  clear: () => void
} {
  const query = signal('')

  const filtered = derived(() => {
    const q = query().trim().toLowerCase()
    const items = readItems(source)
    return q === '' ? items : items.filter((i) => matches(i, q))
  })

  return {
    query,
    filtered,
    setQuery: (q: string) => query.set(q),
    clear: () => query.set(''),
  }
}

/** Substring match over the item's stringified form — the sane default. */
function defaultMatch<T>(item: T, query: string): boolean {
  return String(item).toLowerCase().includes(query)
}
