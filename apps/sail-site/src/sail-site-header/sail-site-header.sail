<!--
  <sail-site-header> — the site-wide chrome: brand, product switcher, theme
  picker, and theme/product application. Written in the language, no CSS.

  This is the ONLY layout that wraps every route (`routes/_layout.sail`
  mounts it at prefix `/`). Per-segment sidebars (Sail/Deck/Components/AI)
  live in their own `routes/<segment>/_layout.sail` + rail component, nested
  inside this one — this file owns none of that, only the always-on top bar.
-->

<controller>
// Controller for <sail-site-header> — brand + product switcher + theme
// picker.
//
// THEME and PRODUCT are separate channels with separate owners. The theme —
// one of @ship/tokens' nine — owns every surface and ink on the page;
// `applyTheme` is its only writer (data-theme + the canonical .dark class),
// and the picker persists the choice. The product — derived from the path —
// owns only the accent hue, on its own attribute (data-product), so crossing
// into Deck turns the brand violet without repainting the reader's theme.
//
// The Sail theme + the framework contracts are imported HERE (not the clients
// seam): this header is in every route's module graph, so these side-effect
// imports can't be tree-shaken — whereas an export-less clients module gets
// dropped. sail-base.css gives every page the design-system typography
// (bare h1/p/ul/code/pre/button/anchor) without a class hook in any template.
import '@ship/tokens/themes.css' // the swappable theme library (paper/halo/glass/console/contrast)
import '@ship/tokens/products.css' // brand hue per data-product; themes own lightness/chroma
import '@ship/tokens/sail-base.css' // sail-base (document floor): bare h1/p/ul/code/pre/button/anchor skins; the design system applied to a Sail document, no class hooks needed
import { applyTheme, isValidTheme, THEME_STORAGE_KEY, DEFAULT_THEME } from '@ship/tokens/themes'

// Ship UI is ONE site with a top product switcher (Sail · Deck · Components · AI).
// The active product is derived from the path; each drives its own segment
// layout/sidebar/accent (see routes/{sail,deck,components,ai}/_layout.sail).
function productFor(p: string): string {
  if (p === '/') return 'ship'
  if (p === '/deck' || p.startsWith('/deck/')) return 'deck'
  if (p === '/ai' || p.startsWith('/ai/')) return 'ai'
  if (p === '/components' || p.startsWith('/components/')) return 'components'
  return 'sail'
}
// The umbrella landing is ship (teal); Sail red; Deck violet; Components
// green; AI blue. `product-accents.css` keys on this attribute.
function productAttr(product: string): string {
  switch (product) {
    case 'ship':
    case 'deck':
    case 'components':
    case 'ai':
      return product
    default:
      return 'sail'
  }
}

export default controller((ctx) => {
    const root = document.documentElement
    // Live pathname from the router when chrome is reused across leaf
    // navigations; fall back to location for first paint / non-router mounts.
    const path = derived(() => {
      const r = (ctx as { route?: { pathname?: () => string } }).route
      if (r?.pathname) return r.pathname()
      return typeof location !== 'undefined' ? location.pathname : '/'
    })
    const product = derived(() => productFor(path()))
    effect(() => {
      root.setAttribute('data-product', productAttr(product()))
    })

    // Theme application. applyTheme is the ONLY writer of data-theme/.dark and
    // does NOT persist (the picker owns localStorage) — so re-running per
    // navigation is free and never overrides an explicit pick.
    //
    // The marketing home (design 1c) leads with the kinetic DARK design, so a
    // first-time reader with no stored preference lands on `ship-dark`; the
    // docs then present in the stored/default theme. An explicit choice in
    // localStorage always wins, on the home or anywhere else.
    const explicitTheme = (): string | null => {
      if (typeof localStorage === 'undefined') return null
      try {
        const raw = localStorage.getItem(THEME_STORAGE_KEY)
        return isValidTheme(raw) ? raw : null
      } catch {
        return null
      }
    }
    effect(() => {
      const chosen = explicitTheme()
      if (chosen) {
        applyTheme(chosen)
        return
      }
      applyTheme(path() === '/' ? 'ship-dark' : DEFAULT_THEME)
    })

    // The umbrella landing (design 1c) is the marketing home — a full-bleed
    // kinetic page (crossing diagonal tickers, giant wordmark) that must NOT
    // sit inside the docs reading gutter, so it renders in a no-padding main
    // like a workbench. Every other route has its own segment layout (Sail,
    // Deck, Components, AI) that supplies its own <main> — this header must
    // not double-wrap those, so it only wraps when nothing downstream will.
    const landing = derived(() => path() === '/')

    return { path, product, landing }
})
</controller>

<scope>
  const brand = style.fontSize(lg).tracking(tight)
  const brandMark = style.fg(primary)
  const productSwitch = style.pad(1).bg(surface-2).borderWidth(1).border(border).radius(full)
  const productLink = style.padX(3).padY(1).radius(full).fontSize(sm).fontWeight(semibold).fg(muted-foreground).when({
    hover: style.fg(foreground),
    current: style.bg(primary).fg(primary-foreground),
  })
  // Clip marketing full-bleed content (tickers use negative inset) at the shell.
  const landingMain = style.basis(0).minH(0).maxW(full).overflowX(hidden).overflowY(visible)
</scope>

<navbar sticky bordered use:chromeOffset>
  <a slot="brand" href="/" aria-label="Ship UI home">
    <row gap="2" align="center">
      <svg width="26" height="26" viewBox="0 0 26 26" fill="none" aria-hidden="true">
        <circle cx="6" cy="6" r="3.4" fill="var(--primary)" />
        <circle cx="20" cy="6" r="3.4" fill="var(--primary)" opacity="0.45" />
        <circle cx="6" cy="20" r="3.4" fill="var(--primary)" opacity="0.45" />
        <circle cx="20" cy="20" r="3.4" fill="var(--primary)" opacity="0.7" />
        <path d="M6 6 L20 20 M6 6 L20 6 M6 6 L6 20" stroke="var(--primary)" stroke-width="1.4" opacity="0.5" stroke-linecap="round" />
      </svg>
      <strong sx={brand}><b sx={brandMark}>SHIP</b> UI</strong>
    </row>
  </a>

  <row slot="center" gap="1" align="center" sx={productSwitch} aria-label="Products">
    <a href="/sail/overview" sx={productLink} aria-current={product() === 'sail' ? 'page' : null}>Sail</a>
    <a href="/deck" sx={productLink} aria-current={product() === 'deck' ? 'page' : null}>Deck</a>
    <a href="/components" sx={productLink} aria-current={product() === 'components' ? 'page' : null}>Components</a>
    <a href="/ai" sx={productLink} aria-current={product() === 'ai' ? 'page' : null}>AI</a>
  </row>

  <theme-picker slot="actions"></theme-picker>
</navbar>

<if cond={landing()}>
  <main grow min-w="0" w="full" sx={landingMain}>
    <slot name="route"></slot>
  </main>
<else>
  <slot name="route"></slot>
</else></if>
