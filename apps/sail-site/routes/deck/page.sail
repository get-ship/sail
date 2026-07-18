<!--
  /deck — Deck overview. The shell sets data-theme="deck" (violet); this page
  is pure content. Adapted from packages/deck/docs/routes/page.sail to the
  sail-site's .doc prose convention (eyebrow / h1 / lead / h2).
-->

<controller>
// /deck — the Deck overview. The shell owns the deck (violet) theme via
// data-theme; this controller only supplies the page's code snippets.
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { quickstart, layers } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ quickstart, layers }))
</controller>

<article sx="max-w-breakpoint-tablet mx-auto">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">deck · 0.2.0</p>
  <h1>The standalone web framework for SAIL.</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    deck mounts SAIL views, routes them from the filesystem, server-renders or
    statically builds them, and serves every asset — with a dev server and
    state-preserving HMR. It is to SAIL what Next is to React.
  </p>

  <alert tone="info">
    <strong>This site is a deck app.</strong> Every page you read is a file-routed
    deck route, server-rendered and hydrated. The docs are deck's own canonical
    example — and its release test.
  </alert>

  <h2>The 30-second app</h2>
  <p>A deck app is a <code>routes/</code> folder and a dev server. No config file, no scaffold step.</p>
  <pre sx="font-mono text-sm leading-relaxed bg-surface-1 border border-border rounded-lg p-4 px-5 whitespace-pre-wrap break-words">{quickstart}</pre>
  <p>That's a running SAIL single-page app, with HMR, on <code>127.0.0.1:7878</code>.</p>

  <h2>Where it sits</h2>
  <pre sx="font-mono text-sm leading-relaxed bg-surface-1 border border-border rounded-lg p-4 px-5 whitespace-pre-wrap break-words">{layers}</pre>
  <p>
    SAIL is the component language — signals, templates, controllers. deck is the
    framework around it — routing, rendering, build, serving. Your app is what
    you write under <code>routes/</code>.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border">
    <a variant="solid" tone="primary" href="/deck/getting-started">Getting started →</a>
  </row>
</article>
