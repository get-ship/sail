import { describe, expect, it } from 'vitest'
import { SAIL_BROWSER_LSP } from '../src/client'

describe('SAIL_BROWSER_LSP', () => {
  it('advertises structure-only (no in-browser TypeScript)', () => {
    expect(SAIL_BROWSER_LSP.structure).toBe(true)
    expect(SAIL_BROWSER_LSP.typescript).toBe(false)
    expect(SAIL_BROWSER_LSP.note).toContain('structure-only')
  })
})
