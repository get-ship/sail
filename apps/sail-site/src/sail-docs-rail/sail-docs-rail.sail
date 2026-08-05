<!--
 <sail-docs-rail> — Sail docs sidebar + content main for every /sail/* route.
 Tutorial LESSON pages (/sail/tutorial/<slug>) drop the rail (full-bleed workbench).

 Geometry (viewport-locked site-header <main> is the parent):
 - shell h(full) + overflow hidden — fill the budget, no page-level window scroll
 - rail: independent overflowY(auto), fixed width, stretches to shell height
 - mainDocs: independent overflowY(auto) — content scrolls; rail does not leave
 - workbench: no rail, main h(fill) for tutorial split panes

 Responsive — CSS media queries, NOT matchMedia unmount:
 - tablet+ (768px): fixed-width side rail
 - below tablet: rail display:none (on a plain div — .sail-col beats
 sx-displaynone); Menu opens a fixed drawer

 Rail is a <div display:flex; flex-dir:column>, never <column>, so
 display(none) is not overridden by .sail-col { display:flex }.
-->

<controller>
/**
 * Pathname that updates on SPA chrome-reuse. Layout controllers stay mounted
 * across leaf navigations; route.pathname alone has been observed to lag, so
 * we also listen to history + subscribe to the route signal.
 */
function useLivePath(ctx: { route?: { pathname?: () => string } }) {
 const path = signal(
 typeof location !== 'undefined' ? location.pathname : '/',
 )
 const sync = (): void => {
 const fromRoute = ctx.route?.pathname?.()
 const next =
 typeof location !== 'undefined' && location.pathname
 ? location.pathname
 : (fromRoute ?? path())
 if (next !== path()) path.set(next)
 }
 // Keep derived/route subscribers honest when the signal moves.
 effect(() => {
 const r = ctx.route?.pathname?.()
 if (typeof r === 'string' && r.length > 0 && r !== path()) path.set(r)
 })
 mount(() => {
 if (typeof window === 'undefined') return
 const onPop = (): void => sync()
 window.addEventListener('popstate', onPop)
 const wrap =
 (fn: History['pushState']) =>
 function (this: History, ...args: Parameters<History['pushState']>) {
 const ret = fn.apply(this, args)
 sync()
 return ret
 }
 const origPush = history.pushState.bind(history)
 const origReplace = history.replaceState.bind(history)
 history.pushState = wrap(origPush)
 history.replaceState = wrap(origReplace)
 // Capture-phase clicks on in-app links (deck may navigate after click).
 const onClick = (): void => {
 queueMicrotask(sync)
 }
 document.addEventListener('click', onClick, true)
 sync()
 return () => {
 window.removeEventListener('popstate', onPop)
 document.removeEventListener('click', onClick, true)
 history.pushState = origPush
 history.replaceState = origReplace
 }
 })
 return path
}

export default controller((ctx) => {
 const path = useLivePath(ctx as { route?: { pathname?: () => string } })
 // Tutorial lessons are full-bleed workbenches (no docs rail). Hub /sail/tutorial
 // stays in the docs chrome; only /sail/tutorial/<slug> is workbench.
 const workbench = derived(() => path().startsWith('/sail/tutorial/'))
 const navOpen = signal(false)
 const openNav = (): void => navOpen.set(true)
 const closeNav = (): void => navOpen.set(false)
 // Close drawer when the viewport crosses into tablet+ (CSS shows the rail).
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
 return { path, workbench, navOpen, openNav, closeNav }
})
</controller>

<scope>
 // Fill the site-header <main> budget. Independent rail + main scrollports.
 const shell = style.minH(0).h(full).minW(0).overflow(hidden)
 // Plain div rail: fixed width, never grows into the content pane.
 // Mobile base = display(none); tablet+ restores flex column.
 // NOT <column> — .sail-col { display:flex } beats .sx-displaynone.
 // 14rem rail (space-56 ≡ --panel-rail-w).
 const rail = style.display(none).flexDir(column).w(56).minH(0).h(full).padY(4).padX(3).borderEWidth(1).border(border).overflowY(auto).overflowX(hidden).bg(navbar).fg(navbar-foreground).when({
 tablet: style.display(flex),
 })
 const railLabel = style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground).mt(4).mb(1)
 // Current must paint over the base muted ink — use a filled pill so the
 // active page is obvious even when cascade order is unkind.
 const railLink = style.display(block).fontSize(sm).padY(1).padX(2).radius(md).fg(muted-foreground).when({
 hover: style.fg(foreground).bg(hover-bg),
 current: style.fg(primary).fontWeight(semibold).bg(muted),
 })
 // Docs main: own scrollport (independent of rail).
 const mainDocs = style.pad(4).basis(0).minH(0).overflowX(hidden).overflowY(auto).when({
 tablet: style.pad(6),
 })
 // Workbench: flex column + h(fill). Route roots must also use h(fill)
 // (tutorial) — grow alone collapses when the slot host is a block wrapper.
 const mainWorkbench = style.display(flex).flexDir(column).basis(0).minH(0).minW(0).h(fill).overflow(hidden)
 // Mobile-only chrome — plain div so display:none wins at tablet+.
 const mobileBar = style.display(flex).gap(2).padX(3).padY(2).borderBWidth(1).borderB(border).bg(navbar).when({
 tablet: style.display(none),
 })
 const contentCol = style.minH(0).minW(0)
</scope>

<if {workbench()}>
 <!-- Full-bleed workbench: no docs rail, fill the viewport shell. -->
 <row nowrap align="stretch" w="full" grow min-w="0" gap="0" sx={shell}>
 <column grow min-w="0" gap="0" sx={contentCol}>
 <main grow min-w="0" w="full" sx={mainWorkbench}>
 <slot name="route"></slot>
 </main>
 </column>
 </row>
<else>
 <row nowrap align="stretch" w="full" grow min-w="0" gap="0" sx={shell}>
 <nav shrink="0" sx={rail} label="Sail docs">
 <p sx={railLabel}>Getting started</p>
 <a href="/sail/overview" sx={railLink} current={path() === '/sail/overview' ? 'page' : null}>Introduction</a>
 <a href="/sail/getting-started" sx={railLink} current={path() === '/sail/getting-started' ? 'page' : null}>Installation</a>
 <a href="/sail/why" sx={railLink} current={path() === '/sail/why' ? 'page' : null}>Why Sail</a>
 <a href="/sail/tutorial" sx={railLink} current={path().startsWith('/sail/tutorial') ? 'page' : null}>Tutorial</a>
 <a href="/showcase" sx={railLink} current={path() === '/showcase' ? 'page' : null}>Showcase</a>

 <p sx={railLabel}>Concepts</p>
 <a href="/sail/language/sail-file" sx={railLink} current={path() === '/sail/language/sail-file' ? 'page' : null}>Components</a>
 <a href="/sail/language/state" sx={railLink} current={path() === '/sail/language/state' ? 'page' : null}>State</a>
 <a href="/sail/language/templates" sx={railLink} current={path() === '/sail/language/templates' ? 'page' : null}>Templates</a>
 <a href="/sail/language/styling" sx={railLink} current={path() === '/sail/language/styling' ? 'page' : null}>Styling</a>
 <a href="/sail/language/layout" sx={railLink} current={path() === '/sail/language/layout' ? 'page' : null}>Layout</a>
 <a href="/sail/language/animation" sx={railLink} current={path() === '/sail/language/animation' ? 'page' : null}>Animation</a>
 <a href="/sail/language/actions" sx={railLink} current={path() === '/sail/language/actions' ? 'page' : null}>Actions</a>
 <a href="/sail/language/data" sx={railLink} current={path() === '/sail/language/data' ? 'page' : null}>Data</a>
 <a href="/sail/language/capabilities" sx={railLink} current={path() === '/sail/language/capabilities' ? 'page' : null}>Capabilities</a>
 <a href="/sail/language/errors" sx={railLink} current={path() === '/sail/language/errors' ? 'page' : null}>Errors</a>
 <a href="/sail/language/theming" sx={railLink} current={path() === '/sail/language/theming' ? 'page' : null}>Theming</a>

 <p sx={railLabel}>Guide</p>
 <a href="/sail/elements" sx={railLink} current={path() === '/sail/elements' ? 'page' : null}>Using components</a>
 <a href="/sail/guide/building-a-component" sx={railLink} current={path() === '/sail/guide/building-a-component' ? 'page' : null}>Building a component</a>
 <a href="/sail/guide/forms" sx={railLink} current={path() === '/sail/guide/forms' ? 'page' : null}>Forms</a>
 <a href="/sail/guide/making-a-theme" sx={railLink} current={path() === '/sail/guide/making-a-theme' ? 'page' : null}>Making a theme</a>
 <a href="/sail/guide/embedding-third-party-code" sx={railLink} current={path() === '/sail/guide/embedding-third-party-code' ? 'page' : null}>Embedding third-party code</a>

 <p sx={railLabel}>Reference</p>
 <a href="/sail/reference" sx={railLink} current={path() === '/sail/reference' ? 'page' : null}>API reference</a>
 </nav>

 <column grow min-w="0" gap="0" sx={contentCol}>
 <div sx={mobileBar}>
 <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
 <small sx={style.fontFamily(mono).fontSize(xs).fg(muted-foreground)}>Sail docs</small>
 </div>
 <main grow min-w="0" sx={mainDocs}>
 <slot name="route"></slot>
 </main>
 </column>
 </row>
</else></if>

<dialog
 id="sail-docs-drawer"
 open={navOpen()}
 side="start"
 size="sm"
 label="Sail docs navigation"
 on:sail-close={closeNav}
>
 <header>
  <row align="center" justify="between" gap="2" w="full">
   <strong sx={style.fontSize(sm)}>Sail docs</strong>
   <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
  </row>
 </header>
 <section>
  <p sx={railLabel}>Getting started</p>
  <a href="/sail/overview" sx={railLink} on:click={closeNav} current={path() === '/sail/overview' ? 'page' : null}>Introduction</a>
  <a href="/sail/getting-started" sx={railLink} on:click={closeNav} current={path() === '/sail/getting-started' ? 'page' : null}>Installation</a>
  <a href="/sail/why" sx={railLink} on:click={closeNav} current={path() === '/sail/why' ? 'page' : null}>Why Sail</a>
  <a href="/sail/tutorial" sx={railLink} on:click={closeNav} current={path().startsWith('/sail/tutorial') ? 'page' : null}>Tutorial</a>
  <a href="/showcase" sx={railLink} on:click={closeNav} current={path() === '/showcase' ? 'page' : null}>Showcase</a>

  <p sx={railLabel}>Concepts</p>
  <a href="/sail/language/sail-file" sx={railLink} on:click={closeNav} current={path() === '/sail/language/sail-file' ? 'page' : null}>Components</a>
  <a href="/sail/language/state" sx={railLink} on:click={closeNav} current={path() === '/sail/language/state' ? 'page' : null}>State</a>
  <a href="/sail/language/templates" sx={railLink} on:click={closeNav} current={path() === '/sail/language/templates' ? 'page' : null}>Templates</a>
  <a href="/sail/language/styling" sx={railLink} on:click={closeNav} current={path() === '/sail/language/styling' ? 'page' : null}>Styling</a>
  <a href="/sail/language/layout" sx={railLink} on:click={closeNav} current={path() === '/sail/language/layout' ? 'page' : null}>Layout</a>
  <a href="/sail/language/animation" sx={railLink} on:click={closeNav} current={path() === '/sail/language/animation' ? 'page' : null}>Animation</a>
  <a href="/sail/language/actions" sx={railLink} on:click={closeNav} current={path() === '/sail/language/actions' ? 'page' : null}>Actions</a>
  <a href="/sail/language/data" sx={railLink} on:click={closeNav} current={path() === '/sail/language/data' ? 'page' : null}>Data</a>
  <a href="/sail/language/capabilities" sx={railLink} on:click={closeNav} current={path() === '/sail/language/capabilities' ? 'page' : null}>Capabilities</a>
  <a href="/sail/language/errors" sx={railLink} on:click={closeNav} current={path() === '/sail/language/errors' ? 'page' : null}>Errors</a>
  <a href="/sail/language/theming" sx={railLink} on:click={closeNav} current={path() === '/sail/language/theming' ? 'page' : null}>Theming</a>

  <p sx={railLabel}>Guide</p>
  <a href="/sail/elements" sx={railLink} on:click={closeNav} current={path() === '/sail/elements' ? 'page' : null}>Using components</a>
  <a href="/sail/guide/building-a-component" sx={railLink} on:click={closeNav} current={path() === '/sail/guide/building-a-component' ? 'page' : null}>Building a component</a>
  <a href="/sail/guide/forms" sx={railLink} on:click={closeNav} current={path() === '/sail/guide/forms' ? 'page' : null}>Forms</a>
  <a href="/sail/guide/making-a-theme" sx={railLink} on:click={closeNav} current={path() === '/sail/guide/making-a-theme' ? 'page' : null}>Making a theme</a>
  <a href="/sail/guide/embedding-third-party-code" sx={railLink} on:click={closeNav} current={path() === '/sail/guide/embedding-third-party-code' ? 'page' : null}>Embedding third-party code</a>

  <p sx={railLabel}>Reference</p>
  <a href="/sail/reference" sx={railLink} on:click={closeNav} current={path() === '/sail/reference' ? 'page' : null}>API reference</a>
 </section>
</dialog>
