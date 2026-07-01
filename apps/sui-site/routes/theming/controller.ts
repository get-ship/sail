// /theming — drives the live light/dark toggle and the inline custom-theme
// demonstration on the page. `mode` is the document-level data-mode; toggling
// it sets/clears the dark CSS class on <html>, and the shell's existing token
// system (see sui-site-shell controller) reads it for the topbar button.

import { controller, signal, effect, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const code = {
  toggleExample: [
    '// document-level data-mode is the single switch every primitive reads',
    'document.documentElement.dataset.mode = isDark ? "dark" : ""',
    '// OR toggle the .dark class directly — either works',
    'document.documentElement.classList.toggle("dark", isDark)',
    '',
    '// The host persists the choice and replays it on next mount',
    'localStorage.setItem("sui-mode", isDark ? "dark" : "light")',
  ].join('\n'),
  defineTheme: [
    '/* Register a theme by adding a [data-theme="…"] override block. */',
    ':root[data-theme="crimson-red"] {',
    '  --primary: oklch(0.62 0.22 25);          /* deep crimson */',
    '  --primary-fg: oklch(0.99 0.005 25);',
    '  --accent: oklch(0.68 0.16 50);           /* warm gold */',
    '  --tone-primary-bg: color-mix(in oklch, var(--primary) 14%, transparent);',
    '  --tone-primary-border: color-mix(in oklch, var(--primary) 32%, transparent);',
    '  --radius-md: 4px;                       /* tighter, edgier */',
    '  --radius-lg: 8px;',
    '}',
  ].join('\n'),
  activate: [
    '// Apply the theme. The data-theme attribute is the single source of truth',
    'document.documentElement.dataset.theme = "crimson-red"',
    '',
    '// Optionally persist the user\'s selection',
    'localStorage.setItem("sui-theme", "crimson-red")',
  ].join('\n'),
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const root = document.documentElement
    const mode = signal<'light' | 'dark'>(
      root.dataset.mode === 'dark' || root.classList.contains('dark') ? 'dark' : 'light',
    )
    effect(() => {
      const m = mode()
      if (m === 'dark') {
        root.dataset.mode = 'dark'
        root.classList.add('dark')
      } else {
        delete root.dataset.mode
        root.classList.remove('dark')
      }
    })

    const crimson = signal(false)
    effect(() => {
      if (crimson()) {
        root.dataset.theme = 'crimson-red'
      } else if (root.dataset.theme === 'crimson-red') {
        delete root.dataset.theme
      }
    })

    // Inject the custom-theme stylesheet (once) so the live toggle actually
    // re-skins the page when activated.
    if (!document.getElementById('sui-crimson-theme')) {
      const style = document.createElement('style')
      style.id = 'sui-crimson-theme'
      style.textContent = code.defineTheme
      document.head.appendChild(style)
    }

    return {
      mode,
      crimson,
      toggleCrimson: (): void => crimson.set(!crimson()),
      code,
    }
  },
})
