// /ai/authoring — the agent authoring loop, adapted from the
// 09-agent-loop tutorial lesson. Snippets live in ./snippets so the literal
// loop-diagram ASCII never leaks into the inferred scope.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { loopDiagram } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ loopDiagram }),
})
