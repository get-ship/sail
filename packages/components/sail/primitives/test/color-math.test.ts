// Unit spec for the <sail-color-picker> color-math module. Guards the runtime
// swatch derivation that replaced the old hardcoded hex palette: OKLCH/RGB →
// hex resolution and the theme-token-driven `deriveSwatches`.

import { afterEach, describe, expect, it } from 'vitest'
import {
  computedColorToHex,
  deriveSwatches,
  hexToRgb,
  hslToHex,
  isHex,
  normHex,
  rgbToHex,
  rgbToHsl,
} from '../src/sail-color-picker/color-math'

const HEX_RE = /^#[0-9a-f]{6}$/

describe('color-math conversions', () => {
  it('isHex accepts #rrggbb and rejects partials', () => {
    expect(isHex('#abcdef')).toBe(true)
    expect(isHex('#ABCDEF')).toBe(true)
    expect(isHex('#abc')).toBe(false)
    expect(isHex('abcdef')).toBe(false)
    expect(isHex('')).toBe(false)
  })

  it('normHex normalizes shorthand, bare, and cased forms', () => {
    expect(normHex('#ABCDEF')).toBe('#abcdef')
    expect(normHex('abcdef')).toBe('#abcdef')
    expect(normHex('#abc')).toBe('#aabbcc')
    expect(normHex('  #FfF  ')).toBe('#ffffff')
    expect(normHex('not a color')).toBe('')
  })

  it('rgb ↔ hex round-trips and clamps', () => {
    expect(rgbToHex(255, 0, 128)).toBe('#ff0080')
    expect(rgbToHex(-5, 300, 128)).toBe('#00ff80')
    expect(hexToRgb('#ff0080')).toEqual({ r: 255, g: 0, b: 128 })
    expect(hexToRgb('nope')).toBeNull()
  })

  it('rgbToHsl reports hue/sat/light', () => {
    expect(rgbToHsl(255, 0, 0)).toEqual({ h: 0, s: 100, l: 50 })
    expect(rgbToHsl(0, 0, 0)).toEqual({ h: 0, s: 0, l: 0 })
    expect(rgbToHsl(255, 255, 255)).toEqual({ h: 0, s: 0, l: 100 })
  })

  it('hslToHex emits valid hex across the wheel', () => {
    for (const h of [0, 90, 180, 270, 360]) {
      expect(hslToHex(h, 68, 55)).toMatch(HEX_RE)
    }
    expect(hslToHex(0, 0, 0)).toBe('#000000')
    expect(hslToHex(0, 0, 100)).toBe('#ffffff')
  })
})

describe('computedColorToHex', () => {
  it('passes through hex (lowercased)', () => {
    expect(computedColorToHex('#AABBCC')).toBe('#aabbcc')
  })

  it('parses an rgb() string (the browser-normalized form)', () => {
    expect(computedColorToHex('rgb(59, 130, 246)')).toBe('#3b82f6')
    expect(computedColorToHex('rgba(255, 0, 0, 0.5)')).toBe('#ff0000')
  })

  it('parses an oklch() string (the OKLCH-native theme form)', () => {
    // Pure white / black anchors are unambiguous across the OKLab transform.
    expect(computedColorToHex('oklch(1 0 0)')).toBe('#ffffff')
    expect(computedColorToHex('oklch(0 0 0)')).toBe('#000000')
    // A mid chroma blue resolves to a valid in-gamut hex.
    expect(computedColorToHex('oklch(0.6 0.2 250)')).toMatch(HEX_RE)
    // Percentage lightness is accepted.
    expect(computedColorToHex('oklch(60% 0.1 30)')).toMatch(HEX_RE)
  })

  it('returns empty for unresolvable input', () => {
    expect(computedColorToHex('')).toBe('')
    expect(computedColorToHex('var(--nope)')).toBe('')
  })
})

describe('deriveSwatches', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('produces a non-empty palette of valid hex even without theme tokens', () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    const swatches = deriveSwatches(host)
    expect(swatches.length).toBeGreaterThanOrEqual(8)
    for (const sw of swatches) {
      expect(sw.hex).toMatch(HEX_RE)
      expect(sw.label.length).toBeGreaterThan(0)
    }
  })

  it('dedupes repeated hex values', () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    const swatches = deriveSwatches(host)
    const hexes = swatches.map((s) => s.hex)
    expect(new Set(hexes).size).toBe(hexes.length)
  })

  it('resolves an inline tone token to a themed swatch', () => {
    const host = document.createElement('div')
    // Inline custom property the browser/jsdom resolves to rgb(...).
    host.style.setProperty('--primary', '#3b82f6')
    document.body.appendChild(host)
    const swatches = deriveSwatches(host)
    expect(swatches.some((s) => s.hex === '#3b82f6' && s.label === 'Primary')).toBe(true)
  })
})
