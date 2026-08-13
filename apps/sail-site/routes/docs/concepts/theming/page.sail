<!--
  /docs/concepts/theming — themes as token sets; live pick repaints the page.
-->

<scope>
  style page = { box: size({ max-w: breakpoint-tablet }).space({ mx: auto }) }
  style eyebrow = { box: space({ mb: 2 }), paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style list = { box: space({ pad-l: 6 }), text: type({ list: decimal }) }
  style footNote = { box: space({ mt: 4 }), paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style footNav = { box: space({ pad-t: 4, mt: 8 }), paint: border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import { type ThemeId, THEMES } from '../../../../src/compositions/tokens-themes'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-theming-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  const active = signal<string>(readStoredTheme())
  const pick = (id: ThemeId): void => {
    applyTheme(id)
    persistTheme(id)
    active.set(id)
  }
  return {
    active,
    pick,
    themes: () => THEMES,
    code,
  }
})
</controller>

<column style(page)>
  <p style(eyebrow)>Concepts</p>
  <h1>Theming</h1>
  <p style(lead)>
    A theme is a complete aesthetic — palette, surfaces, radii, shadows,
    typography, motion — applied as one attribute. Nine ship with the
    framework. There is no light/dark switch; dark is a property a theme
    has, not a mode you manage.
  </p>

  <h2>Try it here</h2>
  <p>
    Every button below makes the same two calls the theme picker in the top
    bar makes. This page — the code blocks, the sidebar, the buttons
    themselves — is the preview.
  </p>
  <row gap="2">
    <for {themes() as t} key={t.id}>
      <button
        on:click={() => pick(t.id)}
        variant={active() === t.id ? 'solid' : 'outline'}
        tone={active() === t.id ? 'primary' : undefined}
        pressed={active() === t.id ? 'true' : 'false'}
      >{t.label}</button>
    </for>
  </row>
  <code-block code={code.apply}></code-block>
  <p>
    The UI for this is one tag — it is the control in the top-right of every
    page on this site:
  </p>
  <code-block code={code.picker}></code-block>

  <h2>Why the whole page follows</h2>
  <p>
    No component in the framework names a color. A card reads
    <code>--surface-1</code>; a primary button reads <code>--primary</code>;
    a destructive alert reads <code>--destructive</code>. A theme is the set
    of values behind those names, so applying one restyles every component at
    once — including components written long before the theme existed.
    Components read tokens, themes define them, and neither knows the other’s
    name.
  </p>

  <h2>The token ladder</h2>
  <p>
    Tokens stack in four layers, highest abstraction first. Read the highest
    layer that captures intent; reach for primitives only when no semantic
    token fits.
  </p>
  <ol style(list)>
    <li>
      <strong>Semantic surfaces</strong> — <code>--surface-0..3</code>,
      <code>--card</code>, <code>--popover</code>, <code>--backdrop</code>.
      The page ground is <code>--surface-0</code>; a card sits on
      <code>--surface-1</code>.
    </li>
    <li>
      <strong>Semantic hues</strong> —
      <code>--primary</code>, <code>--success</code>, <code>--warning</code>,
      <code>--destructive</code>, <code>--info</code>, <code>--accent</code>,
      <code>--muted</code> (and their <code>-foreground</code> partners).
      Status UI spends these directly via Style
      (<code>style&#40;&#123; paint: fill(&#123; color: destructive &#125;) &#125;&#41;</code>,
      <code>style&#40;&#123; paint: fill(&#123; color: primary, mix: 15 &#125;) &#125;&#41;</code>).
      A color-group typed prop (often named <code>tone</code>, type <code>Tone</code>)
      binds in Style paint (<code>fill(&#123; color: tone &#125;)</code>, <code>ink(&#123; color: tone.foreground &#125;)</code>) —
      type-keyed, not a special channel; rest paint is omit the attribute.
    </li>
    <li>
      <strong>Brand / role paint</strong> — <code>--foreground</code>,
      <code>--background</code>, <code>--border</code>,
      <code>--muted-foreground</code>, component slots
      (<code>--card</code>, <code>--button-*</code>). The handful that define a voice.
    </li>
    <li>
      <strong>Primitives</strong> — the spacing, radius, type, shadow and
      easing scales everything above composes from.
    </li>
  </ol>

  <h2>Where themes come from</h2>
  <p>
    A theme is a <strong>token file</strong> — a data file that assigns a
    value to each token name. The nine built-ins are exactly that, checked
    into <code>@ship/tokens</code>. To make your own, start from the
    built-in closest to what you want, copy its file, and change the values;
    because a theme is data rather than CSS rules, the tooling can check it,
    and nothing in it can conflict with a component’s own styles.
    <a href="/docs/guide/making-a-theme">Making a theme</a> walks the exact
    mechanics, start to finish.
  </p>
  <p>
    Token files also layer. The product accents on this site — the shift to
    violet on the Deck pages — are a second, smaller token file applied on
    top of the theme: the theme defines the surfaces, the accent redefines a
    few brand tokens, and every component keeps reading the same names.
  </p>

  <p style(footNote)>
    For the full surface: <a href="/docs/reference/config">reference → Config</a>.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/concepts/errors">← Errors</a>
    <a variant="solid" tone="primary" href="/docs/reference">Reference →</a>
  </row>
</column>
