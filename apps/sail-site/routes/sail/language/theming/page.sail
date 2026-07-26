<!--
  /theming — themes are the argument made visible: components read tokens,
  never colors, so one attribute repaints the world. The page is its own
  demo — every switch below restyles the page you are reading.
-->

<controller>
// /theming — the live theme switchboard. `pick` is the same two calls the
// top-bar <theme-picker> makes: applyTheme (data-theme + the canonical .dark
// class) and persistTheme (survives reload). The page IS the preview — every
// component on it reads tokens, so the pick repaints the reader's world.

import type { ControllerScope } from '@ship/sail-runtime'
import { applyTheme, persistTheme, readStoredTheme, THEMES, type ThemeId } from '@ship/tokens/themes'
import type { Scope } from './page.sail'
import { code } from './snippets'

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

<column sx={style.maxW(breakpoint-tablet).mx(auto)}>
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>Concepts</p>
  <h1>Theming</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>
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
    {#for t of themes() key={t.id}}
      <button
        on:click={() => pick(t.id)}
        variant={active() === t.id ? 'solid' : 'outline'}
        tone={active() === t.id ? 'primary' : 'default'}
        aria-pressed={active() === t.id ? 'true' : 'false'}
      >{t.label}</button>
    {/for}
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
    Components read tokens, themes define them, and neither knows the other's
    name.
  </p>

  <h2>The token ladder</h2>
  <p>
    Tokens stack in four layers, highest abstraction first. Read the highest
    layer that captures intent; reach for primitives only when no semantic
    token fits.
  </p>
  <ol sx={style.list(decimal).padL(6)}>
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
      (<code>style.bg(destructive)</code>, <code>style.colorMix(primary, 15)</code>).
      The L1 <code>tone</code> prop selects which hue a component paints —
      it is not a CSS channel and there is no <code>tone-*-bg/fg</code> ramp.
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
    and nothing in it can conflict with a component's own styles.
    <a href="/sail/guide/making-a-theme">Making a theme</a> walks the exact
    mechanics, start to finish.
  </p>
  <p>
    Token files also layer. The product accents on this site — the shift to
    violet on the Deck pages — are a second, smaller token file applied on
    top of the theme: the theme defines the surfaces, the accent redefines a
    few brand tokens, and every component keeps reading the same names.
  </p>

  <p sx={style.mt(4).fontSize(sm).fg(muted-foreground)}>
    For the full surface: <a href="/sail/reference/config">reference → Config</a>.
  </p>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)} justify="between" gap="2">
    <a variant="default" href="/sail/language/errors">← Errors</a>
    <a variant="solid" tone="primary" href="/sail/reference">Reference →</a>
  </row>
</column>
