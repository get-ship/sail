<!--
  Live [name] param route — heading tracks ctx.route.params.name().
-->
<scope>
  style page = { box: size({ max-w: breakpoint-tablet }).space({ mx: auto }) }
  style eyebrow = { box: space({ mb: 2 }), paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style footNav = { box: space({ pad-t: 4, mt: 8 }), paint: border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

export default controller((ctx: ControllerScope<Scope>) => {
  const name = derived(() => ctx.route.params.name?.() ?? '')
  return { name }
})
</controller>

<article style(page)>
  <p style(eyebrow)>Live example</p>
  <h1>Hello, {name()}.</h1>
  <p style(lead)>
    This page is <code>routes/docs/app/examples/greet/&#91;name&#93;/page.sail</code>. The
    <code>&#91;name&#93;</code> directory is a route parameter, and
    <code>ctx.route.params.name&#40;&#41;</code> reads it as a signal.
  </p>
  <p>
    Change the URL to change the greeting — try
    <a href="/docs/app/examples/greet/ada">/docs/app/examples/greet/ada</a> or
    <a href="/docs/app/examples/greet/deck">/docs/app/examples/greet/deck</a>.
  </p>
  <row style(footNav)>
    <a href="/docs/app/dynamic-routes">← Dynamic routes</a>
  </row>
</article>
