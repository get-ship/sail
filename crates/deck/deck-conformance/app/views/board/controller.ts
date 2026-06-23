// Authored controller for the `board` control-flow proof — a REAL
// `controller({ scope })` DSL export (not the spike's `() => scope` factory).
//
// The synthetic controller composes this by running its `setup(ctx)` — the
// same `resolveController` the browser hydrate path runs — so the authored
// state + methods resolve identically server-side (ship-js SSR) and client-
// side (jsdom mount). The contributed scope type is recovered from the
// `ControllerExport` so the template's `items()`/`add()`/`remove(id)` reads
// type-check against it.
import { controller, signal } from '@ship/sui-runtime'

export interface BoardItem {
  id: string
  label: string
  flagged: boolean
}

export default controller({
  scope: () => {
    const items = signal<BoardItem[]>([
      { id: 'a', label: 'Alpha', flagged: false },
      { id: 'b', label: 'Bravo', flagged: true },
    ])
    const add = (): void => {
      const n = items().length
      items.set([...items(), { id: `n${n}`, label: `Item ${n}`, flagged: false }])
    }
    const remove = (id: string): void => {
      items.set(items().filter((i) => i.id !== id))
    }
    return { items, add, remove }
  },
})
