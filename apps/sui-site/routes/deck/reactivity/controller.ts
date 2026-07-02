// /deck/reactivity — deck's role in the reactive update path. Reactivity
// itself (signal / derived / effect) is SUI's and lives at /reactivity; this
// controller only feeds the page its highlighted code snippets.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { code } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ code }),
})
