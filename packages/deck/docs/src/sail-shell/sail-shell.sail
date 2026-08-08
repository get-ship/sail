<!-- Docs chrome: top bar, sidebar, route slot. Persists across client navigations. -->

<controller>
import { controller, signal } from '@ship/sail-runtime'

const MODE_KEY = 'deck-mode'

export default controller(() => {
  const root = document.documentElement
  root.setAttribute('data-theme', 'deck')

  const stored = (() => {
    try {
      return localStorage.getItem(MODE_KEY)
    } catch {
      return null
    }
  })()
  // Dark-first; only 'light' in storage opts out.
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
      /* no storage */
    }
  }

  // mountRouter rebuilds chrome on each nav; path is re-read at scope time.
  const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')

  return { dark, toggleTheme, path }
})
</controller>

<scope>
  style brand = { text: fontSize(lg).tracking(tight) }
  style brandDeck = { paint: fg(primary) }
  style version = { box: padX(2), paint: fg(muted-foreground).borderWidth(1).border(border).radius(full), text: fontFamily(mono).fontSize(xs) }
  style main = { box: grow(1).minW(0).pad(7).maxW(breakpoint-tablet) }
</scope>

<navbar sticky bordered>
  <a slot="brand" href="/" aria-label="deck home">
    <row gap="2" align="center">
      <svg width="24" height="24" viewBox="0 0 26 26" fill="none" aria-hidden="true">
        <rect x="3" y="7" width="13" height="17" rx="3" fill="var(--accent)" opacity="0.55" />
        <rect x="7" y="4" width="13" height="17" rx="3" fill="var(--primary)" opacity="0.75" />
        <rect x="10.5" y="1.5" width="13" height="17" rx="3" fill="var(--primary)" />
      </svg>
      <strong style(brand)><b style(brandDeck)>deck</b> docs</strong>
      <small style(version)>0.2.0</small>
    </row>
  </a>

  <button slot="actions" variant="outline" size="sm" on:click={() => toggleTheme()}>
    {dark() ? '☀ Light' : '☾ Dark'}
  </button>
</navbar>

<row align="stretch">
  <navbar placement="rail" bordered sticky offset="14">
    <label>Introduction</label>
    <a href="/" aria-current={path() === '/' ? 'page' : null}>Overview</a>
    <a href="/getting-started" aria-current={path() === '/getting-started' ? 'page' : null}>Getting started</a>

    <label>Routing</label>
    <a href="/routing" aria-current={path() === '/routing' ? 'page' : null}>File routing</a>
    <a href="/dynamic-routes" aria-current={path() === '/dynamic-routes' ? 'page' : null}>Dynamic routes</a>
    <a href="/layouts" aria-current={path() === '/layouts' ? 'page' : null}>Layouts</a>

    <label>Reactivity</label>
    <a href="/reactivity" aria-current={path() === '/reactivity' ? 'page' : null}>Reactivity</a>
    <a href="/data" aria-current={path() === '/data' ? 'page' : null}>The data seam</a>

    <label>Rendering &amp; build</label>
    <a href="/rendering" aria-current={path() === '/rendering' ? 'page' : null}>Rendering modes</a>
    <a href="/hmr" aria-current={path() === '/hmr' ? 'page' : null}>HMR</a>
    <a href="/build" aria-current={path() === '/build' ? 'page' : null}>Build &amp; deploy</a>
  </navbar>

  <main style(main)>
    <slot name="route"></slot>
  </main>
</row>
