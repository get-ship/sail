// Shared chart helpers. Kept tiny — no d3, no deps.
// Tone → CSS var, data parsing, path math.

export type ChartTone =
  | 'primary'
  | 'info'
  | 'success'
  | 'warning'
  | 'danger'
  | 'accent'
  | 'muted'

const DEFAULT_TONE = 'var(--foreground)'

const TONE_VARS: Record<string, string> = {
  primary: 'var(--primary)',
  info: 'var(--info)',
  success: 'var(--success)',
  warning: 'var(--warning)',
  danger: 'var(--danger)',
  accent: 'var(--accent)',
  muted: 'var(--muted-foreground)',
}

export function toneColor(tone: string | undefined | null): string {
  if (!tone) return DEFAULT_TONE
  return TONE_VARS[tone] ?? DEFAULT_TONE
}

export function parseNums(s: string | null | undefined): number[] {
  if (!s) return []
  return s
    .split(',')
    .map((p) => Number.parseFloat(p.trim()))
    .filter((n) => Number.isFinite(n))
}

export function parseLabels(s: string | null | undefined): string[] {
  if (!s) return []
  return s.split(',').map((p) => p.trim())
}

export function parseJson<T>(s: string | null | undefined, fallback: T): T {
  if (!s) return fallback
  try {
    return JSON.parse(s) as T
  } catch {
    return fallback
  }
}

export function linePath(
  data: number[],
  w: number,
  h: number,
  pad = 2,
): { d: string; points: Array<[number, number]> } {
  if (data.length === 0) return { d: '', points: [] }
  const min = Math.min(...data)
  const max = Math.max(...data)
  const range = max - min || 1
  const innerH = h - pad * 2
  const step = data.length === 1 ? 0 : w / (data.length - 1)
  const points: Array<[number, number]> = data.map((v, i) => {
    const x = data.length === 1 ? w / 2 : i * step
    const y = pad + innerH - ((v - min) / range) * innerH
    return [x, y]
  })
  const d = points
    .map(([x, y], i) => `${i === 0 ? 'M' : 'L'}${x.toFixed(2)},${y.toFixed(2)}`)
    .join(' ')
  return { d, points }
}

export function polar(cx: number, cy: number, r: number, angleRad: number): [number, number] {
  return [cx + r * Math.cos(angleRad), cy + r * Math.sin(angleRad)]
}

// ── extent / scale / normalize ──────────────────────────────────────────────
// Consolidated from the min/max/normalize math that was duplicated inline in
// every chart controller. Pure scalar helpers; the engine's `scales.ts` wires
// them into the reactive `derived` chain. `null` is a GAP (see types.ChartValue).

/**
 * The finite y-domain of a value list. `null`/non-finite values are ignored.
 * `count` is the number of finite points. An all-null/empty list yields the
 * degenerate `{ min: 0, max: 0, count: 0 }`.
 */
export function extent(values: ReadonlyArray<number | null>): {
  min: number
  max: number
  count: number
} {
  let min = Infinity
  let max = -Infinity
  let count = 0
  for (const v of values) {
    if (v == null || !Number.isFinite(v)) continue
    if (v < min) min = v
    if (v > max) max = v
    count++
  }
  if (count === 0) return { min: 0, max: 0, count: 0 }
  return { min, max, count }
}

/**
 * Linear scale: map `v` from the `[domainMin, domainMax]` domain onto
 * `0..1`. A zero-width domain maps every finite value to `0`. Returns `null`
 * for a `null`/non-finite input (gap preserved).
 */
export function scaleLinear(
  v: number | null,
  domainMin: number,
  domainMax: number,
): number | null {
  if (v == null || !Number.isFinite(v)) return null
  const range = domainMax - domainMin
  if (range === 0) return 0
  return (v - domainMin) / range
}

/**
 * Map a value list to `0..1` of its own (or a supplied) domain, preserving
 * `null` gaps. When `domain` is omitted the list's finite extent is used.
 */
export function normalize(
  values: ReadonlyArray<number | null>,
  domain?: { min: number; max: number },
): Array<number | null> {
  const d = domain ?? extent(values)
  return values.map((v) => scaleLinear(v, d.min, d.max))
}

export function arcPath(
  cx: number,
  cy: number,
  r: number,
  startAngle: number,
  endAngle: number,
): string {
  const [x1, y1] = polar(cx, cy, r, startAngle)
  const [x2, y2] = polar(cx, cy, r, endAngle)
  const large = endAngle - startAngle > Math.PI ? 1 : 0
  const sweep = endAngle > startAngle ? 1 : 0
  return `M${x1.toFixed(2)},${y1.toFixed(2)} A${r},${r} 0 ${large} ${sweep} ${x2.toFixed(2)},${y2.toFixed(2)}`
}
