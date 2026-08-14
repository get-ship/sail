<!-- Build & deploy — sail build static dir, base assets, no dev cruft. -->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { buildCmd, buildOut, baseCmd } from '../../src/compositions/deck-build-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ buildCmd, buildOut, baseCmd }))
</controller>

<scope>
  style page = { maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { mb(2).fg(accent).fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { fg(muted-foreground).fontSize(lg).leading(relaxed) }
  style footNav = { mt(8).padT(4).borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Rendering &amp; build</p>
  <h1>Build &amp; deploy</h1>
  <p style(lead)>One command emits a self-contained static directory a dumb host can serve as-is.</p>

  <h2>Build a static dir</h2>
  <p><code>sail build --project . --bundle-framework</code> materializes everything the app needs into one directory.</p>
  <code-block code={buildCmd} lang="bash"></code-block>
  <code-block code={buildOut} lang="text"></code-block>
  <p>You get <code>index.html</code> (the importmap), <code>_deck/</code> (the framework singletons, materialized), and one code-split, lazily-loaded <code>assets/page-&lt;hash&gt;.js</code> chunk per route.</p>

  <h2>Deploy anywhere</h2>
  <p>Pass <code>--base assets</code> so asset URLs become <code>/assets/&lt;file&gt;</code>, matching the on-disk layout. A dumb static host — Netlify, S3, nginx — then serves the directory directly.</p>
  <code-block code={baseCmd} lang="bash"></code-block>

  <h2>No dev cruft</h2>
  <p>A shipped build carries nothing from dev: no live-reload client, no dev-only metas. Just the markup, the framework singletons, and the route chunks.</p>

  <row justify="between" style(footNav)>
    <a href="/hmr">← HMR</a>
    <span></span>
  </row>
</article>
