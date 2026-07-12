// /motion — the page deals its own cards. `replay` clears the hand and
// re-deals after the exits finish, so the stagger arrival can be watched
// again without a reload. No interval: one deal on arrival, replay on demand.

import { controller, signal, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { code } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const open = signal(true)
    const dealt = signal(true)
    const replay = (): void => {
      dealt.set(false)
      setTimeout(() => dealt.set(true), 450)
    }
    return {
      code,
      open,
      dealt,
      replay,
      hand: signal(['A', 'K', 'Q', 'J', '10']),
    }
  },
})
