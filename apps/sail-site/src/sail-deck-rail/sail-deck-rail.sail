<!--
  <sail-deck-rail> — Deck docs sidebar + content main for every /deck/* route.

  Do NOT use <navbar placement="rail"> here. Top-bar navbar anatomy (brand /
  center / actions + grow) fights a docs rail and collapses the page into a
  centered link column with content crushed to the edge (same Style-cutover
  regression sail-docs-rail already fixed).
-->

<controller>
export default controller(() => {
  const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')
  return { path }
})
</controller>

<scope>
  const shell = style.w(full).grow(1).minW(0).minH(0)
  const rail = style.shrink(0).minW(panel-rail-w).minH(0).padY(5).padX(4).borderEWidth(1).border(border).overflowY(auto).bg(navbar).fg(navbar-foreground)
  const railLabel = style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground).mt(4).mb(1)
  const railLink = style.display(block).fontSize(sm).padY(1).fg(muted-foreground).when({
    hover: style.fg(foreground),
    current: style.fg(primary).fontWeight(semibold),
  })
  const mainDocs = style.pad(6).grow(1).basis(0).minW(0).minH(0).overflowX(hidden)
</scope>

<row nowrap align="stretch" sx={shell}>
  <column gap="0" sx={rail} role="navigation" aria-label="Deck docs">
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
  </column>

  <main sx={mainDocs}>
    <slot name="route"></slot>
  </main>
</row>
