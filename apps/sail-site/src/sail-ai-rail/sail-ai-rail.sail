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
 const shell = style({ box: minH(0).h(full).minW(0).overflow(hidden) })
 const rail = style({ box: display(none).flexDir(column).w(56).minH(0).h(full).padY(4).padX(3).overflowY(auto).overflowX(hidden), paint: borderEWidth(1).border(border).bg(navbar).fg(navbar-foreground) }).when(tablet, { box: display(flex) })
 const railLabel = style({ box: mt(4).mb(1), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) })
 const railLink = style({ box: display(block).padY(1).padX(2), paint: radius(md).fg(muted-foreground), text: fontSize(sm) }).when(hover, { paint: fg(foreground).bg(hover-bg) }).when(current, { paint: fg(primary).bg(muted), text: fontWeight(semibold) })
 const mainDocs = style({ box: pad(4).basis(0).minH(0).overflowX(hidden).overflowY(auto) }).when(tablet, { box: pad(6) })
 const mobileBar = style({ box: display(flex).gap(2).padX(3).padY(2), paint: borderBWidth(1).borderB(border).bg(navbar) }).when(tablet, { box: display(none) })
</scope>

<row nowrap align="stretch" gap="0">
  style([ style(shell), style({ box: w(full).grow(1).minW(0) }) ])

 <nav label="AI docs">
   style([ style(rail), style({ box: shrink(0) }) ])

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

 <column gap="0">
   style({  box: grow(1).minW(0).minH(0)  })

 <div>
   style(mobileBar)

 <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
 <small>
   style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })
AI docs</small>
 </div>
 <main>
   style([ style(mainDocs), style({ box: grow(1).minW(0) }) ])

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
 on-close={closeNav}
>
 <header>
  <row align="center" justify="between" gap="2">
    style({ box: w(full) })

   <strong>
     style({ text: fontSize(sm) })
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
