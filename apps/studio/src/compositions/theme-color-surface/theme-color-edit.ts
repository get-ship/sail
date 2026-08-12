/**
 * Color domain — curated color.* roles. Session owns open/mode/save/path.
 */
import { signal } from '@ship/sail-runtime'
import { getDesignState, getPathValue, setBrandColor, setTokenValue, tryPickerHex } from '../../design/index.ts'
import type { ThemeSession } from '../theme-session/theme-session'

/** Curated write paths — registry keys, not mock CSS var names. */
export const COLOR_ROLES = {
  primary: { path: 'color.primary.default', label: 'Primary', displayPath: 'color.primary', role: 'pr', section: 'brand' },
  secondary: { path: 'color.secondary.default', label: 'Secondary', displayPath: 'color.secondary', role: 'sc', section: 'brand' },
  accent: { path: 'color.brandAccent.default', label: 'Accent', displayPath: 'color.accent', role: 'ac', section: 'brand' },
  bg: { path: 'color.background', label: 'Background', displayPath: 'color.bg', role: 'bg', section: 'surfaces' },
  surface1: { path: 'color.surface.1', label: 'Surface 1', displayPath: 'color.surface.1', role: 's1', section: 'surfaces' },
  surface2: { path: 'color.surface.2', label: 'Surface 2', displayPath: 'color.surface.2', role: 's2', section: 'surfaces' },
  card: { path: 'color.surface.3', label: 'Card', displayPath: 'color.surface.3', role: 'cd', section: 'surfaces' },
  fg: { path: 'color.foreground', label: 'Foreground', displayPath: 'color.foreground', role: 'fg', section: 'text' },
  muted: { path: 'color.muted.foreground', label: 'Muted', displayPath: 'color.muted', role: 'mu', section: 'text' },
  border: { path: 'color.border.default', label: 'Border', displayPath: 'color.border', role: 'bd', section: 'text' },
  ring: { path: 'color.ring.default', label: 'Ring', displayPath: 'color.ring', role: 'rg', section: 'text' },
  success: { path: 'color.success.default', label: 'Success', displayPath: 'color.success', role: 'ok', section: 'semantic' },
  warning: { path: 'color.warning.default', label: 'Warning', displayPath: 'color.warning', role: 'wn', section: 'semantic' },
  danger: { path: 'color.destructive.default', label: 'Danger', displayPath: 'color.danger', role: 'dn', section: 'semantic' },
  info: { path: 'color.info.default', label: 'Info', displayPath: 'color.info', role: 'in', section: 'semantic' },
} as const

export type ColorRoleKey = keyof typeof COLOR_ROLES

const FALLBACKS: Record<ColorRoleKey, string> = {
  primary: '#3b82f6',
  secondary: '#64748b',
  accent: '#a855f7',
  bg: '#f8f7f6',
  surface1: '#ffffff',
  surface2: '#f1efed',
  card: '#ffffff',
  fg: '#1c1917',
  muted: '#78716c',
  border: '#e7e5e4',
  ring: '#3b82f6',
  success: '#16a34a',
  warning: '#d97706',
  danger: '#dc2626',
  info: '#2563eb',
}

function colorHex(
  doc: Record<string, unknown> | null,
  path: string,
  mode: 'light' | 'dark',
  fallback: string,
): string {
  const raw = getPathValue(doc, path, mode)
  // Missing leaf → seed fallback for empty themes. Present but unconvertible → empty (no gray lie).
  if (!raw) return fallback
  return tryPickerHex(raw) ?? ''
}

export function createColorEdit(session: ThemeSession) {
  const primaryHex = signal(FALLBACKS.primary)
  const secondaryHex = signal(FALLBACKS.secondary)
  const accentHex = signal(FALLBACKS.accent)
  const bgHex = signal(FALLBACKS.bg)
  const surface1Hex = signal(FALLBACKS.surface1)
  const surface2Hex = signal(FALLBACKS.surface2)
  const cardHex = signal(FALLBACKS.card)
  const fgHex = signal(FALLBACKS.fg)
  const mutedHex = signal(FALLBACKS.muted)
  const borderHex = signal(FALLBACKS.border)
  const ringHex = signal(FALLBACKS.ring)
  const successHex = signal(FALLBACKS.success)
  const warningHex = signal(FALLBACKS.warning)
  const dangerHex = signal(FALLBACKS.danger)
  const infoHex = signal(FALLBACKS.info)

  const hexByKey: Record<ColorRoleKey, ReturnType<typeof signal<string>>> = {
    primary: primaryHex,
    secondary: secondaryHex,
    accent: accentHex,
    bg: bgHex,
    surface1: surface1Hex,
    surface2: surface2Hex,
    card: cardHex,
    fg: fgHex,
    muted: mutedHex,
    border: borderHex,
    ring: ringHex,
    success: successHex,
    warning: warningHex,
    danger: dangerHex,
    info: infoHex,
  }

  const sync = () => {
    const ds = getDesignState()
    const doc = ds.themeDoc
    const mode = session.writeMode()
    for (const key of Object.keys(COLOR_ROLES) as ColorRoleKey[]) {
      const def = COLOR_ROLES[key]
      hexByKey[key].set(colorHex(doc, def.path, mode, FALLBACKS[key]))
    }
  }

  session.onSync(sync)
  sync()

  const onRoleColor = (detail: { value?: string; token?: string }) => {
    if (!session.guardEdit()) return
    const hex = String(detail?.value ?? '').trim()
    if (!hex) return
    const key = detail?.token as ColorRoleKey | null
    if (!key || !(key in COLOR_ROLES)) return
    const def = COLOR_ROLES[key]
    hexByKey[key].set(tryPickerHex(hex) || hex)
    const write = key === 'primary' ? setBrandColor(hex) : setTokenValue(def.path, hex)
    void write.then(() => session.sync())
  }

  const brandRows = () =>
    (['primary', 'secondary', 'accent'] as const).map((key) => ({
      key,
      role: key,
      label: COLOR_ROLES[key].label,
      path: COLOR_ROLES[key].displayPath,
      hex: hexByKey[key](),
    }))

  const surfaceRows = () =>
    (['bg', 'surface1', 'surface2', 'card'] as const).map((key) => ({
      key,
      role: key,
      label: COLOR_ROLES[key].label,
      path: COLOR_ROLES[key].displayPath,
      chip: COLOR_ROLES[key].role,
      hex: hexByKey[key](),
    }))

  const textRows = () =>
    (['fg', 'muted', 'border', 'ring'] as const).map((key) => ({
      key,
      role: key,
      label: COLOR_ROLES[key].label,
      path: COLOR_ROLES[key].displayPath,
      chip: COLOR_ROLES[key].role,
      hex: hexByKey[key](),
    }))

  const semanticRows = () =>
    (['success', 'warning', 'danger', 'info'] as const).map((key) => ({
      key,
      role: key,
      label: COLOR_ROLES[key].label,
      path: COLOR_ROLES[key].displayPath,
      chip: COLOR_ROLES[key].role,
      hex: hexByKey[key](),
      sem: key,
    }))

  const seedDots = () => [
    { key: 's-primary', hex: primaryHex() },
    { key: 's-secondary', hex: secondaryHex() },
    { key: 's-accent', hex: accentHex() },
    { key: 's-bg', hex: bgHex() },
    { key: 's-fg', hex: fgHex() },
    { key: 's-muted', hex: mutedHex() },
    { key: 's-border', hex: borderHex() },
  ]

  const onSeedGenerate = () => {
    session.setNote('Palette generator — coming soon.', 'muted')
  }

  return {
    primaryHex,
    secondaryHex,
    accentHex,
    bgHex,
    surface1Hex,
    surface2Hex,
    cardHex,
    fgHex,
    mutedHex,
    borderHex,
    ringHex,
    successHex,
    warningHex,
    dangerHex,
    infoHex,
    brandRows,
    surfaceRows,
    textRows,
    semanticRows,
    seedDots,
    onRoleColor,
    onSeedGenerate,
  }
}
