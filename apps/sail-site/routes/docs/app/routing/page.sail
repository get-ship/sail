<!--
  /docs/app/routing — filesystem routes, colocated page+controller, generated tables.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { routeTree, colocated } from '../../../../src/compositions/deck-routing-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ routeTree, colocated }))
</controller>

<article>
  <column gap="5">
    <p>Routing</p>
    <h1>File routing</h1>
    <p>deck routes straight from the filesystem. A directory under <code>routes/</code> is a URL — there is no route-config file to keep in sync.</p>

    <h2>The directory is the URL</h2>
    <p>Drop a <code>page.sail</code> anywhere under <code>routes/</code> and its folder path becomes the route. The tree on disk is the routing table.</p>
    <code-block code={routeTree} lang="text"></code-block>

    <h2>One file is the route</h2>
    <p>The <code>page.sail</code> carries everything: markup, and — when the page has state — a <code>&lt;controller&gt;</code> block in the same file. Nothing else to co-locate, nothing to keep in sync.</p>
    <code-block code={colocated} lang="text"></code-block>

    <h2>Generated tables</h2>
    <p>Walking <code>routes/</code>, deck writes a route table and a small bootstrap. You never edit them. The filesystem is the only config.</p>

    <separator />
    <row gap="2">
      <a href="/docs/app/getting-started">← Getting started</a>
      <a href="/docs/app/dynamic-routes">Dynamic routes →</a>
    </row>
  </column>
</article>
