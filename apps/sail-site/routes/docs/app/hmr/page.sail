<!--
  /docs/app/hmr — route soft-reload with state preserved; component hot-swap; SSE.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { devCmd } from '../../../../src/compositions/deck-hmr-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ devCmd }))
</controller>

<article>
  <column gap="5">
    <p>Rendering &amp; build</p>
    <h1>HMR</h1>
    <p>In dev, editing a route soft-reloads it with signal state preserved — no full document reload.</p>

    <h2>Routes keep their state</h2>
    <p>Save a route and the page updates in place. Counters, inputs, and scroll stay put.</p>
    <code-block code={devCmd} lang="bash"></code-block>

    <h2>Components update in place</h2>
    <p>Editing a component updates every live instance without reloading the page.</p>

    <h2>Dev only</h2>
    <p>The dev server pushes changes to the browser. None of that ships in a production build.</p>

    <separator />
    <row gap="2">
      <a href="/docs/app/rendering">← Rendering modes</a>
      <a href="/docs/app/build">Build &amp; deploy →</a>
    </row>
  </column>
</article>
