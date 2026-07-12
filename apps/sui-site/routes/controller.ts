// The landing's kinetic headline: a signal cycles the verb, and the keyed
// swap plays exit/enter through the motion system — the rotation IS the
// product demonstrating itself, not a CSS keyframe imitating it.

import { controller, signal, effect, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const WORDS = ['build.', 'theme.', 'animate.', 'ship.']

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const i = signal(0)
    effect(() => {
      const t = setInterval(() => i.set((n) => (n + 1) % WORDS.length), 2200)
      return () => clearInterval(t)
    })
    return { word: () => [WORDS[i()]] }
  },
})
