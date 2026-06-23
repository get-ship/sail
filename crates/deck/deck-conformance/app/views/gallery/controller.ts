import { controller, signal, effect } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// Dark themes need the canonical `.dark` class applied in addition to
// `data-theme`, matching the applyTheme contract from @ship/tokens.
const DARK_THEMES = new Set(['ship-dark', 'console', 'contrast'])

export default controller<Scope>({
  scope: () => ({
    activeTheme: signal<string>('ship-light'),
  }),
  mount(state) {
    effect(() => {
      const theme = state.activeTheme
      document.documentElement.setAttribute('data-theme', theme)
      document.documentElement.classList.toggle('dark', DARK_THEMES.has(theme))
    })
  },
})
