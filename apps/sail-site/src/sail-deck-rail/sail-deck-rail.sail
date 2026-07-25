<!--
  <sail-deck-rail> — the Deck docs sidebar + content main, mounted by
  routes/deck/_layout.sail for every /deck/* route.
-->

<controller>
export default controller(() => {
  const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')
  return { path }
})
</controller>

<row nowrap align="stretch">
  <navbar placement="rail" bordered sticky offset="14">
    <label>Introduction</label>
    <a href="/deck" aria-current={path() === '/deck' ? 'page' : null}>Overview</a>
    <a href="/deck/getting-started" aria-current={path() === '/deck/getting-started' ? 'page' : null}>Getting started</a>
    <label>Concepts</label>
    <a href="/deck/routing" aria-current={path() === '/deck/routing' ? 'page' : null}>Routing</a>
    <a href="/deck/dynamic-routes" aria-current={path() === '/deck/dynamic-routes' ? 'page' : null}>Dynamic routes</a>
    <a href="/deck/layouts" aria-current={path() === '/deck/layouts' ? 'page' : null}>Layouts</a>
    <a href="/deck/rendering" aria-current={path() === '/deck/rendering' ? 'page' : null}>Rendering</a>
    <a href="/deck/reactivity" aria-current={path() === '/deck/reactivity' ? 'page' : null}>Reactivity</a>
    <a href="/deck/data" aria-current={path() === '/deck/data' ? 'page' : null}>Live data</a>
    <label>Tooling</label>
    <a href="/deck/hmr" aria-current={path() === '/deck/hmr' ? 'page' : null}>HMR</a>
    <a href="/deck/build" aria-current={path() === '/deck/build' ? 'page' : null}>Build</a>
  </navbar>

  <main sx={style.pad(6).grow(1).minW(0)}>
    <slot name="route"></slot>
  </main>
</row>
