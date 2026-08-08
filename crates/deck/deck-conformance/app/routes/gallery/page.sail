<controller>
import { controller, signal, effect } from '@ship/sail-runtime'

// Dark themes pair with `data-mode="dark"` on <html> so the [data-mode="light"]
// override in deck-theme.css can layer the light palette on top. This is the
// mode-of-theme contract the deck shell uses for its own chrome — the
// gallery reuses the same handle so the conformance routes theme consistently
// with the surrounding host.
const DARK_THEMES = new Set(['console', 'contrast', '8bit', 'synthwave'])

export default controller(() => {
  const activeTheme = signal<string>('ship')

  mount(() => {
    effect(() => {
      const theme = activeTheme()
      document.documentElement.setAttribute('theme', theme)
      document.documentElement.setAttribute('data-mode', DARK_THEMES.has(theme) ? 'dark' : 'light')
    })
  })

  return {
    activeTheme,
  }
})
</controller>

<column gap="6" style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })>
  <header>
    <h1 style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })>Sail Component Gallery</h1>
    <p style({ box: mt(2), paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })>
      Primitives marked <strong style({ paint: fg(foreground) })>sail/*</strong> are real, fully implemented Sail
      components. Sections marked <strong style({ paint: fg(foreground) })>ship/*</strong> are domain stubs
      backed by the deck-conformance in-memory store — they exercise the live-data
      wire but carry no production Ship logic.
    </p>
  </header>

  <column gap="3" style({ box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })>
    <h2 style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) })>Theme</h2>
    <row gap="2">
      <button aria-pressed={activeTheme() === 'ship'} on:click={() => activeTheme.set('ship')} style({ box: padX(3).padY(1), paint: radius(md), when: { pressed: { paint: bg(foreground).fg(background) } } })>
SHIP</button>

      <button aria-pressed={activeTheme() === 'paper'} on:click={() => activeTheme.set('paper')} style({ box: padX(3).padY(1), paint: radius(md), when: { pressed: { paint: bg(foreground).fg(background) } } })>
Paper</button>
      <button aria-pressed={activeTheme() === 'halo'} on:click={() => activeTheme.set('halo')} style({ box: padX(3).padY(1), paint: radius(md), when: { pressed: { paint: bg(foreground).fg(background) } } })>
Halo</button>
      <button aria-pressed={activeTheme() === 'console'} on:click={() => activeTheme.set('console')} style({ box: padX(3).padY(1), paint: radius(md), when: { pressed: { paint: bg(foreground).fg(background) } } })>
Console</button>
      <button aria-pressed={activeTheme() === 'contrast'} on:click={() => activeTheme.set('contrast')} style({ box: padX(3).padY(1), paint: radius(md), when: { pressed: { paint: bg(foreground).fg(background) } } })>
Contrast</button>
    </row>
    <p style({ paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono) })>Active: {activeTheme()}</p>
  </column>

  <column gap="2" style({ box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })>
    <h2 style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) })>Shared components</h2>
    <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>
      demo-chip resolves from the app's component workspace; widgets uses the
      same tag, so the multi-entry build hoists it into one shared chunk.
    </p>
    <demo-chip>gallery-alpha</demo-chip>
  </column>

  <column gap="2" style({ box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })>
     <h2 style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold) })>Pack sections</h2>
    <column gap="1" style({ box: pad(0).m(0), paint: list(none) })>
       <li><a href="/_deck/view/counter">counter — signal basics (sail/*)</a></li>
       <li><a href="/_deck/view/clock">clock — live data tick (&lt;client name="api"&gt;, api/demo.*)</a></li>
       <li><a href="/_deck/view/todos">todos — live list + mutation (ship/demo.*)</a></li>
       <li><a href="/_deck/view/query-demo">query-demo — query()/mutation() derived identity (crm/deals.*)</a></li>
       <li><a href="/_deck/view/boom">boom — error boundary (ship/demo.*)</a></li>
       <li><a href="/_deck/view/greet">greet — static SSR (sail/*)</a></li>
       <li><a href="/_deck/view/hello">hello — constant folding (sail/*)</a></li>
       <li><a href="/_deck/view/widgets">widgets — component composition (sail/*)</a></li>
    </column>
  </column>
</column>
