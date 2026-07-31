// Code snippets for the theming page. Every Sail sample is compiled by the
// build (crates/sail/compiler/tests/site_snippets_compile.rs).
export const code = {
  picker: `<!-- the whole theming UI. Trigger, panel, previews, keyboard,
     persistence — one tag. -->
<theme-picker></theme-picker>`,

  apply: `import { applyTheme, persistTheme } from '@ship/tokens/themes'

// applyTheme is the ONLY writer of the theme state: it sets
// data-theme + data-mode on <html> for light/dark
// themes, and announces ship:theme:change for any chrome listening.
applyTheme('console')
persistTheme('console')   // survives reload

// on boot, restore the reader's pick:
import { readStoredTheme } from '@ship/tokens/themes'
applyTheme(readStoredTheme())`,

}
