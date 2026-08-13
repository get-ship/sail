<!--
  Lesson 07 solved — the theme is one attribute.

  pick() calls applyTheme + persistTheme; everything on the page repaints
  because every surface reads tokens. The two panels below opt OUT: each
  pins its own token table with theme, which is how a picker can show you
  a theme without applying it.
-->

<controller>
import { type Theme, type ThemeId, THEMES } from '@/compositions/theme-catalog'

export default controller(() => {
    const active = signal<ThemeId>(readStoredTheme())

    return {
      themes: (): Theme[] => THEMES,
      active,
      // applyTheme is the only writer of the page theme (data-theme +
      // data-mode on <html>). Components repaint because they read tokens,
      // not colors. persistTheme keeps the choice across reloads.
      pick: (id: ThemeId): void => {
        applyTheme(id)
        persistTheme(id)
        active.set(id)
      },
    }
  })
</controller>

<scope>
  style muted = { paint: ink({ color: muted-foreground }) }
  style preview = { box: space({ pad: 4 }), paint: radius(lg).border({ color: border, width: 1 }).fill({ color: background }).ink({ color: foreground }) }
  style title = { text: type({ size: sm }) }
  style body = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  type PinnedPreviewProps = { pin: string }
</scope>

<partial pinned-preview props: PinnedPreviewProps>
  <column theme={pin} gap="2" style(preview)>
    <small style(muted)>pinned: {pin}</small>
    <card padding="md">
      <column gap="2">
        <row gap="2" align="center">
          <badge tone="success" variant="soft" size="sm">on track</badge>
          <strong style(title)>Q3 launch</strong>
        </row>
        <p style(body)>Same markup as the panel beside me — only the tokens differ.</p>
        <row gap="2">
          <button size="sm" variant="solid" tone="primary">Primary</button>
          <button size="sm" variant="outline" tone="primary">Outline</button>
        </row>
      </column>
    </card>
  </column>
</partial>

<column gap="4">
  <row align="center" justify="between">
    <h3>Theme lab</h3>
    <small style(muted)>data-theme = {active()}</small>
  </row>

  <row gap="1" align="center">
    <for {themes() as t} key={t.id}>
      <button size="sm" tone="primary" variant={active() === t.id ? 'solid' : 'ghost'} on:click={() => pick(t.id)}>
        {t.label}
      </button>
    </for>
  </row>

  <grid cols="2" gap="4">
    <pinned-preview pin="paper" />
    <pinned-preview pin="console" />
  </grid>

  <row gap="2" align="center">
    <small style(muted)>the tone contract:</small>
    <badge variant="soft" size="sm">rest</badge>
    <badge tone="primary" variant="soft" size="sm">primary</badge>
    <badge tone="info" variant="soft" size="sm">info</badge>
    <badge tone="success" variant="soft" size="sm">success</badge>
    <badge tone="warning" variant="soft" size="sm">warning</badge>
    <badge tone="destructive" variant="soft" size="sm">destructive</badge>
  </row>
</column>
