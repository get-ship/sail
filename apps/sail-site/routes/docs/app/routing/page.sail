<!--
  /docs/app/routing — filesystem routes, colocated page+controller, generated tables.
-->
<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style monoBlock = { box: pad(4).padX(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg), text: fontFamily(mono).fontSize(sm).leading(relaxed).whitespace(pre-wrap).break(words) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeTree, colocated } from '../../../../src/compositions/deck-routing-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeTree, colocated }))
</controller>

<article style(page)>
  <p style(eyebrow)>Routing</p>
  <h1>File routing</h1>
  <p style(lead)>deck routes straight from the filesystem. A directory under <code>routes/</code> is a URL — there is no route-config file to keep in sync.</p>

  <h2>The directory is the URL</h2>
  <p>Drop a <code>page.sail</code> anywhere under <code>routes/</code> and its folder path becomes the route. The tree on disk is the routing table.</p>
  <pre style(monoBlock)>{routeTree}</pre>

  <h2>One file is the route</h2>
  <p>The <code>page.sail</code> carries everything: markup, and — when the page has state — a <code>&lt;controller&gt;</code> block in the same file. Nothing else to co-locate, nothing to keep in sync.</p>
  <pre style(monoBlock)>{colocated}</pre>

  <h2>Generated tables</h2>
  <p>Walking <code>routes/</code>, deck generates <code>routes.gen.ts</code> (the typed route table) and <code>entry.gen.ts</code> (the SPA bootstrap that calls <code>mountRouter</code>). You never edit them — they regenerate on every build and every dev run. The filesystem is the only config.</p>

  <row style(footNav)>
    <a href="/docs/app/getting-started">← Getting started</a>
    <a href="/docs/app/dynamic-routes">Dynamic routes →</a>
  </row>
</article>
