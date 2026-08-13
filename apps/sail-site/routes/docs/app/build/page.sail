<!--
  /docs/app/build — adapted from packages/docs/app/docs/routes/build/page.sail.
-->

<controller>
// /docs/app/build — adapted from packages/docs/app/docs/routes/build.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { buildCmd, buildOut, baseCmd, cfDeploy } from '../../../../src/compositions/deck-build-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ buildCmd, buildOut, baseCmd, cfDeploy }))
</controller>

<article style({ box: maxW(breakpoint-tablet).mx(auto) })>
  <p style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Rendering &amp; build</p>
  <h1>Build &amp; deploy</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })>One command emits a self-contained static directory a dumb host can serve as-is.</p>

  <h2>Build a static dir</h2>
  <p><code>sail build --project . --bundle-framework</code> materializes everything the app needs into one directory.</p>
  <code-block code={buildCmd} lang="bash"></code-block>
  <pre style({ box: pad(4).padX(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg), text: fontFamily(mono).fontSize(sm).leading(relaxed).whitespace(pre-wrap).break(words) })>{buildOut}</pre>
  <p>You get one <code>index.html</code> (CSR SPA shell + importmap — not HTML per route), <code>_deck/</code> (framework singletons + used icons), hashed <code>/assets</code> chunks, and <code>sail_compiler_bg.wasm</code> for the playground. The host must SPA-fallback extensionless paths to <code>index.html</code>.</p>

  <h2>Deploy anywhere</h2>
  <p>Pass <code>--base assets</code> so asset URLs become <code>/assets/&lt;file&gt;</code>, matching the on-disk layout. A dumb static host — Cloudflare Pages, Netlify, S3, nginx — then serves the directory directly.</p>
  <code-block code={baseCmd} lang="bash"></code-block>

  <h2>Cloudflare Pages</h2>
  <p>This site ships <code>wrangler.toml</code>, <code>_headers</code> (wasm / woff2 MIME), and <code>_redirects</code> (SPA fallback). After the static build:</p>
  <code-block code={cfDeploy} lang="bash"></code-block>

  <h2>No dev cruft</h2>
  <p>A shipped build carries nothing from dev: no live-reload client, no dev-only metas. Just the markup, the framework singletons, and the route chunks.</p>

  <row style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })>
    <a href="/docs/app/hmr">← HMR</a>
  </row>
</article>
