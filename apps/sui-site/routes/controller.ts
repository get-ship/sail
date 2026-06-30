// Overview (home) — the hero + a LIVE reactive counter built in this page's own
// scope (native SUI, no in-browser compile needed), shown beside its source so
// the demo IS the proof: this code → this running, reactive thing.
import { controller, derived, signal, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { COUNTER_SRC } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const count = signal(0)
    const doubled = derived(() => count() * 2)
    return {
      count,
      doubled,
      counterSrc: COUNTER_SRC,
      inc: () => count.set(count() + 1),
      dec: () => count.set(Math.max(0, count() - 1)),
    }
  },
})
