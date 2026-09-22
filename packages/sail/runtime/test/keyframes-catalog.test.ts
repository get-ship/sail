import { readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import {
  AMBIENT_KEYFRAMES_CSS,
  KEYFRAME_CATALOG,
  STYLE_ANIMATE_NAMES,
  emitKeyframeCatalogCss,
} from '../src/styles/keyframes-catalog'

const sailBase = readFileSync(
  resolve(dirname(fileURLToPath(import.meta.url)), '../../tokens/src/sail-base.css'),
  'utf8',
)

describe('keyframes catalog', () => {
  it('names', () => {
    expect(KEYFRAME_CATALOG.map((e) => e.name)).toEqual([
      'spin',
      'shimmer',
      'progress-indeterminate',
    ])
  })

  it('Style.animate names', () => {
    expect(STYLE_ANIMATE_NAMES).toEqual(['spin'])
  })

  it('emit', () => {
    const css = emitKeyframeCatalogCss()
    for (const e of KEYFRAME_CATALOG) {
      expect(css).toContain(`@keyframes ${e.keyframe} { ${e.frames} }`)
      expect(css).toContain(`--animate-${e.name}: ${e.animation};`)
    }
  })

  it('AMBIENT matches emit', () => {
    expect(AMBIENT_KEYFRAMES_CSS).toBe(emitKeyframeCatalogCss())
  })

  it('sail-base has ambient vars', () => {
    for (const e of KEYFRAME_CATALOG) {
      expect(sailBase).toContain(`@keyframes ${e.keyframe}`)
      expect(sailBase).toContain(`--animate-${e.name}:`)
    }
  })

  it('spin catalog entry', () => {
    const spin = KEYFRAME_CATALOG.find((e) => e.name === 'spin')
    expect(spin?.styleAnimate).toBe(true)
    expect(spin?.animation).toBe('sail-spin 0.7s linear infinite')
    expect(sailBase).toContain('--animate-spin:')
  })
})
