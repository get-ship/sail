// /deck/getting-started — adapted from packages/deck/docs/routes/getting-started.
// Snippets live in ./snippets (a plain data module) so example code containing
// `signal(...)` never leaks into the inferred scope.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { pageExample, controllerExample, devCmd } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ pageExample, controllerExample, devCmd }),
})
