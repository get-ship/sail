<!--
  /deck/getting-started — adapted from packages/deck/docs/routes/getting-started/page.sail.
  Snippets live in ./snippets so literal braces in example code never get scope-scanned.
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { pageExample, controllerExample, devCmd } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ pageExample, controllerExample, devCmd }))
</controller>

<article style({ box: maxW(breakpoint-tablet).mx(auto) })>
  <p style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Introduction</p>
  <h1>Getting started</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })>You need the <code>sail</code> CLI and a folder. No project scaffold, no install, no config to start.</p>

  <h2>1. A page is a file</h2>
  <p>Create <code>routes/page.sail</code>. Whatever markup it holds is your page.</p>
  <code-block code={pageExample} lang="sail"></code-block>

  <h2>2. Add a controller</h2>
  <p>When the page needs state, add a <code>&lt;controller&gt;</code> block to the same file. It returns a scope the template reads by name — one file, template and state together.</p>
  <code-block code={controllerExample} lang="sail"></code-block>

  <h2>3. Run the dev server</h2>
  <code-block code={devCmd} lang="bash"></code-block>
  <alert tone="info">
    <strong>No build step to develop.</strong> deck compiles <code>.sail</code> in-process and serves the SPA with HMR — edit a route and it soft-reloads with state preserved.
  </alert>

  <row style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })>
    <a href="/deck">← Overview</a>
    <a href="/deck/routing">File routing →</a>
  </row>
</article>
