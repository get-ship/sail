<!--
  /docs/app/layouts — _layout.sail chrome, slot, layout chain.
-->

<scope>
  style page = { size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style eyebrow = { margin({ b: 2 }).fg(accent).type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { fg(muted-foreground).type({ size: lg, leading: relaxed }) }
  style monoBlock = { pad({ all: 4, x: 5 }).bg(surface-1).border({ color: border, width: 1 }).radius(lg).type({family: mono, size: sm, leading: relaxed, wrap: pre-wrap, break: words}) }
  style footNav = { pad({ t: 4 }).margin({ t: 8 }).border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { layoutFile, layoutChain } from '../../../../src/compositions/deck-layouts-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ layoutFile, layoutChain }))
</controller>

<article style(page)>
  <p style(eyebrow)>Routing</p>
  <h1>Layouts</h1>
  <p style(lead)>A <code>_layout.sail</code> wraps every route beneath it in shared chrome — a header, a nav, a sidebar — that survives navigation.</p>

  <h2>A layout is chrome</h2>
  <p>deck reads the layout’s <strong>first custom-element tag</strong> as the chrome wrapper, then mounts each route into that element’s <code>&lt;slot name="route"&gt;</code>.</p>
  <code-block code={layoutFile} lang="sail"></code-block>
  <p>This docs site’s <code>routes/_layout.sail</code> is exactly that one line. <code>&lt;sail-site-header&gt;</code> projects a <code>&lt;slot name="route"&gt;</code> — the top bar around this page is that chrome; everything below it is the route.</p>

  <h2>The chrome persists</h2>
  <p>On a client-side navigation only the route slot swaps. The header, nav, and any layout state stay mounted across navigations — no chrome reflow, no flash.</p>

  <h2>Layouts compose</h2>
  <p>Nested <code>_layout.sail</code> files form a layout chain: a route is wrapped by every layout above it, outermost first. This page is a live example — the root layout’s <code>&lt;sail-site-header&gt;</code> wraps <code>routes/docs/app/_layout.sail</code>’s <code>&lt;sail-deck-rail&gt;</code>, which wraps this article.</p>
  <pre style(monoBlock)>{layoutChain}</pre>

  <row style(footNav)>
    <a href="/docs/app/dynamic-routes">← Dynamic routes</a>
    <a href="/docs/app/reactivity">Reactivity →</a>
  </row>
</article>
