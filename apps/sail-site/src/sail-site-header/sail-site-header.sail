<!--
  <sail-site-header> — the site-wide chrome: brand, product switcher, theme
  picker, and theme/product application. Written in the language, no CSS.

  This is the ONLY layout that wraps every route (`routes/_layout.sail`
  mounts it at prefix `/`). Per-segment sidebars (Sail/Deck/Components/AI)
  live in their own `routes/<segment>/_layout.sail` + rail component, nested
  inside this one — this file owns none of that, only the always-on top bar.

  Shell geometry (one chrome tree — theme-picker must NEVER remount on SPA):
  - Outer column is always viewport-locked (`h(screen)`) so fill/grow children
    (docs rails, tutorial workbench) resolve a real height budget.
  - Landing (`/`) uses a single scrollport on <main> (overflowY auto) with
    non-shrinking children so marketing is one continuous scroll, not sections.
  - Document routes use overflow(hidden) on <main> so nested rails own
    independent rail + content scrollports.
  - Tutorial lessons (`/sail/tutorial/*`) and showcase keep the same fill shell;
    sail-docs-rail drops the rail (workbench mode).
-->

<controller>
// Controller for <sail-site-header> — brand + product switcher + theme
// picker.
//
// THEME and PRODUCT are separate channels with separate owners. The theme —
// one of @ship/tokens' nine — owns every surface and ink on the page;
// `applyTheme` is its only writer (theme + data-mode),
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
import { applyTheme, readStoredTheme, isValidTheme, DEFAULT_THEME } from '@ship/tokens/themes'

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

/** Live pathname — subscribe to route.pathname when present, always read location. */
function livePath(ctx: { route?: { pathname?: () => string } }): string {
  const r = ctx.route
  // Touch the signal so SPA navigations re-run this derived when chrome is reused.
  if (r?.pathname) void r.pathname()
  if (typeof location !== 'undefined' && location.pathname) return location.pathname
  return r?.pathname?.() ?? '/'
}

export default controller((ctx) => {
    const root = document.documentElement
    // Live pathname from the router when chrome is reused across leaf
    // navigations; fall back to location for first paint / non-router mounts.
    const path = derived(() => livePath(ctx as { route?: { pathname?: () => string } }))
    const product = derived(() => productFor(path()))
    effect(() => {
      root.setAttribute('data-product', productAttr(product()))
    })

    // Theme application. applyTheme is the ONLY writer of theme/mode and
    // does NOT persist (the picker owns storage via persistTheme) — so
    // re-running per navigation is free and never overrides an explicit pick.
    // Storage reads go through readStoredTheme() (tokens package seam).
    //
    // The marketing home leads with kinetic ship/dark when the stored theme
    // is the package default (unset or default pick).
    effect(() => {
      const stored = readStoredTheme()
      if (path() === '/' && stored === DEFAULT_THEME) {
        applyTheme('ship', document.documentElement, 'dark')
        return
      }
      if (isValidTheme(stored)) applyTheme(stored)
      else applyTheme(DEFAULT_THEME)
    })

    // The umbrella landing (design 1c) is the marketing home — a full-bleed
    // kinetic page that must NOT sit inside the docs reading gutter.
    const landing = derived(() => path() === '/')

    return { path, product, landing }
})
</controller>

<scope>
  const brand = style({ text: fontSize(lg).tracking(tight).whitespace(nowrap) })
  const brandMark = style({ paint: fg(primary) })
  // One fixed-height track. Never wrap — overflow scrolls if needed (last resort).
  const productSwitch = style({ box: pad(half).maxW(full).minW(0).overflowX(auto), paint: bg(muted).borderWidth(1).border(solid).border(border).radius(full) })
  // Compact on phone so Sail·Deck·Components·AI + brand + theme stay one row.
  const productLink = style({ box: display(inline-flex).padX(1_5).h(control-h-sm), paint: radius(full).fg(muted-foreground), text: fontSize(xs).fontWeight(medium).leading(none).underline(none).whitespace(nowrap) }).when(hover, { paint: fg(foreground).bg(hover-bg) }).when(current, { paint: bg(primary).fg(primary-foreground) }).when(tablet, { box: padX(2_5), text: fontSize(sm) })
  // Phone: mark only. Tablet+: SHIP UI wordmark.
  const brandWord = style({ box: display(none), text: whitespace(nowrap) }).when(tablet, { box: display(inline) })
  // Viewport-locked shell so fill/grow children resolve. One tree — never remount
  // the theme-picker when switching landing/docs/tutorial (SPA chrome reuse).
  // Do NOT overflow:hidden the outer column — taffy silent-clip + popover hosts.
  const shell = style({ box: minH(screen).h(screen) })
  // Landing: THE continuous scrollport. Clip ticker X; Y scrolls here only.
  // Children must shrink=0 so sections do not compress into mini-scrollpanes.
  const landingMain = style({ box: basis(0).minH(0).minW(0).maxW(full).overflowX(hidden).overflowY(auto) })
  // App (docs / workbench / catalog): flex column so nested shells resolve h(full).
  // Without display:flex, children size to content → overflow:hidden clips → no scroll
  // (components catalog was unscrollable for exactly this reason).
  const appMain = style({ box: display(flex).flexDir(column).basis(0).minH(0).minW(0).overflow(hidden) })
</scope>

<!-- Single chrome tree: navbar + theme-picker never remount on route change. -->
<column gap="0" w="full">
  style(shell)

  <navbar sticky bordered use:chromeOffset>
    <a slot="brand" href="/" label="SHIP UI home" shrink="0">
      <row gap="2" align="center" nowrap>
        <svg width="22" height="22" viewBox="0 0 26 26" fill="none" hidden="true">
          <circle cx="6" cy="6" r="3.4" fill="var(--primary)" />
          <circle cx="20" cy="6" r="3.4" fill="var(--primary)" opacity="0.45" />
          <circle cx="6" cy="20" r="3.4" fill="var(--primary)" opacity="0.45" />
          <circle cx="20" cy="20" r="3.4" fill="var(--primary)" opacity="0.7" />
          <path d="M6 6 L20 20 M6 6 L20 6 M6 6 L6 20" stroke="var(--primary)" stroke-width="1.4" opacity="0.5" stroke-linecap="round" />
        </svg>
        <strong>
          style(brandWord)
<b>
          style(brandMark)
SHIP</b> UI</strong>
      </row>
    </a>

    <row slot="center" gap="1" align="center" nowrap min-w="0" label="Products">
      style(productSwitch)

      <a href="/sail/overview" shrink="0" current={product() === 'sail' ? 'page' : null}>
        style(productLink)
Sail</a>
      <a href="/deck" shrink="0" current={product() === 'deck' ? 'page' : null}>
        style(productLink)
Deck</a>
      <a href="/components" shrink="0" current={product() === 'components' ? 'page' : null}>
        style(productLink)
Components</a>
      <a href="/ai" shrink="0" current={product() === 'ai' ? 'page' : null}>
        style(productLink)
AI</a>
    </row>

    <theme-picker slot="actions"></theme-picker>
  </navbar>

  <if {landing()}>
    <main grow min-w="0" w="full">
      style(landingMain)

      <slot name="route"></slot>
    </main>
  <else>
    <main grow min-w="0" w="full">
      style(appMain)

      <slot name="route"></slot>
    </main>
  </else></if>
</column>
