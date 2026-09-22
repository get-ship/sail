import { readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const asset = readFileSync(
  resolve(dirname(fileURLToPath(import.meta.url)), '../../tokens/src/sail-base.css'),
  'utf8',
)

describe('sail-base.css', () => {
  it('has layer, ambient keyframes, control baseline', () => {
    expect(asset).toContain('@layer sail')
    expect(asset).toContain('@keyframes sail-spin')
    expect(asset).toContain('--animate-spin')
    expect(asset).toContain('font: inherit')
    expect(asset).toContain('focus-visible')
    expect(asset).not.toContain(':host')
  })

  it('has @font-face', () => {
    expect(asset).toContain('@font-face')
    expect(asset).toContain("url('./fonts/")
  })
})
