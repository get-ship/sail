<!--
  /docs/app/routing — filesystem routes, colocated page+controller, generated tables.
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
  <p>Walking <code>routes/</code>, deck writes a route table and a small bootstrap. You never edit them. The filesystem is the only config.</p>

  <row style(footNav)>
    <a href="/docs/app/getting-started">← Getting started</a>
    <a href="/docs/app/dynamic-routes">Dynamic routes →</a>
  </row>
</article>
