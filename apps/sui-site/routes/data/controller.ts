// /data — a real async source with a visible delay, so the reader watches
// loading → ok and loading → error actually happen. `live()` runs its source
// in an effect, so reading `attempt` inside it makes the button a refetch:
// bump the signal, the source re-runs, the state walks through loading again.

import { controller, live, signal, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { code } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const attempt = signal(1)
    const sabotage = signal(false)
    const users = live(() => {
      attempt() // tracked: bumping it re-runs the source
      const fail = sabotage()
      return new Promise<string[]>((resolve, reject) => {
        setTimeout(() => {
          if (fail) reject(new Error('503 — the demo API was told to fail'))
          else resolve(['Ada', 'Grace', 'Edsger', 'Barbara'])
        }, 900)
      })
    })
    return {
      code,
      users,
      reload: () => {
        sabotage.set(false)
        attempt.set((a) => a + 1)
      },
      breakIt: () => {
        sabotage.set(true)
        attempt.set((a) => a + 1)
      },
    }
  },
})
