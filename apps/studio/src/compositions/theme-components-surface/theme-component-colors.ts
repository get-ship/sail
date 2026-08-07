/**
 * Components facet color resolve — DTCG paths only for display.
 *
 * AUTHOR: component model aliases (`{color.surface.1}`) + theme overrides
 * Registry may store CSS exhaust (`var(--surface-1)`) after generate; we reverse
 * that via registry metadata → path, then read concrete values from the theme doc.
 * Never treat `var(--*)` as product copy.
 */
import {
  getKnob,
  getPathValue,
  tryPickerHex,
  type TokenKnob,
} from '@ship/studio/design'

/**
 * Recover a DTCG alias from a knob base value.
 * Model SSOT is `{color.surface.1}`; generated registry often has `var(--surface-1)`.
 */
export function dtcgDefaultAlias(knob: TokenKnob): string {
  const base = String(knob.base ?? '').trim()
  if (!base) return ''
  if (base.startsWith('{') && base.endsWith('}')) return base
  const cssVar = /^var\(\s*--([a-z0-9-]+)\s*\)$/i.exec(base)
  if (cssVar) {
    const ref = getKnob(cssVar[1]!)
    if (ref?.path) return `{${ref.path}}`
  }
  return base
}

/**
 * Resolve a color string to a concrete oklch/hex value via the theme doc.
 * Follows `{token.path}` aliases and reverse-maps `var(--css-name)` through registry paths.
 * Does not invent colors.
 */
export function resolveColorRaw(
  doc: Record<string, unknown> | null,
  raw: string,
  mode: 'light' | 'dark',
  depth = 0,
): string | null {
  if (!raw || depth > 8) return null
  const v = raw.trim()
  if (!v) return null

  // Direct paint value (oklch / hex / rgb…)
  if (tryPickerHex(v)) return v
  if (
    v.startsWith('oklch(') ||
    v.startsWith('#') ||
    v.startsWith('rgb') ||
    v.startsWith('hsl')
  ) {
    // May still be unparseable (e.g. oklch with var(--brand-hue)) — return for honesty
    return v
  }

  // DTCG alias {color.surface.1}
  const alias = /^\{([^}]+)\}$/.exec(v)
  if (alias) {
    const next = getPathValue(doc, alias[1]!.trim(), mode)
    if (next) return resolveColorRaw(doc, next, mode, depth + 1)
    return null
  }

  // Registry / exhaust: var(--surface-1) → registry path → theme leaf
  const cssVar = /^var\(\s*--([a-z0-9-]+)\s*(?:,[^)]+)?\)$/i.exec(v)
  if (cssVar) {
    const cssName = cssVar[1]!
    const knob = getKnob(cssName)
    if (knob?.path) {
      const next = getPathValue(doc, knob.path, mode)
      if (next) return resolveColorRaw(doc, next, mode, depth + 1)
      // Theme missing leaf: follow registry base if it is a different form
      const base = String(knob.base ?? '').trim()
      if (base && base !== v) return resolveColorRaw(doc, base, mode, depth + 1)
    }
    // Heuristic path guesses when registry entry is missing
    if (cssName.startsWith('surface-')) {
      const next = getPathValue(doc, `color.surface.${cssName.slice('surface-'.length)}`, mode)
      if (next) return resolveColorRaw(doc, next, mode, depth + 1)
    }
    if (cssName === 'foreground' || cssName === 'background') {
      const next =
        getPathValue(doc, `color.${cssName}`, mode) ??
        getPathValue(doc, `color.${cssName}.default`, mode)
      if (next) return resolveColorRaw(doc, next, mode, depth + 1)
    }
    if (cssName === 'muted-foreground') {
      const next = getPathValue(doc, 'color.muted.foreground', mode)
      if (next) return resolveColorRaw(doc, next, mode, depth + 1)
    }
    if (cssName === 'muted') {
      const next =
        getPathValue(doc, 'color.muted.default', mode) ?? getPathValue(doc, 'color.muted', mode)
      if (next) return resolveColorRaw(doc, next, mode, depth + 1)
    }
    if (cssName === 'primary') {
      const next = getPathValue(doc, 'color.primary.default', mode)
      if (next) return resolveColorRaw(doc, next, mode, depth + 1)
    }
    return null
  }

  return null
}

export type EffectiveColor = {
  /** Hex for swatch/picker — empty when unresolved (never invent mid-gray / fake white). */
  hex: string
  /** Theme has an explicit component.* leaf */
  isOverridden: boolean
  /** Raw string for diagnostics (never shown as product when it is var(--*)) */
  raw: string
  /** Short UI label under/near swatch — never contains `var(--` */
  hexLabel: string
  /** DTCG path the default aliases (for honesty), empty when override or unknown */
  defaultPath: string
  canPickColor: boolean
}

function formatDefaultLabel(hex: string, defaultPath: string): string {
  if (hex) return `Default · ${hex}`
  if (defaultPath) return `Default · ${defaultPath}`
  return 'Default'
}

function formatOverrideLabel(hex: string, raw: string): string {
  if (hex) return hex
  // Override present but not hex-convertible (oklch with var, color-mix, …)
  const clean = raw.trim()
  if (!clean || /var\s*\(/i.test(clean)) return 'Override (unresolved)'
  if (clean.length > 48) return `${clean.slice(0, 45)}…`
  return clean
}

/**
 * Effective color for a component knob:
 * 1) theme override at path
 * 2) else model/registry default as DTCG alias, resolved through open theme colors
 *
 * Defaults are NOT in project theme JSON until the user overrides — missing leaf ≠ broken pipeline.
 */
export function effectiveComponentColor(
  doc: Record<string, unknown> | null,
  knob: TokenKnob,
  mode: 'light' | 'dark',
): EffectiveColor {
  const override = getPathValue(doc, knob.path, mode)
  if (override) {
    const concrete = resolveColorRaw(doc, override, mode) ?? override
    const hex = tryPickerHex(concrete) ?? ''
    return {
      hex,
      isOverridden: true,
      raw: override,
      hexLabel: formatOverrideLabel(hex, override),
      defaultPath: '',
      canPickColor: true,
    }
  }

  const alias = dtcgDefaultAlias(knob)
  if (!alias) {
    return {
      hex: '',
      isOverridden: false,
      raw: '',
      hexLabel: 'Default',
      defaultPath: '',
      canPickColor: true,
    }
  }

  const defaultPath = alias.startsWith('{') ? alias.slice(1, -1) : ''
  const concrete = resolveColorRaw(doc, alias, mode)
  const hex = concrete ? tryPickerHex(concrete) ?? '' : ''

  // Never surface CSS exhaust as product copy
  const safeRaw = /var\s*\(/i.test(alias) ? defaultPath || 'theme' : alias

  return {
    hex,
    isOverridden: false,
    raw: safeRaw,
    hexLabel: formatDefaultLabel(hex, defaultPath),
    defaultPath,
    canPickColor: true,
  }
}

/** Global theme path (color.*) for the "related globals" inspector rows. */
export function effectiveGlobalColor(
  doc: Record<string, unknown> | null,
  path: string,
  mode: 'light' | 'dark',
): EffectiveColor {
  const raw = getPathValue(doc, path, mode)
  if (!raw) {
    return {
      hex: '',
      isOverridden: false,
      raw: '',
      hexLabel: 'Default',
      defaultPath: path,
      canPickColor: true,
    }
  }
  const concrete = resolveColorRaw(doc, raw, mode) ?? raw
  const hex = tryPickerHex(concrete) ?? ''
  return {
    hex,
    isOverridden: true,
    raw,
    hexLabel: hex || formatOverrideLabel('', raw),
    defaultPath: '',
    canPickColor: true,
  }
}

/** Guard: product labels must never carry CSS custom-property exhaust. */
export function assertNoCssExhaustLabel(label: string): boolean {
  return !/var\s*\(\s*--/i.test(label)
}
