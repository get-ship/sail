<!--
  /docs/guide/introduction — Introduction. Signals, Style, query, components, theme.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-overview-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
    const playing = signal(true)
    return { code, playing }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style intro = { box: maxW(breakpoint-tablet) }
  style eyebrow = { paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style colGrow = { box: grow(1).basis(breakpoint-mobile).minW(0) }
  style demoPane = { box: grow(1).basis(64).minW(0).pad(4), paint: bg(muted).borderWidth(1).border(border).radius(lg) }
  style pathLabel = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style ctaRow = { box: mt(4).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<column gap="6" style(page)>
  <column gap="3" style(intro)>
    <p style(eyebrow)>Introduction</p>
    <h1>Sail</h1>
    <p style(lead)>
      Sail is a compiled language that includes everything you need to build
      beautiful, consistent, and performant websites. It is optimized for AI
      agents: the language provides the APIs an interface needs — signals for
      state, <code>style&#40;&#123; box, paint, text, animate &#125;&#41;</code> for styling,
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
      <column gap="2" style(colGrow)>
        <code-block code={code.hello}></code-block>
      </column>
      <column gap="2" justify="center" style(demoPane)>
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

  <column gap="3" style(intro)>
    <h2>The compiler pushes back</h2>
    <row gap="6" align="start">
      <column gap="3" style(colGrow)>
        <p>
          The compiler checks more than syntax: a colour that isn’t a token, a
          stylesheet beside a component, an animation outside the motion
          system — these don’t compile, and every refusal names the correct
          form. How strict is a per-directory contract: the config on the
          right is what these docs pages compile under. Grants reach beyond
          styling — our code-editor declares <code>"unconfined": true</code>
          to embed CodeMirror; the canvas component may fetch from
          <code>"network": ["self"]</code> and nowhere else. The full
          surface today is seven keys — <code>preset</code>,
          <code>sx</code>, <code>variants</code>,
          <code>raw_containers</code>, <code>stylesheets</code>,
          <code>network</code>, <code>unconfined</code> — each a
          per-directory grant, and an unrecognized key refuses loudly. What
          isn’t granted doesn’t compile.
        </p>
      </column>
      <column gap="2" style(colGrow)>
        <p style(pathLabel)>apps/sail-site/routes/sail.config.json</p>
        <sail-json-view data={code.configJson} expanded depth={2}></sail-json-view>
      </column>
    </row>

    <h2>What’s in the box</h2>
    <p>
      <a href="/components">Interface patterns</a> — dialogs, comboboxes,
      tables, tabs, toasts — are tags, keyboard-complete and accessible,
      skinned by your <a href="/docs/concepts/theming">theme</a>.
      <a href="/docs/concepts/layout">Layout</a> is its own vocabulary
      (<code>row</code>, <code>column</code>, <code>grid</code>) rather than a
      CSS technique. <a href="/docs/concepts/animation">Animation</a> is declarative and respects
      reduced motion by default. <a href="/docs/concepts/data">Data</a> has a first-class
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
      as their own runtime on top of the framework’s.
    </p>

    <p style(mutedSm)>
      Sail is in alpha. It builds real websites today — this site is one —
      and the language is still settling.
    </p>

    <row gap="2" style(ctaRow)>
      <a variant="solid" tone="primary" href="/docs/guide/getting-started">Installation →</a>
      <a variant="default" href="/playground">Playground</a>
      <a variant="default" href="/showcase">Showcase</a>
      <a variant="default" href="/docs/guide/why">Why Sail</a>
    </row>
  </column>
</column>
