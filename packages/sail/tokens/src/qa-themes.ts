/**
 * Theme catalog for node QA harnesses (probe / sweep).
 *
 * SSOT is generated THEME_META (`sail tokens`). Do not hand-list ids here.
 */
import { THEME_META, type ThemeMeta } from './generated/themes-meta.ts'

export type ThemeCatalogRow = { id: string; dark: boolean; defaultMode?: string }

export const THEME_CATALOG: readonly ThemeCatalogRow[] = Object.freeze(
  THEME_META.map((t: ThemeMeta) => ({
    id: t.id,
    dark: t.dark,
    defaultMode: t.defaultMode ?? undefined,
  })),
)

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
 */
export function parseThemeList(arg: string | undefined | null): string[] {
  if (arg == null || arg === '' || arg === 'all') return [...ALL_THEME_IDS]
  return arg.split(',').map((s) => s.trim()).filter(Boolean)
}
