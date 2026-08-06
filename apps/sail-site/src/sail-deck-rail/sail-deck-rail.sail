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
</scope>

<row nowrap align="stretch" w="full" grow min-w="0" gap="0">
  style(shell)

 <nav shrink="0" label="Deck docs">
   style(rail)

 <p>
   style(railLabel)
Introduction</p>
 <a href="/deck" current={path() === '/deck' ? 'page' : null}>
   style(railLink)
Overview</a>
 <a href="/deck/getting-started" current={path() === '/deck/getting-started' ? 'page' : null}>
   style(railLink)
Getting started</a>

 <p>
   style(railLabel)
Concepts</p>
 <a href="/deck/routing" current={path() === '/deck/routing' ? 'page' : null}>
   style(railLink)
Routing</a>
 <a href="/deck/dynamic-routes" current={path() === '/deck/dynamic-routes' ? 'page' : null}>
   style(railLink)
Dynamic routes</a>
 <a href="/deck/layouts" current={path() === '/deck/layouts' ? 'page' : null}>
   style(railLink)
Layouts</a>
 <a href="/deck/rendering" current={path() === '/deck/rendering' ? 'page' : null}>
   style(railLink)
Rendering</a>
 <a href="/deck/reactivity" current={path() === '/deck/reactivity' ? 'page' : null}>
   style(railLink)
Reactivity</a>
 <a href="/deck/data" current={path() === '/deck/data' ? 'page' : null}>
   style(railLink)
Live data</a>

 <p>
   style(railLabel)
Tooling</p>
 <a href="/deck/hmr" current={path() === '/deck/hmr' ? 'page' : null}>
   style(railLink)
HMR</a>
 <a href="/deck/build" current={path() === '/deck/build' ? 'page' : null}>
   style(railLink)
Build</a>
 </nav>

 <column grow min-w="0" gap="0">
   style.minH(0).minW(0)

 <div>
   style(mobileBar)

 <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
 <small>
   style.fontFamily(mono).fontSize(xs).fg(muted-foreground)
Deck docs</small>
 </div>
 <main grow min-w="0">
   style(mainDocs)

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
 on:sail-close={closeNav}
>
 <header>
  <row align="center" justify="between" gap="2" w="full">
   <strong>
     style.fontSize(sm)
Deck docs</strong>
   <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
  </row>
 </header>
 <section>
  <p>
    style(railLabel)
Introduction</p>
  <a href="/deck" on:click={closeNav} current={path() === '/deck' ? 'page' : null}>
    style(railLink)
Overview</a>
  <a href="/deck/getting-started" on:click={closeNav} current={path() === '/deck/getting-started' ? 'page' : null}>
    style(railLink)
Getting started</a>
  <p>
    style(railLabel)
Concepts</p>
  <a href="/deck/routing" on:click={closeNav} current={path() === '/deck/routing' ? 'page' : null}>
    style(railLink)
Routing</a>
  <a href="/deck/dynamic-routes" on:click={closeNav} current={path() === '/deck/dynamic-routes' ? 'page' : null}>
    style(railLink)
Dynamic routes</a>
  <a href="/deck/layouts" on:click={closeNav} current={path() === '/deck/layouts' ? 'page' : null}>
    style(railLink)
Layouts</a>
  <a href="/deck/rendering" on:click={closeNav} current={path() === '/deck/rendering' ? 'page' : null}>
    style(railLink)
Rendering</a>
  <a href="/deck/reactivity" on:click={closeNav} current={path() === '/deck/reactivity' ? 'page' : null}>
    style(railLink)
Reactivity</a>
  <a href="/deck/data" on:click={closeNav} current={path() === '/deck/data' ? 'page' : null}>
    style(railLink)
Live data</a>
  <p>
    style(railLabel)
Tooling</p>
  <a href="/deck/hmr" on:click={closeNav} current={path() === '/deck/hmr' ? 'page' : null}>
    style(railLink)
HMR</a>
  <a href="/deck/build" on:click={closeNav} current={path() === '/deck/build' ? 'page' : null}>
    style(railLink)
Build</a>
 </section>
</dialog>
