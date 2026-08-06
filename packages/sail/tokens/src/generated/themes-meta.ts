/**
 * GENERATED FILE — do not edit by hand.
 *
 * Theme catalog meta from packages/sail/tokens/model/themes/**.
 * Includes modes-in-file (Phase D). Regenerate: sail tokens
 */

export interface ThemeMeta {
  id: string
  /** Whether the default mode is a dark surface (data-mode dark). */
  dark: boolean
  /** Mode names when modes-in-file; empty = legacy single-block theme. */
  modes: readonly string[]
  defaultMode: string | null
}

export const THEME_META: readonly ThemeMeta[] = [
  { id: '8bit', dark: true, modes: ['dark'], defaultMode: 'dark' },
  { id: 'brut', dark: false, modes: ['light'], defaultMode: 'light' },
  { id: 'console', dark: true, modes: ['dark'], defaultMode: 'dark' },
  { id: 'contrast', dark: true, modes: ['dark'], defaultMode: 'dark' },
  { id: 'glass', dark: false, modes: ['light'], defaultMode: 'light' },
  { id: 'halo', dark: false, modes: ['light'], defaultMode: 'light' },
  { id: 'neutral', dark: false, modes: ['light', 'dark'], defaultMode: 'light' },
  { id: 'paper', dark: false, modes: ['light'], defaultMode: 'light' },
  { id: 'sail', dark: false, modes: ['light', 'dark'], defaultMode: 'light' },
  { id: 'ship', dark: false, modes: ['light', 'dark'], defaultMode: 'light' },
  { id: 'synthwave', dark: true, modes: ['dark'], defaultMode: 'dark' },
] as const

const BY_ID: Record<string, ThemeMeta> = Object.fromEntries(
  THEME_META.map((t) => [t.id, t]),
)

export function themeMeta(id: string): ThemeMeta | undefined {
  return BY_ID[id]
}
