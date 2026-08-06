<!--
  /deck — Deck overview. The shell sets theme="deck" (violet); this page
  is pure content. Adapted from packages/deck/docs/routes/page.sail to the
  sail-site's .doc prose convention (eyebrow / h1 / lead / h2).
-->

<controller>
// /deck — the Deck overview. The shell owns the deck (violet) theme via
// data-theme; this controller only supplies the page's code snippets.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { quickstart, layers } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ quickstart, layers }))
</controller>

<article>
  style.maxW(breakpoint-tablet).mx(auto)

  <p>
    style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)
deck · 0.2.0</p>
  <h1>The standalone web framework for Sail.</h1>
  <p>
    style.fontSize(lg).fg(muted-foreground).leading(relaxed)

    deck mounts Sail routes, routes them from the filesystem, server-renders or
    statically builds them, and serves every asset — with a dev server and
    state-preserving HMR. It is to Sail what Next is to React.
  </p>

  <alert tone="info">
    <strong>This site is a deck app.</strong> Every page you read is a file-routed
    deck route, server-rendered and hydrated. The docs are deck's own canonical
    example — and its release test.
  </alert>

  <h2>The 30-second app</h2>
  <p>A deck app is a <code>routes/</code> folder and a dev server. No config file, no scaffold step.</p>
  <pre>
    style.fontFamily(mono).fontSize(sm).leading(relaxed).bg(surface-1).borderWidth(1).border(border).radius(lg).pad(4).padX(5).whitespace(pre-wrap).break(words)
{quickstart}</pre>
  <p>That's a running Sail single-page app, with HMR, on <code>127.0.0.1:7878</code>.</p>

  <h2>Where it sits</h2>
  <pre>
    style.fontFamily(mono).fontSize(sm).leading(relaxed).bg(surface-1).borderWidth(1).border(border).radius(lg).pad(4).padX(5).whitespace(pre-wrap).break(words)
{layers}</pre>
  <p>
    Sail is the component language — signals, templates, controllers. deck is the
    framework around it — routing, rendering, build, serving. Your app is what
    you write under <code>routes/</code>.
  </p>

  <row>
    style.mt(8).padT(4).borderTWidth(1).borderT(border)

    <a variant="solid" tone="primary" href="/deck/getting-started">Getting started →</a>
  </row>
</article>
