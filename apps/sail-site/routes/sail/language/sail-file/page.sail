<!--
  /language/sail-file — THE LANGUAGE, chapter one: The .sail file.
  Frame (ruled): file walk with the data-flow through-line (props in →
  signals between → events out). The walked file is the intro's real
  <demo-player>, shown in SLICES, never dumped. Restraint: styling, blocks
  depth, signals depth, data, motion each get one line + a link, no more.
  Truth-map: setup-return rule + boundary-redirects experience-verified;
  component-vs-page distinction taught explicitly (confusion risk, #89);
  slots kept brief (depth under-tested — audit item); ctx phrasing chosen to
  survive #110 either way.
-->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="4">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">The language</p>
  <h1>The .sail file</h1>

  <p sx="text-lg text-muted-foreground leading-relaxed">
    One <code>.sail</code> file is one component, and a component is a tag.
    There is no wrapper class, no registration step, no separate stylesheet
    or test-id ceremony — the file is the whole thing. This chapter walks one
    real file, the music player from the <a href="/sail/overview">Introduction</a>,
    part by part.
  </p>

  <h2>The shape</h2>
  <p>
    A <code>.sail</code> file has up to three parts, each with one job. The
    template is the markup. <code>&lt;scope&gt;</code> declares the
    component's contract. <code>&lt;controller&gt;</code> holds its behavior.
    Only the template is required — a static component is just markup.
  </p>
  <code-block code={code.shape}></code-block>

  <h2>The template</h2>
  <p>
    Markup that reads like HTML, because it mostly is. Braces are
    expressions: <code>&#123;position()&#125;</code> reads a value, and when that
    value changes, only that binding updates — the template reads state, and
    nothing in it ever computes state. Flow is spelled with blocks —
    <code>&#123;#if&#125;</code> here swaps the equalizer for idle bars; lists use
    <code>&#123;#for&#125;</code> with a required <code>key</code>. Literal braces
    in text are escaped as strings.
  </p>
  <code-block code={code.template}></code-block>
  <p sx="text-sm text-muted-foreground">
    Blocks, keying, and expression rules in depth: <a href="/sail/templates">Templates</a>.
    The <code>sx</code> attributes are the styling system — one line here,
    a whole chapter at <a href="/sail/styling">Styling</a>.
  </p>

  <h2>The scope</h2>
  <p>
    The contract. Props are declared with defaults, and the declaration is
    what the compiler types the component with — a consumer passing a prop
    that isn't declared, or the wrong shape, is a compile error, not a
    runtime surprise. This is the language's standing rule: the surface is
    declared, never discovered.
  </p>
  <code-block code={code.scope}></code-block>

  <h2>The controller</h2>
  <p>
    TypeScript that runs when the component mounts. Props arrive on the
    setup's input, typed from the scope; signals hold local state; and the
    one rule that governs everything: <strong>what setup returns is the
    template's entire vocabulary.</strong> A name the template uses that
    setup didn't return is a compile error that lists what is in scope.
    Lifecycle is ambient — an <code>effect</code> cleans up by returning a
    function, and the component cleans up on unmount without a hook to
    remember.
  </p>
  <code-block code={code.controller}></code-block>
  <p>
    Read the effect closely and you have the whole reactive model: it tracks
    <code>ctx.playing()</code>, so pausing re-runs it, and its cleanup stops
    the clock. Signals in depth: <a href="/sail/language/state">State</a>.
  </p>

  <h2>The flow</h2>
  <p>
    Props in, signals between, events out. The Introduction drives this exact
    component with one signal — and the wiring is one attribute:
  </p>
  <code-block code={code.usage}></code-block>
  <p>
    Events go the other way with <code>on:</code> — a handler writes a
    signal, the template reads it, and the loop closes. That loop is the
    entire mental model.
  </p>

  <h2>Children</h2>
  <p>
    A component receives content the platform way: <code>&lt;slot&gt;</code>.
  </p>
  <code-block code={code.slot}></code-block>

  <h2>The boundaries</h2>
  <p>
    Each part refuses what isn't its job: the template can only speak names
    the scope and setup expose; the controller has unrestricted logic but no
    styling access; styling speaks tokens. When you cross a line, the error
    names the correct form — the boundaries are how five hundred files stay
    one system.
  </p>

  <p sx="text-sm text-muted-foreground">
    A page (<code>routes/…/page.sail</code>) is this same file — routing and
    mounting are <a href="/deck">deck</a>'s job.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="solid" tone="primary" href="/sail/language/state">State →</a>
    <a variant="default" href="/sail/templates">Templates</a>
    <a variant="default" href="/sail/styling">Styling</a>
  </row>
</column>
