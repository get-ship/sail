<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeExample, readExample } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeExample, readExample }))
</controller>

<article data-doc="true">
  <p role="doc-eyebrow">Routing</p>
  <h1>Dynamic routes</h1>
  <p role="doc-lead">A directory segment in brackets is a parameter. <code>routes/posts/[id]/page.sail</code> matches <code>/posts/anything</code>, and the route reads the value as a signal.</p>

  <h2>The [param] segment</h2>
  <p>deck emits a typed pattern — <code>/posts/:id</code> — for any <code>[param]</code> directory. Read the value with <code>route.params.&lt;name&gt;()</code>. It's a signal, so the route re-renders when the parameter changes during client-side navigation.</p>
  <pre data-code-block="true">{routeExample}</pre>
  <pre data-code-block="true">{readExample}</pre>

  <h2>Try it</h2>
  <p>This site has a live <code>[name]</code> route at <code>routes/examples/greet/[name]/</code>:</p>
  <p><a variant="solid" href="/examples/greet/ada">Open /examples/greet/ada →</a></p>

  <div role="navigation">
    <a href="/routing">← File routing</a>
    <a href="/layouts">Layouts →</a>
  </div>
</article>
