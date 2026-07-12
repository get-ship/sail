// Code snippets for the theming page. Every SUI sample is compiled by the
// build (crates/sui/compiler/tests/site_snippets_compile.rs).
export const code = {
  picker: `<!-- the whole theming UI. Trigger, panel, previews, keyboard,
     persistence — one tag. -->
<theme-picker></theme-picker>`,

  apply: `import { applyTheme, persistTheme } from '@ship/tokens/themes'

// applyTheme is the ONLY writer of the theme state: it sets
// data-theme on <html>, toggles the canonical .dark class for dark
// themes, and announces ship:theme:change for any chrome listening.
applyTheme('console')
persistTheme('console')   // survives reload

// on boot, restore the reader's pick:
import { readStoredTheme } from '@ship/tokens/themes'
applyTheme(readStoredTheme())`,

  defineTheme: `/* A theme is a [data-theme] block over the token ladder. Override
   any subset — palette, surfaces, radii, shadows, type — and every
   component follows, because components read tokens, never colors. */
:root[data-theme='crimson'] {
  --primary: oklch(0.62 0.22 25);
  --primary-foreground: oklch(0.99 0.005 25);
  --accent: oklch(0.68 0.16 50);
  --background: oklch(0.98 0.008 30);
  --radius-md: 4px;                 /* tighter, edgier */
  --shadow-sm: 0 1px 2px oklch(0.3 0.05 25 / 20%);
}`,

  accentChannel: `/* Composition: a second attribute that overrides ONLY the brand
   channel. This site does exactly this — data-product turns the accent
   violet on Deck pages without touching the reader's theme. */
:root[data-product='deck'] {
  --primary: oklch(0.55 0.2 285);
  --ring: oklch(0.55 0.2 285 / 40%);
  --accent: oklch(0.54 0.17 280);
}
:root.dark[data-product='deck'] {
  --primary: oklch(0.68 0.19 285);  /* brighter on dark surfaces */
}`,
}
