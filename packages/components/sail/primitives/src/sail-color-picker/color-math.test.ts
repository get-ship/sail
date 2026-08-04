/**
 * Pure color-math behavior for sail-color-picker — no component mount / NAPI.
 * These are the conversions the picker commits when the user picks a color.
 */
import { describe, expect, it } from 'vitest'
import {
  hexToRgb,
  hslToHex,
  isHex,
  normHex,
  rgbToHsl,
  rgbToHex,
} from './color-math'

describe('color-math (picker value model)', () => {
  it('normHex accepts #rgb, #rrggbb, and bare hex', () => {
    expect(normHex('#ABC')).toBe('#aabbcc')
    expect(normHex('#aAbBcC')).toBe('#aabbcc')
    expect(normHex('ff0000')).toBe('#ff0000')
    expect(normHex('  #00ff00  ')).toBe('#00ff00')
    expect(normHex('not-a-color')).toBe('')
  })

  it('isHex is strict 7-char lowercase-tolerant', () => {
    expect(isHex('#ff0000')).toBe(true)
    expect(isHex('#FF0000')).toBe(true)
    expect(isHex('#fff')).toBe(false)
    expect(isHex('ff0000')).toBe(false)
  })

  it('hex ↔ rgb round-trips primary colors', () => {
    expect(hexToRgb('#ff0000')).toEqual({ r: 255, g: 0, b: 0 })
    expect(hexToRgb('#00ff00')).toEqual({ r: 0, g: 255, b: 0 })
    expect(hexToRgb('#0000ff')).toEqual({ r: 0, g: 0, b: 255 })
    expect(rgbToHex(255, 0, 0)).toBe('#ff0000')
    expect(rgbToHex(0, 128, 255)).toBe('#0080ff')
  })

  it('hex → HSL → hex preserves pure red / green / blue', () => {
    for (const hex of ['#ff0000', '#00ff00', '#0000ff', '#ffffff', '#000000']) {
      const rgb = hexToRgb(hex)!
      const hsl = rgbToHsl(rgb.r, rgb.g, rgb.b)
      const back = hslToHex(hsl.h, hsl.s, hsl.l)
      expect(back).toBe(hex)
    }
  })

  it('slider math: hue shift from red toward cyan produces a different hex', () => {
    const red = rgbToHsl(255, 0, 0)
    expect(red.h).toBe(0)
    const shifted = hslToHex(180, red.s, red.l)
    expect(isHex(shifted)).toBe(true)
    expect(shifted).not.toBe('#ff0000')
    // Cyan-ish: green+blue high, red low
    const rgb = hexToRgb(shifted)!
    expect(rgb.r).toBeLessThan(40)
    expect(rgb.g).toBeGreaterThan(200)
    expect(rgb.b).toBeGreaterThan(200)
  })

  it('lightness 0 always black; lightness 100 always white', () => {
    expect(hslToHex(40, 80, 0)).toBe('#000000')
    expect(hslToHex(40, 80, 100)).toBe('#ffffff')
  })
})
