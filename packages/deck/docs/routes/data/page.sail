<controller>
// Data-seam controller. Reads the persisted `store` client off the clients seam
// (ctx.clients.store) and exposes its `visits` signal + actions for the live
// demo. No `signal(` in this file's source, so scope inference stays clean.
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { configExample, usageExample } from './snippets'

export default controller((ctx: ControllerScope<Scope>) => ({
    configExample,
    usageExample,
    visits: ctx.clients.store.visits,
    record: (): void => ctx.clients.store.noteVisit(),
    reset: (): void => ctx.clients.store.reset(),
  }))
</controller>

<article data-doc="true">
  <p role="doc-eyebrow">Reactivity</p>
  <h1>The data seam</h1>
  <p role="doc-lead">deck mounts routes but owns no data. Your app exposes named clients through one seam; every route reads them off <code>ctx.clients</code>.</p>

  <h2>The clients seam</h2>
  <p>Declare a clients module in <code>deck.jsonc</code>. deck wires its exports into the router, and every route reads them by name.</p>
  <pre data-code-block="true">{configExample}</pre>
  <pre data-code-block="true">{usageExample}</pre>

  <h2>Live: durable state</h2>
  <p>This site exposes a <code>store</code> client with a persisted <code>visits</code> counter. It lives <em>above</em> the router, so it survives client-side navigation <strong>and</strong> a full reload — unlike a route signal, which resets when you navigate away.</p>
  <div data-demo="true">
    <p data-big="true">{visits()}</p>
    <div data-row="true">
      <button variant="solid" type="button" on:click={() => record()}>Record a visit</button>
      <button variant="ghost" type="button" on:click={() => reset()}>Reset</button>
    </div>
    <p data-muted="true">Bump it, navigate to another page, come back — the number holds.</p>
  </div>

  <h2>Live server data</h2>
  <p>A <code>&lt;data source="…" live&gt;</code> binding subscribes a route to live server data. That transport is <strong>host-wired</strong>: a host (like shipd) mounts it and supplies a resolver. Standalone <code>deck dev</code> serves routing, assets, SSR and HMR but not live server data yet — that's the server-functions work (#32). Use a client store for state until then.</p>

  <div role="navigation">
    <a href="/reactivity">← Reactivity</a>
    <a href="/rendering">Rendering modes →</a>
  </div>
</article>
