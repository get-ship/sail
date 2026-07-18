<!--
  <sail-docs-rail> — the Sail-language docs sidebar + content main, mounted
  by routes/sail/_layout.sail for every /sail/* route. A tutorial LESSON page
  (/sail/tutorial/<slug>) is a fixed-viewport workbench and drops the rail +
  reading padding; the /sail/tutorial index itself is a normal docs page and
  keeps both.
-->

<controller>
export default controller(() => {
  const path = signal<string>(typeof location !== 'undefined' ? location.pathname : '/')
  const workbench = derived(() => path().startsWith('/sail/tutorial/'))
  return { path, workbench }
})
</controller>

<row nowrap align="stretch">
  {#if !workbench()}
    <navbar placement="rail" bordered sticky offset="14">
      <label>Getting started</label>
      <a href="/sail/overview" aria-current={path() === '/sail/overview' ? 'page' : null}>Introduction</a>
      <a href="/sail/getting-started" aria-current={path() === '/sail/getting-started' ? 'page' : null}>Installation</a>
      <a href="/sail/why" aria-current={path() === '/sail/why' ? 'page' : null}>Why Sail</a>
      <a href="/sail/tutorial" aria-current={path().startsWith('/sail/tutorial') ? 'page' : null}>Tutorial</a>
      <a href="/sail/showcase" aria-current={path() === '/sail/showcase' ? 'page' : null}>Showcase</a>

      <label>Concepts</label>
      <a href="/sail/language/sail-file" aria-current={path() === '/sail/language/sail-file' ? 'page' : null}>Components</a>
      <a href="/sail/language/state" aria-current={path() === '/sail/language/state' ? 'page' : null}>State</a>
      <a href="/sail/language/templates" aria-current={path() === '/sail/language/templates' ? 'page' : null}>Templates</a>
      <a href="/sail/language/styling" aria-current={path() === '/sail/language/styling' ? 'page' : null}>Styling</a>
      <a href="/sail/language/layout" aria-current={path() === '/sail/language/layout' ? 'page' : null}>Layout</a>
      <a href="/sail/language/motion" aria-current={path() === '/sail/language/motion' ? 'page' : null}>Motion</a>
      <a href="/sail/language/actions" aria-current={path() === '/sail/language/actions' ? 'page' : null}>Actions</a>
      <a href="/sail/language/data" aria-current={path() === '/sail/language/data' ? 'page' : null}>Data</a>
      <a href="/sail/language/theming" aria-current={path() === '/sail/language/theming' ? 'page' : null}>Theming</a>

      <label>Guide</label>
      <a href="/sail/elements" aria-current={path() === '/sail/elements' ? 'page' : null}>Using components</a>

      <label>Reference</label>
      <a href="/sail/reference" aria-current={path() === '/sail/reference' ? 'page' : null}>API reference</a>
    </navbar>
  {/if}

  {#if workbench()}
    <main sx="grow-1 min-w-0">
      <slot name="route"></slot>
    </main>
  {:else}
    <main sx="p-6 grow-1 min-w-0">
      <slot name="route"></slot>
    </main>
  {/if}
</row>
