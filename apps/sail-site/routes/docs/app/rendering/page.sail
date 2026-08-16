<!--
  /docs/app/rendering — CSR default, SSR opt-in, sail build static out.
-->

<scope>
  style page = { size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style eyebrow = { margin({ b: 2 }).fg(accent).type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { fg(muted-foreground).type({ size: lg, leading: relaxed }) }
  style footNav = { pad({ t: 4 }).margin({ t: 8 }).border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { ssrConfig, buildCmd } from '../../../../src/compositions/deck-rendering-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ ssrConfig, buildCmd }))
</controller>

<article style(page)>
  <p style(eyebrow)>Rendering &amp; build</p>
  <h1>Rendering modes</h1>
  <p style(lead)>The default app is a single-page app. The browser loads one document and paints each route as you navigate.</p>

  <h2>What the first document is</h2>
  <p>The built <code>index.html</code> is a shell: title and description from <code>sail.config.jsonc</code> <code>app</code>, then a root element the app mounts into. Route markup is drawn in the browser. Deep links still start from the site title until the app boots — the host must serve this same document for every path (see Build).</p>

  <h2>Server rendering</h2>
  <p>Set <code>"render": "ssr"</code> under the <code>deck</code> object in <code>sail.config.jsonc</code> if you want the matched route in the first HTML. This site ships as a client-rendered SPA.</p>
  <code-block code={ssrConfig} lang="json"></code-block>

  <h2>One static directory</h2>
  <p><code>sail build</code> writes a folder you can host as-is.</p>
  <code-block code={buildCmd} lang="bash"></code-block>

  <row style(footNav)>
    <a href="/docs/app/data">← The data seam</a>
    <a href="/docs/app/hmr">HMR →</a>
  </row>
</article>
