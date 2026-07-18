<!--
  <sail-ai-rail> — the AI docs sidebar + content main, mounted by
  routes/ai/_layout.sail for every /ai/* route.
-->

<controller>
export default controller(() => {
  const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')
  return { path }
})
</controller>

<row nowrap align="stretch">
  <navbar placement="rail" bordered sticky offset="14">
    <label>AI</label>
    <a href="/ai" aria-current={path() === '/ai' ? 'page' : null}>Overview</a>
    <a href="/ai/authoring" aria-current={path() === '/ai/authoring' ? 'page' : null}>Authoring</a>
    <a href="/ai/tooling" aria-current={path() === '/ai/tooling' ? 'page' : null}>Tooling</a>
    <a href="/ai/eval" aria-current={path() === '/ai/eval' ? 'page' : null}>Eval</a>
  </navbar>

  <main sx="p-6 grow-1 min-w-0">
    <slot name="route"></slot>
  </main>
</row>
