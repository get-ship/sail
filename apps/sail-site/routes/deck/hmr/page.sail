<!--
  /deck/hmr — route soft-reload with state preserved; component hot-swap; SSE.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { devCmd } from '../../../src/compositions/deck-hmr-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ devCmd }))
</controller>

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Rendering &amp; build</p>
  <h1>HMR</h1>
  <p style(lead)>In dev, editing a route soft-reloads it with signal state preserved — no full document reload.</p>

  <h2>Routes reload with state intact</h2>
  <p>Save a route’s <code>.sail</code> and deck swaps the freshly-bundled module into the already-running router — via a <code>globalThis.__deckHmr</code> handle — then re-mounts. Signal state survives the swap, so counters, inputs, and scroll stay put.</p>
  <code-block code={devCmd} lang="bash"></code-block>

  <h2>Components hot-swap in place</h2>
  <p>Editing a <code>&lt;sail-*&gt;</code> component hot-swaps its live instances where they sit — every mounted copy updates without a reload.</p>

  <h2>Pushed over SSE</h2>
  <p>The dev server pushes changes to the browser over an <code>SSE</code> stream. HMR is a dev-only convenience; it ships in no build.</p>

  <row style(footNav)>
    <a href="/deck/rendering">← Rendering modes</a>
    <a href="/deck/build">Build &amp; deploy →</a>
  </row>
</article>
