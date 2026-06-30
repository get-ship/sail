// Data-seam controller. Reads the persisted `store` client off the clients seam
// (ctx.clients.store) and exposes its `visits` signal + actions for the live
// demo. No `signal(` in this file's source, so scope inference stays clean.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { configExample, usageExample } from './snippets'

export default controller({
  scope: (ctx: ViewScope<Scope>) => ({
    configExample,
    usageExample,
    visits: ctx.clients.store.visits,
    record: (): void => ctx.clients.store.noteVisit(),
    reset: (): void => ctx.clients.store.reset(),
  }),
})
