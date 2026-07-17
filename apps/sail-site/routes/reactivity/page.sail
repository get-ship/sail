<controller>
// /reactivity — the live graph the page describes. One signal, one derived,
// one effect writing a visible log: the reader clicks and watches all three
// layers respond.

import { controller, signal, derived, effect, type ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ViewScope<Scope>) => {
  const count = signal(0)
  const doubled = derived(() => count() * 2)
  const log = signal<string[]>([])
  effect(() => {
    const n = count()
    log.set((prev) => [...prev.slice(-3), `effect ran: count is ${n}`])
  })
  return { code, count, doubled, log }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Building</p>
  <h1>Reactivity</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    SAIL's reactivity is three small primitives: a <strong>signal</strong> holds a
    value, a <strong>derived</strong> computes one from others, and an
    <strong>effect</strong> runs a side-effect when its inputs change. No
    proxies, no dependency arrays — you read the graph.
  </p>

  <h2>The graph, live</h2>
  <grid cols="2" gap="4">
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Live</p>
      <column sx="p-6 border border-border rounded-lg min-h-8 bg-surface-1" gap="3">
        <row gap="3" align="center">
          <button on:click={() => count.set(count() + 1)}>count.set({count() + 1})</button>
          <p><strong>{count()}</strong> <small sx="text-muted-foreground">signal</small></p>
          <p><strong>{doubled()}</strong> <small sx="text-muted-foreground">derived</small></p>
        </row>
        <column gap="1">
          {#for line of log() key={line}}
            <p sx="text-sm text-muted-foreground">{line}</p>
          {/for}
        </column>
      </column>
    </column>
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Source</p>
      <code-block code={code.graph}></code-block>
    </column>
  </row>

  <h2>Signals</h2>
  <p>
    A signal is a readable, writable value. Call it to read; call
    <code>.set()</code> to write. Reading a signal inside a derived or effect
    subscribes to it automatically.
  </p>
  <code-block code={code.signal}></code-block>

  <h2>Derived</h2>
  <p>
    A derived value recomputes from the signals it reads. It's lazy and cached —
    it only recomputes when you read it <em>and</em> an input actually changed.
  </p>
  <code-block code={code.derived}></code-block>

  <h2>Effects</h2>
  <p>
    An effect runs immediately, then re-runs whenever a signal it read changes.
    Use it for the edges of your app — logging, the DOM, network, storage. Return
    a function and the effect cleans up after itself: that cleanup runs before the
    next run and when the effect is disposed, so a timer, socket, or listener is
    never left dangling. <code>effect()</code> returns a disposer to stop it
    entirely.
  </p>
  <code-block code={code.effect}></code-block>

  <h2>Escape hatches</h2>
  <p>
    Two tools for when the default tracking isn't what you want.
    <code>batch()</code> coalesces several writes into a single update pass, so
    effects run once instead of once per write. <code>untrack()</code> reads a
    signal without subscribing to it — useful when an effect should react to one
    input but only sample another.
  </p>
  <code-block code={code.escape}></code-block>

  <h2>In a component</h2>
  <p>
    Templates bind to signals directly. When the signal changes, only the exact
    DOM that read it updates — no virtual DOM, no re-render of the component.
  </p>
  <code-block code={code.template}></code-block>
  <p sx="border border-border border-l-3 border-l-primary rounded-md p-3 px-4 text-sm text-muted-foreground">
    A text binding <code>&#123;count()&#125;</code> is a compiled effect. When the
    signal changes, the compiler updates only the one text node that read it
    — nothing else in the tree re-runs. That is the whole runtime.
  </p>

  <h2>Lifecycle: <code>mount</code> and cleanup</h2>
  <p>
    Some state has to reach outside the component — a
    <code>ResizeObserver</code>, a socket, a timer. That is what
    <code>mount</code> is for: it runs once, after the element is in the DOM, and
    receives the scope plus the element itself. Register the undo with
    <code>onCleanup</code>, and the runtime calls it when the component goes away.
  </p>
  <code-block code={code.lifecycle}></code-block>
  <p sx="border border-border border-l-3 border-l-primary rounded-md p-3 px-4 text-sm text-muted-foreground">
    <strong>There is no dependency array.</strong> An effect subscribes to
    whatever it reads, and <code>onCleanup</code> is the only teardown there is —
    so a listener cannot be left dangling because someone forgot to list it.
  </p>

  <h2>Compositions</h2>
  <p>
    A <strong>composition</strong> is a plain function over these primitives.
    There is no type to implement, nothing to register, no base class. The rule is
    one line: <strong>if it touches the DOM it is an
    <a href="/behavior">action</a>; if it returns state it is a
    composition.</strong>
  </p>
  <p>
    They ship with the language — <code>form()</code>, <code>selection()</code>,
    <code>filteredList()</code> — because you no more install <code>form()</code>
    than you install <code>signal()</code>. And yours look exactly like ours,
    because there is nothing privileged about ours.
  </p>
  <code-block code={code.composition}></code-block>

  <row sx="mt-8 pt-4 border-t border-t-border">
    <a variant="default" href="/getting-started">← Getting started</a>
    <a variant="solid" tone="primary" href="/templates">Templates →</a>
  </row>
</column>
