<!--
  /sail/guide/making-a-theme — the task: copy the closest built-in theme,
  change values, apply it. Everything here is checked against
  packages/sail/tokens (themes.css, themes.ts, products.css). The one thing
  that is NOT possible today — registering a theme from outside the package —
  is stated plainly rather than papered over.
-->

<controller>
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
  return { active, pick, themes: () => THEMES, code }
})
</controller>

<column gap="5">
  style({ box: maxW(breakpoint-tablet).mx(auto) })

  <p>
    style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })
Guide</p>
  <h1>Making a theme</h1>
  <p>
    style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })

    Nine themes ship with the framework. Making a tenth is a copy-and-edit
    job: take the block of token values closest to what you want, rename it,
    change the numbers, and register it so the framework will apply it. This
    guide does exactly that, and is honest about where the seam runs out.
  </p>

  <h2>What a theme actually is</h2>
  <p>
    A <strong>token</strong> is a named value — <code>--background</code>,
    <code>--primary</code>, <code>--radius</code>,
    <code>--duration-fast</code>. Components never write a color or a
    corner radius; they read a token. A <strong>theme</strong> is one set of
    values for those names.
  </p>
  <p>
    Concretely, a theme is two things in
    <code>packages/sail/tokens</code>, and nothing else:
  </p>
  <ol>
    style({ box: padL(6), paint: list(decimal) })

    <li>
      a block in <code>src/themes.css</code> that assigns the tokens, keyed
      on <code>&#91;theme='…'&#93;</code>;
    </li>
    <li>
      an entry in the <code>THEMES</code> array in <code>src/themes.ts</code>,
      carrying its id, label, whether it is dark, and four preview swatches.
    </li>
  </ol>
  <p>
    That is the whole model. <code>src/tokens.css</code> declares every token
    once, with the default palette; a theme block overrides the ones it wants
    and inherits the rest, so a theme file only carries what makes it
    different.
  </p>

  <h2>Step 1 — pick the closest built-in</h2>
  <p>
    Start from the theme nearest your intent, not from an empty block. Switch
    between them here to find it — every button repaints this page:
  </p>
  <row gap="2">
    <for {themes()} as="t" key={t.id}>
      <button
        on:click={() => pick(t.id)}
        variant={active() === t.id ? 'solid' : 'outline'}
        tone={active() === t.id ? 'primary' : 'default'}
        pressed={active() === t.id ? 'true' : 'false'}
      >{t.label}</button>
    </for>
  </row>
  <p>
    Say the target is a warmer, squarer version of Paper. Here is Paper's
    block, abridged — the real one runs about ninety lines and continues
    through shadows, sidebar chrome, typography and interaction tints:
  </p>
  <code-block code={code.paperBlock} lang="css"></code-block>
  <p>
    Two details in there are worth understanding before you copy it. The
    selector is doubled: <code>:root[theme='paper']</code> repaints the
    document, and <code>&#91;theme='paper'&#93;</code> repaints one
    subtree — that is how a swatch shows a theme without switching the page.
    And the brand tokens are written
    <code>oklch(L C var(--brand-hue, 30))</code>: lightness and chroma are
    the theme's, while the hue can be moved from outside. More on that at the
    end.
  </p>

  <h2>Step 2 — copy the block, change the values</h2>
  <p>
    Paste it back into <code>src/themes.css</code> under a new id and edit.
    Colors are <a href="https://oklch.com" rel="noreferrer">OKLCH</a> —
    lightness, chroma, hue — which is why nudging a palette warmer is a hue
    change rather than a rewrite.
  </p>
  <code-block code={code.mineBlock} lang="css"></code-block>
  <p>
    Change values, not names. An invented token name is dead — nothing reads
    it — and a token you delete falls back to the default from
    <code>tokens.css</code>, which is usually not what you meant.
  </p>

  <h2>Step 3 — register it</h2>
  <p>
    A CSS block alone is inert: the framework will not apply an id it does
    not know. Add the id to the <code>ThemeId</code> union and an entry to
    <code>THEMES</code>, both in <code>src/themes.ts</code>.
  </p>
  <code-block code={code.themeType} lang="ts"></code-block>
  <p>
    The <code>swatches</code> are the four colors the theme picker shows in
    its preview tile: background, card, primary, accent. They are display
    only — the values that matter are the ones in the CSS block.
  </p>
  <code-block code={code.themeEntry} lang="ts"></code-block>
  <p>
    <code>dark: true</code> is not cosmetic. It decides whether the
    <code>data-mode</code> attribute goes on the document, which a number of
    component skins key on. Set it to match the theme you actually built.
  </p>
  <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })

    The two files are kept in step by the token package's own tests: for
    every id in <code>THEMES</code>, one asserts a matching
    <code>:root[theme]</code> block exists in
    <code>themes.css</code>. Edit one file and forget the other and the suite
    says so.
  </p>

  <h2>Step 4 — apply it</h2>
  <p>
    <code>applyTheme</code> is the only writer of theme state.
    <code>persistTheme</code> is separate, and remembers the choice.
  </p>
  <code-block code={code.apply} lang="ts"></code-block>
  <p>Its whole implementation:</p>
  <code-block code={code.applySource} lang="ts"></code-block>
  <p>
    Three things happen. <code>theme</code> goes on the document, which
    activates your CSS block. The <code>data-mode</code> attribute is set from
    the entry's <code>dark</code> flag. A <code>ship:theme:change</code>
    event is dispatched, for chrome that needs to react beyond CSS.
  </p>
  <p>
    Note the first line: an id that is not in <code>THEMES</code> is not an
    error — it silently becomes <code>ship</code>. If a new theme
    applies as the default, the missing piece is the <code>THEMES</code>
    entry, not the CSS.
  </p>
  <p>
    To restore the reader's choice on boot, read it back and apply it:
    <code>applyTheme(readStoredTheme())</code>. And the entire switching UI —
    trigger, previews, keyboard, persistence — is one tag,
    <code>&lt;theme-picker&gt;</code>, the control in this site's top bar.
  </p>

  <h2>The limit, stated plainly</h2>
  <alert tone="warning">
    A theme cannot be registered from your own project today: there is no
    registration API and no config key, so steps 2 and 3 mean editing
    <code>@ship/tokens</code> itself.
  </alert>
  <p>
    If editing the package is not available to you, the one path that works
    is to ship your own stylesheet with a
    <code>:root[theme="mine"]</code> block and set
    <code>document.documentElement.setAttribute('theme', 'mine')</code> yourself, never
    calling <code>applyTheme</code>. You then also own the <code>data-mode</code>
    class, the change event and persistence, and your theme will not appear
    in the picker. Deck's own palette is built this way, so the approach is
    proven — but it is a bypass, not a seam.
  </p>

  <h2>One more axis: product hue</h2>
  <p>
    Before writing a whole theme, check whether you only need a different
    hue. The brand tokens read <code>var(--brand-hue, …)</code>, and a second,
    much smaller file sets that one variable:
  </p>
  <code-block code={code.products} lang="css"></code-block>
  <p>
    A theme owns lightness and chroma — its character; a product owns hue —
    its brand. They compose: with no product set the theme looks exactly as
    it always did, and with one set only the hue moves. It is why the Deck
    pages on this site run violet while keeping whichever theme you picked
    above. Five lines of CSS, and only <code>--primary</code>,
    <code>--accent</code>, <code>--ring</code> and
    <code>--sidebar-primary</code> follow it; surfaces and ink stay put.
  </p>

  <row justify="between" gap="2">
    style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })

    <a variant="default" href="/sail/guide/forms">← Forms</a>
    <a variant="solid" tone="primary" href="/sail/guide/embedding-third-party-code">Embedding third-party code →</a>
  </row>
</column>
