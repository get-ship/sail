<!--
  /deck/data — adapted from packages/deck/docs/routes/data/page.sail. The
  source site exercises a live `store` client (with a `visits` counter
  persisted to localStorage); that client lives in packages/deck/docs's
  own clients.ts. The sail-site doesn't expose a `store` client (its
  clients seam is intentionally empty), so the live demo is omitted
  here — the prose + config + usage snippets still document the seam.
-->

<controller>
// /deck/data — adapted from packages/deck/docs/routes/data. The live `visits`
// demo is omitted here because the sail-site doesn't expose a `store` client
// (its clients seam is intentionally empty). The config + usage snippets
// still document the seam.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { configExample, usageExample } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ configExample, usageExample }))
</controller>

<article sx={style.maxW(breakpoint-tablet).mx(auto)}>
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>Reactivity</p>
  <h1>The data seam</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>deck mounts routes but owns no data. Your app exposes named clients through one seam; every route reads them straight off <code>ctx</code> — flattened, no nested bag.</p>

  <h2>The clients seam</h2>
  <p>Declare a clients module in <code>deck.jsonc</code>. deck wires its exports into the router, and every route reads them by name.</p>
  <code-block code={configExample} lang="json"></code-block>
  <code-block code={usageExample} lang="ts"></code-block>

  <h2>Durable state vs. route state</h2>
  <p>The store lives <em>above</em> the router, so it survives client-side navigation <strong>and</strong> a full reload — unlike a route signal, which resets when you navigate away. A persisted counter (an example) demonstrates the difference: bump it, navigate to another page, come back — the number holds.</p>

  <h2>Live server data</h2>
  <p>A <code>&lt;data source="…" live&gt;</code> binding subscribes a route to live server data. That transport is <strong>host-wired</strong>: a host (like shipd) mounts it and supplies a resolver. Standalone <code>sail dev</code> serves routing, assets, SSR and HMR but not live server data yet — that's the server-functions work (#32). Use a client store for state until then.</p>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)}>
    <a href="/deck/reactivity">← Reactivity</a>
    <a href="/deck/rendering">Rendering modes →</a>
  </row>
</article>
