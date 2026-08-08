<!--
  <sail-deck-rail> — Deck docs sidebar + content main.
  Geometry matches sail-docs-rail: shell fill, independent rail/main scrollports.
  Rail is a plain <nav> (not <column>) so display(none) wins below tablet.
-->

<controller>
export default controller((ctx) => {
  // Layout chrome stays mounted across leaf routes — re-read path for current=.
  const path = derived(() => {
    const fromRoute = (ctx as { route?: { pathname?: () => string } }).route?.pathname?.()
    if (typeof fromRoute === 'string' && fromRoute.length > 0) return fromRoute
    if (typeof location !== 'undefined' && location.pathname) return location.pathname
    return '/'
  })
  const navOpen = signal(false)
  const openNav = (): void => navOpen.set(true)
  const closeNav = (): void => navOpen.set(false)
  const noopNav = (): void => {}
  // Close drawer when viewport crosses into tablet+ (CSS shows the rail).
  mount(() => {
    if (typeof window === 'undefined') return
    const mq = window.matchMedia('(min-width: 768px)')
    const onChange = (): void => {
      if (mq.matches) navOpen.set(false)
    }
    mq.addEventListener('change', onChange)
    return () => mq.removeEventListener('change', onChange)
  })
  effect(() => {
    path()
    navOpen.set(false)
  })
  return { path, navOpen, openNav, closeNav, noopNav }
})
</controller>

<scope>
  style shell = { box: minH(0).h(full).minW(0).overflow(hidden) }
  style rail = { box: display(none).flexDir(column).w(56).minH(0).h(full).padY(4).padX(3).overflowY(auto).overflowX(hidden), paint: borderEWidth(1).border(border).bg(background).fg(foreground), when: { tablet: { box: display(flex) } } }
  style railLabel = { box: mt(4).mb(1), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style railLink = { box: display(block).padY(1).padX(2), paint: radius(md).fg(muted-foreground), text: fontSize(sm), when: { hover: { paint: fg(foreground).bg(hover-bg) }, current: { paint: fg(primary).bg(muted), text: fontWeight(semibold) } } }
  style mainDocs = { box: pad(4).basis(0).minH(0).overflowX(hidden).overflowY(auto), when: { tablet: { box: pad(6) } } }
  style mobileBar = { box: display(flex).gap(2).padX(3).padY(2), paint: borderBWidth(1).borderB(border).bg(background), when: { tablet: { box: display(none) } } }
  style contentCol = { box: grow(1).minW(0).minH(0) }
  style mobileLabel = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) }
  style drawerTitle = { text: fontSize(sm) }
  style fullW = { box: w(full) }

  type DeckNavProps = {
    path: any
    onNav: any
  }
</scope>

<!-- Same link tree for side rail and mobile drawer; onNav closes drawer (noop on rail). -->
<partial deck-nav props: DeckNavProps>
  <p style(railLabel)>Introduction</p>
  <a href="/deck" on:click={onNav} current={path() === '/deck' ? 'page' : null} style(railLink)>Overview</a>
  <a href="/deck/getting-started" on:click={onNav} current={path() === '/deck/getting-started' ? 'page' : null} style(railLink)>Getting started</a>

  <p style(railLabel)>Concepts</p>
  <a href="/deck/routing" on:click={onNav} current={path() === '/deck/routing' ? 'page' : null} style(railLink)>Routing</a>
  <a href="/deck/dynamic-routes" on:click={onNav} current={path() === '/deck/dynamic-routes' ? 'page' : null} style(railLink)>Dynamic routes</a>
  <a href="/deck/layouts" on:click={onNav} current={path() === '/deck/layouts' ? 'page' : null} style(railLink)>Layouts</a>
  <a href="/deck/rendering" on:click={onNav} current={path() === '/deck/rendering' ? 'page' : null} style(railLink)>Rendering</a>
  <a href="/deck/reactivity" on:click={onNav} current={path() === '/deck/reactivity' ? 'page' : null} style(railLink)>Reactivity</a>
  <a href="/deck/data" on:click={onNav} current={path() === '/deck/data' ? 'page' : null} style(railLink)>Live data</a>

  <p style(railLabel)>Tooling</p>
  <a href="/deck/hmr" on:click={onNav} current={path() === '/deck/hmr' ? 'page' : null} style(railLink)>HMR</a>
  <a href="/deck/build" on:click={onNav} current={path() === '/deck/build' ? 'page' : null} style(railLink)>Build</a>
</partial>

<row nowrap align="stretch" gap="0" style(shell, { box: w(full).grow(1).minW(0) })>
  <nav label="Deck docs" style(rail, { box: shrink(0) })>
    <deck-nav path={path} onNav={noopNav}></deck-nav>
  </nav>

  <column gap="0" style(contentCol)>
    <div style(mobileBar)>
      <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
      <small style(mobileLabel)>Deck docs</small>
    </div>
    <main style(mainDocs, { box: grow(1).minW(0) })>
      <slot name="route"></slot>
    </main>
  </column>
</row>

<dialog
  id="sail-deck-drawer"
  open={navOpen()}
  side="start"
  size="sm"
  label="Deck docs navigation"
  on-close={closeNav}
>
  <header>
    <row align="center" justify="between" gap="2" style(fullW)>
      <strong style(drawerTitle)>Deck docs</strong>
      <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
    </row>
  </header>
  <section>
    <deck-nav path={path} onNav={closeNav}></deck-nav>
  </section>
</dialog>
