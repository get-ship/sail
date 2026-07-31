// Color conversion + theme-token resolution for <sail-color-picker>.
// The picker's value model is a 7-char `#rrggbb` hex, so the swatch palette
// has to resolve concrete hex strings. Instead of a hardcoded brand palette
// (which would ignore the active theme), the swatches are DERIVED at runtime:
// a deterministic neutral ramp built from HSL math (always valid, theme-free)
// plus the design-system tone tokens resolved through computed style. The
// platform is OKLCH-native, so token resolution handles both the browser's
// `rgb(...)` normalization and a raw `oklch(...)` literal (jsdom does not
// resolve oklch, but the neutral ramp keeps the palette non-empty there).

export interface Swatch {
  hex: string
  label: string
}

const clamp255 = (n: number): number => Math.max(0, Math.min(255, Math.round(n)))

export const isHex = (s: string): boolean => /^#[0-9a-f]{6}$/i.test(s)

export const normHex = (s: string): string => {
  const t = s.trim().toLowerCase()
  if (/^#[0-9a-f]{6}$/.test(t)) return t
  if (/^[0-9a-f]{6}$/.test(t)) return `#${t}`
  if (/^#[0-9a-f]{3}$/.test(t)) {
    return `#${t[1]}${t[1]}${t[2]}${t[2]}${t[3]}${t[3]}`
  }
  return ''
}

export const hexToRgb = (hex: string): { r: number; g: number; b: number } | null => {
  if (!isHex(hex)) return null
  return {
    r: parseInt(hex.slice(1, 3), 16),
    g: parseInt(hex.slice(3, 5), 16),
    b: parseInt(hex.slice(5, 7), 16),
  }
}

export const rgbToHex = (r: number, g: number, b: number): string => {
  const to2 = (n: number) => clamp255(n).toString(16).padStart(2, '0')
  return `#${to2(r)}${to2(g)}${to2(b)}`
}

export const rgbToHsl = (r: number, g: number, b: number): { h: number; s: number; l: number } => {
  const rn = r / 255, gn = g / 255, bn = b / 255
  const max = Math.max(rn, gn, bn)
  const min = Math.min(rn, gn, bn)
  const l = (max + min) / 2
  let h = 0, s = 0
  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    if (max === rn) h = ((gn - bn) / d + (gn < bn ? 6 : 0))
    else if (max === gn) h = (bn - rn) / d + 2
    else h = (rn - gn) / d + 4
    h *= 60
  }
  return { h: Math.round(h), s: Math.round(s * 100), l: Math.round(l * 100) }
}

export const hslToHex = (h: number, s: number, l: number): string => {
  const sn = s / 100, ln = l / 100
  const c = (1 - Math.abs(2 * ln - 1)) * sn
  const hp = ((h % 360) + 360) % 360 / 60
  const x = c * (1 - Math.abs((hp % 2) - 1))
  let r1 = 0, g1 = 0, b1 = 0
  if (hp < 1) { r1 = c; g1 = x; b1 = 0 }
  else if (hp < 2) { r1 = x; g1 = c; b1 = 0 }
  else if (hp < 3) { r1 = 0; g1 = c; b1 = x }
  else if (hp < 4) { r1 = 0; g1 = x; b1 = c }
  else if (hp < 5) { r1 = x; g1 = 0; b1 = c }
  else { r1 = c; g1 = 0; b1 = x }
  const m = ln - c / 2
  return rgbToHex((r1 + m) * 255, (g1 + m) * 255, (b1 + m) * 255)
}

// --- computed-color string → hex -------------------------------------------

const parseRgb = (s: string): { r: number; g: number; b: number } | null => {
  const m = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)/i.exec(s.trim())
  if (!m) return null
  return { r: Number(m[1]), g: Number(m[2]), b: Number(m[3]) }
}

const srgbGamma = (c: number): number =>
  c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055

// OKLCH → sRGB (Björn Ottosson's OKLab). The platform is OKLCH-native, so
// browser-resolved theme tokens come back as `oklch(L C H)`.
const parseOklch = (s: string): { r: number; g: number; b: number } | null => {
  const m = /^oklch\(\s*([\d.]+%?)\s+([\d.]+)\s+([\d.]+)/i.exec(s.trim())
  if (!m) return null
  const Lraw = m[1]!
  const L = Lraw.endsWith('%') ? Number(Lraw.slice(0, -1)) / 100 : Number(Lraw)
  const C = Number(m[2])
  const hDeg = Number(m[3])
  const hr = (hDeg * Math.PI) / 180
  const a = C * Math.cos(hr)
  const bb = C * Math.sin(hr)
  const l_ = L + 0.3963377774 * a + 0.2158037573 * bb
  const m_ = L - 0.1055613458 * a - 0.0638541728 * bb
  const s_ = L - 0.0894841775 * a - 1.291485548 * bb
  const lc = l_ ** 3, mc = m_ ** 3, sc = s_ ** 3
  const r = 4.0767416621 * lc - 3.3077115913 * mc + 0.2309699292 * sc
  const g = -1.2684380046 * lc + 2.6097574011 * mc - 0.3413193965 * sc
  const b = -0.0041960863 * lc - 0.7034186147 * mc + 1.707614701 * sc
  return {
    r: clamp255(srgbGamma(r) * 255),
    g: clamp255(srgbGamma(g) * 255),
    b: clamp255(srgbGamma(b) * 255),
  }
}

// Resolve any computed color string (`rgb()`, `oklch()`, hex) to `#rrggbb`,
// or `''` when unresolvable (e.g. an unset `var()` literal under jsdom).
export const computedColorToHex = (s: string): string => {
  if (!s) return ''
  if (isHex(s)) return s.toLowerCase()
  const rgb = parseRgb(s)
  if (rgb) return rgbToHex(rgb.r, rgb.g, rgb.b)
  const ok = parseOklch(s)
  if (ok) return rgbToHex(ok.r, ok.g, ok.b)
  return ''
}

/** OKLCH string → `#rrggbb` for picker internals (invisible conversion). */
export const oklchToHex = (s: string): string => computedColorToHex(s)

const lin = (c: number): number => {
  const x = c / 255
  return x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4
}

/**
 * `#rrggbb` → approximate `oklch(L C H)` for token write-back.
 * Users never see hex; SSOT remains oklch.
 */
export const hexToOklch = (hex: string, alpha?: number): string => {
  const rgb = hexToRgb(normHex(hex) || hex)
  if (!rgb) return 'oklch(0 0 0)'
  const r = lin(rgb.r)
  const g = lin(rgb.g)
  const b = lin(rgb.b)
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
  const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s
  const a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s
  const bb = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s
  const C = Math.sqrt(a * a + bb * bb)
  let H = (Math.atan2(bb, a) * 180) / Math.PI
  if (H < 0) H += 360
  const fmt = (n: number) => String(Math.round(n * 1000) / 1000)
  if (alpha != null && alpha < 1) {
    return `oklch(${fmt(L)} ${fmt(C)} ${fmt(H)} / ${fmt(alpha)})`
  }
  return `oklch(${fmt(L)} ${fmt(C)} ${fmt(H)})`
}

export const isOklch = (s: string): boolean => /^oklch\(/i.test(String(s ?? '').trim())

// --- swatch palette derivation ----------------------------------------------

// Design-system tone tokens the swatches pull from — never a hardcoded brand
// hex. Resolved through computed style against the host so the palette tracks
// the active theme (ship/paper/halo/console/contrast — modes via data-mode).
const TONE_TOKENS: { token: string; label: string }[] = [
  { token: '--primary', label: 'Primary' },
  { token: '--accent', label: 'Accent' },
  { token: '--info', label: 'Info' },
  { token: '--success', label: 'Success' },
  { token: '--warning', label: 'Warning' },
  { token: '--destructive', label: 'Destructive' },
]

// Deterministic base palette — pure HSL math, no theme dependency and no
// hardcoded brand hex. Guarantees the palette is never empty (and gives the
// picker a usable spectrum even before the design-system tokens are loaded,
// e.g. under jsdom). A neutral greyscale ramp plus an evenly-spaced hue wheel
// at a fixed saturation/lightness.
const baseRamp = (): Swatch[] => {
  const neutrals: { l: number; label: string }[] = [
    { l: 0, label: 'Black' },
    { l: 33, label: 'Dark gray' },
    { l: 66, label: 'Light gray' },
    { l: 100, label: 'White' },
  ]
  const hues: { h: number; label: string }[] = [
    { h: 0, label: 'Red' },
    { h: 30, label: 'Orange' },
    { h: 50, label: 'Amber' },
    { h: 140, label: 'Green' },
    { h: 175, label: 'Teal' },
    { h: 210, label: 'Blue' },
    { h: 265, label: 'Indigo' },
    { h: 320, label: 'Magenta' },
  ]
  return [
    ...neutrals.map(({ l, label }) => ({ hex: hslToHex(0, 0, l), label })),
    ...hues.map(({ h, label }) => ({ hex: hslToHex(h, 68, 55), label })),
  ]
}

const dedupe = (swatches: Swatch[]): Swatch[] => {
  const seen = new Set<string>()
  return swatches.filter((s) => {
    if (seen.has(s.hex)) return false
    seen.add(s.hex)
    return true
  })
}

// Build the swatch palette: themed tone tokens (resolved against the host)
// first, then the theme-free base ramp. Tone tokens that fail to resolve
// (jsdom / unloaded theme) are simply dropped, so the palette degrades to the
// base ramp rather than emitting an invalid swatch.
export const deriveSwatches = (host: HTMLElement): Swatch[] => {
  const cs = getComputedStyle(host)
  const toned: Swatch[] = []
  for (const { token, label } of TONE_TOKENS) {
    const hex = computedColorToHex(cs.getPropertyValue(token).trim())
    if (hex) toned.push({ hex, label })
  }
  return dedupe([...toned, ...baseRamp()])
}
