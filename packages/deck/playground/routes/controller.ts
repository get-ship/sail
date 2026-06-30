// Overview controller. A route-local signal counter (resets on nav) plus the
// persisted `visits` store off the clients seam (survives nav + reload).
import { controller, signal, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const count = signal(0)

    // Durable: increment + persist this visit. Because the store lives above the
    // router, navigating away and back re-runs this and the count climbs.
    ctx.clients.store.noteVisit()

    return {
      count,
      inc: (): void => count.set(count() + 1),
      dec: (): void => count.set(count() - 1),
      reset: (): void => count.set(0),
      visits: ctx.clients.store.visits,
      resetVisits: (): void => ctx.clients.store.reset(),
    }
  },
})
