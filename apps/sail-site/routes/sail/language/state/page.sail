<!--
  /sail/language/state — chapter 2. The live graph the page describes: one
  signal, one derived, one effect writing a VISIBLE log (the snippet shows
  the same code — no console.log needles). Written for a reader we respect,
  not for a React developer: no dependency-array talk, no useEffect
  comparisons, concepts before examples. Stores lead with the problem and
  demonstrate with two different components sharing one store, live.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  const count = signal(0)
  const doubled = derived(() => count() * 2)
  const log = signal<string[]>([])
  effect(() => {
    const n = count()
    log.set((prev) => [...prev.slice(-2), `effect ran — count is now ${n}`])
  })
  return { code, count, doubled, log }
})
</controller>

<column gap="5" style({ box: maxW(breakpoint-tablet).mx(auto) })>
  <p style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>The language</p>
  <h1>State</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })>
    State is three primitives. A <strong>signal</strong> holds a value. A
    <strong>derived</strong> computes a value from other state. An
    <strong>effect</strong> runs code when the state it reads changes.
    Everything else in this chapter is those three, composed — and the
    direction never varies: the template reads state, handlers write it,
    effects react to it.
  </p>

  <h2>The graph, live</h2>
  <row gap="4" align="start">
    <column style({  box: grow(1).basis(breakpoint-mobile).minW(0)  })>
      <p style({ box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Live</p>
      <column gap="4" style({ box: pad(6).minH(8), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })>
        <row gap="5" align="center">
          <button on:click={() => count.set(count() + 1)}>+1</button>
          <column gap="1" align="center">
            <small style({ paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) })>signal</small>
            <strong style({ text: fontSize(lg) })>{count()}</strong>
          </column>
          <column gap="1" align="center">
            <small style({ paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) })>derived ×2</small>
            <strong style({ text: fontSize(lg) })>{doubled()}</strong>
          </column>
        </row>
        <column gap="1">
          <small style({ paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) })>effect output</small>
          <for {log() as line} key={line}>
            <p style({ paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono) })>{line}</p>
          </for>
        </column>
      </column>
    </column>
    <column style({  box: grow(1).basis(breakpoint-mobile).minW(0)  })>
      <p style({ box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Source</p>
      <code-block code={code.graph}></code-block>
    </column>
  </row>
  <p>
    One click moves all three layers: the write lands in the signal, the
    derived recomputes because its input changed, and the effect runs and
    appends a line you can see. Nothing subscribed to anything by hand.
  </p>

  <h2>Signals</h2>
  <p>
    A signal is a readable, writable value. Call it to read; call
    <code>.set()</code> to write. Reading a signal inside a derived, an
    effect, or a template binding subscribes automatically — that is the
    entire wiring model.
  </p>
  <code-block code={code.signal}></code-block>

  <h2>Derived</h2>
  <p>
    A derived value recomputes from the signals it reads. It is lazy and
    cached — it only recomputes when you read it <em>and</em> an input
    actually changed.
  </p>
  <code-block code={code.derived}></code-block>

  <h2>Effects</h2>
  <p>
    An effect runs immediately, then re-runs whenever a signal it reads
    changes. Effects are for the edges — the places state has to touch the
    world outside the graph: the document, a network call, storage. If the
    effect returns a function, that function is the undo: it runs before the
    next run and when the effect is disposed, so a timer or listener the
    effect started is never left running after it’s gone.
  </p>
  <code-block code={code.effect}></code-block>
  <p style({ box: pad(3).padX(4), paint: borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).fg(muted-foreground), text: fontSize(sm) })>
    An effect subscribes to what it <em>actually read on this run</em> — so
    a read the code never reached is a read the effect never subscribed to.
    If an early <code>return</code> or an <code>if</code> skips past a
    signal, the effect will not re-run when that signal changes. The habit
    that avoids the surprise: read your inputs at the top of the effect,
    then branch on the values.
  </p>

  <h2>The template is a subscriber</h2>
  <p>
    Everything above also describes the template — a binding like
    <code>&#123;count()&#125;</code> is a compiled effect whose job is to
    update one text node. The live demo at the top of this page is exactly
    this; here is its markup:
  </p>
  <code-block code={code.template}></code-block>
  <p>
    When <code>count</code> changes, the two bindings that read it update,
    the derived’s binding updates, and nothing else runs. There is no
    re-render to think about, because there is no re-render.
  </p>

  <h2>Lifecycle: <code>mount</code> and cleanup</h2>
  <p>
    A component appears, does work in the world, and eventually goes away —
    and anything it started has to stop when it does. That is the whole
    lifecycle story, and it has two words: <code>mount</code> runs once,
    after the element is on the page, and hands you the element; the
    function you return from it is the undo, called when the component
    leaves.
  </p>
  <p>
    Here that pattern watches the component’s own size, using
    <code>ResizeObserver</code> — a browser API that calls you whenever an
    element’s measured size changes:
  </p>
  <code-block code={code.lifecycle}></code-block>

  <h2><code>batch</code> and <code>untrack</code></h2>
  <p>
    Two tools that adjust <em>when</em> the graph reacts. You will not need
    them on day one, but you will meet them in real code, so they get real
    explanations.
  </p>
  <p>
    <strong><code>batch</code> makes several writes count as one.</strong>
    Setting two signals normally means dependents update twice — usually
    invisible, but if an effect saves to disk or an animation keys off the
    change, twice is wrong. Wrap the writes and the graph settles once,
    after the last one:
  </p>
  <code-block code={code.batch}></code-block>
  <p>
    <strong><code>untrack</code> reads without subscribing.</strong>
    Sometimes an effect should <em>react</em> to one thing but merely
    <em>use</em> another — an autosave that fires when the draft changes,
    and includes the current user, but should not re-fire on user switch:
  </p>
  <code-block code={code.untrack}></code-block>

  <h2>Stores — when state belongs to no one component</h2>
  <p>
    Sooner or later two components on opposite ends of a page need the same
    value: the cart total in the header and in the checkout pane, the
    signed-in user everywhere. Threading it through props means every
    component between them carries state it doesn’t care about. A store is
    the way out: state declared <em>outside</em> any component, that any
    component can reach by name.
  </p>
  <row gap="4" align="start">
    <column style({  box: grow(1).basis(breakpoint-mobile).minW(0)  })>
      <p style({ box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Live — click either card</p>
      <column gap="3" style({ box: pad(6), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })>
        <grid cols="2" gap="3">
          <store-counter label="one component"></store-counter>
          <store-counter label="a different instance"></store-counter>
        </grid>
        <store-total></store-total>
      </column>
    </column>
    <column style({  box: grow(1).basis(breakpoint-mobile).minW(0)  })>
      <p style({ box: mb(2), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>The store</p>
      <code-block code={code.storeDef}></code-block>
    </column>
  </row>
  <p>
    Three components share that store and none of them knows the others
    exist — no props carry the count, no events bubble it. Using one is one
    line:
  </p>
  <code-block code={code.storeUse}></code-block>
  <p>
    <code>useStore</code> resolves the nearest instance up the component
    tree, creating one at the root the first time anyone asks. That
    tree-scoping matters more than it looks: because instances live on the
    tree rather than in the module, two server-rendered requests can never
    bleed state into each other, and a subtree can be handed its own private
    copy with <code>provideStore</code> while its siblings keep the shared
    one.
  </p>

  <h2>Compositions</h2>
  <p>
    When the same few signals and effects keep appearing together, you fold
    them into a <strong>composition</strong> — a function that builds a
    piece of the graph for whoever calls it. There is nothing to register
    and no type to implement:
  </p>
  <code-block code={code.composition}></code-block>
  <p>
    The contract is worth saying precisely: a composition has access to
    <em>exactly what you pass it</em> — its arguments, and the primitives.
    It cannot see the caller’s template, its props, or its element. That
    boundary is what makes one reusable anywhere. Logic that needs the DOM
    is the other kind of reuse — an <a href="/sail/language/actions">action</a>,
    which is handed the element explicitly.
  </p>

  <p style({ box: mt(4), paint: fg(muted-foreground), text: fontSize(sm) })>
    For the author runtime (tier B): <a href="/sail/reference/runtime">reference → Author runtime</a>.
  </p>

  <row justify="between" gap="2" style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })>
    <a variant="default" href="/sail/language/sail-file">← Components</a>
    <a variant="solid" tone="primary" href="/sail/language/templates">Templates →</a>
  </row>
</column>
