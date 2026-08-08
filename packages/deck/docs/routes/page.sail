<!-- Overview (`/`) — what deck is, 30-second app, SAIL layering. -->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

// Snippets as strings — literal `{` in a template would parse as interpolation.
const quickstart = `my-app/
└─ routes/
   └─ page.sail        <h1>Hello, deck.</h1>

$ sail dev
  ▸ serving http://127.0.0.1:7878`

const layers = `SAIL    :  deck   :  your app
React  :  Next   :  your app`

export default controller((_ctx: ControllerScope<Scope>) => ({ quickstart, layers }))
</controller>

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style callout = { box: padY(3).padX(4), paint: borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).bg(surface-1), text: fontSize(sm) }
  style calloutStrong = { paint: fg(foreground) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>deck · 0.2.0</p>
  <h1>The standalone web framework for SAIL.</h1>
  <p style(lead)>
    deck mounts SAIL routes, routes them from the filesystem, server-renders or
    statically builds them, and serves every asset — with a dev server and
    state-preserving HMR. It is to SAIL what Next is to React.
  </p>

  <div style(callout)>
    <strong style(calloutStrong)>This site is a deck app.</strong> Every page you read is a file-routed
    deck route, server-rendered and hydrated. The docs are deck’s own canonical
    example — and its release test.
  </div>

  <h2>The 30-second app</h2>
  <p>A deck app is a <code>routes/</code> folder and a dev server. No config file, no scaffold step.</p>
  <code-block code={quickstart} lang="text"></code-block>
  <p>That’s a running SAIL single-page app, with HMR, on <code>127.0.0.1:7878</code>.</p>

  <h2>Where it sits</h2>
  <code-block code={layers} lang="text"></code-block>
  <p>
    SAIL is the component language — signals, templates, controllers. deck is the
    framework around it — routing, rendering, build, serving. Your app is what
    you write under <code>routes/</code>.
  </p>

  <row justify="end" style(footNav)>
    <a variant="solid" href="/getting-started">Getting started →</a>
  </row>
</article>
