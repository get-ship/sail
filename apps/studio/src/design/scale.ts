
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
 */
export function setScaleStep(
  family: ScaleFamily,
  key: string,
  nextPx: number,
): { family: ScaleFamily; error: string | null } {
  const px = Math.max(0, nextPx)
  const idx = family.steps.findIndex((s) => s.key === key)
  if (idx < 0) {
    return { family, error: "That size step isn't available" }
  }
  let steps: ScaleStep[]
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
    // After proportional scale, enforce non-decreasing with light fix-up.
    steps = enforceMonotonic(steps, family.ordered)
  } else {
    steps = family.steps.map((s) =>
      s.key === key ? { key: s.key, valuePx: px, raw: formatPx(px) } : s,
    )
  }
  const next: ScaleFamily = { ...family, steps }
  const error = validateMonotonic(next.steps, next.ordered, next.path)
  return { family: next, error }
}

/** Nudge values so ordered steps are non-decreasing (min bump 0). */
export function enforceMonotonic(steps: ScaleStep[], ordered: string[]): ScaleStep[] {
  const byKey = new Map(steps.map((s) => [s.key, { ...s }]))
  let prevPx = -Infinity
  for (const key of ordered) {
    const s = byKey.get(key)
    if (!s) continue
    if (s.valuePx < prevPx) {
      s.valuePx = prevPx
      s.raw = formatPx(prevPx)
    }
    prevPx = s.valuePx
  }
  return steps.map((s) => byKey.get(s.key) ?? s)
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

/** Apply scale family into a mutable theme doc at `family.path` (e.g. radius, icon). */
export function applyScaleToDoc(
  doc: Record<string, unknown>,
  family: ScaleFamily,
): Record<string, unknown> {
  const next = structuredClone(doc) as Record<string, unknown>
  const key = family.path
  const group: Record<string, unknown> =
    next[key] && typeof next[key] === 'object' && !Array.isArray(next[key])
      ? { ...(next[key] as Record<string, unknown>) }
      : {}
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
  next[key] = group
  return next
}

/** Apply scale family into a mutable theme doc (root-level radius). */
export function applyRadiusToDoc(
  doc: Record<string, unknown>,
  family: ScaleFamily,
): Record<string, unknown> {
  return applyScaleToDoc(doc, { ...family, path: 'radius' })
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
