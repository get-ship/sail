/**
 * Brand color helpers — OKLCH ↔ hex for sail-color-picker, ramp + contrast.
 * Tokens stay OKLCH; hex is picker-only.
 */

export type Oklch = { L: number; C: number; H: number; a?: number | undefined }

export function parseOklch(s: string): Oklch | null {
  const m = /^oklch\(\s*([\d.]+%?)\s+([\d.]+)\s+([\d.]+)(?:\s*\/\s*([\d.]+%?))?\s*\)/i.exec(
    String(s ?? '').trim(),
  )
  if (!m) return null
  const Lraw = m[1]!
  const L = Lraw.endsWith('%') ? Number(Lraw.slice(0, -1)) / 100 : Number(Lraw)
  const C = Number(m[2])
  const H = Number(m[3])
  let a: number | undefined
  if (m[4] != null) {
    const ar = m[4]
    a = ar.endsWith('%') ? Number(ar.slice(0, -1)) / 100 : Number(ar)
  }
  if (![L, C, H].every(Number.isFinite)) return null
  return { L, C, H, a }
}

export function formatOklch(c: Oklch): string {
  const fmt = (n: number) => String(Math.round(n * 1000) / 1000)
  if (c.a != null && c.a < 1) {
    return `oklch(${fmt(c.L)} ${fmt(c.C)} ${fmt(c.H)} / ${fmt(c.a)})`
  }
  return `oklch(${fmt(c.L)} ${fmt(c.C)} ${fmt(c.H)})`
}

const clamp255 = (n: number) => Math.max(0, Math.min(255, Math.round(n)))

function srgbGamma(c: number): number {
  return c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055
}

function oklchToRgb(c: Oklch): { r: number; g: number; b: number } {
  const hr = (c.H * Math.PI) / 180
  const a = c.C * Math.cos(hr)
  const bb = c.C * Math.sin(hr)
  const l_ = c.L + 0.3963377774 * a + 0.2158037573 * bb
  const m_ = c.L - 0.1055613458 * a - 0.0638541728 * bb
  const s_ = c.L - 0.0894841775 * a - 1.291485548 * bb
  const lc = l_ ** 3
  const mc = m_ ** 3
  const sc = s_ ** 3
  const r = 4.0767416621 * lc - 3.3077115913 * mc + 0.2309699292 * sc
  const g = -1.2684380046 * lc + 2.6097574011 * mc - 0.3413193965 * sc
  const b = -0.0041960863 * lc - 0.7034186147 * mc + 1.707614701 * sc
  return {
    r: clamp255(srgbGamma(r) * 255),
    g: clamp255(srgbGamma(g) * 255),
    b: clamp255(srgbGamma(b) * 255),
  }
}

function rgbToHex(r: number, g: number, b: number): string {
  const to2 = (n: number) => clamp255(n).toString(16).padStart(2, '0')
  return `#${to2(r)}${to2(g)}${to2(b)}`
}

export function isHex(s: string): boolean {
  return /^#[0-9a-f]{6}$/i.test(String(s ?? '').trim())
}

export function normHex(s: string): string {
  const t = String(s ?? '').trim().toLowerCase()
  if (/^#[0-9a-f]{6}$/.test(t)) return t
  if (/^[0-9a-f]{6}$/.test(t)) return `#${t}`
  if (/^#[0-9a-f]{3}$/.test(t)) {
    return `#${t[1]}${t[1]}${t[2]}${t[2]}${t[3]}${t[3]}`
  }
  return ''
}

/**
 * OKLCH (or hex) → `#rrggbb` for sail-color-picker.
 * Returns null when the value cannot be converted — **never** invent mid-gray.
 * Callers must treat null as "unresolved / not pickable as hex".
 */
export function tryPickerHex(value: string): string | null {
  const v = String(value ?? '').trim()
  if (!v) return null
  if (isHex(v)) return v.toLowerCase()
  const n = normHex(v)
  if (n) return n
  // color-mix / var() / multi-shadow — not a single swatch hex
  if (/var\(|color-mix\(|gradient\(|url\(/i.test(v)) return null
  const ok = parseOklch(v)
  if (!ok) return null
  const rgb = oklchToRgb(ok)
  return rgbToHex(rgb.r, rgb.g, rgb.b)
}

/**
 * OKLCH (or hex) → `#rrggbb`. Empty string when unconvertible (honest UI).
 * Prefer {@link tryPickerHex} when you need null vs empty.
 */
export function toPickerHex(value: string): string {
  return tryPickerHex(value) ?? ''
}

const lin = (c: number): number => {
  const x = c / 255
  return x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4
}

/** `#rrggbb` → oklch string for token write-back. Throws on invalid hex. */
export function hexToOklchString(hex: string): string {
  const h = normHex(hex) || hex
  const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(h)
  if (!m) throw new Error('invalid hex color')
  const r = lin(parseInt(m[1]!, 16))
  const g = lin(parseInt(m[2]!, 16))
  const b = lin(parseInt(m[3]!, 16))
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
  const mm = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
  const L = 0.2104542553 * l + 0.793617785 * mm - 0.0040720468 * s
  const a = 1.9779984951 * l - 2.428592205 * mm + 0.4505937099 * s
  const bb = 0.0259040371 * l + 0.7827717662 * mm - 0.808675766 * s
  const C = Math.sqrt(a * a + bb * bb)
  let H = (Math.atan2(bb, a) * 180) / Math.PI
  if (H < 0) H += 360
  return formatOklch({ L, C, H })
}

export type RampStep = {
  key: string
  label: string
  value: string
  hex: string
}

/** Derive a 5-step brand ramp around primary (monotonic L). */
export function deriveBrandRamp(primary: string): RampStep[] {
  const ok = parseOklch(primary) ?? { L: 0.62, C: 0.16, H: 38 }
  const steps: { key: string; label: string; dL: number; dC: number }[] = [
    { key: 'soft', label: 'Soft', dL: 0.22, dC: -0.04 },
    { key: 'light', label: 'Light', dL: 0.1, dC: -0.01 },
    { key: 'base', label: 'Brand', dL: 0, dC: 0 },
    { key: 'deep', label: 'Deep', dL: -0.1, dC: 0.02 },
    { key: 'ink', label: 'Ink', dL: -0.22, dC: 0.01 },
  ]
  return steps.map((s) => {
    const L = Math.max(0.08, Math.min(0.96, ok.L + s.dL))
    const C = Math.max(0, ok.C + s.dC)
    const value = formatOklch({ L, C, H: ok.H })
    return { key: s.key, label: s.label, value, hex: toPickerHex(value) }
  })
}

function relativeLuminance(hex: string): number {
  const h = normHex(hex) || hex
  const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(h)
  if (!m) return 0
  const r = lin(parseInt(m[1]!, 16))
  const g = lin(parseInt(m[2]!, 16))
  const b = lin(parseInt(m[3]!, 16))
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
}

/** WCAG contrast ratio between two color strings (oklch/hex). 0 when either is unconvertible. */
export function contrastRatio(fg: string, bg: string): number {
  const a = tryPickerHex(fg)
  const b = tryPickerHex(bg)
  if (!a || !b) return 0
  const L1 = relativeLuminance(a)
  const L2 = relativeLuminance(b)
  const lighter = Math.max(L1, L2)
  const darker = Math.min(L1, L2)
  return (lighter + 0.05) / (darker + 0.05)
}

export type ContrastFlag = {
  id: string
  label: string
  ratio: number
  passAA: boolean
}

/** Check brand pairs used by specimen controls. */
export function brandContrastFlags(opts: {
  primary: string
  primaryFg: string
  background: string
  foreground: string
}): ContrastFlag[] {
  const pairs: { id: string; label: string; fg: string; bg: string }[] = [
    {
      id: 'primary-on-bg',
      label: 'Brand on page',
      fg: opts.primary,
      bg: opts.background,
    },
    {
      id: 'fg-on-primary',
      label: 'Label on brand',
      fg: opts.primaryFg,
      bg: opts.primary,
    },
    {
      id: 'body-on-bg',
      label: 'Body on page',
      fg: opts.foreground,
      bg: opts.background,
    },
  ]
  return pairs.map((p) => {
    const ratio = Math.round(contrastRatio(p.fg, p.bg) * 100) / 100
    return {
      id: p.id,
      label: p.label,
      ratio,
      passAA: ratio >= 4.5,
    }
  })
}

export const BRAND_PRIMARY_PATH = 'color.primary.default'
export const BRAND_FG_PATH = 'color.primary.foreground'
