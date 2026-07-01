// /ai/eval — scoring with sui-eval, adapted from the 11-sui-eval tutorial
// lesson. Snippets live in ./snippets so literal braces in the example block
// and the sniffEscapeHatches output never leak into the inferred scope.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { evalExample, hatchExample } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ evalExample, hatchExample }),
})
