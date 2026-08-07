/**
 * GENERATED FILE — do not edit by hand.
 *
 * Theme catalog meta + THEMES-1 schema lock from
 * packages/sail/tokens/model/themes/**.
 * Includes modes-in-file (Phase D) + mode capability (light|dark|both).
 * Regenerate: sail tokens
 */

/** Allowed top-level keys on a theme document. */
export const THEME_ROOT_KEYS = ['$description', '$extensions', 'modes'] as const
export type ThemeRootKey = (typeof THEME_ROOT_KEYS)[number]

/** Allowed keys under `$extensions.sail` on a theme. */
export const THEME_SAIL_KEYS = ['layer', 'themeId', 'defaultMode', 'supportedModes', 'density', 'ordered'] as const
export type ThemeSailKey = (typeof THEME_SAIL_KEYS)[number]

/** Allowed paint mode names (`data-mode` values). */
export const THEME_PAINT_MODES = ['light', 'dark'] as const
export type ThemePaintMode = (typeof THEME_PAINT_MODES)[number]

/** Allowed top-level groups inside a mode body (scale + component overrides). */
export const THEME_MODE_GROUP_KEYS = ['$extensions', 'color', 'component', 'control', 'interaction', 'motion', 'radius', 'scrollbar', 'shadow', 'typography', 'space', 'icon', 'density', 'panel', 'z', 'breakpoint'] as const
export type ThemeModeGroupKey = (typeof THEME_MODE_GROUP_KEYS)[number]

/**
 * Override resolution order (highest wins first).
 * Freeze §6 / inv-theme-override-order: instance (style/channels) >
 * component tokens > global theme > baked defaults.
 */
export const THEME_OVERRIDE_ORDER = ['instance', 'component', 'global', 'baked'] as const
export type ThemeOverrideLayer = (typeof THEME_OVERRIDE_ORDER)[number]

/** Per-theme declared mode capability (VSCode model — not universal). */
export type ModeCapability = 'light' | 'dark' | 'both'

export interface ThemeMeta {
  id: string
  /** Whether the default mode is a dark surface (data-mode dark). */
  dark: boolean
  /** Mode names when modes-in-file; empty = legacy single-block theme. */
  modes: readonly string[]
  defaultMode: string | null
  /** Declared capability derived from modes: light | dark | both. */
  capability: ModeCapability
}

export const THEME_META: readonly ThemeMeta[] = [
  { id: '8bit', dark: true, modes: ['dark'], defaultMode: 'dark', capability: 'dark' },
  { id: 'brut', dark: false, modes: ['light'], defaultMode: 'light', capability: 'light' },
  { id: 'console', dark: true, modes: ['dark'], defaultMode: 'dark', capability: 'dark' },
  { id: 'contrast', dark: true, modes: ['dark'], defaultMode: 'dark', capability: 'dark' },
  { id: 'glass', dark: false, modes: ['light'], defaultMode: 'light', capability: 'light' },
  { id: 'halo', dark: false, modes: ['light'], defaultMode: 'light', capability: 'light' },
  { id: 'neutral', dark: false, modes: ['light', 'dark'], defaultMode: 'light', capability: 'both' },
  { id: 'paper', dark: false, modes: ['light'], defaultMode: 'light', capability: 'light' },
  { id: 'sail', dark: false, modes: ['light', 'dark'], defaultMode: 'light', capability: 'both' },
  { id: 'ship', dark: false, modes: ['light', 'dark'], defaultMode: 'light', capability: 'both' },
  { id: 'synthwave', dark: true, modes: ['dark'], defaultMode: 'dark', capability: 'dark' },
] as const

const BY_ID: Record<string, ThemeMeta> = Object.fromEntries(
  THEME_META.map((t) => [t.id, t]),
)

export function themeMeta(id: string): ThemeMeta | undefined {
  return BY_ID[id]
}

/** Capability for a theme id (defaults to both when unknown). */
export function modeCapability(id: string): ModeCapability {
  return themeMeta(id)?.capability ?? 'both'
}
