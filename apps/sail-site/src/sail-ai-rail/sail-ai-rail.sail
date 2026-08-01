<!--
  <sail-ai-rail> — AI docs sidebar + content main.
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
  // Fill site-header <main>; rail + content independent scrollports.
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
  <div shrink="0" sx={rail} role="navigation" aria-label="AI docs">
    <p sx={railLabel}>AI</p>
    <a href="/ai" sx={railLink} aria-current={path() === '/ai' ? 'page' : null}>Overview</a>
    <a href="/ai/authoring" sx={railLink} aria-current={path() === '/ai/authoring' ? 'page' : null}>Authoring</a>
    <a href="/ai/tooling" sx={railLink} aria-current={path() === '/ai/tooling' ? 'page' : null}>Tooling</a>
    <a href="/ai/eval" sx={railLink} aria-current={path() === '/ai/eval' ? 'page' : null}>Eval</a>
  </div>

  <column grow min-w="0" gap="0" sx={style.minH(0).minW(0)}>
    <div sx={mobileBar}>
      <button type="button" variant="outline" size="sm" on:click={openNav} aria-expanded={navOpen() ? 'true' : 'false'} aria-controls="sail-ai-drawer">Menu</button>
      <small sx={style.fontFamily(mono).fontSize(xs).fg(muted-foreground)}>AI docs</small>
    </div>
    <main grow min-w="0" sx={mainDocs}>
      <slot name="route"></slot>
    </main>
  </column>
</row>

<if {navOpen()}>
  <div sx={drawer} id="sail-ai-drawer" role="dialog" aria-modal="true" aria-label="AI docs navigation">
    <div sx={drawerPanel}>
      <row align="center" justify="between" gap="2" sx={style.padB(3).borderBWidth(1).borderB(border).mb(2)}>
        <strong sx={style.fontSize(sm)}>AI docs</strong>
        <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
      </row>
      <p sx={railLabel}>AI</p>
      <a href="/ai" sx={railLink} on:click={closeNav} aria-current={path() === '/ai' ? 'page' : null}>Overview</a>
      <a href="/ai/authoring" sx={railLink} on:click={closeNav} aria-current={path() === '/ai/authoring' ? 'page' : null}>Authoring</a>
      <a href="/ai/tooling" sx={railLink} on:click={closeNav} aria-current={path() === '/ai/tooling' ? 'page' : null}>Tooling</a>
      <a href="/ai/eval" sx={railLink} on:click={closeNav} aria-current={path() === '/ai/eval' ? 'page' : null}>Eval</a>
    </div>
    <div grow sx={backdrop} on:click={closeNav} aria-hidden="true"></div>
  </div>
</if>
