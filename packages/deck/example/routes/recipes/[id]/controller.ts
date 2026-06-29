// Recipe detail controller. The `[id]` segment arrives as the reactive signal
// `route.params.id` (note the call: `ctx.route.params.id()`). Reading it inside
// a `derived` re-runs the lookup whenever the URL id changes — so client-side
// navigation between two recipes updates the page without a remount.
import { controller, derived, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: (ctx: ViewScope<Scope>) => ({
    recipe: derived(() => ctx.clients.data.get(ctx.route.params.id())),
  }),
})
