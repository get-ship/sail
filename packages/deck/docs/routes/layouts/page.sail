<!-- Layouts — _layout.sail chrome, persistence, nested layout chain. -->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { layoutFile, layoutChain } from '../../src/compositions/deck-layouts-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ layoutFile, layoutChain }))
</controller>

<scope>
  style page = { maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { mb(2).fg(accent).fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { fg(muted-foreground).fontSize(lg).leading(relaxed) }
  style footNav = { mt(8).padT(4).borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Routing</p>
  <h1>Layouts</h1>
  <p style(lead)>A <code>_layout.sail</code> wraps every route beneath it in shared chrome — a header, a nav, a sidebar — that survives navigation.</p>

  <h2>A layout is chrome</h2>
  <p>deck reads the layout’s <strong>first custom-element tag</strong> as the chrome wrapper, then mounts each route into that element’s <code>&lt;slot name="route"&gt;</code>.</p>
  <code-block code={layoutFile} lang="sail"></code-block>
  <p>This docs site’s <code>routes/_layout.sail</code> is exactly that one line. <code>&lt;sail-shell&gt;</code> projects a <code>&lt;slot name="route"&gt;</code> — the sidebar and top bar around this page are the chrome; the article is the route.</p>

  <h2>The chrome persists</h2>
  <p>On a client-side navigation only the route slot swaps. The header, nav, and any layout state stay mounted across navigations — no chrome reflow, no flash.</p>

  <h2>Layouts compose</h2>
  <p>Nested <code>_layout.sail</code> files form a layout chain: a route is wrapped by every layout above it, outermost first.</p>
  <code-block code={layoutChain} lang="text"></code-block>

  <row justify="between" style(footNav)>
    <a href="/dynamic-routes">← Dynamic routes</a>
    <a href="/reactivity">Reactivity →</a>
  </row>
</article>
