<controller>
// Getting-started controller. Snippets live in ./snippets (a plain data module)
// so example code containing `signal(...)` never leaks into the inferred scope.
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { pageExample, controllerExample, devCmd } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ pageExample, controllerExample, devCmd }))
</controller>

<article data-doc="true">
  <p role="doc-eyebrow">Introduction</p>
  <h1>Getting started</h1>
  <p role="doc-lead">You need the <code>deck</code> CLI and a folder. No project scaffold, no install, no config to start.</p>

  <h2>1. A page is a file</h2>
  <p>Create <code>routes/page.sail</code>. Whatever markup it holds is your page.</p>
  <pre data-code-block="true">{pageExample}</pre>

  <h2>2. Add a controller</h2>
  <p>Co-locate <code>routes/controller.ts</code> for reactive state. It returns a scope the template reads by name.</p>
  <pre data-code-block="true">{controllerExample}</pre>

  <h2>3. Run the dev server</h2>
  <pre data-code-block="true">{devCmd}</pre>
  <div role="note">
    <strong>No build step to develop.</strong> deck compiles <code>.sail</code> in-process and serves the SPA with HMR — edit a route and it soft-reloads with state preserved.
  </div>

  <div role="navigation">
    <a href="/">← Overview</a>
    <a href="/routing">File routing →</a>
  </div>
</article>
