// Snippets for /sail/guide/making-a-theme. Every excerpt is VERBATIM from
// packages/sail/tokens — themes.css, themes.ts, tokens.test.ts, products.css.
// Never invent a token name here: the guide's whole value is that a reader
// can paste it.
export const code = {
  // packages/sail/tokens/src/themes.css:56-100, abridged where marked.
  paperBlock: `:root[data-theme='paper'],
[data-theme-preview='paper'] {
  --background: oklch(0.985 0.020 75);
  --foreground: oklch(0.20 0.015 60);
  --card: oklch(0.995 0.012 75);
  --card-foreground: oklch(0.20 0.015 60);
  --muted: oklch(0.92 0.020 75);
  --muted-foreground: oklch(0.45 0.015 60);
  --border: oklch(0.20 0.015 60 / 14%);
  --primary: oklch(0.50 0.16 var(--brand-hue, 30));
  --primary-foreground: oklch(0.985 0.02 75);
  --accent: oklch(0.45 0.10 var(--brand-hue, 280));
  --ring: oklch(0.50 0.16 var(--brand-hue, 30)/ 35%);

  /* Paper surfaces: slightly brighter than the page so cards lift
   * off the pulp. */
  --surface-0: oklch(0.985 0.020 75);
  --surface-1: oklch(0.995 0.012 75);
  --surface-2: oklch(1.000 0.000 75);
  --surface-3: oklch(1.000 0.000 75);
  --border-subtle: oklch(0.20 0.015 60 / 8%);
  --border-strong: oklch(0.20 0.015 60 / 30%);

  /* Rounded book-corners. */
  --radius: 10px;
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 12px;
  --radius-xl: 18px;

  /* Slower, calmer motion — long reading sessions. */
  --duration-fast: 200ms;
  --duration-medium: 300ms;
  --duration-slow: 420ms;
  --duration-deliberate: 560ms;
  --ease-out: var(--ease-out-soft);

  /* …shadows, sidebar chrome, typography, interaction tints… */
}`,

  // The copy: same block, new id, three values changed.
  mineBlock: `:root[data-theme='sandstone'],
[data-theme-preview='sandstone'] {
  --background: oklch(0.960 0.030 85);   /* warmer, darker paper */
  --foreground: oklch(0.24 0.020 55);
  --card: oklch(0.985 0.022 85);
  --card-foreground: oklch(0.24 0.020 55);
  --muted: oklch(0.90 0.028 85);
  --muted-foreground: oklch(0.47 0.020 55);
  --border: oklch(0.24 0.020 55 / 14%);
  --primary: oklch(0.52 0.14 var(--brand-hue, 55));   /* clay, not brick */
  --primary-foreground: oklch(0.985 0.02 85);
  --accent: oklch(0.45 0.10 var(--brand-hue, 250));
  --ring: oklch(0.52 0.14 var(--brand-hue, 55)/ 35%);

  --surface-0: oklch(0.960 0.030 85);
  --surface-1: oklch(0.985 0.022 85);
  --surface-2: oklch(1.000 0.000 85);
  --surface-3: oklch(1.000 0.000 85);
  --border-subtle: oklch(0.24 0.020 55 / 8%);
  --border-strong: oklch(0.24 0.020 55 / 30%);

  --radius: 4px;                          /* squarer than Paper */
  --radius-sm: 2px;
  --radius-md: 3px;
  --radius-lg: 6px;
  --radius-xl: 10px;
}`,

  // packages/sail/tokens/src/themes.ts:16-24 and one THEMES entry.
  themeType: `export type ThemeId =
  | 'ship-light' | 'ship-dark' | 'paper' | 'halo' | 'console'
  | 'contrast' | 'glass' | 'neutral-light' | 'neutral-dark'

export interface Theme {
  id: ThemeId
  label: string
  dark: boolean
  swatches: readonly [string, string, string, string]
}`,

  themeEntry: `  {
    id: 'sandstone', label: 'Sandstone', dark: false,
    swatches: [
      'oklch(0.960 0.030 85)',   // background
      'oklch(0.985 0.022 85)',   // card
      'oklch(0.52 0.14 55)',     // primary
      'oklch(0.45 0.10 250)',    // accent
    ],
  },`,

  // packages/sail/tokens/src/themes.ts:144-160, verbatim.
  applySource: `export function applyTheme(
  id: unknown,
  root: HTMLElement = document.documentElement,
): ThemeId {
  const next: ThemeId = isValidTheme(id) ? id : DEFAULT_THEME
  const dark = isDarkTheme(next)
  root.dataset.theme = next
  root.classList.toggle('dark', dark)
  if (typeof document !== 'undefined' && typeof CustomEvent !== 'undefined') {
    document.dispatchEvent(
      new CustomEvent('ship:theme:change', { detail: { id: next, dark } }),
    )
  }
  return next
}`,

  apply: `import { applyTheme, persistTheme } from '@ship/tokens/themes'

applyTheme('sandstone')     // sets data-theme, toggles .dark, announces the change
persistTheme('sandstone')   // localStorage, and the host transport if one is set`,

  // packages/sail/tokens/src/products.css, verbatim (the whole file body).
  products: `:root[data-product='ship']       { --brand-hue: 195; }  /* teal   — umbrella */
:root[data-product='sail']        { --brand-hue: 27; }   /* red    — the framework */
:root[data-product='deck']       { --brand-hue: 285; }  /* violet — the app framework */
:root[data-product='components'] { --brand-hue: 155; }  /* green  — the library */
:root[data-product='ai']         { --brand-hue: 255; }  /* blue   — agent tooling */`,
}
