<controller>
// Build & deploy controller. Snippets live in ./snippets (a plain data module)
// so example code containing `{ ... }` is never scope-scanned by the compiler.
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { buildCmd, buildOut, baseCmd } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ buildCmd, buildOut, baseCmd }))
</controller>

<article data-doc="true">
  <p role="doc-eyebrow">Rendering &amp; build</p>
  <h1>Build &amp; deploy</h1>
  <p role="doc-lead">One command emits a self-contained static directory a dumb host can serve as-is.</p>

  <h2>Build a static dir</h2>
  <p><code>sail build --project . --bundle-framework</code> materializes everything the app needs into one directory.</p>
  <pre data-code-block="true">{buildCmd}</pre>
  <pre data-code-block="true">{buildOut}</pre>
  <p>You get <code>index.html</code> (the importmap), <code>_deck/</code> (the framework singletons, materialized), and one code-split, lazily-loaded <code>assets/page-&lt;hash&gt;.js</code> chunk per route.</p>

  <h2>Deploy anywhere</h2>
  <p>Pass <code>--base assets</code> so asset URLs become <code>/assets/&lt;file&gt;</code>, matching the on-disk layout. A dumb static host — Netlify, S3, nginx — then serves the directory directly.</p>
  <pre data-code-block="true">{baseCmd}</pre>

  <h2>No dev cruft</h2>
  <p>A shipped build carries nothing from dev: no live-reload client, no dev-only metas. Just the markup, the framework singletons, and the route chunks.</p>

  <div role="navigation">
    <a href="/hmr">← HMR</a>
    <span></span>
  </div>
</article>
