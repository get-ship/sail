/**
 * THEMES-1 theme data-model schema lock (typed + generated constants).
 *
 * The freeze artifact is this schema — not the DTCG file format.
 * DTCG JSON is storage/exchange only; studio is the authoring surface.
 *
 * A theme = {
 *   name (themeId),
 *   modes: declared capability (light | dark | both),
 *   scale overrides (mode-body groups),
 *   component-token overrides (modes.*.component)
 * }
 *
 * Schema keys live in generated/themes-meta.ts; this module validates
 * documents against them and documents override order resolution.
 */

import {
  THEME_ROOT_KEYS,
  THEME_SAIL_KEYS,
  THEME_PAINT_MODES,
  THEME_MODE_GROUP_KEYS,
  THEME_OVERRIDE_ORDER,
  modeCapability as generatedModeCapability,
  themeMeta,
  type ModeCapability,
  type ThemeOverrideLayer,
  type ThemePaintMode,
} from './generated/themes-meta'

export {
  THEME_ROOT_KEYS,
  THEME_SAIL_KEYS,
  THEME_PAINT_MODES,
  THEME_MODE_GROUP_KEYS,
  THEME_OVERRIDE_ORDER,
  type ModeCapability,
  type ThemeOverrideLayer,
  type ThemePaintMode,
}

export type ThemeSchemaError = {
  path: string
  message: string
}

function closestKey(unknown: string, valid: readonly string[]): string | null {
  let best: { d: number; k: string } | null = null
  for (const k of valid) {
    const d = levenshtein(unknown, k)
    const maxD = Math.max(Math.floor(Math.max(k.length, unknown.length) / 3), 2)
    if (d <= maxD && (best === null || d < best.d)) best = { d, k }
  }
  return best?.k ?? null
}

function levenshtein(a: string, b: string): number {
  if (a === b) return 0
  if (!a.length) return b.length
  if (!b.length) return a.length
  const prev = Array.from({ length: b.length + 1 }, (_, i) => i)
  const cur = new Array<number>(b.length + 1)
  for (let i = 1; i <= a.length; i++) {
    cur[0] = i
    for (let j = 1; j <= b.length; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1
      cur[j] = Math.min(prev[j]! + 1, cur[j - 1]! + 1, prev[j - 1]! + cost)
    }
    for (let j = 0; j <= b.length; j++) prev[j] = cur[j]!
  }
  return prev[b.length]!
}

function unknownKeyMessage(where: string, key: string, valid: readonly string[]): string {
  const validList = valid.join(', ')
  const closest = closestKey(key, valid)
  if (closest) {
    return `unknown ${where} key "${key}" (closest valid key: "${closest}"; valid: ${validList})`
  }
  return `unknown ${where} key "${key}" (valid: ${validList})`
}

/**
 * Validate a theme document against the THEMES-1 schema lock.
 * Returns [] when ok; otherwise error messages that name valid keys.
 */
export function validateThemeDoc(doc: unknown, label = 'theme'): ThemeSchemaError[] {
  const errors: ThemeSchemaError[] = []
  if (!doc || typeof doc !== 'object' || Array.isArray(doc)) {
    return [{ path: '', message: `${label}: root must be a JSON object` }]
  }
  const obj = doc as Record<string, unknown>
  const rootValid = THEME_ROOT_KEYS as readonly string[]

  for (const key of Object.keys(obj)) {
    if (!rootValid.includes(key)) {
      errors.push({
        path: key,
        message: `${label}: ${unknownKeyMessage('root', key, rootValid)}`,
      })
    }
  }

  const ext = obj.$extensions
  if (ext !== undefined) {
    if (!ext || typeof ext !== 'object' || Array.isArray(ext)) {
      errors.push({ path: '$extensions', message: `${label}: $extensions must be an object` })
    } else {
      const extObj = ext as Record<string, unknown>
      for (const key of Object.keys(extObj)) {
        if (key !== 'sail') {
          errors.push({
            path: `$extensions.${key}`,
            message: `${label}: ${unknownKeyMessage('$extensions', key, ['sail'])}`,
          })
        }
      }
      const sail = extObj.sail
      if (sail !== undefined) {
        if (!sail || typeof sail !== 'object' || Array.isArray(sail)) {
          errors.push({
            path: '$extensions.sail',
            message: `${label}: $extensions.sail must be an object`,
          })
        } else {
          const sailObj = sail as Record<string, unknown>
          const sailValid = THEME_SAIL_KEYS as readonly string[]
          for (const key of Object.keys(sailObj)) {
            if (!sailValid.includes(key)) {
              errors.push({
                path: `$extensions.sail.${key}`,
                message: `${label}: ${unknownKeyMessage('$extensions.sail', key, sailValid)}`,
              })
            }
          }
          const dm = sailObj.defaultMode
          if (typeof dm === 'string' && !(THEME_PAINT_MODES as readonly string[]).includes(dm)) {
            errors.push({
              path: '$extensions.sail.defaultMode',
              message: `${label}: ${unknownKeyMessage('defaultMode', dm, THEME_PAINT_MODES)}`,
            })
          }
          const sm = sailObj.supportedModes
          if (Array.isArray(sm)) {
            for (const m of sm) {
              if (typeof m !== 'string') {
                errors.push({
                  path: '$extensions.sail.supportedModes',
                  message: `${label}: supportedModes entries must be strings`,
                })
              } else if (!(THEME_PAINT_MODES as readonly string[]).includes(m)) {
                errors.push({
                  path: '$extensions.sail.supportedModes',
                  message: `${label}: ${unknownKeyMessage('supportedModes', m, THEME_PAINT_MODES)}`,
                })
              }
            }
          }
          if (typeof sailObj.layer === 'string' && sailObj.layer !== 'theme') {
            errors.push({
              path: '$extensions.sail.layer',
              message: `${label}: $extensions.sail.layer must be "theme" (got ${JSON.stringify(sailObj.layer)})`,
            })
          }
          if (typeof sailObj.density === 'string') {
            if (!['compact', 'default', 'spacious'].includes(sailObj.density)) {
              errors.push({
                path: '$extensions.sail.density',
                message: `${label}: density must be compact|default|spacious (got ${JSON.stringify(sailObj.density)})`,
              })
            }
          }
        }
      }
    }
  }

  const modes = obj.modes
  if (modes !== undefined) {
    if (!modes || typeof modes !== 'object' || Array.isArray(modes)) {
      errors.push({ path: 'modes', message: `${label}: modes must be an object` })
    } else {
      const modesObj = modes as Record<string, unknown>
      const paintValid = THEME_PAINT_MODES as readonly string[]
      const groupValid = THEME_MODE_GROUP_KEYS as readonly string[]
      for (const [modeName, body] of Object.entries(modesObj)) {
        if (!paintValid.includes(modeName)) {
          errors.push({
            path: `modes.${modeName}`,
            message: `${label}: ${unknownKeyMessage('modes', modeName, paintValid)}`,
          })
          continue
        }
        if (!body || typeof body !== 'object' || Array.isArray(body)) {
          errors.push({
            path: `modes.${modeName}`,
            message: `${label}: modes.${modeName} must be an object`,
          })
          continue
        }
        for (const key of Object.keys(body as object)) {
          if (!groupValid.includes(key)) {
            errors.push({
              path: `modes.${modeName}.${key}`,
              message: `${label}: ${unknownKeyMessage(`modes.${modeName}`, key, groupValid)}`,
            })
          }
        }
      }
    }
  }

  return errors
}

/** True when validateThemeDoc returns no errors. */
export function isValidThemeDoc(doc: unknown): boolean {
  return validateThemeDoc(doc).length === 0
}

/** Derive ModeCapability from an explicit modes list. */
export function modeCapabilityFromModes(modes: readonly string[]): ModeCapability {
  const hasLight = modes.includes('light')
  const hasDark = modes.includes('dark')
  if (hasLight && hasDark) return 'both'
  if (hasDark) return 'dark'
  return 'light'
}

/** Capability for a catalog theme id (generated meta). Unknown → both. */
export function modeCapabilityFor(id: string): ModeCapability {
  return generatedModeCapability(id)
}

/**
 * Resolve which override layer won for a token probe.
 *
 * Layers (THEME_OVERRIDE_ORDER, highest first):
 *   instance → component → global → baked
 *
 * Callers supply presence flags for each layer; first true wins.
 */
export function resolveOverrideLayer(flags: {
  instance?: boolean
  component?: boolean
  global?: boolean
  baked?: boolean
}): ThemeOverrideLayer {
  for (const layer of THEME_OVERRIDE_ORDER) {
    if (flags[layer]) return layer
  }
  return 'baked'
}

/**
 * Studio-facing resolution for a token path against a theme doc mode table.
 * Instance is always caller-supplied (style/channels on the host).
 */
export function resolveTokenLayer(opts: {
  /** Instance (style/channels) override present on the element. */
  instance?: boolean
  /** Theme has modes.*.component.* leaf for this token. */
  componentOverride?: boolean
  /** Theme has modes.*.color|radius|… global scale leaf. */
  globalOverride?: boolean
}): { layer: ThemeOverrideLayer; order: typeof THEME_OVERRIDE_ORDER } {
  const layer = resolveOverrideLayer({
    instance: opts.instance === true,
    component: opts.componentOverride === true,
    global: opts.globalOverride === true,
    baked: true,
  })
  return { layer, order: THEME_OVERRIDE_ORDER }
}

/** Picker honesty: which mode tabs to offer for a theme. */
export function pickerModesForTheme(id: string): {
  capability: ModeCapability
  paintModes: readonly ThemePaintMode[]
  /** System preference only when capability is both. */
  allowSystem: boolean
} {
  const meta = themeMeta(id)
  const capability = meta?.capability ?? modeCapabilityFromModes(meta?.modes ?? ['light', 'dark'])
  const paintModes = (meta?.modes?.length
    ? meta.modes.filter((m): m is ThemePaintMode =>
        (THEME_PAINT_MODES as readonly string[]).includes(m),
      )
    : capability === 'dark'
      ? (['dark'] as const)
      : capability === 'light'
        ? (['light'] as const)
        : THEME_PAINT_MODES) as readonly ThemePaintMode[]
  return {
    capability,
    paintModes,
    allowSystem: capability === 'both',
  }
}
