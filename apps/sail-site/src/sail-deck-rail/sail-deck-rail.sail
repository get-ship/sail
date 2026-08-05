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

<row nowrap align="stretch" w="full" grow min-w="0" gap="0" sx={shell}>
 <nav shrink="0" sx={rail} label="Deck docs">
 <p sx={railLabel}>Introduction</p>
 <a href="/deck" sx={railLink} current={path() === '/deck' ? 'page' : null}>Overview</a>
 <a href="/deck/getting-started" sx={railLink} current={path() === '/deck/getting-started' ? 'page' : null}>Getting started</a>

 <p sx={railLabel}>Concepts</p>
 <a href="/deck/routing" sx={railLink} current={path() === '/deck/routing' ? 'page' : null}>Routing</a>
 <a href="/deck/dynamic-routes" sx={railLink} current={path() === '/deck/dynamic-routes' ? 'page' : null}>Dynamic routes</a>
 <a href="/deck/layouts" sx={railLink} current={path() === '/deck/layouts' ? 'page' : null}>Layouts</a>
 <a href="/deck/rendering" sx={railLink} current={path() === '/deck/rendering' ? 'page' : null}>Rendering</a>
 <a href="/deck/reactivity" sx={railLink} current={path() === '/deck/reactivity' ? 'page' : null}>Reactivity</a>
 <a href="/deck/data" sx={railLink} current={path() === '/deck/data' ? 'page' : null}>Live data</a>

 <p sx={railLabel}>Tooling</p>
 <a href="/deck/hmr" sx={railLink} current={path() === '/deck/hmr' ? 'page' : null}>HMR</a>
 <a href="/deck/build" sx={railLink} current={path() === '/deck/build' ? 'page' : null}>Build</a>
 </nav>

 <column grow min-w="0" gap="0" sx={style.minH(0).minW(0)}>
 <div sx={mobileBar}>
 <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
 <small sx={style.fontFamily(mono).fontSize(xs).fg(muted-foreground)}>Deck docs</small>
 </div>
 <main grow min-w="0" sx={mainDocs}>
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
   <strong sx={style.fontSize(sm)}>Deck docs</strong>
   <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
  </row>
 </header>
 <section>
  <p sx={railLabel}>Introduction</p>
  <a href="/deck" sx={railLink} on:click={closeNav} current={path() === '/deck' ? 'page' : null}>Overview</a>
  <a href="/deck/getting-started" sx={railLink} on:click={closeNav} current={path() === '/deck/getting-started' ? 'page' : null}>Getting started</a>
  <p sx={railLabel}>Concepts</p>
  <a href="/deck/routing" sx={railLink} on:click={closeNav} current={path() === '/deck/routing' ? 'page' : null}>Routing</a>
  <a href="/deck/dynamic-routes" sx={railLink} on:click={closeNav} current={path() === '/deck/dynamic-routes' ? 'page' : null}>Dynamic routes</a>
  <a href="/deck/layouts" sx={railLink} on:click={closeNav} current={path() === '/deck/layouts' ? 'page' : null}>Layouts</a>
  <a href="/deck/rendering" sx={railLink} on:click={closeNav} current={path() === '/deck/rendering' ? 'page' : null}>Rendering</a>
  <a href="/deck/reactivity" sx={railLink} on:click={closeNav} current={path() === '/deck/reactivity' ? 'page' : null}>Reactivity</a>
  <a href="/deck/data" sx={railLink} on:click={closeNav} current={path() === '/deck/data' ? 'page' : null}>Live data</a>
  <p sx={railLabel}>Tooling</p>
  <a href="/deck/hmr" sx={railLink} on:click={closeNav} current={path() === '/deck/hmr' ? 'page' : null}>HMR</a>
  <a href="/deck/build" sx={railLink} on:click={closeNav} current={path() === '/deck/build' ? 'page' : null}>Build</a>
 </section>
</dialog>
