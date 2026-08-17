<!--
  Live [name] param route — heading tracks ctx.route.params.name().
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

export default controller((ctx: ControllerScope<Scope>) => {
  const name = derived(() => ctx.route.params.name?.() ?? '')
  return { name }
})
</controller>

<article>
  <column gap="5">
    <p>Live example</p>
    <h1>Hello, {name()}.</h1>
    <p>
      This page is <code>routes/docs/app/examples/greet/&#91;name&#93;/page.sail</code>. The
      <code>&#91;name&#93;</code> directory is a route parameter, and
      <code>ctx.route.params.name&#40;&#41;</code> reads it as a signal.
    </p>
    <p>
      Change the URL to change the greeting — try
      <a href="/docs/app/examples/greet/ada">/docs/app/examples/greet/ada</a> or
      <a href="/docs/app/examples/greet/deck">/docs/app/examples/greet/deck</a>.
    </p>
    <separator />
    <row gap="2">
      <a href="/docs/app/dynamic-routes">← Dynamic routes</a>
    </row>
  </column>
</article>
