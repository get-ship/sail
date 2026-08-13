<!--
  <sail-docs-rail> — single docs sidebar + content main for every /docs/* route.
  Workbench routes (/playground, tutorial lessons) use their OWN layout prefixes —
  never a path-if branch inside this rail.

  Geometry (viewport-locked site-header <main> is the parent):
  - shell h(full) + overflow hidden — fill the budget, no page-level scroll
  - rail: independent overflowY(auto), fixed width, stretches to shell height
  - mainDocs: independent overflowY(auto) — content scrolls; rail does not leave

  Responsive — style when:{ tablet } + structural <when {tablet}> for drawer:
  - tablet+ (768px): fixed-width side rail
  - below tablet: rail display:none (on a plain div — .sail-col beats
    s-displaynone); Menu opens a fixed drawer

  Rail is a <div display:flex; flex-dir:column>, never <column>, so
  display(none) is not overridden by .sail-col { display:flex }.
-->

<scope>
  // Fill the site-header <main> budget. Independent rail + main scrollports.
  style shell = { box: minH(0).h(full).minW(0).overflow(hidden) }
  // Plain div rail: fixed width, never grows into the content pane.
  // Mobile base = display(none); tablet+ restores flex column.
  // NOT <column> — .sail-col { display:flex } beats .s-displaynone.
  // 14rem rail (space-56 ≡ --panel-rail-w).
  style rail = { box: display(none).flexDir(column).w(56).minH(0).h(full).padY(4).padX(3).overflowY(auto).overflowX(hidden), paint: borderEWidth(1).border(border).bg(background).fg(foreground), when: { tablet: { box: display(flex) } } }
  style railLabel = { box: mt(4).mb(1), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  // Current must paint over the base muted ink — filled pill for active page.
  style railLink = { box: display(block).padY(1).padX(2), paint: radius(md).fg(muted-foreground), text: fontSize(sm), when: { hover: { paint: fg(foreground).bg(hover-bg) }, current: { paint: fg(primary).bg(muted), text: fontWeight(semibold) } } }
  style mainDocs = { box: pad(4).basis(0).minH(0).overflowX(hidden).overflowY(auto), when: { tablet: { box: pad(6) } } }
  // Mobile-only chrome — plain div so display:none wins at tablet+.
  style mobileBar = { box: display(flex).gap(2).padX(3).padY(2), paint: borderBWidth(1).borderB(border).bg(background), when: { tablet: { box: display(none) } } }
  style contentCol = { box: minH(0).minW(0) }
  style mobileLabel = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) }
  style drawerTitle = { text: fontSize(sm) }
  style fullW = { box: w(full) }

  type DocsNavProps = {
    path: any
    onNav: any
  }
</scope>

<controller>
import { currentNav } from '../compositions/current-nav'
/**
 * Live pathname for aria-current. Layout chrome stays mounted across leaf
 * navigations under /docs/* — deck route.pathname updates synchronously on
 * navigate (pathname SSOT), so the rail tracks the leaf without browser API wraps.
 * currentNav is the DOM fallback when this layout CE does not receive ctx.route.
 */
export default controller((ctx) => {
  const path = derived(() => ctx.route?.pathname?.() ?? '')
  const navOpen = signal(false)
  const openNav = (): void => navOpen.set(true)
  const closeNav = (): void => navOpen.set(false)
  const noopNav = (): void => {}
  // Close the mobile drawer on every leaf nav (chrome stays mounted).
  effect(() => {
    path()
    navOpen.set(false)
  })
  return { path, navOpen, openNav, closeNav, noopNav, currentNav }
})
</controller>

<!-- Same link tree for side rail and mobile drawer; onNav closes drawer (noop on rail). -->
<partial docs-nav props: DocsNavProps>
  <p style(railLabel)>Guide</p>
  <a href="/docs" on:click={onNav} current={path() === '/docs' ? 'page' : null} style(railLink)>Docs hub</a>
  <a href="/docs/guide/introduction" on:click={onNav} current={path() === '/docs/guide/introduction' ? 'page' : null} style(railLink)>Introduction</a>
  <a href="/docs/guide/why" on:click={onNav} current={path() === '/docs/guide/why' ? 'page' : null} style(railLink)>Why Sail</a>
  <a href="/docs/guide/getting-started" on:click={onNav} current={path() === '/docs/guide/getting-started' ? 'page' : null} style(railLink)>Installation</a>
  <a href="/docs/guide/using-components" on:click={onNav} current={path() === '/docs/guide/using-components' ? 'page' : null} style(railLink)>Using components</a>
  <a href="/docs/guide/building-a-component" on:click={onNav} current={path() === '/docs/guide/building-a-component' ? 'page' : null} style(railLink)>Building a component</a>
  <a href="/docs/guide/forms" on:click={onNav} current={path() === '/docs/guide/forms' ? 'page' : null} style(railLink)>Forms</a>
  <a href="/docs/guide/making-a-theme" on:click={onNav} current={path() === '/docs/guide/making-a-theme' ? 'page' : null} style(railLink)>Making a theme</a>
  <a href="/docs/guide/embedding-third-party-code" on:click={onNav} current={path() === '/docs/guide/embedding-third-party-code' ? 'page' : null} style(railLink)>Embedding third-party code</a>

  <p style(railLabel)>Concepts</p>
  <a href="/docs/concepts/sail-file" on:click={onNav} current={path() === '/docs/concepts/sail-file' ? 'page' : null} style(railLink)>Components</a>
  <a href="/docs/concepts/state" on:click={onNav} current={path() === '/docs/concepts/state' ? 'page' : null} style(railLink)>State</a>
  <a href="/docs/concepts/templates" on:click={onNav} current={path() === '/docs/concepts/templates' ? 'page' : null} style(railLink)>Templates</a>
  <a href="/docs/concepts/styling" on:click={onNav} current={path() === '/docs/concepts/styling' ? 'page' : null} style(railLink)>Styling</a>
  <a href="/docs/concepts/layout" on:click={onNav} current={path() === '/docs/concepts/layout' ? 'page' : null} style(railLink)>Layout</a>
  <a href="/docs/concepts/animation" on:click={onNav} current={path() === '/docs/concepts/animation' ? 'page' : null} style(railLink)>Animation</a>
  <a href="/docs/concepts/actions" on:click={onNav} current={path() === '/docs/concepts/actions' ? 'page' : null} style(railLink)>Actions</a>
  <a href="/docs/concepts/data" on:click={onNav} current={path() === '/docs/concepts/data' ? 'page' : null} style(railLink)>Data</a>
  <a href="/docs/concepts/capabilities" on:click={onNav} current={path() === '/docs/concepts/capabilities' ? 'page' : null} style(railLink)>Capabilities</a>
  <a href="/docs/concepts/errors" on:click={onNav} current={path() === '/docs/concepts/errors' ? 'page' : null} style(railLink)>Errors</a>
  <a href="/docs/concepts/theming" on:click={onNav} current={path() === '/docs/concepts/theming' ? 'page' : null} style(railLink)>Theming</a>

  <p style(railLabel)>App</p>
  <a href="/docs/app" on:click={onNav} current={path() === '/docs/app' ? 'page' : null} style(railLink)>Overview</a>
  <a href="/docs/app/getting-started" on:click={onNav} current={path() === '/docs/app/getting-started' ? 'page' : null} style(railLink)>Getting started</a>
  <a href="/docs/app/routing" on:click={onNav} current={path() === '/docs/app/routing' ? 'page' : null} style(railLink)>Routing</a>
  <a href="/docs/app/dynamic-routes" on:click={onNav} current={path() === '/docs/app/dynamic-routes' ? 'page' : null} style(railLink)>Dynamic routes</a>
  <a href="/docs/app/layouts" on:click={onNav} current={path() === '/docs/app/layouts' ? 'page' : null} style(railLink)>Layouts</a>
  <a href="/docs/app/rendering" on:click={onNav} current={path() === '/docs/app/rendering' ? 'page' : null} style(railLink)>Rendering</a>
  <a href="/docs/app/reactivity" on:click={onNav} current={path() === '/docs/app/reactivity' ? 'page' : null} style(railLink)>Reactivity</a>
  <a href="/docs/app/data" on:click={onNav} current={path() === '/docs/app/data' ? 'page' : null} style(railLink)>Live data</a>
  <a href="/docs/app/hmr" on:click={onNav} current={path() === '/docs/app/hmr' ? 'page' : null} style(railLink)>HMR</a>
  <a href="/docs/app/build" on:click={onNav} current={path() === '/docs/app/build' ? 'page' : null} style(railLink)>Build</a>

  <p style(railLabel)>AI</p>
  <a href="/docs/ai" on:click={onNav} current={path() === '/docs/ai' ? 'page' : null} style(railLink)>Overview</a>
  <a href="/docs/ai/authoring" on:click={onNav} current={path() === '/docs/ai/authoring' ? 'page' : null} style(railLink)>Authoring</a>
  <a href="/docs/ai/tooling" on:click={onNav} current={path() === '/docs/ai/tooling' ? 'page' : null} style(railLink)>Tooling</a>
  <a href="/docs/ai/eval" on:click={onNav} current={path() === '/docs/ai/eval' ? 'page' : null} style(railLink)>Eval</a>

  <p style(railLabel)>Reference</p>
  <a href="/docs/reference" on:click={onNav} current={path() === '/docs/reference' ? 'page' : null} style(railLink)>API reference</a>
  <a href="/docs/reference/sx" on:click={onNav} current={path() === '/docs/reference/sx' ? 'page' : null} style(railLink)>Style</a>
  <a href="/docs/reference/template" on:click={onNav} current={path() === '/docs/reference/template' ? 'page' : null} style(railLink)>Template / callbacks</a>
  <a href="/docs/reference/regions" on:click={onNav} current={path() === '/docs/reference/regions' ? 'page' : null} style(railLink)>Region matrix</a>
</partial>

<row nowrap align="stretch" gap="0" style(shell, { box: w(full).grow(1).minW(0) })>
  <nav label="Docs" testid="docs-rail" use:currentNav style(rail, { box: shrink(0) })>
    <docs-nav path={path} onNav={noopNav}></docs-nav>
  </nav>

  <column gap="0" style(contentCol, { box: grow(1).minW(0) })>
    <div style(mobileBar)>
      <button type="button" variant="outline" size="sm" on:click={openNav} expanded={navOpen() ? 'true' : 'false'}>Menu</button>
      <small style(mobileLabel)>Docs</small>
    </div>
    <main style(mainDocs, { box: grow(1).minW(0) })>
      <slot name="route"></slot>
    </main>
  </column>
</row>

<!-- Mobile drawer only below tablet; side rail owns nav at tablet+. -->
<when {tablet}>
{else =>
<dialog
  id="sail-docs-drawer"
  open={navOpen()}
  side="start"
  size="sm"
  label="Docs navigation"
  on-close={closeNav}
>
  <header>
    <row align="center" justify="between" gap="2" style(fullW)>
      <strong style(drawerTitle)>Docs</strong>
      <button type="button" variant="ghost" size="sm" on:click={closeNav}>Close</button>
    </row>
  </header>
  <section>
    <docs-nav path={path} onNav={closeNav}></docs-nav>
  </section>
</dialog>
}</when>
