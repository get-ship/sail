<controller>
import { controller, signal, effect } from '@ship/sail-runtime'

// Dark themes pair with `data-mode="dark"` on <html> so the [data-mode="light"]
// override in deck-theme.css can layer the light palette on top. This is the
// mode-of-data-theme contract the deck shell uses for its own chrome — the
// gallery reuses the same handle so the conformance views theme consistently
// with the surrounding host.
const DARK_THEMES = new Set(['ship-dark', 'console', 'contrast'])

export default controller(() => {
  const activeTheme = signal<string>('ship-light')

  mount(() => {
    effect(() => {
      const theme = activeTheme()
      document.documentElement.setAttribute('data-theme', theme)
      document.documentElement.setAttribute('data-mode', DARK_THEMES.has(theme) ? 'dark' : 'light')
    })
  })

  return {
    activeTheme,
  }
})
</controller>

<column gap="6" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <header>
    <h1 sx="text-2xl font-bold text-foreground">SUI Component Gallery</h1>
    <p sx="text-muted-foreground text-sm leading-relaxed mt-2">
      Primitives marked <strong sx="text-foreground">sui/*</strong> are real, fully implemented SUI
      components. Sections marked <strong sx="text-foreground">ship/*</strong> are domain stubs
      backed by the deck-conformance in-memory store — they exercise the live-data
      wire but carry no production Ship logic.
    </p>
  </header>

  <column gap="3" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground">Theme</h2>
    <row gap="2">
      <button aria-pressed={activeTheme() === 'ship-light'} on:click={() => activeTheme.set('ship-light')} sx="pressed:bg-foreground pressed:text-background rounded-md px-3 py-1">SHIP Light</button>
      <button aria-pressed={activeTheme() === 'ship-dark'} on:click={() => activeTheme.set('ship-dark')} sx="pressed:bg-foreground pressed:text-background rounded-md px-3 py-1">SHIP Dark</button>
      <button aria-pressed={activeTheme() === 'paper'} on:click={() => activeTheme.set('paper')} sx="pressed:bg-foreground pressed:text-background rounded-md px-3 py-1">Paper</button>
      <button aria-pressed={activeTheme() === 'halo'} on:click={() => activeTheme.set('halo')} sx="pressed:bg-foreground pressed:text-background rounded-md px-3 py-1">Halo</button>
      <button aria-pressed={activeTheme() === 'console'} on:click={() => activeTheme.set('console')} sx="pressed:bg-foreground pressed:text-background rounded-md px-3 py-1">Console</button>
      <button aria-pressed={activeTheme() === 'contrast'} on:click={() => activeTheme.set('contrast')} sx="pressed:bg-foreground pressed:text-background rounded-md px-3 py-1">Contrast</button>
    </row>
    <p sx="text-muted-foreground text-sm font-mono">Active: {activeTheme()}</p>
  </column>

  <column gap="2" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground">Shared components</h2>
    <p sx="text-muted-foreground text-sm">
      demo-chip resolves from the app's component workspace; widgets uses the
      same tag, so the multi-entry build hoists it into one shared chunk.
    </p>
    <demo-chip>gallery-alpha</demo-chip>
  </column>

  <column gap="2" sx="border border-border rounded-lg bg-surface-1 p-5">
     <h2 sx="text-lg font-semibold text-foreground">Pack sections</h2>
    <column gap="1" sx="list-none p-0 m-0">
       <li><a href="/_deck/view/counter">counter — signal basics (sui/*)</a></li>
       <li><a href="/_deck/view/clock">clock — live data tick (&lt;client name="api"&gt;, api/demo.*)</a></li>
       <li><a href="/_deck/view/todos">todos — live list + mutation (ship/demo.*)</a></li>
       <li><a href="/_deck/view/query-demo">query-demo — query()/mutation() derived identity (crm/deals.*)</a></li>
       <li><a href="/_deck/view/boom">boom — error boundary (ship/demo.*)</a></li>
       <li><a href="/_deck/view/greet">greet — static SSR (sui/*)</a></li>
       <li><a href="/_deck/view/hello">hello — constant folding (sui/*)</a></li>
       <li><a href="/_deck/view/widgets">widgets — component composition (sui/*)</a></li>
    </column>
  </column>
</column>
