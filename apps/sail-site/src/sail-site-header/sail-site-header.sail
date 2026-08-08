<!--
  <sail-site-header> — site chrome: brand, product switcher, theme picker.
  Mounted by routes/_layout.sail at prefix `/`. Segment sidebars live in
  routes/<segment>/_layout.sail + rail — not here.

  Shell: outer column is viewport-locked (h(screen)) so fill/grow children
  resolve height. Landing (`/`) scrolls on <main>; document routes hide
  overflow on <main> so nested rails own scrollports. One chrome tree so
  theme-picker never remounts on SPA navigation.
-->

<controller>
// Theme (applyTheme) and product accent (data-product from path) are
// separate channels. Side-effect CSS imports live here so the module graph
// keeps them (export-less clients modules get tree-shaken).
import { siteTokenCss } from '../compositions/site-token-css'
import { applyTheme, readStoredTheme, isValidTheme, DEFAULT_THEME } from '@ship/tokens/themes'
void siteTokenCss

function productFor(p: string): string {
  if (p === '/') return 'ship'
  if (p === '/deck' || p.startsWith('/deck/')) return 'deck'
  if (p === '/ai' || p.startsWith('/ai/')) return 'ai'
  if (p === '/components' || p.startsWith('/components/')) return 'components'
  return 'sail'
}

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
    const path = derived(() => livePath(ctx as { route?: { pathname?: () => string } }))
    const product = derived(() => productFor(path()))
    effect(() => {
      root.setAttribute('data-product', productAttr(product()))
    })

    // applyTheme writes theme/mode only; picker owns persistence. Marketing
    // home leads with ship/dark when stored theme is still the package default.
    effect(() => {
      const stored = readStoredTheme()
      if (path() === '/' && stored === DEFAULT_THEME) {
        applyTheme('ship', document.documentElement, 'dark')
        return
      }
      if (isValidTheme(stored)) applyTheme(stored)
      else applyTheme(DEFAULT_THEME)
    })

    const landing = derived(() => path() === '/')

    return { path, product, landing }
})
</controller>

<scope>
  style brand = { text: fontSize(lg).tracking(tight).whitespace(nowrap) }
  style brandMark = { paint: fg(primary) }
  // Fixed-height track; overflow-x scrolls if product links overflow.
  style productSwitch = { box: pad(half).maxW(full).minW(0).overflowX(auto), paint: bg(muted).borderWidth(1).border(solid).border(border).radius(full) }
  style productLink = { box: display(inline-flex).padX(1_5).h(control-h-sm), paint: radius(full).fg(muted-foreground), text: fontSize(xs).fontWeight(medium).leading(none).underline(none).whitespace(nowrap), when: { hover: { paint: fg(foreground).bg(hover-bg) }, current: { paint: bg(primary).fg(primary-foreground) }, tablet: { box: padX(2_5), text: fontSize(sm) } } }
  // Phone: logo only. Tablet+: SHIP UI wordmark.
  style brandWord = { box: display(none), text: whitespace(nowrap), when: { tablet: { box: display(inline) } } }
  // Do NOT overflow:hidden the outer column — taffy silent-clip + popover hosts.
  style shell = { box: minH(screen).h(screen) }
  // Landing: continuous scrollport. Children shrink=0 so sections do not compress.
  style landingMain = { box: basis(0).minH(0).minW(0).maxW(full).overflowX(hidden).overflowY(auto) }
  // Docs/workbench: flex column so nested shells resolve h(full). Without
  // display:flex, overflow:hidden on main clips and catalog cannot scroll.
  style appMain = { box: display(flex).flexDir(column).basis(0).minH(0).minW(0).overflow(hidden) }
</scope>

<column gap="0" style(shell, { box: w(full) })>
  <navbar sticky bordered use:chromeOffset>
    <a slot="brand" href="/" label="SHIP UI home" style({ box: shrink(0) })>
      <row gap="2" align="center" nowrap>
        <svg width="22" height="22" viewBox="0 0 26 26" fill="none" hidden="true">
          <circle cx="6" cy="6" r="3.4" fill="var(--primary)" />
          <circle cx="20" cy="6" r="3.4" fill="var(--primary)" opacity="0.45" />
          <circle cx="6" cy="20" r="3.4" fill="var(--primary)" opacity="0.45" />
          <circle cx="20" cy="20" r="3.4" fill="var(--primary)" opacity="0.7" />
          <path d="M6 6 L20 20 M6 6 L20 6 M6 6 L6 20" stroke="var(--primary)" stroke-width="1.4" opacity="0.5" stroke-linecap="round" />
        </svg>
        <strong style(brandWord)><b style(brandMark)>SHIP</b> UI</strong>
      </row>
    </a>

    <row slot="center" gap="1" align="center" nowrap label="Products" style(productSwitch, { box: minW(0) })>
      <a href="/sail/overview" current={product() === 'sail' ? 'page' : null} style(productLink, { box: shrink(0) })>Sail</a>
      <a href="/deck" current={product() === 'deck' ? 'page' : null} style(productLink, { box: shrink(0) })>Deck</a>
      <a href="/components" current={product() === 'components' ? 'page' : null} style(productLink, { box: shrink(0) })>Components</a>
      <a href="/ai" current={product() === 'ai' ? 'page' : null} style(productLink, { box: shrink(0) })>AI</a>
    </row>

    <theme-picker slot="actions"></theme-picker>
  </navbar>

  <if {landing()}>
    <main style(landingMain, { box: grow(1).minW(0).w(full) })>
      <slot name="route"></slot>
    </main>
  {else => <main style(appMain, { box: grow(1).minW(0).w(full) })>
      <slot name="route"></slot>
    </main>}</if>
</column>
