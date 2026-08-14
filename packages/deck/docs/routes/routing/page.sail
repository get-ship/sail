<!-- File routing — directory is the URL, co-located files, generated tables. -->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeTree, colocated } from '../../src/compositions/deck-routing-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeTree, colocated }))
</controller>

<scope>
  style page = { maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { mb(2).fg(accent).fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { fg(muted-foreground).fontSize(lg).leading(relaxed) }
  style footNav = { mt(8).padT(4).borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Routing</p>
  <h1>File routing</h1>
  <p style(lead)>deck routes straight from the filesystem. A directory under <code>routes/</code> is a URL — there is no route-config file to keep in sync.</p>

  <h2>The directory is the URL</h2>
  <p>Drop a <code>page.sail</code> anywhere under <code>routes/</code> and its folder path becomes the route. The tree on disk is the routing table.</p>
  <code-block code={routeTree} lang="text"></code-block>

  <h2>Co-located files</h2>
  <p>A route may sit beside a <code>controller.ts</code> for reactive state and a <code>{'style.css'}</code> for page styles. They belong to that route alone.</p>
  <code-block code={colocated} lang="text"></code-block>

  <h2>Generated tables</h2>
  <p>Walking <code>routes/</code>, deck generates <code>routes.gen.ts</code> (the typed route table) and <code>entry.gen.ts</code> (the SPA bootstrap that calls <code>mountRouter</code>). You never edit them — they regenerate on every build and every dev run. The filesystem is the only config.</p>

  <row justify="between" style(footNav)>
    <a href="/getting-started">← Getting started</a>
    <a href="/dynamic-routes">Dynamic routes →</a>
  </row>
</article>
