<!--
  <sail-site-shell> — the docs chrome, written in the language, with no CSS.

  The product switcher (SAIL · Deck · Components · AI) is the navbar's center
  slot with the pill written in sx — container atoms on the <row>, link atoms
  plus `current:` on each segment. The navbar contributes placement and
  regions; the look is authored here, where it belongs. The sidebar is the
  same element in the rail placement.

  This file used to be 100 lines of markup over 4.8 kB of hand-rolled CSS that
  reimplemented <navbar> badly. Now the structure lives in the standard
  library and the styling is sx — which is the argument the site makes,
  running in its own chrome.
-->

<controller>
// Controller for <sail-site-shell> — the docs chrome (top bar + sidebar +
// content).
//
// THEME and PRODUCT are separate channels with separate owners. The theme —
// one of @ship/tokens' nine — owns every surface and ink on the page;
// `applyTheme` is its only writer (data-theme + the canonical .dark class),
// and the picker persists the choice. The product — derived from the path —
// owns only the accent hue, on its own attribute (data-product), so crossing
// into Deck turns the brand violet without repainting the reader's theme.
//
// The SAIL theme + the framework contracts are imported HERE (not the clients
// seam): the shell is in every route's module graph, so these side-effect
// imports can't be tree-shaken — whereas an export-less clients module gets
// dropped. contracts.css gives every page the design-system typography
// (bare h1/p/ul/code/pre/button/anchor) without a class hook in any template.
import '@ship/tokens/themes.css' // the swappable theme library (paper/halo/glass/console/contrast)
import '@ship/tokens/products.css' // brand hue per data-product; themes own lightness/chroma
import '@ship/tokens/contracts.css' // framework contracts: bare h1/p/ul/code/pre/button/anchor skins; the design system applied to a SAIL document, no class hooks needed
import { applyTheme, isValidTheme, THEME_STORAGE_KEY, DEFAULT_THEME } from '@ship/tokens/themes'

// Ship UI is ONE site with a top product switcher (SAIL · Deck · Components · AI),
// SolidJS-style. The active product is derived from the path; each drives its own
// sidebar and accent. Components renders full-screen (no sidebar) — the
// operator's "no nested nav bars" rule: one switcher + one sidebar, never
// stacked. The tutorial is SAIL product and KEEPS the rail: a lesson is a
// docs page, and the reader needs the way back.
function productFor(p: string): string {
  if (p === '/') return 'ship'
  if (p === '/deck' || p.startsWith('/deck/')) return 'deck'
  if (p === '/ai' || p.startsWith('/ai/')) return 'ai'
  if (p === '/components' || p.startsWith('/components/')) return 'components'
  return 'sail'
}
// The umbrella landing is ship (teal); SAIL red; Deck violet; Components
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

export default controller(() => {
    const root = document.documentElement
    const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')
    const product = derived(() => productFor(path()))
    root.setAttribute('data-product', productAttr(product()))

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

    // The component catalog and the tutorial LESSON pages own the full width
    // (no sidebar). The /tutorial index is a docs page and keeps the rail; a
    // lesson is a two-pane workbench and needs the whole viewport. The umbrella
    // landing (design 1c) is the marketing home — a full-bleed kinetic page
    // (crossing diagonal tickers, giant wordmark) that must NOT sit inside the
    // docs rail or the reading gutter, so it opts out of both.
    const fullscreen = derived(() => {
      const p = path()
      return (
        p === '/' ||
        p === '/components' ||
        p.startsWith('/components/') ||
        p.startsWith('/tutorial/')
      )
    })

    // The marketing landing owns the whole width edge-to-edge (its tickers and
    // wordmark bleed past the reading gutter), so it renders in a no-padding
    // main like the workbench — its own sections carry their own insets.
    const landing = derived(() => path() === '/')

    // A workbench route (the tutorial lesson) is a fixed-viewport app: no
    // reading padding, and its page sizes itself with h:fill against the
    // chrome offset this shell publishes.
    const workbench = derived(() => path().startsWith('/tutorial/'))

    // The chrome offset — the sticky top bar's height — is published as
    // --chrome-offset on :root by `use:chromeOffset` on the <navbar> below.
    // The `h:fill` scale is `calc(100dvh - var(--chrome-offset, 0px))`; without
    // a value it falls back to the whole viewport, so a workbench pane placed
    // BELOW the bar overflows by exactly the bar's height (the page scrolls).
    // The measurement is geometry the action owns — the controller never
    // touches .style (owner ruling 2026-07-18).

    return { path, product, fullscreen, workbench, landing }
})
</controller>


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
      <strong sx="text-lg tracking-tight"><b sx="text-primary">SHIP</b> UI</strong>
    </row>
  </a>

  <row slot="center" gap="1" align="center" sx="p-1 bg-surface-2 border border-border rounded-full" aria-label="Products">
    <a href="/overview" sx="px-3 py-1 rounded-full text-sm font-semibold text-muted-foreground hover:text-foreground current:bg-primary current:text-primary-foreground" aria-current={product() === 'sail' ? 'page' : null}>SAIL</a>
    <a href="/deck" sx="px-3 py-1 rounded-full text-sm font-semibold text-muted-foreground hover:text-foreground current:bg-primary current:text-primary-foreground" aria-current={product() === 'deck' ? 'page' : null}>Deck</a>
    <a href="/components" sx="px-3 py-1 rounded-full text-sm font-semibold text-muted-foreground hover:text-foreground current:bg-primary current:text-primary-foreground" aria-current={product() === 'components' ? 'page' : null}>Components</a>
    <a href="/ai" sx="px-3 py-1 rounded-full text-sm font-semibold text-muted-foreground hover:text-foreground current:bg-primary current:text-primary-foreground" aria-current={product() === 'ai' ? 'page' : null}>AI</a>
  </row>

  <theme-picker slot="actions"></theme-picker>
</navbar>

<row nowrap align="stretch">
  {#if product() === 'deck'}
    <navbar placement="rail" bordered sticky offset="14">
      <label>Introduction</label>
      <a href="/deck" aria-current={path() === '/deck' ? 'page' : null}>Overview</a>
      <a href="/deck/getting-started" aria-current={path() === '/deck/getting-started' ? 'page' : null}>Getting started</a>
      <label>Concepts</label>
      <a href="/deck/routing" aria-current={path() === '/deck/routing' ? 'page' : null}>Routing</a>
      <a href="/deck/dynamic-routes" aria-current={path() === '/deck/dynamic-routes' ? 'page' : null}>Dynamic routes</a>
      <a href="/deck/layouts" aria-current={path() === '/deck/layouts' ? 'page' : null}>Layouts</a>
      <a href="/deck/rendering" aria-current={path() === '/deck/rendering' ? 'page' : null}>Rendering</a>
      <a href="/deck/reactivity" aria-current={path() === '/deck/reactivity' ? 'page' : null}>Reactivity</a>
      <a href="/deck/data" aria-current={path() === '/deck/data' ? 'page' : null}>Live data</a>
      <label>Tooling</label>
      <a href="/deck/hmr" aria-current={path() === '/deck/hmr' ? 'page' : null}>HMR</a>
      <a href="/deck/build" aria-current={path() === '/deck/build' ? 'page' : null}>Build</a>
    </navbar>
  {:else if product() === 'ai'}
    <navbar placement="rail" bordered sticky offset="14">
      <label>AI</label>
      <a href="/ai" aria-current={path() === '/ai' ? 'page' : null}>Overview</a>
      <a href="/ai/authoring" aria-current={path() === '/ai/authoring' ? 'page' : null}>Authoring</a>
      <a href="/ai/tooling" aria-current={path() === '/ai/tooling' ? 'page' : null}>Tooling</a>
      <a href="/ai/eval" aria-current={path() === '/ai/eval' ? 'page' : null}>Eval</a>
    </navbar>
  {:else if !fullscreen()}
    <navbar placement="rail" bordered sticky offset="14">
      <label>Start here</label>
      <a href="/overview" aria-current={path() === '/' || path() === '/overview' ? 'page' : null}>Introduction</a>
      <a href="/thesis" aria-current={path() === '/thesis' ? 'page' : null}>Thesis</a>
      <a href="/getting-started" aria-current={path() === '/getting-started' ? 'page' : null}>Installation</a>
      <a href="/why" aria-current={path() === '/why' ? 'page' : null}>Why SAIL</a>
      <a href="/tutorial" aria-current={path().startsWith('/tutorial') ? 'page' : null}>Tutorial</a>
      <a href="/showcase" aria-current={path() === '/showcase' ? 'page' : null}>Showcase</a>

      <label>Components</label>
      <a href="/elements" aria-current={path() === '/elements' ? 'page' : null}>Using components</a>
      <a href="/layout" aria-current={path() === '/layout' ? 'page' : null}>Layout</a>
      <a href="/styling" aria-current={path() === '/styling' ? 'page' : null}>Styling</a>
      <a href="/theming" aria-current={path() === '/theming' ? 'page' : null}>Theming</a>
      <a href="/motion" aria-current={path() === '/motion' ? 'page' : null}>Animation</a>
      <a href="/behavior" aria-current={path() === '/behavior' ? 'page' : null}>Actions</a>

      <label>Building</label>
      <a href="/reactivity" aria-current={path() === '/reactivity' ? 'page' : null}>State</a>
      <a href="/templates" aria-current={path() === '/templates' ? 'page' : null}>Templates</a>
      <a href="/data" aria-current={path() === '/data' ? 'page' : null}>Data &amp; forms</a>

      <label>Reference</label>
      <a href="/reference" aria-current={path() === '/reference' ? 'page' : null}>API reference</a>
    </navbar>
  {/if}

  {#if workbench() || landing()}
    <main sx="grow-1 min-w-0">
      <slot name="route"></slot>
    </main>
  {:else}
    <main sx="p-6 grow-1 min-w-0">
      <slot name="route"></slot>
    </main>
  {/if}
</row>
