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
</scope>

<row nowrap align="stretch" w="full" grow min-w="0" gap="0">
  style(shell)

 <nav shrink="0" label="AI docs">
   style(rail)

 <p>
   style(railLabel)
AI</p>
 <a href="/ai" current={path() === '/ai' ? 'page' : null}>
   style(railLink)
Overview</a>
 <a href="/ai/authoring" current={path() === '/ai/authoring' ? 'page' : null}>
   style(railLink)
Authoring</a>
 <a href="/ai/tooling" current={path() === '/ai/tooling' ? 'page' : null}>
   style(railLink)
Tooling</a>
 <a href="/ai/eval" current={path() === '/ai/eval' ? 'page' : null}>
   style(railLink)
Eval</a>
 </nav>

 <column grow min-w="0" gap="0">
   style.minH(0).minW(0)

 <div>
   style(mobileBar)

 <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
 <small>
   style.fontFamily(mono).fontSize(xs).fg(muted-foreground)
AI docs</small>
 </div>
 <main grow min-w="0">
   style(mainDocs)

 <slot name="route"></slot>
 </main>
 </column>
</row>

<dialog
 id="sail-ai-drawer"
 open={navOpen()}
 side="start"
 size="sm"
 label="AI docs navigation"
 on:sail-close={closeNav}
>
 <header>
  <row align="center" justify="between" gap="2" w="full">
   <strong>
     style.fontSize(sm)
AI docs</strong>
   <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
  </row>
 </header>
 <section>
  <p>
    style(railLabel)
AI</p>
  <a href="/ai" on:click={closeNav} current={path() === '/ai' ? 'page' : null}>
    style(railLink)
Overview</a>
  <a href="/ai/authoring" on:click={closeNav} current={path() === '/ai/authoring' ? 'page' : null}>
    style(railLink)
Authoring</a>
  <a href="/ai/tooling" on:click={closeNav} current={path() === '/ai/tooling' ? 'page' : null}>
    style(railLink)
Tooling</a>
  <a href="/ai/eval" on:click={closeNav} current={path() === '/ai/eval' ? 'page' : null}>
    style(railLink)
Eval</a>
 </section>
</dialog>
