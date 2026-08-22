<!-- Component gallery hub — theme switcher + links into pack proof routes. -->

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style lead = { margin({ t: 2 }).fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style strong = { fg(foreground) }
  style section = { pad(5).border({ color: border, width: 1 }).radius(lg).bg(surface) }
  style sectionTitle = { fg(foreground).type({ size: lg, weight: semibold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style monoMuted = { fg(muted-foreground).type({ size: sm, family: mono }) }
  style themeBtn = { pad({ x: 3, y: 1 }).radius(md), when: { pressed: bg(foreground).fg(background) } }
  style packList = { pad(0).margin(0).type({ list: none }) }
</scope>

<controller>
// Themes that pair with data-mode=dark on <html> (deck shell mode-of-theme).
const THEME_OPTIONS = [
  { id: 'sail', label: 'SAIL' },
  { id: 'paper', label: 'Paper' },
  { id: 'halo', label: 'Halo' },
  { id: 'console', label: 'Console' },
  { id: 'contrast', label: 'Contrast' },
] as const

export default controller(() => {
  const activeTheme = signal<string>('sail')
  const themes = THEME_OPTIONS

  return {
    activeTheme,
    themes,
  }
})
</controller>

<column gap="6" style(page)>
  <header>
    <h1 style(title)>Sail Component Gallery</h1>
    <p style(lead)>
      Primitives marked <strong style(strong)>{'sail/*'}</strong> are real, fully implemented Sail
      components. Sections marked <strong style(strong)>{'ship/*'}</strong> are domain stubs
      backed by the deck-conformance in-memory store — they exercise the live-data
      wire but carry no production Ship logic.
    </p>
  </header>

  <column gap="3" style(section)>
    <h2 style(sectionTitle)>Theme</h2>
    <row gap="2">
      <for {themes as t} key={t.id}>
        <button
          aria-pressed={activeTheme() === t.id}
          on:click={() => activeTheme.set(t.id)}
          style(themeBtn)
        >{t.label}</button>
      </for>
    </row>
    <p style(monoMuted)>Active: {activeTheme()}</p>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>Shared components</h2>
    <p style(muted)>
      demo-chip resolves from the app’s component workspace; widgets uses the
      same tag, so the multi-entry build hoists it into one shared chunk.
    </p>
    <demo-chip>gallery-alpha</demo-chip>
  </column>

  <column gap="2" style(section)>
    <h2 style(sectionTitle)>Pack sections</h2>
    <column gap="1" style(packList)>
      <li><a href="/_deck/view/counter">counter — signal basics ({'sail/*'})</a></li>
      <li><a href="/_deck/view/clock">clock — live data tick (&lt;client name="api"&gt;, api/demo.*)</a></li>
      <li><a href="/_deck/view/todos">todos — live list + mutation (ship/demo.*)</a></li>
      <li><a href="/_deck/view/query-demo">query-demo — query()/mutation() derived identity (crm/deals.*)</a></li>
      <li><a href="/_deck/view/boom">boom — error boundary (ship/demo.*)</a></li>
      <li><a href="/_deck/view/greet">greet — static SSR ({'sail/*'})</a></li>
      <li><a href="/_deck/view/hello">hello — constant folding ({'sail/*'})</a></li>
      <li><a href="/_deck/view/widgets">widgets — component composition ({'sail/*'})</a></li>
    </column>
  </column>
</column>
