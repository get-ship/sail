// /reactivity — the live graph the page describes. One signal, one derived,
// one effect writing a visible log: the reader clicks and watches all three
// layers respond.

import { controller, signal, derived, effect, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { code } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const count = signal(0)
    const doubled = derived(() => count() * 2)
    const log = signal<string[]>([])
    effect(() => {
      const n = count()
      log.set((prev) => [...prev.slice(-3), `effect ran: count is ${n}`])
    })
    return { code, count, doubled, log }
  },
})
