<!--
  Overview (`/`) — the docs landing. What deck is, the 30-second app, where it
  sits relative to SAIL. Code snippets come from the controller as strings
  (literal `{` in a template would parse as interpolation).
-->

<controller>
// Overview controller — supplies the page's code snippets as strings (literal
// braces in a .sail template parse as interpolation, so code lives here).
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const quickstart = `my-app/
└─ routes/
   └─ page.sail        <h1>Hello, deck.</h1>

$ deck dev
  ▸ serving http://127.0.0.1:7878`

const layers = `SAIL    :  deck   :  your app
React  :  Next   :  your app`

export default controller((_ctx: ControllerScope<Scope>) => ({ quickstart, layers }))
</controller>

<article data-doc="true">
  <p role="doc-eyebrow">deck · 0.2.0</p>
  <h1>The standalone web framework for SAIL.</h1>
  <p role="doc-lead">
    deck mounts SAIL views, routes them from the filesystem, server-renders or
    statically builds them, and serves every asset — with a dev server and
    state-preserving HMR. It is to SAIL what Next is to React.
  </p>

  <div role="note">
    <strong>This site is a deck app.</strong> Every page you read is a file-routed
    deck route, server-rendered and hydrated. The docs are deck's own canonical
    example — and its release test.
  </div>

  <h2>The 30-second app</h2>
  <p>A deck app is a <code>routes/</code> folder and a dev server. No config file, no scaffold step.</p>
  <pre data-code-block="true">{quickstart}</pre>
  <p>That's a running SAIL single-page app, with HMR, on <code>127.0.0.1:7878</code>.</p>

  <h2>Where it sits</h2>
  <pre data-code-block="true">{layers}</pre>
  <p>
    SAIL is the component language — signals, templates, controllers. deck is the
    framework around it — routing, rendering, build, serving. Your app is what
    you write under <code>routes/</code>.
  </p>

  <div role="navigation">
    <span></span>
    <a variant="solid" href="/getting-started">Getting started →</a>
  </div>
</article>
