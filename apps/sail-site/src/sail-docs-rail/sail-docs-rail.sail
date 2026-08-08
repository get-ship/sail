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
 s-displaynone); Menu opens a fixed drawer

 Rail is a <div display:flex; flex-dir:column>, never <column>, so
 display(none) is not overridden by .sail-col { display:flex }.
-->

<controller>
/**
 * Path for aria-current on rail links. Layout chrome stays mounted across leaf
 * navigations — subscribe to route.pathname (Deck updates the shared signal on
 * navigate). No history monkey-patch / bare mount required for current state.
 */
function useLivePath(ctx: { route?: { pathname?: () => string } }) {
 return derived(() => {
 const fromRoute = ctx.route?.pathname?.()
 if (typeof fromRoute === 'string' && fromRoute.length > 0) return fromRoute
 if (typeof location !== 'undefined' && location.pathname) return location.pathname
 return '/'
 })
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
 style shell = { box: minH(0).h(full).minW(0).overflow(hidden) }
 // Plain div rail: fixed width, never grows into the content pane.
 // Mobile base = display(none); tablet+ restores flex column.
 // NOT <column> — .sail-col { display:flex } beats .s-displaynone.
 // 14rem rail (space-56 ≡ --panel-rail-w).
 style rail = { box: display(none).flexDir(column).w(56).minH(0).h(full).padY(4).padX(3).overflowY(auto).overflowX(hidden), paint: borderEWidth(1).border(border).bg(background).fg(foreground), when: { tablet: { box: display(flex) } } }
 style railLabel = { box: mt(4).mb(1), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
 // Current must paint over the base muted ink — use a filled pill so the
 // active page is obvious even when cascade order is unkind.
 style railLink = { box: display(block).padY(1).padX(2), paint: radius(md).fg(muted-foreground), text: fontSize(sm), when: { hover: { paint: fg(foreground).bg(hover-bg) }, current: { paint: fg(primary).bg(muted), text: fontWeight(semibold) } } }
 // Docs main: own scrollport (independent of rail).
 style mainDocs = { box: pad(4).basis(0).minH(0).overflowX(hidden).overflowY(auto), when: { tablet: { box: pad(6) } } }
 // Workbench: flex column + h(fill). Route roots must also use h(fill)
 // (tutorial) — grow alone collapses when the slot host is a block wrapper.
 style mainWorkbench = { box: display(flex).flexDir(column).basis(0).minH(0).minW(0).h(fill).overflow(hidden) }
 // Mobile-only chrome — plain div so display:none wins at tablet+.
 style mobileBar = { box: display(flex).gap(2).padX(3).padY(2), paint: borderBWidth(1).borderB(border).bg(background), when: { tablet: { box: display(none) } } }
 style contentCol = { box: minH(0).minW(0) }
</scope>

<if {workbench()}>
 <!-- Full-bleed workbench: no docs rail, fill the viewport shell. -->
 <row nowrap align="stretch" gap="0" style(shell, { box: w(full).grow(1).minW(0) })>
 <column gap="0" style(contentCol, { box: grow(1).minW(0) })>
 <main style(mainWorkbench, { box: grow(1).minW(0).w(full) })>
 <slot name="route"></slot>
 </main>
 </column>
 </row>
{else => <row nowrap align="stretch" gap="0" style(shell, { box: w(full).grow(1).minW(0) })>
 <nav label="Sail docs" style(rail, { box: shrink(0) })>
 <p style(railLabel)>Getting started</p>
 <a href="/sail/overview" current={path() === '/sail/overview' ? 'page' : null} style(railLink)>Introduction</a>
 <a href="/sail/getting-started" current={path() === '/sail/getting-started' ? 'page' : null} style(railLink)>Installation</a>
 <a href="/sail/why" current={path() === '/sail/why' ? 'page' : null} style(railLink)>Why Sail</a>
 <a href="/sail/tutorial" current={path().startsWith('/sail/tutorial') ? 'page' : null} style(railLink)>Tutorial</a>
 <a href="/showcase" current={path() === '/showcase' ? 'page' : null} style(railLink)>Showcase</a>

 <p style(railLabel)>Concepts</p>
 <a href="/sail/language/sail-file" current={path() === '/sail/language/sail-file' ? 'page' : null} style(railLink)>Components</a>
 <a href="/sail/language/state" current={path() === '/sail/language/state' ? 'page' : null} style(railLink)>State</a>
 <a href="/sail/language/templates" current={path() === '/sail/language/templates' ? 'page' : null} style(railLink)>Templates</a>
 <a href="/sail/language/styling" current={path() === '/sail/language/styling' ? 'page' : null} style(railLink)>Styling</a>
 <a href="/sail/language/layout" current={path() === '/sail/language/layout' ? 'page' : null} style(railLink)>Layout</a>
 <a href="/sail/language/animation" current={path() === '/sail/language/animation' ? 'page' : null} style(railLink)>Animation</a>
 <a href="/sail/language/actions" current={path() === '/sail/language/actions' ? 'page' : null} style(railLink)>Actions</a>
 <a href="/sail/language/data" current={path() === '/sail/language/data' ? 'page' : null} style(railLink)>Data</a>
 <a href="/sail/language/capabilities" current={path() === '/sail/language/capabilities' ? 'page' : null} style(railLink)>Capabilities</a>
 <a href="/sail/language/errors" current={path() === '/sail/language/errors' ? 'page' : null} style(railLink)>Errors</a>
 <a href="/sail/language/theming" current={path() === '/sail/language/theming' ? 'page' : null} style(railLink)>Theming</a>

 <p style(railLabel)>Guide</p>
 <a href="/sail/elements" current={path() === '/sail/elements' ? 'page' : null} style(railLink)>Using components</a>
 <a href="/sail/guide/building-a-component" current={path() === '/sail/guide/building-a-component' ? 'page' : null} style(railLink)>Building a component</a>
 <a href="/sail/guide/forms" current={path() === '/sail/guide/forms' ? 'page' : null} style(railLink)>Forms</a>
 <a href="/sail/guide/making-a-theme" current={path() === '/sail/guide/making-a-theme' ? 'page' : null} style(railLink)>Making a theme</a>
 <a href="/sail/guide/embedding-third-party-code" current={path() === '/sail/guide/embedding-third-party-code' ? 'page' : null} style(railLink)>Embedding third-party code</a>

 <p style(railLabel)>Reference</p>
 <a href="/sail/reference" current={path() === '/sail/reference' ? 'page' : null} style(railLink)>API reference</a>
 <a href="/sail/reference/sx" current={path() === '/sail/reference/sx' ? 'page' : null} style(railLink)>Style</a>
 <a href="/sail/reference/template" current={path() === '/sail/reference/template' ? 'page' : null} style(railLink)>Template / callbacks</a>
 <a href="/sail/reference/regions" current={path() === '/sail/reference/regions' ? 'page' : null} style(railLink)>Region matrix</a>
 </nav>

 <column gap="0" style(contentCol, { box: grow(1).minW(0) })>
 <div style(mobileBar)>
 <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
 <small style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>Sail docs</small>
 </div>
 <main style(mainDocs, { box: grow(1).minW(0) })>
 <slot name="route"></slot>
 </main>
 </column>
 </row>}</if>

<dialog
 id="sail-docs-drawer"
 open={navOpen()}
 side="start"
 size="sm"
 label="Sail docs navigation"
 on-close={closeNav}
>
 <header>
  <row align="center" justify="between" gap="2" style({ box: w(full) })>
   <strong style({ text: fontSize(sm) })>Sail docs</strong>
   <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
  </row>
 </header>
 <section>
  <p style(railLabel)>Getting started</p>
  <a href="/sail/overview" on:click={closeNav} current={path() === '/sail/overview' ? 'page' : null} style(railLink)>Introduction</a>
  <a href="/sail/getting-started" on:click={closeNav} current={path() === '/sail/getting-started' ? 'page' : null} style(railLink)>Installation</a>
  <a href="/sail/why" on:click={closeNav} current={path() === '/sail/why' ? 'page' : null} style(railLink)>Why Sail</a>
  <a href="/sail/tutorial" on:click={closeNav} current={path().startsWith('/sail/tutorial') ? 'page' : null} style(railLink)>Tutorial</a>
  <a href="/showcase" on:click={closeNav} current={path() === '/showcase' ? 'page' : null} style(railLink)>Showcase</a>

  <p style(railLabel)>Concepts</p>
  <a href="/sail/language/sail-file" on:click={closeNav} current={path() === '/sail/language/sail-file' ? 'page' : null} style(railLink)>Components</a>
  <a href="/sail/language/state" on:click={closeNav} current={path() === '/sail/language/state' ? 'page' : null} style(railLink)>State</a>
  <a href="/sail/language/templates" on:click={closeNav} current={path() === '/sail/language/templates' ? 'page' : null} style(railLink)>Templates</a>
  <a href="/sail/language/styling" on:click={closeNav} current={path() === '/sail/language/styling' ? 'page' : null} style(railLink)>Styling</a>
  <a href="/sail/language/layout" on:click={closeNav} current={path() === '/sail/language/layout' ? 'page' : null} style(railLink)>Layout</a>
  <a href="/sail/language/animation" on:click={closeNav} current={path() === '/sail/language/animation' ? 'page' : null} style(railLink)>Animation</a>
  <a href="/sail/language/actions" on:click={closeNav} current={path() === '/sail/language/actions' ? 'page' : null} style(railLink)>Actions</a>
  <a href="/sail/language/data" on:click={closeNav} current={path() === '/sail/language/data' ? 'page' : null} style(railLink)>Data</a>
  <a href="/sail/language/capabilities" on:click={closeNav} current={path() === '/sail/language/capabilities' ? 'page' : null} style(railLink)>Capabilities</a>
  <a href="/sail/language/errors" on:click={closeNav} current={path() === '/sail/language/errors' ? 'page' : null} style(railLink)>Errors</a>
  <a href="/sail/language/theming" on:click={closeNav} current={path() === '/sail/language/theming' ? 'page' : null} style(railLink)>Theming</a>

  <p style(railLabel)>Guide</p>
  <a href="/sail/elements" on:click={closeNav} current={path() === '/sail/elements' ? 'page' : null} style(railLink)>Using components</a>
  <a href="/sail/guide/building-a-component" on:click={closeNav} current={path() === '/sail/guide/building-a-component' ? 'page' : null} style(railLink)>Building a component</a>
  <a href="/sail/guide/forms" on:click={closeNav} current={path() === '/sail/guide/forms' ? 'page' : null} style(railLink)>Forms</a>
  <a href="/sail/guide/making-a-theme" on:click={closeNav} current={path() === '/sail/guide/making-a-theme' ? 'page' : null} style(railLink)>Making a theme</a>
  <a href="/sail/guide/embedding-third-party-code" on:click={closeNav} current={path() === '/sail/guide/embedding-third-party-code' ? 'page' : null} style(railLink)>Embedding third-party code</a>

  <p style(railLabel)>Reference</p>
  <a href="/sail/reference" on:click={closeNav} current={path() === '/sail/reference' ? 'page' : null} style(railLink)>API reference</a>
  <a href="/sail/reference/sx" on:click={closeNav} current={path() === '/sail/reference/sx' ? 'page' : null} style(railLink)>Style</a>
  <a href="/sail/reference/template" on:click={closeNav} current={path() === '/sail/reference/template' ? 'page' : null} style(railLink)>Template / callbacks</a>
  <a href="/sail/reference/regions" on:click={closeNav} current={path() === '/sail/reference/regions' ? 'page' : null} style(railLink)>Region matrix</a>
 </section>
</dialog>
