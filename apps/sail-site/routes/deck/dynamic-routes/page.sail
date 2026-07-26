<!--
  /deck/dynamic-routes — adapted from packages/deck/docs/routes/dynamic-routes/page.sail.
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeExample, readExample } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeExample, readExample }))
</controller>

<article sx={style.maxW(breakpoint-tablet).mx(auto)}>
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>Routing</p>
  <h1>Dynamic routes</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>A directory segment in brackets is a parameter. <code>routes/posts/[id]/page.sail</code> matches <code>/posts/anything</code>, and the route reads the value as a signal.</p>

  <h2>The [param] segment</h2>
  <p>deck emits a typed pattern — <code>/posts/:id</code> — for any <code>[param]</code> directory. Read the value with <code>route.params.&lt;name&gt;()</code>. It's a signal, so the route re-renders when the parameter changes during client-side navigation.</p>
  <pre sx={style.fontFamily(mono).fontSize(sm).leading(relaxed).bg(surface-1).borderWidth(1).border(border).radius(lg).pad(4).padX(5).whitespace(pre-wrap).break(words)}>{routeExample}</pre>
  <code-block code={readExample} lang="sail"></code-block>

  <h2>Try it</h2>
  <p>This site has a live <code>[name]</code> route at <code>routes/examples/greet/[name]/</code>:</p>
  <p><a variant="solid" tone="primary" href="/deck/examples/greet/ada">Open /deck/examples/greet/ada →</a></p>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)}>
    <a href="/deck/routing">← File routing</a>
    <a href="/deck/layouts">Layouts →</a>
  </row>
</article>
