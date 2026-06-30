// Controller for <sui-shell> — the playground chrome.
//
// Owns the theme: it brands <html> with `data-theme="deck"` (which makes the
// deck palette win specificity over the entry-contract tokens.css) and manages
// dark/light via `data-mode`. Dark is the default (deck is dark-first); the
// choice persists. The `dark` signal seeds from the live DOM so the toggle
// label stays correct even though mountRouter rebuilds the chrome on every
// navigation (the attributes live on <html>, outside the chrome).
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
    // Dark-first: only light if explicitly chosen.
    const dark = signal<boolean>(stored !== 'light')

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
        /* no storage — fine */
      }
    }

    return { dark, toggleTheme }
  },
})
