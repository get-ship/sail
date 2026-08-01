
import {
  ICON_ORDERED,
  RADIUS_ORDERED,
  type ScaleFamily,
  type ScaleStep,
} from './types'

export const NON_MONOTONIC_ERROR = 'NonMonotonicScale'

/** Parse dimension-like value to px (aliases return null). */
export function parseDimensionPx(raw: string): number | null {
  const s = String(raw).trim()
  if (!s || s.startsWith('{')) return null
  if (s.endsWith('px')) {
    const n = Number(s.slice(0, -2).trim())
    return Number.isFinite(n) ? n : null
  }
  if (s.endsWith('rem')) {
    const n = Number(s.slice(0, -3).trim())
    return Number.isFinite(n) ? n * 16 : null
  }
  if (s.endsWith('em')) {
    const n = Number(s.slice(0, -2).trim())
    return Number.isFinite(n) ? n * 16 : null
  }
  const n = Number(s)
  return Number.isFinite(n) ? n : null
}

export function formatPx(n: number): string {
  const r = Math.round(n * 1000) / 1000
  return Number.isInteger(r) ? `${r}px` : `${r}px`
}

/**
 * True when each consecutive ordered step is non-decreasing.
 * Missing / alias steps are skipped (same as ops).
 */
export function isMonotonic(steps: ScaleStep[], ordered: string[]): boolean {
  return validateMonotonic(steps, ordered) === null
}

/** Named error message matching ops, or null if ok. */
export function validateMonotonic(
  steps: ScaleStep[],
  ordered: string[],
  path = 'radius',
): string | null {
  const byKey = new Map(steps.map((s) => [s.key, s]))
  let prev: ScaleStep | null = null
  for (const key of ordered) {
    const step = byKey.get(key)
    if (!step || step.valuePx == null || Number.isNaN(step.valuePx)) continue
    if (prev && step.valuePx + Number.EPSILON < prev.valuePx) {
      return `${NON_MONOTONIC_ERROR}: ${path} step ${key}=${step.raw} is less than ${prev.key}=${prev.raw} (ordered requires non-decreasing)`
    }
    prev = step
  }
  return null
}

/** Default radius family (base model values). */
export function defaultRadiusFamily(scaleTogether = true): ScaleFamily {
  const defaults: Record<string, number> = { sm: 4, md: 6, lg: 10, xl: 14 }
  const ordered = [...RADIUS_ORDERED]
  return {
    path: 'radius',
    ordered,
    scaleTogether,
    steps: ordered.map((key) => ({
      key,
      valuePx: defaults[key],
      raw: formatPx(defaults[key]),
    })),
  }
}

/**
 * Set one step. When scaleTogether, scale all steps proportionally around
 * the dragged key's previous value; then clamp to keep monotonic.
 * `clampNote` explains neighbor clamps in plain language (never silent).
 */
export function setScaleStep(
  family: ScaleFamily,
  key: string,
  nextPx: number,
): { family: ScaleFamily; error: string | null; clampNote: string | null } {
  const px = Math.max(0, nextPx)
  const idx = family.steps.findIndex((s) => s.key === key)
  if (idx < 0) {
    return { family, error: "That size step isn't available", clampNote: null }
  }
  let steps: ScaleStep[]
  let clampNote: string | null = null
  if (family.scaleTogether) {
    const prev = family.steps[idx].valuePx
    const factor = prev > 0 ? px / prev : 1
    steps = family.steps.map((s, i) => {
      if (i === idx) {
        return { key: s.key, valuePx: px, raw: formatPx(px) }
      }
      const v = Math.max(0, Math.round(s.valuePx * factor * 1000) / 1000)
      return { key: s.key, valuePx: v, raw: formatPx(v) }
    })
    const enforced = enforceMonotonicWithNotes(steps, family.ordered)
    steps = enforced.steps
    clampNote = enforced.note
  } else {
    steps = family.steps.map((s) =>
      s.key === key ? { key: s.key, valuePx: px, raw: formatPx(px) } : s,
    )
    // Neighbor clamp so the dragged step can't break order, then full-family fix-up.
    const ordered = family.ordered
    const oIdx = ordered.indexOf(key)
    if (oIdx >= 0) {
      const byKey = new Map(steps.map((s) => [s.key, s]))
      const prevKey = oIdx > 0 ? ordered[oIdx - 1] : null
      const nextKey = oIdx < ordered.length - 1 ? ordered[oIdx + 1] : null
      const prevPx = prevKey != null ? byKey.get(prevKey)?.valuePx : null
      const nextPxVal = nextKey != null ? byKey.get(nextKey)?.valuePx : null
      let clamped = px
      if (prevPx != null && clamped < prevPx) {
        clamped = prevPx
        clampNote = `${key} can't go below ${prevKey} — adjusted`
      }
      if (nextPxVal != null && clamped > nextPxVal) {
        clamped = nextPxVal
        clampNote = `${key} can't exceed ${nextKey} — adjusted`
      }
      if (clamped !== px) {
        steps = steps.map((s) =>
          s.key === key ? { key: s.key, valuePx: clamped, raw: formatPx(clamped) } : s,
        )
      }
    }
    const enforced = enforceMonotonicWithNotes(steps, family.ordered)
    steps = enforced.steps
    if (enforced.note) clampNote = clampNote ?? enforced.note
  }
  const next: ScaleFamily = { ...family, steps }
  const error = validateMonotonic(next.steps, next.ordered, next.path)
  return { family: next, error, clampNote }
}

/** Nudge values so ordered steps are non-decreasing (min bump 0). */
export function enforceMonotonic(steps: ScaleStep[], ordered: string[]): ScaleStep[] {
  return enforceMonotonicWithNotes(steps, ordered).steps
}

function enforceMonotonicWithNotes(
  steps: ScaleStep[],
  ordered: string[],
): { steps: ScaleStep[]; note: string | null } {
  const byKey = new Map(steps.map((s) => [s.key, { ...s }]))
  let prevPx = -Infinity
  let prevKey = ''
  let note: string | null = null
  for (const key of ordered) {
    const s = byKey.get(key)
    if (!s) continue
    if (s.valuePx < prevPx) {
      note = `${key} can't go below ${prevKey} — adjusted`
      s.valuePx = prevPx
      s.raw = formatPx(prevPx)
    }
    prevPx = s.valuePx
    prevKey = key
  }
  return { steps: steps.map((s) => byKey.get(s.key) ?? s), note }
}

/** Build ScaleFamily from a DTCG radius object. */
export function radiusFamilyFromDoc(
  radiusObj: Record<string, unknown> | null | undefined,
  scaleTogether = true,
): ScaleFamily {
  const base = defaultRadiusFamily(scaleTogether)
  if (!radiusObj || typeof radiusObj !== 'object') return base
  const ext = radiusObj.$extensions as Record<string, unknown> | undefined
  let ordered = base.ordered
  if (ext && Array.isArray(ext.ordered)) {
    ordered = (ext.ordered as unknown[]).filter((x): x is string => typeof x === 'string')
  }
  const steps: ScaleStep[] = []
  for (const key of ordered) {
    const step = radiusObj[key] as { $value?: unknown } | undefined
    const raw = step && step.$value != null ? String(step.$value) : null
    const px = raw != null ? parseDimensionPx(raw) : null
    const fallback = base.steps.find((s) => s.key === key)
    steps.push({
      key,
      valuePx: px ?? fallback?.valuePx ?? 0,
      raw: raw ?? fallback?.raw ?? '0px',
    })
  }
  return { path: 'radius', ordered, scaleTogether, steps }
}

/**
 * Where to write a scale group for modes-in-file themes.
 * Prefer existing active-mode group, then root if no modes; never the inactive mode.
 */
export function resolveScaleWritePath(
  doc: Record<string, unknown>,
  groupKey: string,
  modeId: 'light' | 'dark' = 'light',
): string {
  const modes = doc.modes as Record<string, unknown> | undefined
  const active = modes?.[modeId] as Record<string, unknown> | undefined
  if (active?.[groupKey] && typeof active[groupKey] === 'object') {
    return `modes.${modeId}.${groupKey}`
  }
  if (doc[groupKey] && typeof doc[groupKey] === 'object' && !Array.isArray(doc[groupKey])) {
    // Modes-in-file themes should not accumulate dual root + mode sources.
    if (modes && typeof modes === 'object') {
      return `modes.${modeId}.${groupKey}`
    }
    return groupKey
  }
  if (modes && typeof modes === 'object') {
    return `modes.${modeId}.${groupKey}`
  }
  return groupKey
}

function setNestedGroup(
  doc: Record<string, unknown>,
  dottedPath: string,
  group: Record<string, unknown>,
): void {
  const parts = dottedPath.split('.')
  let cur: Record<string, unknown> = doc
  for (let i = 0; i < parts.length - 1; i++) {
    const p = parts[i]!
    const child = cur[p]
    if (!child || typeof child !== 'object' || Array.isArray(child)) {
      cur[p] = {}
    }
    cur = cur[p] as Record<string, unknown>
  }
  cur[parts[parts.length - 1]!] = group
}

function getNestedGroup(
  doc: Record<string, unknown>,
  dottedPath: string,
): Record<string, unknown> | null {
  const parts = dottedPath.split('.')
  let cur: unknown = doc
  for (const p of parts) {
    if (!cur || typeof cur !== 'object' || Array.isArray(cur)) return null
    cur = (cur as Record<string, unknown>)[p]
  }
  if (!cur || typeof cur !== 'object' || Array.isArray(cur)) return null
  return cur as Record<string, unknown>
}

/** Apply scale family into a theme doc at root or modes.{modeId}.{path}. */
export function applyScaleToDoc(
  doc: Record<string, unknown>,
  family: ScaleFamily,
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> {
  const next = structuredClone(doc) as Record<string, unknown>
  const key = family.path
  const target = resolveScaleWritePath(next, key, modeId)
  const prevGroup = getNestedGroup(next, target) ?? {}
  const group: Record<string, unknown> = { ...prevGroup }
  for (const step of family.steps) {
    const prev = (group[step.key] as Record<string, unknown> | undefined) ?? {}
    group[step.key] = {
      ...prev,
      $value: step.raw,
      $type: prev.$type ?? 'dimension',
    }
  }
  group.$extensions = {
    ...((group.$extensions as object) ?? {}),
    ordered: family.ordered,
  }
  setNestedGroup(next, target, group)
  // Drop stale root copy when writing under modes (avoids dual sources).
  if (target.startsWith('modes.') && next[key]) {
    delete next[key]
  }
  return next
}

/** Apply radius scale family (mode-aware). */
export function applyRadiusToDoc(
  doc: Record<string, unknown>,
  family: ScaleFamily,
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> {
  return applyScaleToDoc(doc, { ...family, path: 'radius' }, modeId)
}

/** Default icon size family (sm…xl subset). */
export function defaultIconFamily(scaleTogether = true): ScaleFamily {
  const defaults: Record<string, number> = { sm: 14, md: 16, lg: 18, xl: 20 }
  const ordered = [...ICON_ORDERED]
  return {
    path: 'icon',
    ordered,
    scaleTogether,
    steps: ordered.map((key) => ({
      key,
      valuePx: defaults[key],
      raw: formatPx(defaults[key]),
    })),
  }
}

/** Build icon ScaleFamily from a DTCG icon object. */
export function iconFamilyFromDoc(
  iconObj: Record<string, unknown> | null | undefined,
  scaleTogether = true,
): ScaleFamily {
  const base = defaultIconFamily(scaleTogether)
  if (!iconObj || typeof iconObj !== 'object') return base
  const ext = iconObj.$extensions as Record<string, unknown> | undefined
  let ordered = base.ordered
  if (ext && Array.isArray(ext.ordered)) {
    ordered = (ext.ordered as unknown[]).filter((x): x is string => typeof x === 'string')
  }
  const steps: ScaleStep[] = []
  for (const key of ordered) {
    const step = iconObj[key] as { $value?: unknown } | undefined
    const raw = step && step.$value != null ? String(step.$value) : null
    const px = raw != null ? parseDimensionPx(raw) : null
    const fallback = base.steps.find((s) => s.key === key)
    steps.push({
      key,
      valuePx: px ?? fallback?.valuePx ?? 0,
      raw: raw ?? fallback?.raw ?? '0px',
    })
  }
  return { path: 'icon', ordered, scaleTogether, steps }
}
