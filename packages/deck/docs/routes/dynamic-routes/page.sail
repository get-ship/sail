<!-- Dynamic routes — [param] segments, route.params signals, live greet example. -->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeExample, readExample } from '../../src/compositions/deck-dynamic-routes-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeExample, readExample }))
</controller>

<scope>
  style page = { maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { mb(2).fg(accent).fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { fg(muted-foreground).fontSize(lg).leading(relaxed) }
  style footNav = { mt(8).padT(4).borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Routing</p>
  <h1>Dynamic routes</h1>
  <p style(lead)>A directory segment in brackets is a parameter. <code>routes/posts/[id]/page.sail</code> matches <code>/posts/anything</code>, and the route reads the value as a signal.</p>

  <h2>The {'[param]'} segment</h2>
  <p>deck emits a typed pattern — <code>/posts/:id</code> — for any <code>{'[param]'}</code> directory. Read the value with <code>route.params.&lt;name&gt;()</code>. It’s a signal, so the route re-renders when the parameter changes during client-side navigation.</p>
  <code-block code={routeExample} lang="text"></code-block>
  <code-block code={readExample} lang="sail"></code-block>

  <h2>Try it</h2>
  <p>This site has a live <code>{'[name]'}</code> route at <code>routes/examples/greet/[name]/</code>:</p>
  <p><a variant="solid" href="/examples/greet/ada">Open /examples/greet/ada →</a></p>

  <row justify="between" style(footNav)>
    <a href="/routing">← File routing</a>
    <a href="/layouts">Layouts →</a>
  </row>
</article>
