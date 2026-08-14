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
  <p style(lead)>deck paints a route one of two ways. The default renders on the client; opt into server rendering for first paint and SEO.</p>

  <h2>CSR (default)</h2>
  <p>deck serves an empty <code>&lt;div id="root"&gt;</code> and an importmap shell. The client entry mounts the <code>routes/</code> SPA and paints. The initial document is near-empty; everything is drawn in the browser.</p>

  <h2>SSR</h2>
  <p>Set <code>"render": "ssr"</code> in <code>sail.config.jsonc</code>. The matched route is server-rendered (via ship-js) into <code>#root</code>, and the client hydrates it in place.</p>
  <code-block code={ssrConfig} lang="json"></code-block>
  <p>With SSR the route’s markup is in the <strong>raw server bytes</strong> — good for first paint and for crawlers. (This docs site itself runs CSR; deck’s SSR path is proven by the framework’s own browser test harness.)</p>

  <h2>Either way, one static dir</h2>
  <p>Whichever mode you pick, <code>sail build</code> produces a self-contained static directory.</p>
  <code-block code={buildCmd} lang="bash"></code-block>

  <row style(footNav)>
    <a href="/docs/app/data">← The data seam</a>
    <a href="/docs/app/hmr">HMR →</a>
  </row>
</article>
