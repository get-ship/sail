import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

const ROWS = Array.from({ length: 200 }, (_, i) => ({ id: i, label: `r${i}` }))

export default controller<Scope>({
  scope: () => {
    const rows = signal(ROWS)
    const selectedId = signal(0)
    // Wrap the single selection in a Set so the template can use the
    // allowlisted `has` method call — `selectedSet().has(r.id)` is a
    // keyed read; only the two affected rows re-evaluate on change.
    const selectedSet = derived(() => new Set([selectedId()]))

    return {
      rows, selectedId, selectedSet,
      move: (delta: number): void => {
        const next = (selectedId() + delta + ROWS.length) % ROWS.length
        selectedId.set(next)
      },
    }
  },
})
