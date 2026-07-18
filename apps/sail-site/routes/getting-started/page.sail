<!--
  /getting-started — the real workflow, end to end: toolchain, first app,
  the edit loop with the compiler as feedback, state, routes, production
  build, and the standalone component path. Honest about what Sail is — a
  Rust toolchain plus JS packages — not a fake one-liner.
-->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Introduction</p>
  <h1>Getting started</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    Your first page is a folder and a dev server away. This walks the whole
    path — install once, make a page, feel the edit loop, add state and
    routes, ship it.
  </p>

  <h2>1 · Install (once)</h2>
  <code-block code={code.toolchain}></code-block>
  <p>
    Sail's compiler and dev server are one Rust binary; the runtime and
    tokens are JS packages. The install story is decided and landing: a
    single downloadable binary and an npm wrapper that fetches it — at which
    point this section becomes one command and the Rust requirement
    disappears. Until then, the source build above takes a few minutes, once,
    and <code>deck</code> is an ordinary command on your path.
  </p>

  <h2>2 · Start an app</h2>
  <p>
    A deck app is a <code>routes/</code> folder and a dev server — no config
    file, no scaffold step.
  </p>
  <code-block code={code.app}></code-block>
  <p>The first page is markup. Nothing is imported; the components are part of the language.</p>
  <code-block code={code.firstPage}></code-block>

  <h2>3 · The edit loop</h2>
  <p>
    Save and the browser reloads with state preserved. The part that changes
    how you work: <strong>the compiler answers back.</strong> Write a color
    that is not on the design system, a motion preset that does not exist, a
    prop off the component's type — and the build fails with the vocabulary
    of the fix, at the line that caused it:
  </p>
  <code-block code={code.feedback}></code-block>
  <p>
    This is deliberate. There is no class soup to drift into and no silent
    no-op to ship — the system either accepts the page or tells you exactly
    what it would take. Reviews stop being about spacing.
  </p>

  <h2>4 · Add state</h2>
  <p>
    State lives in a <code>&lt;controller&gt;</code> block in the same file.
    The template can bind only what the controller's setup returns — the
    compiler enforces the boundary.
  </p>
  <code-block code={code.state}></code-block>
  <p>
    Signals are the whole model: <code>signal()</code> holds a value,
    <code>derived()</code> computes from others, <code>effect()</code> runs
    side effects and cleans up on unmount. <a href="/reactivity">State →</a>
  </p>

  <h2>5 · Add routes</h2>
  <p>Directories are the router. A folder is a segment; brackets are params.</p>
  <code-block code={code.routesTree}></code-block>

  <h2>6 · Ship it</h2>
  <code-block code={code.build}></code-block>
  <p>
    Server rendering, hydration, and code-splitting per route come with the
    build — they are deck's job, not configuration. <a href="/deck">Deck →</a>
  </p>

  <h2>Without deck: components in an existing stack</h2>
  <p>
    The compiler's output is a standard custom element, so Sail components can
    live inside any page — a React app, a CMS template, plain HTML. You lose
    the app half (routing, SSR, the dev loop) and keep the language.
  </p>
  <code-block code={code.install}></code-block>
  <code-block code={code.component}></code-block>
  <code-block code={code.compile}></code-block>
  <code-block code={code.use}></code-block>
  <p>
    Interactivity works the same way in both worlds — one file, template and
    controller together:
  </p>
  <code-block code={code.interactive}></code-block>

  <p sx="text-sm text-muted-foreground">
    Working with an agent? Point it at the generated authoring skill
    (<code>.claude/skills/sail-authoring</code>) — the language's rules in a
    form a model follows. The compiler corrects the rest.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="default" href="/overview">← Introduction</a>
    <a variant="solid" tone="primary" href="/reactivity">State →</a>
  </row>
</column>
