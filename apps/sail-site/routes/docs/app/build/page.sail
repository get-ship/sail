<!--
  /docs/app/build — adapted from packages/docs/app/docs/routes/build/page.sail.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { buildCmd, buildOut, baseCmd, spaFallback } from '../../../../src/compositions/deck-build-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ buildCmd, buildOut, baseCmd, spaFallback }))
</controller>

<article style({ size({ max-w: breakpoint-tablet }).margin({ x: auto }) })>
  <p style({ margin({ b: 2 }).fg(accent).type({ family: mono, size: xs, tracking: wide, case: uppercase }) })>Rendering &amp; build</p>
  <h1>Build &amp; deploy</h1>
  <p style({ fg(muted-foreground).type({ size: lg, leading: relaxed }) })>One command writes a folder you can put on any static host.</p>

  <h2>Build</h2>
  <p><code>sail build</code> writes <code>dist/</code> by default: the document, hashed assets, and whatever you put in <code>public/</code>.</p>
  <code-block code={buildCmd} lang="bash"></code-block>
  <pre style({ pad({ all: 4, x: 5 }).bg(surface-1).border({ color: border, width: 1 }).radius(lg).type({family: mono, size: sm, leading: relaxed, wrap: pre-wrap, break: words}) })>{buildOut}</pre>
  <p>Files under <code>public/</code> copy to the output root — favicons, <code>_headers</code>, <code>_redirects</code>, anything else the host should see as a real URL.</p>

  <h2>Deep links</h2>
  <p>This is a single-page app. <code>/about</code> is not a file; it is a route. The host must serve <code>index.html</code> for paths that are not files. Cloudflare Pages reads <code>_redirects</code>; nginx and similar need the same rewrite.</p>
  <code-block code={spaFallback} lang="text"></code-block>

  <h2>Asset URLs</h2>
  <p>Pass <code>--base assets</code> so asset URLs become <code>/assets/&lt;file&gt;</code>, matching the on-disk layout.</p>
  <code-block code={baseCmd} lang="bash"></code-block>

  <h2>What ships</h2>
  <p>A production build does not include the dev server or live reload. Just the document, the framework files, and the route chunks.</p>

  <row style({ pad({ t: 4 }).margin({ t: 8 }).border({ color: border, width: 1, edge: t }) })>
    <a href="/docs/app/hmr">← HMR</a>
  </row>
</article>
