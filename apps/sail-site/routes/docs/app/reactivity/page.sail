<!--
  Deck’s role in the reactive path: URL, live data, durable state, HMR.
  Sail’s signal model is taught at /docs/concepts/state — not re-taught here.
-->
<scope>
  style page = { size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style eyebrow = { margin({ b: 2 }).fg(accent).type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { fg(muted-foreground).type({ size: lg, leading: relaxed }) }
  style footNav = { pad({ t: 4 }).margin({ t: 8 }).border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/deck-reactivity-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ code }))
</controller>

<article style(page)>
  <p style(eyebrow)>Reactivity</p>
  <h1>deck in the reactive path</h1>
  <p style(lead)>
    Reactivity is Sail’s, not deck’s. Sail gives a route signals, derived values,
    and effects; deck’s job is to keep that graph fed from the things a
    framework owns and changes — the URL, live server data, and state that
    outlives a route.
  </p>

  <alert tone="info">
    <strong>The signal model lives in Sail.</strong> Signals, <code>derived</code>,
    <code>effect</code>, batching, and the glitch-free update pass are all the Sail
    runtime — deck adds no reactive primitive of its own. For how the graph
    recomputes, see <a href="/docs/concepts/state">Sail · Reactivity</a>.
  </alert>

  <h2>The URL is reactive</h2>
  <p>
    deck injects a <code>route</code> handle into every route scope, and its fields
    are signals — <code>route.params.&lt;name&gt;()</code>,
    <code>route.query.&lt;name&gt;()</code>, <code>route.pathname&#40;&#41;</code>, and
    <code>route.hash&#40;&#41;</code>. A client-side navigation updates them in place, so
    only the bindings that read the URL re-run: no document reload, no re-mount.
  </p>
  <code-block code={code.routeSignals}></code-block>
  <p>
    See it live: the <a href="/docs/app/examples/greet/ada">greet</a> route reads
    <code>route.params.name&#40;&#41;</code>, so its heading tracks the URL as you follow
    the links.
  </p>

  <h2>Live server data flows in the same way</h2>
  <p>
    A <code>&lt;data source="…" live&gt;</code> binding is just another reactive
    source: deck subscribes the route to a server stream and patches the signal as
    values arrive, so only the bindings that read it re-render. That transport is
    <strong>host-wired</strong> — a host (like shipd) mounts it and supplies the
    resolver.
  </p>
  <code-block code={code.liveData}></code-block>
  <p>
    Standalone <code>sail dev</code> serves routing, assets, SSR and HMR, but not
    live server data yet — that’s the server-functions work (#32). Until then,
    use a client store for shared state; the <a href="/docs/app/data">data seam</a>
    covers both.
  </p>

  <h2>State that outlives a route</h2>
  <p>
    A signal declared in a route’s <code>&lt;controller&gt;</code> block is scoped to that
    route — it resets when you navigate away. State that must survive navigation
    lives <em>above</em> the router, on deck’s clients seam, where it persists
    across navigations and full reloads. The <a href="/docs/app/data">data seam</a>
    page shows the wiring.
  </p>

  <h2>Edits preserve state</h2>
  <p>
    In dev, saving a route swaps the freshly-bundled module into the running
    router and re-mounts it — <strong>signal state survives the swap</strong>, so
    counters, inputs, and scroll stay put. That’s deck’s
    <a href="/docs/app/hmr">HMR</a>, not a reactive primitive: the graph is simply
    never torn down.
  </p>

  <row style(footNav)>
    <a href="/docs/app/layouts">← Layouts</a>
    <a href="/docs/app/data">The data seam →</a>
  </row>
</article>
