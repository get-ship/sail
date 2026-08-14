<!--
  /docs/app/build — adapted from packages/docs/app/docs/routes/build/page.sail.
-->

<controller>
// /docs/app/build — adapted from packages/docs/app/docs/routes/build.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { buildCmd, buildOut, baseCmd } from '../../../../src/compositions/deck-build-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ buildCmd, buildOut, baseCmd }))
</controller>

<article style({ size({ max-w: breakpoint-tablet }).margin({ x: auto }) })>
  <p style({ margin({ b: 2 }).fg(accent).type({ family: mono, size: xs, tracking: wide, case: uppercase }) })>Rendering &amp; build</p>
  <h1>Build &amp; deploy</h1>
  <p style({ fg(muted-foreground).type({ size: lg, leading: relaxed }) })>One command emits a self-contained static directory a dumb host can serve as-is.</p>

  <h2>Build a static dir</h2>
  <p><code>sail build --project . --bundle-framework</code> materializes everything the app needs into one directory.</p>
  <code-block code={buildCmd} lang="bash"></code-block>
  <pre style({ pad({ all: 4, x: 5 }).bg(surface-1).border({ color: border, width: 1 }).radius(lg).type({family: mono, size: sm, leading: relaxed, wrap: pre-wrap, break: words}) })>{buildOut}</pre>
  <p>You get <code>index.html</code> (the importmap), <code>_deck/</code> (the framework singletons, materialized), and one code-split, lazily-loaded <code>assets/page-&lt;hash&gt;.js</code> chunk per route.</p>

  <h2>Deploy anywhere</h2>
  <p>Pass <code>--base assets</code> so asset URLs become <code>/assets/&lt;file&gt;</code>, matching the on-disk layout. A dumb static host — Netlify, S3, nginx — then serves the directory directly.</p>
  <code-block code={baseCmd} lang="bash"></code-block>

  <h2>No dev cruft</h2>
  <p>A shipped build carries nothing from dev: no live-reload client, no dev-only metas. Just the markup, the framework singletons, and the route chunks.</p>

  <row style({ pad({ t: 4 }).margin({ t: 8 }).border({ color: border, width: 1, edge: t }) })>
    <a href="/docs/app/hmr">← HMR</a>
  </row>
</article>
