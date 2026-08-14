<!-- Data seam — clients module, durable store demo, live server data note. -->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { configExample, usageExample } from '../../src/compositions/deck-data-snippets'

export default controller((ctx: ControllerScope<Scope>) => ({
  configExample,
  usageExample,
  visits: ctx.clients.store.visits,
  record: (): void => ctx.clients.store.noteVisit(),
  reset: (): void => ctx.clients.store.reset(),
}))
</controller>

<scope>
  style page = { maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { mb(2).fg(accent).fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { fg(muted-foreground).fontSize(lg).leading(relaxed) }
  style demo = { pad(5).borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style big = { mb(3).fg(foreground).fontFamily(mono).fontWeight(bold).fontSize(4xl).leading(none) }
  style demoMuted = { mt(3).fg(muted-foreground).fontSize(sm) }
  style footNav = { mt(8).padT(4).borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Reactivity</p>
  <h1>The data seam</h1>
  <p style(lead)>deck mounts routes but owns no data. Your app exposes named clients through one seam; every route reads them off <code>ctx.clients</code>.</p>

  <h2>The clients seam</h2>
  <p>Declare a clients module in <code>sail.config.jsonc</code>. deck wires its exports into the router, and every route reads them by name.</p>
  <code-block code={configExample} lang="json"></code-block>
  <code-block code={usageExample} lang="ts"></code-block>

  <h2>Live: durable state</h2>
  <p>This site exposes a <code>store</code> client with a persisted <code>visits</code> counter. It lives <em>above</em> the router, so it survives client-side navigation <strong>and</strong> a full reload — unlike a route signal, which resets when you navigate away.</p>
  <column gap="3" style(demo)>
    <p style(big)>{visits()}</p>
    <row gap="2">
      <button variant="solid" type="button" on:click={() => record()}>Record a visit</button>
      <button variant="ghost" type="button" on:click={() => reset()}>Reset</button>
    </row>
    <p style(demoMuted)>Bump it, navigate to another page, come back — the number holds.</p>
  </column>

  <h2>Live server data</h2>
  <p>A <code>&lt;data source="…" live&gt;</code> binding subscribes a route to live server data. That transport is <strong>host-wired</strong>: a host (like shipd) mounts it and supplies a resolver. Standalone <code>sail dev</code> serves routing, assets, SSR and HMR but not live server data yet — server-functions work is still landing. Use a client store for state until then.</p>

  <row justify="between" style(footNav)>
    <a href="/reactivity">← Reactivity</a>
    <a href="/rendering">Rendering modes →</a>
  </row>
</article>
