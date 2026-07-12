// /theming — the live theme switchboard. `pick` is the same two calls the
// top-bar <theme-picker> makes: applyTheme (data-theme + the canonical .dark
// class) and persistTheme (survives reload). The page IS the preview — every
// component on it reads tokens, so the pick repaints the reader's world.

import { controller, signal, type ViewScope } from '@ship/sui-runtime'
import { applyTheme, persistTheme, readStoredTheme, THEMES, type ThemeId } from '@ship/tokens/themes'
import type { Scope } from './page.sui'
import { code } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const active = signal<string>(readStoredTheme())
    const pick = (id: ThemeId): void => {
      applyTheme(id)
      persistTheme(id)
      active.set(id)
    }
    return {
      active,
      pick,
      themes: () => THEMES,
      code,
    }
  },
})
