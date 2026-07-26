<!--
  <sail-docs-rail> — Sail docs sidebar + content main for every /sail/* route.
  Tutorial LESSON pages (/sail/tutorial/<slug>) drop the rail (workbench).

  Style cutover note: do NOT use <navbar placement="rail"> for this shell.
  Navbar's top-bar anatomy (brand/center/actions + grow) fights a docs rail
  and left the page as a centered link column with empty main. A plain
  column + main row is the product layout for docs.
-->

<controller>
export default controller((ctx) => {
  const path = derived(() => {
    const r = (ctx as { route?: { pathname?: () => string } }).route
    if (r?.pathname) return r.pathname()
    return typeof location !== 'undefined' ? location.pathname : '/'
  })
  const workbench = derived(() => path().startsWith('/sail/tutorial/'))
  return { path, workbench }
})
</controller>

<scope>
  const shell = style.minH(0)
  const rail = style.minH(0).padY(5).padX(4).borderEWidth(1).border(border).overflowY(auto).bg(navbar).fg(navbar-foreground)
  const railLabel = style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground).mt(4).mb(1)
  const railLink = style.display(block).fontSize(sm).padY(1).fg(muted-foreground).when({
    hover: style.fg(foreground),
    current: style.fg(primary).fontWeight(semibold),
  })
  const mainDocs = style.pad(6).basis(0).minH(0).overflowX(hidden)
  const mainWorkbench = style.basis(0).minH(0).h(fill).overflow(hidden)
</scope>

<row nowrap align="stretch" w="full" grow min-w="0" sx={shell}>
  <if cond={!workbench()}>
    <column gap="0" shrink min-w="panel-rail-w" sx={rail} role="navigation" aria-label="Sail docs">
      <p sx={railLabel}>Getting started</p>
      <a href="/sail/overview" sx={railLink} aria-current={path() === '/sail/overview' ? 'page' : null}>Introduction</a>
      <a href="/sail/getting-started" sx={railLink} aria-current={path() === '/sail/getting-started' ? 'page' : null}>Installation</a>
      <a href="/sail/why" sx={railLink} aria-current={path() === '/sail/why' ? 'page' : null}>Why Sail</a>
      <a href="/sail/tutorial" sx={railLink} aria-current={path().startsWith('/sail/tutorial') ? 'page' : null}>Tutorial</a>
      <a href="/showcase" sx={railLink} aria-current={path() === '/showcase' ? 'page' : null}>Showcase</a>

      <p sx={railLabel}>Concepts</p>
      <a href="/sail/language/sail-file" sx={railLink} aria-current={path() === '/sail/language/sail-file' ? 'page' : null}>Components</a>
      <a href="/sail/language/state" sx={railLink} aria-current={path() === '/sail/language/state' ? 'page' : null}>State</a>
      <a href="/sail/language/templates" sx={railLink} aria-current={path() === '/sail/language/templates' ? 'page' : null}>Templates</a>
      <a href="/sail/language/styling" sx={railLink} aria-current={path() === '/sail/language/styling' ? 'page' : null}>Styling</a>
      <a href="/sail/language/layout" sx={railLink} aria-current={path() === '/sail/language/layout' ? 'page' : null}>Layout</a>
      <a href="/sail/language/animation" sx={railLink} aria-current={path() === '/sail/language/animation' ? 'page' : null}>Animation</a>
      <a href="/sail/language/actions" sx={railLink} aria-current={path() === '/sail/language/actions' ? 'page' : null}>Actions</a>
      <a href="/sail/language/data" sx={railLink} aria-current={path() === '/sail/language/data' ? 'page' : null}>Data</a>
      <a href="/sail/language/capabilities" sx={railLink} aria-current={path() === '/sail/language/capabilities' ? 'page' : null}>Capabilities</a>
      <a href="/sail/language/errors" sx={railLink} aria-current={path() === '/sail/language/errors' ? 'page' : null}>Errors</a>
      <a href="/sail/language/theming" sx={railLink} aria-current={path() === '/sail/language/theming' ? 'page' : null}>Theming</a>

      <p sx={railLabel}>Guide</p>
      <a href="/sail/elements" sx={railLink} aria-current={path() === '/sail/elements' ? 'page' : null}>Using components</a>
      <a href="/sail/guide/building-a-component" sx={railLink} aria-current={path() === '/sail/guide/building-a-component' ? 'page' : null}>Building a component</a>
      <a href="/sail/guide/forms" sx={railLink} aria-current={path() === '/sail/guide/forms' ? 'page' : null}>Forms</a>
      <a href="/sail/guide/making-a-theme" sx={railLink} aria-current={path() === '/sail/guide/making-a-theme' ? 'page' : null}>Making a theme</a>
      <a href="/sail/guide/embedding-third-party-code" sx={railLink} aria-current={path() === '/sail/guide/embedding-third-party-code' ? 'page' : null}>Embedding third-party code</a>

      <p sx={railLabel}>Reference</p>
      <a href="/sail/reference" sx={railLink} aria-current={path() === '/sail/reference' ? 'page' : null}>API reference</a>
    </column>
  </if>

  <if cond={workbench()}>
    <main grow min-w="0" sx={mainWorkbench}>
      <slot name="route"></slot>
    </main>
  <else>
    <main grow min-w="0" sx={mainDocs}>
      <slot name="route"></slot>
    </main>
  </else></if>
</row>
