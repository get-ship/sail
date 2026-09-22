<!--
  /docs/app — Deck overview. Shell sets theme="deck"; this page is pure content.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { quickstart, layers } from '../../../src/compositions/deck-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ quickstart, layers }))
</controller>

<article>
  <column gap="5">
    <p>deck · 0.2.0</p>
    <h1>The standalone web framework for Sail.</h1>
    <p>
      deck hosts Sail pages. It turns a <code>routes/</code> folder into a running
      app — file routing, a dev server with HMR, and a static build you can put
      on any host. It is to Sail what Next is to React.
    </p>

    <alert tone="info">
      <strong>This site is a deck app.</strong> Every page you read is a file in
      <code>routes/</code>.
    </alert>

    <h2>The 30-second app</h2>
    <p>A deck app is a <code>routes/</code> folder and a dev server. No config file, no scaffold step.</p>
    <code-block code={quickstart} lang="text"></code-block>
    <p>That’s a running Sail single-page app, with HMR, on <code>127.0.0.1:7878</code>.</p>

    <h2>Where it sits</h2>
    <code-block code={layers} lang="text"></code-block>
    <p>
      Sail is the component language — signals, templates, controllers. deck is the
      framework around it — routing, rendering, build, serving. Your app is what
      you write under <code>routes/</code>.
    </p>

    <separator />
    <row gap="2">
      <a variant="solid" tone="primary" href="/docs/app/getting-started">Getting started →</a>
    </row>
  </column>
</article>
