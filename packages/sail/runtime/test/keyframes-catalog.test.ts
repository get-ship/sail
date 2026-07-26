// Closed product keyframe catalog — ambient CSS loops for Style.animate
// and L1 skins. Pins emit shape + that LIGHT_CONTRACTS owns the sheet
// (no hand @keyframes left in skins).

import { describe, expect, it } from 'vitest'
import {
  AMBIENT_KEYFRAMES_CSS,
  KEYFRAME_CATALOG,
  STYLE_ANIMATE_NAMES,
  emitKeyframeCatalogCss,
} from '../src/styles/keyframes-catalog'
import { LIGHT_CONTRACTS_CSS } from '../src/styles/index'

describe('keyframes catalog', () => {
  it('lists the closed product names', () => {
    expect(KEYFRAME_CATALOG.map((e) => e.name)).toEqual([
      'spin',
      'pulse',
      'shimmer',
      'progress-indeterminate',
    ])
  })

  it('exposes Style.animate names (spin only today)', () => {
    expect(STYLE_ANIMATE_NAMES).toEqual(['spin'])
  })

  it('emits @keyframes + --animate-* vars for every entry', () => {
    const css = emitKeyframeCatalogCss()
    for (const e of KEYFRAME_CATALOG) {
      expect(css).toContain(`@keyframes ${e.keyframe} { ${e.frames} }`)
      expect(css).toContain(`--animate-${e.name}: ${e.animation};`)
    }
    expect(css).toContain(
      '@media (prefers-reduced-motion: reduce) { :root { --animate-spin: sail-spin 1.5s linear infinite; } }',
    )
  })

  it('AMBIENT_KEYFRAMES_CSS matches emitKeyframeCatalogCss()', () => {
    expect(AMBIENT_KEYFRAMES_CSS).toBe(emitKeyframeCatalogCss())
  })

  it('LIGHT_CONTRACTS_CSS includes the ambient sheet and skins use catalog vars', () => {
    expect(LIGHT_CONTRACTS_CSS.startsWith(AMBIENT_KEYFRAMES_CSS)).toBe(true)
    // Skins reference vars — they must not re-declare @keyframes.
    const afterAmbient = LIGHT_CONTRACTS_CSS.slice(AMBIENT_KEYFRAMES_CSS.length)
    expect(afterAmbient).not.toMatch(/@keyframes\s+sail-/)
    // spin PE is used by spinner element-skins / Style.animate; contracts
    // only declare the var. Skins below live in contracts:
    expect(afterAmbient).toContain('animation: var(--animate-pulse)')
    expect(afterAmbient).toContain('animation: var(--animate-shimmer)')
    expect(afterAmbient).toContain(
      'animation: var(--animate-progress-indeterminate)',
    )
  })

  it('Style.animate(spin) PE still lowers via --animate-spin (catalog entry)', () => {
    const spin = KEYFRAME_CATALOG.find((e) => e.name === 'spin')
    expect(spin?.styleAnimate).toBe(true)
    expect(spin?.animation).toBe('sail-spin 0.7s linear infinite')
    expect(AMBIENT_KEYFRAMES_CSS).toContain('--animate-spin: sail-spin 0.7s linear infinite')
  })
})
