<!--
  Lesson 07 starter — the picker updates a label, not the page.

  Fix pick() in the controller so the theme actually applies, then pin
  the second preview panel: data-theme-preview scopes the token table to
  a subtree, which is how two themes sit side by side on one page.
-->

<controller>
import { controller, signal } from '@ship/sail-runtime'
import { THEMES, readStoredTheme, type Theme, type ThemeId } from '@ship/tokens/themes'

export default controller(() => {
    const active = signal<ThemeId>(readStoredTheme())

    return {
      themes: (): Theme[] => THEMES,
      active,
      // TODO 1: this only updates the label — the page never repaints.
      // applyTheme(id) is the ONLY writer of the theme state (data-theme on
      // <html> plus the canonical .dark class); persistTheme(id) makes the
      // choice survive a reload. Import both from '@ship/tokens/themes'
      // and call them here.
      pick: (id: ThemeId): void => {
        active.set(id)
      },
    }
  })
</controller>

<column gap="4">
  <row align="center" justify="between">
    <h3>Theme lab</h3>
    <small sx="text-muted-foreground">data-theme = {active()}</small>
  </row>

  <row gap="1" align="center">
    {#for t of themes() key={t.id}}
      <button size="sm" tone="primary" variant={active() === t.id ? 'solid' : 'ghost'} on:click={() => pick(t.id)}>
        {t.label}
      </button>
    {/for}
  </row>

  <grid cols="2" gap="4">
    <column data-theme-preview="paper" gap="2" sx="p-4 rounded-lg border border-border bg-background text-foreground">
      <small sx="text-muted-foreground">pinned: paper</small>
      <card padding="md">
        <column gap="2">
          <row gap="2" align="center">
            <badge tone="success" variant="soft" size="sm">on track</badge>
            <strong sx="text-sm">Q3 launch</strong>
          </row>
          <p sx="text-sm text-muted-foreground">Same markup as the panel beside me — only the tokens differ.</p>
          <row gap="2">
            <button size="sm" variant="solid" tone="primary">Primary</button>
            <button size="sm" variant="outline" tone="primary">Outline</button>
          </row>
        </column>
      </card>
    </column>

    <!-- TODO 2: this panel follows the page theme. Pin it to 'console'
         with data-theme-preview so the two previews stop agreeing. -->
    <column gap="2" sx="p-4 rounded-lg border border-border bg-background text-foreground">
      <small sx="text-muted-foreground">pinned: (not yet)</small>
      <card padding="md">
        <column gap="2">
          <row gap="2" align="center">
            <badge tone="success" variant="soft" size="sm">on track</badge>
            <strong sx="text-sm">Q3 launch</strong>
          </row>
          <p sx="text-sm text-muted-foreground">Same markup as the panel beside me — only the tokens differ.</p>
          <row gap="2">
            <button size="sm" variant="solid" tone="primary">Primary</button>
            <button size="sm" variant="outline" tone="primary">Outline</button>
          </row>
        </column>
      </card>
    </column>
  </grid>
</column>
