<!-- HMR — soft-reload with signal state, component hot-swap, SSE push. -->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { devCmd } from '../../src/compositions/deck-hmr-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ devCmd }))
</controller>

<scope>
  style page = { maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { mb(2).fg(accent).fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { fg(muted-foreground).fontSize(lg).leading(relaxed) }
  style footNav = { mt(8).padT(4).borderTWidth(1).borderT(border) }
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

  <row justify="between" style(footNav)>
    <a href="/rendering">← Rendering modes</a>
    <a href="/build">Build &amp; deploy →</a>
  </row>
</article>
