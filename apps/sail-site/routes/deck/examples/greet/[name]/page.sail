<!--
  /deck/examples/greet/[name] — adapted from
  packages/deck/docs/routes/examples/greet/[name]/page.sail. A live [param]
  route: routes/deck/examples/greet/[name]/page.sail matches
  /deck/examples/greet/:name. The controller reads ctx.route.params.name()
  (host-granted on route files) so the heading tracks the URL.
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

export default controller((ctx: ControllerScope<Scope>) => {
  const name = derived(() => ctx.route.params.name?.() ?? '')
  return { name }
})
</controller>

<article style({ box: maxW(breakpoint-tablet).mx(auto) })>
  <p style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Live example</p>
  <h1>Hello, {name()}.</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })>
    This page is <code>routes/deck/examples/greet/&#91;name&#93;/page.sail</code>. The
    <code>&#91;name&#93;</code> directory is a route parameter, and
    <code>ctx.route.params.name&#40;&#41;</code> reads it as a signal.
  </p>
  <p>
    Change the URL to change the greeting — try
    <a href="/deck/examples/greet/ada">/deck/examples/greet/ada</a> or
    <a href="/deck/examples/greet/deck">/deck/examples/greet/deck</a>.
  </p>
  <row style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })>
    <a href="/deck/dynamic-routes">← Dynamic routes</a>
  </row>
</article>
