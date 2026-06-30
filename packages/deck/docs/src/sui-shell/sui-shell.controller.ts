// Controller for <sui-shell> — the docs chrome (top bar + sidebar + content).
//
// Owns the theme (brands <html data-theme="deck">, manages dark/light, dark-
// first, persisted) and the current path for sidebar active-state. mountRouter
// rebuilds the chrome on every client-side navigation, so reading
// location.pathname at scope time always reflects the active route.
import { controller, signal } from '@ship/sui-runtime'

const MODE_KEY = 'deck-mode'

export default controller({
  scope: () => {
    const root = document.documentElement
    root.setAttribute('data-theme', 'deck')

    const stored = (() => {
      try {
        return localStorage.getItem(MODE_KEY)
      } catch {
        return null
      }
    })()
    const dark = signal<boolean>(stored !== 'light') // dark-first
    const apply = (isDark: boolean): void => {
      if (isDark) root.removeAttribute('data-mode')
      else root.setAttribute('data-mode', 'light')
    }
    apply(dark())

    const toggleTheme = (): void => {
      const next = !dark()
      dark.set(next)
      apply(next)
      try {
        localStorage.setItem(MODE_KEY, next ? 'dark' : 'light')
      } catch {
        /* no storage */
      }
    }

    const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')

    return { dark, toggleTheme, path }
  },
})
