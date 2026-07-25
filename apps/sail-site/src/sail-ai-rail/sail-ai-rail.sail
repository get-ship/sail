<!--
  <sail-ai-rail> — AI docs sidebar + content main for every /ai/* route.

  Same layout contract as sail-docs-rail / sail-deck-rail: plain column rail,
  not <navbar placement="rail"> (top-bar anatomy collapses the shell).
-->

<controller>
export default controller((ctx) => {
  const path = derived(() => {
    const r = (ctx as { route?: { pathname?: () => string } }).route
    if (r?.pathname) return r.pathname()
    return typeof location !== 'undefined' ? location.pathname : '/'
  })
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
  <column gap="0" sx={rail} role="navigation" aria-label="AI docs">
    <p sx={railLabel}>AI</p>
    <a href="/ai" sx={railLink} aria-current={path() === '/ai' ? 'page' : null}>Overview</a>
    <a href="/ai/authoring" sx={railLink} aria-current={path() === '/ai/authoring' ? 'page' : null}>Authoring</a>
    <a href="/ai/tooling" sx={railLink} aria-current={path() === '/ai/tooling' ? 'page' : null}>Tooling</a>
    <a href="/ai/eval" sx={railLink} aria-current={path() === '/ai/eval' ? 'page' : null}>Eval</a>
  </column>

  <main sx={mainDocs}>
    <slot name="route"></slot>
  </main>
</row>
