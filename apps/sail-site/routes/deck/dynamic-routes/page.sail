<!--
  /deck/dynamic-routes — [param] segments and route.params signals.
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeExample, readExample } from '../../../src/compositions/deck-dynamic-routes-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeExample, readExample }))
</controller>

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style monoBlock = { box: pad(4).padX(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg), text: fontFamily(mono).fontSize(sm).leading(relaxed).whitespace(pre-wrap).break(words) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Routing</p>
  <h1>Dynamic routes</h1>
  <p style(lead)>A directory segment in brackets is a parameter. <code>routes/posts/&#91;id&#93;/page.sail</code> matches <code>/posts/anything</code>, and the route reads the value as a signal.</p>

  <h2>The &#91;param&#93; segment</h2>
  <p>deck emits a typed pattern — <code>/posts/:id</code> — for any <code>&#91;param&#93;</code> directory. Read the value with <code>route.params.&lt;name&gt;()</code>. It’s a signal, so the route re-renders when the parameter changes during client-side navigation.</p>
  <pre style(monoBlock)>{routeExample}</pre>
  <code-block code={readExample} lang="sail"></code-block>

  <h2>Try it</h2>
  <p>This site has a live <code>&#91;name&#93;</code> route at <code>routes/examples/greet/&#91;name&#93;/</code>:</p>
  <p><a variant="solid" tone="primary" href="/deck/examples/greet/ada">Open /deck/examples/greet/ada →</a></p>

  <row style(footNav)>
    <a href="/deck/routing">← File routing</a>
    <a href="/deck/layouts">Layouts →</a>
  </row>
</article>
