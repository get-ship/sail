<!--
  /docs/app/getting-started — page file, controller, sail dev.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { pageExample, controllerExample, devCmd, siteConfig } from '../../../../src/compositions/deck-getting-started-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ pageExample, controllerExample, devCmd, siteConfig }))
</controller>

<article>
  <column gap="5">
    <p>Introduction</p>
    <h1>Getting started</h1>
    <p>You need the <code>sail</code> CLI and a folder. No project scaffold, no install, no config to start.</p>

    <h2>1. A page is a file</h2>
    <p>Create <code>routes/page.sail</code>. Whatever markup it holds is your page.</p>
    <code-block code={pageExample} lang="sail"></code-block>

    <h2>2. Add a controller</h2>
    <p>When the page needs state, add a <code>&lt;controller&gt;</code> block to the same file. It returns a scope the template reads by name — one file, template and state together.</p>
    <code-block code={controllerExample} lang="sail"></code-block>

    <h2>3. Title the app</h2>
    <p>App-wide title and description go in <code>sail.config.jsonc</code> under <code>app</code>. The first document uses these.</p>
    <code-block code={siteConfig} lang="json"></code-block>

    <h2>4. Run the dev server</h2>
    <code-block code={devCmd} lang="bash"></code-block>
    <alert tone="info">
      <strong>No build step to develop.</strong> Edit a route and the page updates. Signal state is kept.
    </alert>

    <separator />
    <row gap="2">
      <a href="/docs/app">← Overview</a>
      <a href="/docs/app/routing">File routing →</a>
    </row>
  </column>
</article>
