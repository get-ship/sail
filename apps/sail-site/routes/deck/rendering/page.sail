<!--
  /deck/rendering — adapted from packages/deck/docs/routes/rendering/page.sail.
-->

<controller>
// /deck/rendering — adapted from packages/deck/docs/routes/rendering.
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { ssrConfig, buildCmd } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ ssrConfig, buildCmd }))
</controller>

<article sx={style.maxW(breakpoint-tablet).mx(auto)}>
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>Rendering &amp; build</p>
  <h1>Rendering modes</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>deck paints a route one of two ways. The default renders on the client; opt into server rendering for first paint and SEO.</p>

  <h2>CSR (default)</h2>
  <p>deck serves an empty <code>&lt;div id="root"&gt;</code> and an importmap shell. The client entry mounts the <code>routes/</code> SPA and paints. The initial document is near-empty; everything is drawn in the browser.</p>

  <h2>SSR</h2>
  <p>Set <code>"render": "ssr"</code> in <code>deck.jsonc</code>. The matched route is server-rendered (via ship-js) into <code>#root</code>, and the client hydrates it in place.</p>
  <code-block code={ssrConfig} lang="json"></code-block>
  <p>With SSR the route's markup is in the <strong>raw server bytes</strong> — good for first paint and for crawlers. (This docs site itself runs CSR; deck's SSR path is proven by the framework's own browser test harness.)</p>

  <h2>Either way, one static dir</h2>
  <p>Whichever mode you pick, <code>deck build</code> produces a self-contained static directory.</p>
  <code-block code={buildCmd} lang="bash"></code-block>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)}>
    <a href="/deck/data">← The data seam</a>
    <a href="/deck/hmr">HMR →</a>
  </row>
</article>
