<!--
  /overview — the Introduction (pairing draft v3).
  v3 rulings applied: query added to the opener's API list; hover feedback is
  CSS (sx hover: + transition atoms), NOT motion: — motion:hover ran a full
  WAAPI pass on enter AND leave (jitter on mouse sweep); prose and demos sit
  side by side; the bundle widget is GONE (bundle story ≠ compiler tunability
  — separate artifact, built when we're ready to compare honestly); tuning
  section is prose about the real policy settings.
-->

<controller>
import { controller, signal, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
    const playing = signal(true)
    return { code, playing }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="6">
  <column sx="max-w-breakpoint-tablet" gap="3">
    <p sx="font-mono text-xs uppercase tracking-wide text-accent">Introduction</p>
    <h1>Sail</h1>
    <p sx="text-lg text-muted-foreground leading-relaxed">
      Sail is a compiled language that includes everything you need to build
      beautiful, consistent, and performant websites. It is optimized for AI
      agents: the language provides the APIs an interface needs — signals for
      state, <code>sx</code> for styling, <code>motion</code> for animation,
      <code>query</code> for data, a component library that already behaves —
      and a strict compiler enforces how they are used. That is how best
      practices and consistency hold across an entire application, not just
      the files someone reviewed.
    </p>
  </column>

  <column gap="3">
    <h2>It reads like HTML, it runs on signals</h2>
    <p>
      A component is one file: a template that reads like HTML, TypeScript
      beneath it, signals connecting the two. Updates are fine-grained — the
      template binds signals directly, and only what changed re-renders. No
      virtual DOM, no diffing. One signal, threaded through a nested
      component — and the same lines running:
    </p>
    <row gap="6" align="start">
      <column gap="2" sx="grow-1 basis-breakpoint-mobile min-w-0">
        <code-block code={code.hello}></code-block>
      </column>
      <column gap="2" sx="grow-1 basis-64 min-w-0 p-4 bg-muted border border-border rounded-lg" justify="center">
        <demo-player playing={playing()}></demo-player>
        <row gap="2" align="center">
          <switch bind:checked={playing}></switch>
          <badge>{playing() ? 'playing' : 'paused'}</badge>
        </row>
      </column>
    </row>
  </column>

  <column gap="3">
    <h2>Design is data</h2>
    <p>
      Styling draws from a token system the compiler enforces — colors,
      spacing, and type resolve through the active theme, and components never
      name a colour. Which means a theme is data, and swapping it re-skins
      everything written in the language. Pick one — this entire site is
      written in Sail:
    </p>
    <theme-picker inline></theme-picker>
  </column>

  <column sx="max-w-breakpoint-tablet" gap="3">
    <h2>The compiler pushes back</h2>
    <row gap="6" align="start">
      <column gap="3" sx="grow-1 basis-breakpoint-mobile min-w-0">
        <p>
          The compiler checks more than syntax: a colour that isn't a token, a
          stylesheet beside a component, an animation outside the motion
          system — these don't compile, and every refusal names the correct
          form. How strict is a per-directory contract: the config on the
          right is what these docs pages compile under. Grants reach beyond
          styling — our code-editor declares <code>"unconfined": true</code>
          to embed CodeMirror; the canvas component may fetch from
          <code>"network": ["self"]</code> and nowhere else. What isn't
          granted doesn't compile.
        </p>
      </column>
      <column gap="2" sx="grow-1 basis-breakpoint-mobile min-w-0">
        <p sx="font-mono text-xs text-muted-foreground">apps/sail-site/routes/sail.config.json</p>
        <sail-json-view data={code.configJson} expanded depth={2}></sail-json-view>
      </column>
    </row>

    <h2>What's in the box</h2>
    <p>
      <a href="/components">Interface patterns</a> — dialogs, comboboxes,
      tables, tabs, toasts — are tags, keyboard-complete and accessible,
      skinned by your <a href="/sail/theming">theme</a>.
      <a href="/sail/language/layout">Layout</a> is its own vocabulary
      (<code>row</code>, <code>column</code>, <code>grid</code>) rather than a
      CSS technique. <a href="/sail/motion">Motion</a> is declarative and respects
      reduced motion by default. <a href="/sail/data">Data</a> has a first-class
      shape: <code>query()</code> turns a client call into loading, error,
      and value states the template has to handle. Forms, icons, and a
      typographic system for prose round it out. The practical difference:
      there is no stack to choose — and nothing can drift out of step with
      your design system, because everything reads the same tokens.
    </p>

    <h2>Performance</h2>
    <p>
      Templates compile ahead of time and updates are fine-grained signal
      writes — no virtual DOM, nothing to diff. The runtime is small and
      tree-shakable, and the design system ships as CSS: components render
      and style without JavaScript, with behavior arriving as progressive
      enhancement over native elements. That matters most compared to the
      usual stack, where the component library and the design system arrive
      as their own runtime on top of the framework's.
    </p>

    <p sx="text-sm text-muted-foreground">
      Sail is in alpha. It builds real websites today — this site is one —
      and the language is still settling.
    </p>

    <row sx="mt-4 pt-4 border-t border-t-border" gap="2">
      <a variant="solid" tone="primary" href="/sail/getting-started">Installation →</a>
      <a variant="default" href="/sail/tutorial">Tutorial</a>
      <a variant="default" href="/sail/showcase">Showcase</a>
      <a variant="default" href="/sail/why">Why Sail</a>
    </row>
  </column>
</column>
