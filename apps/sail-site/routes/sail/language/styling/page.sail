<!--
  /sail/language/styling — chapter 4. One way to style, and no way out.

  Claims are checked against the tables: the prefix sets are STATE_PREFIXES /
  CONDITION_PREFIXES in sail-lint's sx rule (the diagnostics render FROM those
  tables), both quoted errors are captured compiler output, the (--x)
  reflection is register-properties.ts, variants is the scope_block grammar.
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
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">The language</p>
  <h1>Styling</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    <code>sx</code> is a closed vocabulary of design-system atoms. Not a
    shorthand for CSS — a <em>subset</em>, on purpose. Every atom resolves to a
    design token, which is what makes the theme switch in the header work:
    swap the theme and every component in the app repaints, because no
    component ever held a raw value to begin with.
  </p>

  <code-block code={code.sx}></code-block>
  <p>
    The full vocabulary lives on the <a href="/sail/reference">reference
    page</a>, rendered from the same table the compiler enforces — the docs
    and the errors cannot disagree.
  </p>

  <h2>When it refuses, it redirects</h2>
  <p>
    A value off the system is a compile error, and the error names what the
    scale actually holds:
  </p>
  <code-block code={code.sxError}></code-block>

  <h2>States are a prefix</h2>
  <p>
    <code>hover:</code>, <code>focus:</code>, <code>focus-within:</code>,
    <code>active:</code>, <code>disabled:</code> — a state is a prefix on an
    atom, not a rule in a stylesheet somewhere else.
    <code>focus:</code> lowers to <code>:focus-visible</code>, so keyboard
    users get the ring and mouse clicks don't.
  </p>
  <code-block code={code.states}></code-block>
  <p>
    The rest of the state set keys on ARIA attributes:
    <code>current:</code>, <code>selected:</code>, <code>expanded:</code>,
    <code>checked:</code>, <code>pressed:</code>, <code>busy:</code>. The
    style is only reachable by stamping the semantics — the attribute you set
    for assistive tech is the style hook, one source of truth for both. This
    site's product switcher is built on exactly this — the shape of it:
  </p>
  <code-block code={code.current}></code-block>

  <h2>Conditions are a prefix too</h2>
  <p>
    Responsive and preference styling use the same shape:
    <code>mobile:</code>, <code>tablet:</code>, <code>desktop:</code>
    (mobile-first <code>min-width</code>, and the widths are the breakpoint
    tokens), plus <code>dark:</code>, <code>light:</code>,
    <code>motion-reduce:</code>, <code>motion-safe:</code>, and
    <code>forced-colors:</code>. The set is closed — there is no arbitrary
    <code>@media</code>, because that would be raw CSS with extra steps.
  </p>
  <code-block code={code.conditions}></code-block>
  <p>An unknown prefix gets the same treatment as an unknown atom:</p>
  <code-block code={code.prefixError}></code-block>

  <h2>Tones, not colors</h2>
  <p>
    You do not pick a red. You declare a <strong>tone</strong>, and the
    framework resolves <code>--tone-fg</code>, <code>--tone-bg</code>,
    <code>--tone-border</code> and <code>--tone-ring</code> for the subtree.
    That is why a destructive button in one app matches a destructive alert in
    another — and why <a href="/sail/theming">swapping the theme</a> repaints
    both.
  </p>
  <code-block code={code.tones}></code-block>

  <h2>Variants</h2>
  <p>
    When a component has named looks, the looks are data, not code:
    a <code>variants:</code> table in <code>&lt;scope&gt;</code>. Each axis
    becomes a typed prop on the component — the first key is the default —
    and each value is ordinary <code>sx</code>, keyed off the attribute.
  </p>
  <code-block code={code.variants}></code-block>
  <p>Callers pick a look the same way they set any prop:</p>
  <code-block code={code.variantUse}></code-block>

  <h2>Dynamic values</h2>
  <p>
    Atoms are static by design, so computed style has exactly one channel:
    a signal. Read a custom property in <code>sx</code> with
    <code>(--name)</code>; when the name matches a signal the controller
    returned, the compiler registers the property and reflects the signal's
    value onto the element. State stays in the controller, and the style
    layer just reads it — the <a href="/sail/motion">motion chapter</a> drives
    entire choreographies through this.
  </p>
  <code-block code={code.dynamic}></code-block>

  <h2>Structure is not styling</h2>
  <p>
    <code>sx</code> has no <code>flex-direction</code>, no
    <code>align-items</code>, no <code>grid-template-columns</code> — and that
    is not a gap. Structure belongs to the layout elements, where the axis is
    the name and there is nothing to get backwards. The
    <a href="/sail/language/layout">next chapter</a> is exactly that.
  </p>
  <code-block code={code.layout}></code-block>

  <h2>There is no escape hatch</h2>
  <p>
    This is the part that matters. <strong>Authored CSS is a compile
    error.</strong> Not discouraged, not linted with a warning — an error, in
    every SAIL file. The only exception is the stdlib elements inside the
    compiler, because they <em>are</em> the styling layer.
  </p>
  <p>
    An escape hatch is not a safety valve; it is where the design system goes
    to die. One component reaches for a raw rule, then ten do, and now the
    "system" is a suggestion.
  </p>
  <alert tone="warning">
    <strong>If you cannot express something, that is a conversation — not a
    hatch.</strong> Most of the time the answer already exists: the element
    (<code>&lt;navbar&gt;</code>), the contract (<code>tone</code>), or the
    rule (<code>code</code>). When it genuinely is a gap — per-side borders
    were one — the language grows a token, not a hole.
  </alert>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="default" href="/sail/language/templates">← Templates</a>
    <a variant="solid" tone="primary" href="/sail/language/layout">Layout →</a>
  </row>
</column>
