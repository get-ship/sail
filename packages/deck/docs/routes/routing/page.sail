<controller>
// File-routing controller. Snippets live in ./snippets (a plain data module) so
// example code is never scope-scanned by the compiler.
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeTree, colocated } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeTree, colocated }))
</controller>

<article data-doc="true">
  <p role="doc-eyebrow">Routing</p>
  <h1>File routing</h1>
  <p role="doc-lead">deck routes straight from the filesystem. A directory under <code>routes/</code> is a URL — there is no route-config file to keep in sync.</p>

  <h2>The directory is the URL</h2>
  <p>Drop a <code>page.sail</code> anywhere under <code>routes/</code> and its folder path becomes the route. The tree on disk is the routing table.</p>
  <pre data-code-block="true">{routeTree}</pre>

  <h2>Co-located files</h2>
  <p>A route may sit beside a <code>controller.ts</code> for reactive state and a <code style.css</code> for page styles. They belong to that route alone.</p>
  <pre data-code-block="true">{colocated}></pre>

  <h2>Generated tables</h2>
  <p>Walking <code>routes/</code>, deck generates <code>routes.gen.ts</code> (the typed route table) and <code>entry.gen.ts</code> (the SPA bootstrap that calls <code>mountRouter</code>). You never edit them — they regenerate on every build and every dev run. The filesystem is the only config.</p>

  <div role="navigation">
    <a href="/getting-started">← Getting started</a>
    <a href="/dynamic-routes">Dynamic routes →</a>
  </div>
</article>
