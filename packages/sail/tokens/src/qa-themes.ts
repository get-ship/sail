/**
 * Theme catalog for node QA harnesses (probe / sweep).
 *
 * MUST stay aligned with `themes.ts` THEMES[] and `generated/unions.ts` THEME_IDS.
 * Dark is a **mode** (`mode=`), not a twin theme id.
 *
 * When you add a theme: update model/themes/*.json, regenerate tokens,
 * then add one row here (or teach the tokens generator to emit this file).
 */

export type ThemeCatalogRow = { id: string; dark: boolean; defaultMode?: string }

export const THEME_CATALOG: readonly ThemeCatalogRow[] = Object.freeze([
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

export const DARK_THEME_IDS: ReadonlySet<string> = new Set(
  THEME_CATALOG.filter((t) => t.dark).map((t) => t.id),
)

export function isDarkTheme(id: string): boolean {
  return DARK_THEME_IDS.has(id)
}

/**
 * Parse a CLI theme list.
 * - omitted / "all" → every catalog theme
 * - "a,b,c" → those ids (unknown ids kept so the run surfaces them)
 *
 */
export function parseThemeList(arg: string | undefined | null): string[] {
  if (arg == null || arg === '' || arg === 'all') return [...ALL_THEME_IDS]
  return arg.split(',').map((s) => s.trim()).filter(Boolean)
}
