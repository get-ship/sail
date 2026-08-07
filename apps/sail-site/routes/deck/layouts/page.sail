<!--
  /deck/layouts — adapted from packages/deck/docs/routes/layouts/page.sail.
-->

<controller>
// /deck/layouts — adapted from packages/deck/docs/routes/layouts.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { layoutFile, layoutChain } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ layoutFile, layoutChain }))
</controller>

<article style({ box: maxW(breakpoint-tablet).mx(auto) })>
  <p style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Routing</p>
  <h1>Layouts</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })>A <code>_layout.sail</code> wraps every route beneath it in shared chrome — a header, a nav, a sidebar — that survives navigation.</p>

  <h2>A layout is chrome</h2>
  <p>deck reads the layout’s <strong>first custom-element tag</strong> as the chrome wrapper, then mounts each route into that element’s <code>&lt;slot name="route"&gt;</code>.</p>
  <code-block code={layoutFile} lang="sail"></code-block>
  <p>This docs site’s <code>routes/_layout.sail</code> is exactly that one line. <code>&lt;sail-site-header&gt;</code> projects a <code>&lt;slot name="route"&gt;</code> — the top bar around this page is that chrome; everything below it is the route.</p>

  <h2>The chrome persists</h2>
  <p>On a client-side navigation only the route slot swaps. The header, nav, and any layout state stay mounted across navigations — no chrome reflow, no flash.</p>

  <h2>Layouts compose</h2>
  <p>Nested <code>_layout.sail</code> files form a layout chain: a route is wrapped by every layout above it, outermost first. This page is a live example — the root layout’s <code>&lt;sail-site-header&gt;</code> wraps <code>routes/deck/_layout.sail</code>’s <code>&lt;sail-deck-rail&gt;</code>, which wraps this article.</p>
  <pre style({ box: pad(4).padX(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg), text: fontFamily(mono).fontSize(sm).leading(relaxed).whitespace(pre-wrap).break(words) })>{layoutChain}</pre>

  <row style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })>
    <a href="/deck/dynamic-routes">← Dynamic routes</a>
    <a href="/deck/reactivity">Reactivity →</a>
  </row>
</article>
