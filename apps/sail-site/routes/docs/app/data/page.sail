<!--
  /docs/app/data — clients seam, durable store vs route state, live server data.
  Live store demo omitted: sail-site clients seam is empty by design.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { configExample, usageExample } from '../../../../src/compositions/deck-data-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ configExample, usageExample }))
</controller>

<article>
  <column gap="5">
    <p>Reactivity</p>
    <h1>The data seam</h1>
    <p>deck mounts routes but owns no data. Your app exposes named clients through one seam; every route reads them straight off <code>ctx</code> — flattened, no nested bag.</p>

    <h2>The clients seam</h2>
    <p>Declare a clients module in <code>sail.config.jsonc</code>. deck wires its exports into the router, and every route reads them by name.</p>
    <code-block code={configExample} lang="json"></code-block>
    <code-block code={usageExample} lang="ts"></code-block>

    <h2>Durable state vs. route state</h2>
    <p>The store lives <em>above</em> the router, so it survives client-side navigation <strong>and</strong> a full reload — unlike a route signal, which resets when you navigate away. A persisted counter (an example) demonstrates the difference: bump it, navigate to another page, come back — the number holds.</p>

    <h2>Live server data</h2>
    <p>A <code>&lt;data source="…" live&gt;</code> binding subscribes a route to live server data. That transport is <strong>host-wired</strong>: a host (like shipd) mounts it and supplies a resolver. Standalone <code>sail dev</code> serves routing, assets, SSR and HMR but not live server data yet — that’s the server-functions work (#32). Use a client store for state until then.</p>

    <separator />
    <row gap="2">
      <a href="/docs/app/reactivity">← Reactivity</a>
      <a href="/docs/app/rendering">Rendering modes →</a>
    </row>
  </column>
</article>
