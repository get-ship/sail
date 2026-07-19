<!--
  /sail/language/styling — chapter 4, round 2 per owner review: say what it
  IS first (Tailwind-shaped utilities, Rust-compiled, closed), THEN the
  restraint. Every section renders its styles LIVE above the code. One
  refusal shown, not two. ARIA framed as default-not-extra. Tones get a real
  introduction. Dynamic styles framed positively. The disallowed material is
  one section. Reference linked in the first paragraph.
-->

<controller>
import { controller, signal, effect, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  // The dynamic-styles demo: a range input writes `shift` (strings, as
  // form values are); the effect mirrors it into `px`, which the template
  // reads through (--px).
  const shift = signal('0')
  const px = signal('0px')
  effect(() => px.set(`${shift()}px`))
  return { code, shift, px }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">The language</p>
  <h1>Styling</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    Styling is the <code>sx</code> attribute: utility classes, written the
    way Tailwind writes them — <code>p-4</code>, <code>bg-surface-1</code>,
    <code>rounded-lg</code>, <code>hover:bg-muted</code>. The full
    vocabulary is on the <a href="/sail/reference">reference page</a>,
    generated from the same table the compiler enforces. The difference
    from Tailwind is what the vocabulary is made of: a closed set, and
    every atom resolves to a design token.
  </p>
  <p>
    The syntax is deliberately familiar — an agent or a person who knows
    Tailwind already knows how to write <code>sx</code>. It exists as its
    own system for two reasons: the toolchain is Rust with no Node in it,
    and a design system needs values that can be constrained. Closed and
    token-backed is what makes the theme switch in this site's header work
    at all — swap the theme and everything repaints, because nothing ever
    held a raw hex value.
  </p>

  <h2>The atoms</h2>
  <card sx="p-5 bg-surface-1 border border-border rounded-lg shadow-sm">
    <p sx="text-muted-foreground text-sm">This card is styled by the code below it.</p>
  </card>
  <code-block code={code.sx}></code-block>
  <p>
    A value off the system is a compile error, and the error names what the
    scale holds:
  </p>
  <code-block code={code.sxError}></code-block>

  <h2>States</h2>
  <p>
    A state is a prefix on an atom. Hover these:
  </p>
  <row gap="3" sx="p-5 bg-surface-1 border border-border rounded-lg">
    <button sx="hover:bg-muted">hover:bg-muted</button>
    <button sx="hover:text-primary">hover:text-primary</button>
    <button sx="active:bg-primary active:text-primary-foreground">active:…</button>
  </row>
  <code-block code={code.states}></code-block>
  <p>
    <code>hover:</code>, <code>focus:</code>, <code>focus-within:</code>,
    <code>active:</code>, and <code>disabled:</code> cover the pointer and
    keyboard states — <code>focus:</code> lowers to
    <code>:focus-visible</code>, so the ring appears for keyboard users and
    not on mouse clicks. The rest key on ARIA attributes that Sail
    components set by default: <code>current:</code>,
    <code>selected:</code>, <code>expanded:</code>, <code>checked:</code>,
    <code>pressed:</code>. The attribute is already there for assistive
    tech; the prefix just styles off it. This site's product switcher:
  </p>
  <code-block code={code.current}></code-block>

  <h2>Responsive and preference prefixes</h2>
  <p>
    Responsive styling is the same prefix shape. Three breakpoints —
    <code>mobile:</code> (390px), <code>tablet:</code> (768px),
    <code>desktop:</code> (1280px) — apply their atom from that width up,
    so the unprefixed atom is the phone style and prefixes layer on top.
    This panel pads tighter below 768px; resize the window to see it:
  </p>
  <column gap="2" sx="p-3 tablet:p-6 bg-surface-1 border border-border rounded-lg">
    <p sx="text-sm">p-3 on a phone, tablet:p-6 from 768px up.</p>
  </column>
  <code-block code={code.conditions}></code-block>
  <p>
    Preference queries use the same shape — <code>dark:</code>,
    <code>light:</code>, <code>motion-reduce:</code>,
    <code>motion-safe:</code>, <code>forced-colors:</code> — and the set is
    closed: those prefixes are the media queries there are.
  </p>

  <h2>Tones</h2>
  <p>
    A <strong>tone</strong> is a named intent — <code>primary</code>,
    <code>success</code>, <code>warning</code>, <code>destructive</code>,
    <code>muted</code> — that stands in for a color. Declaring
    <code>tone="destructive"</code> on an element resolves a coordinated
    set of tokens for its subtree: a background, a foreground, a border, a
    focus ring, chosen by the theme to work together. Components then read
    those tokens instead of picking colors:
  </p>
  <row gap="2" sx="p-5 bg-surface-1 border border-border rounded-lg" align="center">
    <badge tone="primary">primary</badge>
    <badge tone="success">success</badge>
    <badge tone="warning">warning</badge>
    <badge tone="destructive">destructive</badge>
    <badge tone="muted">muted</badge>
  </row>
  <code-block code={code.tones}></code-block>
  <p>
    This is why a destructive button and a destructive alert match without
    coordinating, and why a theme can re-decide what "destructive" looks
    like for every component at once.
  </p>

  <h2>Variants</h2>
  <p>
    When a component has named looks, the looks are data: a
    <code>variants:</code> table in <code>&lt;scope&gt;</code>. Each axis
    becomes a typed prop — the first key is the default — and each value is
    ordinary <code>sx</code>. Variant tables are a per-directory compiler
    permit (<code>"variants": true</code> in that directory's
    <code>sail.config.json</code>) — component libraries turn them on;
    plain pages usually don't need them.
  </p>
  <code-block code={code.variants}></code-block>
  <p>Callers pick a look the way they set any prop:</p>
  <code-block code={code.variantUse}></code-block>

  <h2>Styles can be dynamic too</h2>
  <p>
    Computed style flows through signals. Read a custom property in
    <code>sx</code> with <code>(--name)</code>; when the name matches a
    signal the controller returned, the compiler registers the property and
    reflects the signal's value onto the element. Drag:
  </p>
  <column gap="3" sx="p-5 bg-surface-1 border border-border rounded-lg">
    <input type="range" min="0" max="200" step="1" bind:value={shift} sx="min-w-0" />
    <card sx="w-9 h-9 bg-primary rounded-md translate-x-(--px)"></card>
  </column>
  <code-block code={code.dynamic}></code-block>
  <p>
    State stays in the controller and the style layer reads it — the
    <a href="/sail/language/motion">motion chapter</a> drives whole
    choreographies through this one channel.
  </p>

  <h2>What sx does not do</h2>
  <p>
    Two things are outside the vocabulary on purpose. <strong>Layout</strong>
    — there is no <code>flex-direction</code> or
    <code>grid-template-columns</code>; the parent decides how children sit,
    with the <a href="/sail/language/layout">layout elements</a>. And
    <strong>authored CSS</strong> — <code>&lt;style&gt;</code> blocks and
    <code>style=</code> attributes are compile errors in application code,
    because one raw rule becomes ten, and then the system is a suggestion.
    When the vocabulary genuinely can't express something you need, that is
    a gap to file — the system grows a token, not a hole; per-side borders
    were added exactly that way.
  </p>

  <p sx="mt-4 text-sm text-muted-foreground">
    For the full surface: <a href="/sail/reference/sx">reference → sx</a>.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" justify="between" gap="2">
    <a variant="default" href="/sail/language/templates">← Templates</a>
    <a variant="solid" tone="primary" href="/sail/language/layout">Layout →</a>
  </row>
</column>
