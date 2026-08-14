<!--
  Lesson 07 starter — the picker updates a label, not the page.

  Fix pick() in the controller so the theme actually applies, then pin
  the second preview panel: theme scopes the token table to a subtree,
  which is how two themes sit side by side on one page.
-->

<controller>
import { type Theme, type ThemeId, THEMES } from '@/compositions/theme-catalog'

export default controller(() => {
  const active = signal<ThemeId>(readStoredTheme())

  return {
    themes: (): Theme[] => THEMES,
    active,
    // TODO 1: this only updates the label — the page never repaints.
    // applyTheme(id) is the only writer of the theme state (data-theme +
    // mode on <html>); persistTheme(id) makes the choice survive a
    // reload. Call ambient applyTheme + persistTheme here (no package import).
    pick: (id: ThemeId): void => {
      active.set(id)
    },
  }
})
</controller>

<scope>
  style muted = { fg(muted-foreground) }
  style preview = { pad(4).radius(lg).border({ color: border, width: 1 }).bg(background).fg(foreground) }
  style title = { type({ size: sm }) }
  style body = { fg(muted-foreground).type({ size: sm }) }
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

    <!-- TODO 2: this panel follows the page theme. Pin it to 'console'
         with theme so the two previews stop agreeing. Prefer the same
         <pinned-preview pin="console" /> shape as the panel above. -->
    <column gap="2" style(preview)>
      <small style(muted)>pinned: (not yet)</small>
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
  </grid>
</column>
