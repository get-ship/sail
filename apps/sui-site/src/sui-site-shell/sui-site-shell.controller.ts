// Controller for <sui-site-shell> — the docs chrome (top bar + sidebar +
// content).
//
// Owns the theme (brands <html data-theme="sui">, LIGHT-first, persisted) and
// the current path for sidebar active-state. The router rebuilds the chrome on
// every client-side navigation, so reading location.pathname at scope time
// always reflects the active route.
//
// The SUI theme + doc base styles are imported HERE (not the clients seam): the
// shell is in every route's module graph, so these side-effect imports can't be
// tree-shaken — whereas an export-less clients module gets dropped.
import '../sui-theme.css'
import '@ship/sui-code-editor' // side-effect: defines <sui-code-editor> for the tutorial + future live surfaces
import '@ship/sui-viewer'       // side-effect: defines <sui-component-viewer> + children for /components
import { controller, signal } from '@ship/sui-runtime'

const MODE_KEY = 'sui-mode'

export default controller({
  scope: () => {
    const root = document.documentElement
    root.setAttribute('data-theme', 'sui')

    const stored = (() => {
      try {
        return localStorage.getItem(MODE_KEY)
      } catch {
        return null
      }
    })()
    const dark = signal<boolean>(stored === 'dark') // light-first
    const apply = (isDark: boolean): void => {
      if (isDark) root.setAttribute('data-mode', 'dark')
      else root.removeAttribute('data-mode')
    }
    apply(dark())

    const toggleTheme = (): void => {
      const next = !dark()
      dark.set(next)
      apply(next)
      try {
        localStorage.setItem(MODE_KEY, next ? 'dark' : 'light')
      } catch {
        /* no storage — still toggles in-session */
      }
    }

    const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')

    return { dark, toggleTheme, path }
  },
})
