/**
 * Theme catalog for node QA harnesses (probe / sweep / l1-conformance).
 *
 * MUST stay aligned with `themes.ts` THEMES[] and `generated/unions.ts` THEME_IDS.
 * Dark is a **mode** (`data-mode`), not a twin theme id.
 *
 * When you add a theme: update model/themes/*.json, regenerate tokens,
 * then add one row here (or teach the tokens generator to emit this file).
 */

/** @type {readonly { id: string, dark: boolean, defaultMode?: string }[]} */
export const THEME_CATALOG = Object.freeze([
  { id: 'ship', dark: false, defaultMode: 'light' },
  { id: 'neutral', dark: false, defaultMode: 'light' },
  { id: 'paper', dark: false, defaultMode: 'light' },
  { id: 'halo', dark: false, defaultMode: 'light' },
  { id: 'console', dark: true, defaultMode: 'dark' },
  { id: 'contrast', dark: true, defaultMode: 'dark' },
  { id: 'glass', dark: false, defaultMode: 'light' },
  { id: 'brut', dark: false, defaultMode: 'light' },
  { id: '8bit', dark: true, defaultMode: 'dark' },
  { id: 'synthwave', dark: true, defaultMode: 'dark' },
])

/** Every theme id, stable order for reports + screenshot grids. */
export const ALL_THEME_IDS = Object.freeze(THEME_CATALOG.map((t) => t.id))

/** @type {ReadonlySet<string>} */
export const DARK_THEME_IDS = new Set(THEME_CATALOG.filter((t) => t.dark).map((t) => t.id))

/** @param {string} id */
export function isDarkTheme(id) {
  return DARK_THEME_IDS.has(id)
}

/**
 * Parse a CLI theme list.
 * - omitted / "all" → every catalog theme
 * - "a,b,c" → those ids (unknown ids kept so the run surfaces them)
 *
 * @param {string | undefined | null} arg
 * @returns {string[]}
 */
export function parseThemeList(arg) {
  if (arg == null || arg === '' || arg === 'all') return [...ALL_THEME_IDS]
  return arg.split(',').map((s) => s.trim()).filter(Boolean)
}
