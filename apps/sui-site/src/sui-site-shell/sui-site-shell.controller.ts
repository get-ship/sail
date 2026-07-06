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
import '../deck-theme.css' // deck (violet) product theme; SUI red is the default
import '@ship/tokens/themes.css' // the swappable theme library (paper/halo/glass/console/contrast)
import '../base.css' // doc typography + .doc reading column (the comment above promised this; it was missing)
import '@ship/sui-code-editor' // side-effect: defines <sui-code-editor> for the tutorial + future live surfaces
import '@ship/sui-viewer'       // side-effect: defines <sui-component-viewer> + children for /components
import { controller, signal, derived } from '@ship/sui-runtime'

const MODE_KEY = 'sui-mode'

// Ship UI is ONE site with a top product switcher (SUI · Deck · Components · AI),
// SolidJS-style. The active product is derived from the path; each drives its own
// sidebar and accent. Tutorial + Components render full-screen (no sidebar) — the
// operator's "no nested nav bars" rule: one switcher + one sidebar, never stacked.
function productFor(p: string): string {
  if (p === '/') return 'ship'
  if (p === '/deck' || p.startsWith('/deck/')) return 'deck'
  if (p === '/ai' || p.startsWith('/ai/')) return 'ai'
  if (p === '/components' || p.startsWith('/components/')) return 'components'
  if (p.startsWith('/tutorial')) return 'tutorial'
  return 'sui'
}
// Each product has its own accent (data-theme). The umbrella landing is ship
// (teal); SUI red; Deck violet; Components green; AI blue. Tutorial rides SUI's.
// data-theme is the single source of truth — set here so no route manages it.
function themeFor(product: string): string {
  switch (product) {
    case 'ship': return 'ship'
    case 'deck': return 'deck'
    case 'components': return 'components'
    case 'ai': return 'ai'
    default: return 'sui'
  }
}

// Selectable site themes for the header switcher. The product accent is the
// default; a manual pick overrides it (persisted). Names are `data-theme`
// values shipped by @ship/tokens/themes.css (imported in the .sui).
const THEME_KEY = 'sui-site-theme'
export const SITE_THEMES: Array<{ id: string; label: string }> = [
  { id: '', label: 'Auto (per product)' },
  { id: 'paper', label: 'Paper' },
  { id: 'halo', label: 'Halo' },
  { id: 'glass', label: 'Glass' },
  { id: 'console', label: 'Console' },
  { id: 'contrast', label: 'Contrast' },
]

export default controller({
  scope: () => {
    const root = document.documentElement
    const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')
    const product = derived(() => productFor(path()))

    // Manual theme override (persisted). Empty string = follow the product.
    const storedTheme = (() => {
      try { return localStorage.getItem(THEME_KEY) ?? '' } catch { return '' }
    })()
    const themeName = signal<string>(storedTheme)
    const applyTheme = (): void => {
      root.setAttribute('data-theme', themeName() || themeFor(product()))
    }
    applyTheme()
    const setTheme = (id: string): void => {
      themeName.set(id)
      applyTheme()
      try { id ? localStorage.setItem(THEME_KEY, id) : localStorage.removeItem(THEME_KEY) } catch { /* no storage */ }
    }

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

    // Component viewer + individual tutorial LESSONS render full-screen (no docs
    // sidebar, full width). The /tutorial index (the TOC) keeps the normal
    // padded sidebar layout — full-screen padding:0 pushed its list flush-left.
    const fullscreen = derived(() => {
      const p = path()
      // The umbrella landing, the component catalog, and tutorial lessons own the
      // full width (no sidebar). Docs pages keep the per-product sidebar.
      return p === '/' || p === '/components' || p.startsWith('/components/') || p.startsWith('/tutorial/')
    })

    return { dark, toggleTheme, themeName, setTheme, themes: SITE_THEMES, path, product, fullscreen }
  },
})
