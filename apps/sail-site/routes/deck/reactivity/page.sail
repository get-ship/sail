<!--
  /deck/reactivity — deck's role in the reactive update path. Reactivity
  itself (signal / derived / effect, the glitch-free update pass) is SAIL's;
  that model lives at /reactivity and is not re-taught here. This page is
  about the sources deck owns and feeds into the graph: the URL, live server
  data, durable app state, and dev edits. Code blocks are highlighted via
  <code-block> (strings from ./snippets).
-->
<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ code }))
</controller>

<article sx="max-w-breakpoint-tablet mx-auto">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Reactivity</p>
  <h1>deck in the reactive path</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    Reactivity is SAIL's, not deck's. SAIL gives a view signals, derived values,
    and effects; deck's job is to keep that graph fed from the things a
    framework owns and changes — the URL, live server data, and state that
    outlives a route.
  </p>

  <alert tone="info">
    <strong>The signal model lives in SAIL.</strong> Signals, <code>derived</code>,
    <code>effect</code>, batching, and the glitch-free update pass are all the SAIL
    runtime — deck adds no reactive primitive of its own. For how the graph
    recomputes, see <a href="/reactivity">SAIL · Reactivity</a>.
  </alert>

  <h2>The URL is reactive</h2>
  <p>
    deck injects a <code>route</code> handle into every view scope, and its fields
    are signals — <code>route.params.&lt;name&gt;()</code>,
    <code>route.query.&lt;name&gt;()</code>, <code>route.pathname()</code>, and
    <code>route.hash()</code>. A client-side navigation updates them in place, so
    only the bindings that read the URL re-run: no document reload, no re-mount.
  </p>
  <code-block code={code.routeSignals}></code-block>
  <p>
    See it live: the <a href="/deck/examples/greet/ada">greet</a> route reads
    <code>route.params.name()</code>, so its heading tracks the URL as you follow
    the links.
  </p>

  <h2>Live server data flows in the same way</h2>
  <p>
    A <code>&lt;data source="…" live&gt;</code> binding is just another reactive
    source: deck subscribes the view to a server stream and patches the signal as
    values arrive, so only the bindings that read it re-render. That transport is
    <strong>host-wired</strong> — a host (like shipd) mounts it and supplies the
    resolver.
  </p>
  <code-block code={code.liveData}></code-block>
  <p>
    Standalone <code>deck dev</code> serves routing, assets, SSR and HMR, but not
    live server data yet — that's the server-functions work (#32). Until then,
    use a client store for shared state; the <a href="/deck/data">data seam</a>
    covers both.
  </p>

  <h2>State that outlives a route</h2>
  <p>
    A signal declared in a route's <code>&lt;controller&gt;</code> block is scoped to that
    route — it resets when you navigate away. State that must survive navigation
    lives <em>above</em> the router, on deck's clients seam, where it persists
    across navigations and full reloads. The <a href="/deck/data">data seam</a>
    page shows the wiring.
  </p>

  <h2>Edits preserve state</h2>
  <p>
    In dev, saving a route swaps the freshly-bundled module into the running
    router and re-mounts it — <strong>signal state survives the swap</strong>, so
    counters, inputs, and scroll stay put. That's deck's
    <a href="/deck/hmr">HMR</a>, not a reactive primitive: the graph is simply
    never torn down.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border">
    <a href="/deck/layouts">← Layouts</a>
    <a href="/deck/data">The data seam →</a>
  </row>
</article>
