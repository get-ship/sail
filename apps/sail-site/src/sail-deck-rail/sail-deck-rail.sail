<!--
  <sail-deck-rail> — Deck docs sidebar + content main.
  CSS media rail (plain div) + mobile Menu drawer. Same contract as sail-docs-rail.
-->

<controller>
export default controller((ctx) => {
  const path = derived(() => {
    const r = (ctx as { route?: { pathname?: () => string } }).route
    if (r?.pathname) void r.pathname()
    if (typeof location !== 'undefined' && location.pathname) return location.pathname
    return r?.pathname?.() ?? '/'
  })
  const navOpen = signal(false)
  const openNav = (): void => navOpen.set(true)
  const closeNav = (): void => navOpen.set(false)
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
  return { path, navOpen, openNav, closeNav }
})
</controller>

<scope>
  // Fill site-header <main>; rail + content are independent scrollports.
  const shell = style.minH(0).h(full).minW(0).overflow(hidden)
  const rail = style.display(none).flexDir(column).w(56).minH(0).h(full).padY(4).padX(3).borderEWidth(1).border(border).overflowY(auto).overflowX(hidden).bg(navbar).fg(navbar-foreground).when({
    tablet: style.display(flex),
  })
  const railLabel = style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground).mt(4).mb(1)
  const railLink = style.display(block).fontSize(sm).padY(1).padX(2).radius(md).fg(muted-foreground).when({
    hover: style.fg(foreground).bg(hover-bg),
    current: style.fg(primary).fontWeight(semibold).bg(muted),
  })
  const mainDocs = style.pad(4).basis(0).minH(0).overflowX(hidden).overflowY(auto).when({
    tablet: style.pad(6),
  })
  const mobileBar = style.display(flex).gap(2).padX(3).padY(2).borderBWidth(1).borderB(border).bg(navbar).when({
    tablet: style.display(none),
  })
  const drawer = style.position(fixed).inset(0).z(popover).display(flex)
  const drawerPanel = style.display(flex).flexDir(column).w(56).maxW(full).h(full).padY(4).padX(3).bg(navbar).fg(navbar-foreground).overflowY(auto).overflowX(hidden).borderEWidth(1).border(border).shadow(lg)
  const backdrop = style.bg(foreground).opacity(40)
</scope>

<row nowrap align="stretch" w="full" grow min-w="0" gap="0" sx={shell}>
  <div shrink="0" sx={rail} role="navigation" aria-label="Deck docs">
    <p sx={railLabel}>Introduction</p>
    <a href="/deck" sx={railLink} aria-current={path() === '/deck' ? 'page' : null}>Overview</a>
    <a href="/deck/getting-started" sx={railLink} aria-current={path() === '/deck/getting-started' ? 'page' : null}>Getting started</a>

    <p sx={railLabel}>Concepts</p>
    <a href="/deck/routing" sx={railLink} aria-current={path() === '/deck/routing' ? 'page' : null}>Routing</a>
    <a href="/deck/dynamic-routes" sx={railLink} aria-current={path() === '/deck/dynamic-routes' ? 'page' : null}>Dynamic routes</a>
    <a href="/deck/layouts" sx={railLink} aria-current={path() === '/deck/layouts' ? 'page' : null}>Layouts</a>
    <a href="/deck/rendering" sx={railLink} aria-current={path() === '/deck/rendering' ? 'page' : null}>Rendering</a>
    <a href="/deck/reactivity" sx={railLink} aria-current={path() === '/deck/reactivity' ? 'page' : null}>Reactivity</a>
    <a href="/deck/data" sx={railLink} aria-current={path() === '/deck/data' ? 'page' : null}>Live data</a>

    <p sx={railLabel}>Tooling</p>
    <a href="/deck/hmr" sx={railLink} aria-current={path() === '/deck/hmr' ? 'page' : null}>HMR</a>
    <a href="/deck/build" sx={railLink} aria-current={path() === '/deck/build' ? 'page' : null}>Build</a>
  </div>

  <column grow min-w="0" gap="0" sx={style.minH(0).minW(0)}>
    <div sx={mobileBar}>
      <button type="button" variant="outline" size="sm" on:click={openNav} aria-expanded={navOpen() ? 'true' : 'false'} aria-controls="sail-deck-drawer">Menu</button>
      <small sx={style.fontFamily(mono).fontSize(xs).fg(muted-foreground)}>Deck docs</small>
    </div>
    <main grow min-w="0" sx={mainDocs}>
      <slot name="route"></slot>
    </main>
  </column>
</row>

<if {navOpen()}>
  <div sx={drawer} id="sail-deck-drawer" role="dialog" aria-modal="true" aria-label="Deck docs navigation">
    <div sx={drawerPanel}>
      <row align="center" justify="between" gap="2" sx={style.padB(3).borderBWidth(1).borderB(border).mb(2)}>
        <strong sx={style.fontSize(sm)}>Deck docs</strong>
        <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
      </row>
      <p sx={railLabel}>Introduction</p>
      <a href="/deck" sx={railLink} on:click={closeNav} aria-current={path() === '/deck' ? 'page' : null}>Overview</a>
      <a href="/deck/getting-started" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/getting-started' ? 'page' : null}>Getting started</a>
      <p sx={railLabel}>Concepts</p>
      <a href="/deck/routing" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/routing' ? 'page' : null}>Routing</a>
      <a href="/deck/dynamic-routes" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/dynamic-routes' ? 'page' : null}>Dynamic routes</a>
      <a href="/deck/layouts" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/layouts' ? 'page' : null}>Layouts</a>
      <a href="/deck/rendering" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/rendering' ? 'page' : null}>Rendering</a>
      <a href="/deck/reactivity" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/reactivity' ? 'page' : null}>Reactivity</a>
      <a href="/deck/data" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/data' ? 'page' : null}>Live data</a>
      <p sx={railLabel}>Tooling</p>
      <a href="/deck/hmr" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/hmr' ? 'page' : null}>HMR</a>
      <a href="/deck/build" sx={railLink} on:click={closeNav} aria-current={path() === '/deck/build' ? 'page' : null}>Build</a>
    </div>
    <div grow sx={backdrop} on:click={closeNav} aria-hidden="true"></div>
  </div>
</if>
