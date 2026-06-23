// useSelection — single-select over a list of items.
//
// Bundles `state.selectedId`, `derived.selected`, `on.select` /
// `on.clear` / `on.toggle` as a coherent slice. Each item's identity
// is taken from a user-supplied `keyOf` function.
//
// Source shape mirrors `useFilteredList` — a plain `Signal<T[]>` or
// the standard live envelope `LiveSignal<{ items: T[] }>`.

import type { ControllerHook } from '../controller'
import type { FilteredListSource } from './filtered-list'

/** Read items out of a source signal — see `filtered-list.ts` for the
 * full discrimination. Duplicated here to keep the hook self-contained
 * (no cross-file private helpers). */
function readItems<T>(source: FilteredListSource<T>): T[] {
  const v = source()
  if (Array.isArray(v)) return v
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
 * Single-select over a reactive list.
 *
 * Returns a controller hook with:
 *
 * - `state.selectedId` — `K | null` (the currently-selected item's key)
 * - `derived.selected` — the current `T | null` resolved through the source
 * - `on.select(id)` — select by key
 * - `on.clear()` — clear the selection
 * - `on.toggle(id)` — select if not selected; clear if already selected
 *
 * @param source A signal or live signal of items.
 * @param keyOf `(item) => K`. Stable identity per item.
 *
 * @example
 * ```ts
 * use: ({ agents }) => ({
 *   sel: useSelection(agents, (a) => a.id),
 * })
 * ```
 *
 * Template:
 * ```html
 * {#for a of items() key={a.id}}
 *   <ship-card
 *     tone={sel.selected()?.id === a.id ? 'primary' : 'default'}
 *     on:click={() => sel.select(a.id)}
 *   >{a.name}</ship-card>
 * {/for}
 * ```
 */
export function useSelection<T, K extends string | number = string>(
  source: FilteredListSource<T>,
  keyOf: (item: T) => K,
): ControllerHook<{ selectedId: K | null }> {
  return {
    state: { selectedId: null as K | null },
    derived: {
      selected: ({ selectedId }) => {
        const id = (selectedId as () => K | null)()
        if (id === null) return null
        return readItems(source).find((i) => keyOf(i) === id) ?? null
      },
    },
    on: {
      select: (s: { selectedId: K | null }, id: K) => {
        s.selectedId = id
      },
      clear: (s: { selectedId: K | null }) => {
        s.selectedId = null
      },
      toggle: (s: { selectedId: K | null }, id: K) => {
        s.selectedId = s.selectedId === id ? null : id
      },
    },
  }
}
