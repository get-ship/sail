<!--
  /docs/app/dynamic-routes — [param] segments and route.params signals.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeExample, readExample } from '../../../../src/compositions/deck-dynamic-routes-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeExample, readExample }))
</controller>

<article>
  <column gap="5">
    <p>Routing</p>
    <h1>Dynamic routes</h1>
    <p>A directory segment in brackets is a parameter. <code>routes/posts/&#91;id&#93;/page.sail</code> matches <code>/posts/anything</code>, and the route reads the value as a signal.</p>

    <h2>The &#91;param&#93; segment</h2>
    <p>deck emits a typed pattern — <code>/posts/:id</code> — for any <code>&#91;param&#93;</code> directory. Read the value with <code>route.params.&lt;name&gt;()</code>. It’s a signal, so the route re-renders when the parameter changes during client-side navigation.</p>
    <code-block code={routeExample} lang="text"></code-block>
    <code-block code={readExample} lang="sail"></code-block>

    <h2>Try it</h2>
    <p>This site has a live <code>&#91;name&#93;</code> route at <code>routes/examples/greet/&#91;name&#93;/</code>:</p>
    <p><a variant="solid" tone="primary" href="/docs/app/examples/greet/ada">Open /docs/app/examples/greet/ada →</a></p>

    <separator />
    <row gap="2">
      <a href="/docs/app/routing">← File routing</a>
      <a href="/docs/app/layouts">Layouts →</a>
    </row>
  </column>
</article>
