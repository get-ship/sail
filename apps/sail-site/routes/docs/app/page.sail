<!--
  /docs/app — Deck overview. Shell sets theme="deck"; this page is pure content.
-->

<scope>
  style page = { box: size({ max-w: breakpoint-tablet }).space({ mx: auto }) }
  style eyebrow = { box: space({ mb: 2 }), paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style codeBlock = { box: space({ pad: 4, pad-x: 5 }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(lg), text: type({ family: mono, size: sm, leading: relaxed }).whitespace(pre-wrap).break(words) }
  style footNav = { box: space({ pad-t: 4, mt: 8 }), paint: border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { quickstart, layers } from '../../../src/compositions/deck-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ quickstart, layers }))
</controller>

<article style(page)>
  <p style(eyebrow)>deck · 0.2.0</p>
  <h1>The standalone web framework for Sail.</h1>
  <p style(lead)>
    deck mounts Sail routes, routes them from the filesystem, server-renders or
    statically builds them, and serves every asset — with a dev server and
    state-preserving HMR. It is to Sail what Next is to React.
  </p>

  <alert tone="info">
    <strong>This site is a deck app.</strong> Every page you read is a file-routed
    deck route, server-rendered and hydrated. The docs are deck’s own canonical
    example — and its release test.
  </alert>

  <h2>The 30-second app</h2>
  <p>A deck app is a <code>routes/</code> folder and a dev server. No config file, no scaffold step.</p>
  <pre style(codeBlock)>{quickstart}</pre>
  <p>That’s a running Sail single-page app, with HMR, on <code>127.0.0.1:7878</code>.</p>

  <h2>Where it sits</h2>
  <pre style(codeBlock)>{layers}</pre>
  <p>
    Sail is the component language — signals, templates, controllers. deck is the
    framework around it — routing, rendering, build, serving. Your app is what
    you write under <code>routes/</code>.
  </p>

  <row style(footNav)>
    <a variant="solid" tone="primary" href="/docs/app/getting-started">Getting started →</a>
  </row>
</article>
