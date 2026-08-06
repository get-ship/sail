<!--
  /deck/build — adapted from packages/deck/docs/routes/build/page.sail.
-->

<controller>
// /deck/build — adapted from packages/deck/docs/routes/build.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { buildCmd, buildOut, baseCmd } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ buildCmd, buildOut, baseCmd }))
</controller>

<article>
  style.maxW(breakpoint-tablet).mx(auto)

  <p>
    style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)
Rendering &amp; build</p>
  <h1>Build &amp; deploy</h1>
  <p>
    style.fontSize(lg).fg(muted-foreground).leading(relaxed)
One command emits a self-contained static directory a dumb host can serve as-is.</p>

  <h2>Build a static dir</h2>
  <p><code>sail build --project . --bundle-framework</code> materializes everything the app needs into one directory.</p>
  <code-block code={buildCmd} lang="bash"></code-block>
  <pre>
    style.fontFamily(mono).fontSize(sm).leading(relaxed).bg(surface-1).borderWidth(1).border(border).radius(lg).pad(4).padX(5).whitespace(pre-wrap).break(words)
{buildOut}</pre>
  <p>You get <code>index.html</code> (the importmap), <code>_deck/</code> (the framework singletons, materialized), and one code-split, lazily-loaded <code>assets/page-&lt;hash&gt;.js</code> chunk per route.</p>

  <h2>Deploy anywhere</h2>
  <p>Pass <code>--base assets</code> so asset URLs become <code>/assets/&lt;file&gt;</code>, matching the on-disk layout. A dumb static host — Netlify, S3, nginx — then serves the directory directly.</p>
  <code-block code={baseCmd} lang="bash"></code-block>

  <h2>No dev cruft</h2>
  <p>A shipped build carries nothing from dev: no live-reload client, no dev-only metas. Just the markup, the framework singletons, and the route chunks.</p>

  <row>
    style.mt(8).padT(4).borderTWidth(1).borderT(border)

    <a href="/deck/hmr">← HMR</a>
  </row>
</article>
