<!--
  Lesson 07 solved — the theme is one attribute.

  pick() calls applyTheme + persistTheme; everything on the page repaints
  because every surface reads tokens. The two panels below opt OUT: each
  pins its own token table with theme, which is how a picker
  can show you a theme without applying it.
-->

<controller>
import {
  THEMES,
  applyTheme,
  persistTheme,
  readStoredTheme,
  type Theme,
  type ThemeId,
} from '@ship/tokens/themes'

export default controller(() => {
    const active = signal<ThemeId>(readStoredTheme())

    return {
      themes: (): Theme[] => THEMES,
      active,
      // applyTheme is the ONLY writer of the theme state: it sets
      // data-theme + data-mode on <html>.
      // Every component repaints because components read tokens, never
      // colors. persistTheme makes the choice survive a reload.
      pick: (id: ThemeId): void => {
        applyTheme(id)
        persistTheme(id)
        active.set(id)
      },
    }
  })
</controller>

<scope>
  const muted = style.fg(muted-foreground)
  const preview = style.pad(4).radius(lg).borderWidth(1).border(border).bg(background).fg(foreground)
  const title = style.fontSize(sm)
  const body = style.fontSize(sm).fg(muted-foreground)
</scope>

<column gap="4">
  <row align="center" justify="between">
    <h3>Theme lab</h3>
    <small sx={muted}>data-theme = {active()}</small>
  </row>

  <row gap="1" align="center">
    <for {themes()} as="t" key={t.id}>
      <button size="sm" tone="primary" variant={active() === t.id ? 'solid' : 'ghost'} on:click={() => pick(t.id)}>
        {t.label}
      </button>
    </for>
  </row>

  <grid cols="2" gap="4">
    <column theme="paper" gap="2" sx={preview}>
      <small sx={muted}>pinned: paper</small>
      <card padding="md">
        <column gap="2">
          <row gap="2" align="center">
            <badge tone="success" variant="soft" size="sm">on track</badge>
            <strong sx={title}>Q3 launch</strong>
          </row>
          <p sx={body}>Same markup as the panel beside me — only the tokens differ.</p>
          <row gap="2">
            <button size="sm" variant="solid" tone="primary">Primary</button>
            <button size="sm" variant="outline" tone="primary">Outline</button>
          </row>
        </column>
      </card>
    </column>

    <column theme="console" gap="2" sx={preview}>
      <small sx={muted}>pinned: console</small>
      <card padding="md">
        <column gap="2">
          <row gap="2" align="center">
            <badge tone="success" variant="soft" size="sm">on track</badge>
            <strong sx={title}>Q3 launch</strong>
          </row>
          <p sx={body}>Same markup as the panel beside me — only the tokens differ.</p>
          <row gap="2">
            <button size="sm" variant="solid" tone="primary">Primary</button>
            <button size="sm" variant="outline" tone="primary">Outline</button>
          </row>
        </column>
      </card>
    </column>
  </grid>

  <row gap="2" align="center">
    <small sx={muted}>the tone contract:</small>
    <badge tone="default" variant="soft" size="sm">default</badge>
    <badge tone="primary" variant="soft" size="sm">primary</badge>
    <badge tone="info" variant="soft" size="sm">info</badge>
    <badge tone="success" variant="soft" size="sm">success</badge>
    <badge tone="warning" variant="soft" size="sm">warning</badge>
    <badge tone="destructive" variant="soft" size="sm">destructive</badge>
  </row>
</column>
