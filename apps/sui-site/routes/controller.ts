// Landing: the headline rotates WHICH battery is included (enter-only fade —
// no exit animation, so there is no double-exposure ghost), and the theme
// section opens the site's real theme picker instead of demo cards.

import { controller, signal, effect, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const WORDS = ['Batteries', 'Components', 'Styling', 'State', 'Motion', 'Themes']

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const i = signal(0)
    effect(() => {
      const t = setInterval(() => i.set((n) => (n + 1) % WORDS.length), 2800)
      return () => clearInterval(t)
    })
    const openThemes = (): void => {
      const panel = document.getElementById('sui-theme-panel') as HTMLElement & { showPopover?: () => void }
      try {
        panel?.showPopover?.()
      } catch {
        /* already open */
      }
    }
    return { word: () => [WORDS[i()]], openThemes }
  },
})
