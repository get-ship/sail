<!-- Component gallery hub — theme switcher + links into pack proof routes. -->

<controller>
import { controller, signal, effect } from '@ship/sail-runtime'

// Themes that pair with data-mode=dark on <html> (deck shell mode-of-theme).
const DARK_THEMES = new Set(['console', 'contrast', '8bit', 'synthwave'])

const THEME_OPTIONS = [
  { id: 'ship', label: 'SHIP' },
  { id: 'paper', label: 'Paper' },
  { id: 'halo', label: 'Halo' },
  { id: 'console', label: 'Console' },
  { id: 'contrast', label: 'Contrast' },
] as const

export default controller(() => {
  const activeTheme = signal<string>('ship')
  const themes = THEME_OPTIONS

  mount(() => {
    effect(() => {
      const theme = activeTheme()
      document.documentElement.setAttribute('theme', theme)
      document.documentElement.setAttribute('data-mode', DARK_THEMES.has(theme) ? 'dark' : 'light')
    })
  })

  return {
    activeTheme,
    themes,
  }
})
</controller>

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style lead = { box: mt(2), paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style strong = { paint: fg(foreground) }
  style section = { box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style sectionTitle = { paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style monoMuted = { paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono) }
  style themeBtn = { box: padX(3).padY(1), paint: radius(md), when: { pressed: { paint: bg(foreground).fg(background) } } }
  style packList = { box: pad(0).m(0), paint: list(none) }
</scope>

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
