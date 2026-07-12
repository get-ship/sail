// Controller for <sui-site-shell> — the docs chrome (top bar + sidebar +
// content).
//
// THEME and PRODUCT are separate channels with separate owners. The theme —
// one of @ship/tokens' nine — owns every surface and ink on the page;
// `applyTheme` is its only writer (data-theme + the canonical .dark class),
// and the picker persists the choice. The product — derived from the path —
// owns only the accent hue, on its own attribute (data-product), so crossing
// into Deck turns the brand violet without repainting the reader's theme.
//
// The SUI theme + doc base styles are imported HERE (not the clients seam): the
// shell is in every route's module graph, so these side-effect imports can't be
// tree-shaken — whereas an export-less clients module gets dropped.
import '../product-accents.css'
import '@ship/tokens/themes.css' // the swappable theme library (paper/halo/glass/console/contrast)
import '../base.css' // doc typography + .doc reading column (the comment above promised this; it was missing)
import { applyTheme, readStoredTheme } from '@ship/tokens/themes'
import { controller, signal, derived } from '@ship/sui-runtime'

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
// The umbrella landing is ship (teal); SUI red; Deck violet; Components
// green; AI blue. `product-accents.css` keys on this attribute.
function productAttr(product: string): string {
  switch (product) {
    case 'ship':
    case 'deck':
    case 'components':
    case 'ai':
      return product
    default:
      return 'sui'
  }
}

export default controller({
  scope: () => {
    const root = document.documentElement
    const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')
    const product = derived(() => productFor(path()))
    root.setAttribute('data-product', productAttr(product()))

    // Restore the reader's theme. applyTheme is the ONLY writer of
    // data-theme/.dark; idempotent, so re-running per navigation is free.
    applyTheme(readStoredTheme())

    // The umbrella landing and the component catalog own the full width
    // (no sidebar). Docs pages keep the per-product sidebar.
    const fullscreen = derived(() => {
      const p = path()
      return p === '/' || p === '/components' || p.startsWith('/components/')
    })

    return { path, product, fullscreen }
  },
})
