import { signal, derived } from '@ship/sail-runtime'
import type { Signal } from '@ship/sail-runtime'
import { type ListSource } from './filtered-list'

/**
 * `selection(source, keyOf)` — single-select over a list.
 *
 * Returns `{ selectedId, selected, select, clear, toggle, isSelected }`.
 *
 * ```ts
 * export const sel = selection(rows, (r) => r.id)
 * ```
 * ```html
 * {#each rows() as r}
 *   <card aria-selected={sel.isSelected(r.id)} on:click={() => sel.select(r.id)}>{r.name}</card>
 * {/each}
 * ```
 *
 * Compare this to what it replaced: a `ControllerHook` declaring `{ state, derived,
 * on }` that a resolver inflated into signals and namespaced under a `use:` key —
 * a mini-DSL, a framework feature, a thing to learn. It is forty lines of plain
 * functions instead, and it does strictly more (it hands you `isSelected`, which the
 * declarative shape had no way to express).
 */
export function selection<T, K extends string | number = string>(
  source: ListSource<T>,
  keyOf: (item: T) => K,
): {
  selectedId: Signal<K | null>
  selected: () => T | null
  select: (id: K) => void
  clear: () => void
  toggle: (id: K) => void
  isSelected: (id: K) => boolean
} {
  const selectedId = signal<K | null>(null)

  const selected = derived(() => {
    const id = selectedId()
    if (id === null) return null
    const items = Array.isArray(source) ? source : ((source as () => T[])() ?? [])
    return (Array.isArray(items) ? items : []).find((i) => keyOf(i) === id) ?? null
  })

  return {
    selectedId,
    selected,
    select: (id: K) => selectedId.set(id),
    clear: () => selectedId.set(null),
    toggle: (id: K) => selectedId.set(selectedId() === id ? null : id),
    // Reading the signal inside means a template calling isSelected(id) TRACKS it —
    // the row re-renders when the selection moves, with nothing wired by hand.
    isSelected: (id: K) => selectedId() === id,
  }
}
