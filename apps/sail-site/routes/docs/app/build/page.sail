<!--
  /docs/app/build — adapted from packages/docs/app/docs/routes/build/page.sail.
-->

<controller>
// /docs/app/build — adapted from packages/docs/app/docs/routes/build.
import { buildCmd, buildOut, baseCmd, spaFallback } from '../../../../src/compositions/deck-build-snippets'

export default controller(() => ({ buildCmd, buildOut, baseCmd, spaFallback }))
</controller>

<article>
  <column gap="5">
    <p>Rendering &amp; build</p>
    <h1>Build &amp; deploy</h1>
    <p>One command writes a folder you can put on any static host.</p>

    <h2>Build</h2>
    <p><code>sail build</code> writes <code>dist/</code> by default: the document, hashed assets, and whatever you put in <code>public/</code>.</p>
    <code-block code={buildCmd} lang="bash"></code-block>
    <code-block code={buildOut} lang="text"></code-block>
    <p>Files under <code>public/</code> copy to the output root — favicons, <code>_headers</code>, <code>_redirects</code>, anything else the host should see as a real URL.</p>

    <h2>Deep links</h2>
    <p>This is a single-page app. <code>/about</code> is not a file; it is a route. The host must serve <code>index.html</code> for paths that are not files. Cloudflare Pages reads <code>_redirects</code>; nginx and similar need the same rewrite.</p>
    <code-block code={spaFallback} lang="text"></code-block>

    <h2>Asset URLs</h2>
    <p>Pass <code>--base assets</code> so asset URLs become <code>/assets/&lt;file&gt;</code>, matching the on-disk layout.</p>
    <code-block code={baseCmd} lang="bash"></code-block>

    <h2>What ships</h2>
    <p>A production build does not include the dev server or live reload. Just the document, the framework files, and the route chunks.</p>

    <separator />
    <row gap="2">
      <a href="/docs/app/hmr">← HMR</a>
    </row>
  </column>
</article>
