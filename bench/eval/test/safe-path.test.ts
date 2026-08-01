import { describe, expect, it } from 'vitest'
import { isPathInside } from '../src/naming/safe-path.ts'

describe('isPathInside', () => {
  it('accepts root and children', () => {
    expect(isPathInside('/tmp/sail-lab', '/tmp/sail-lab')).toBe(true)
    expect(isPathInside('/tmp/sail-lab', '/tmp/sail-lab/runs/x')).toBe(true)
  })

  it('rejects sibling prefix escape', () => {
    expect(isPathInside('/tmp/sail-lab', '/tmp/sail-lab2/secret')).toBe(false)
    expect(isPathInside('/tmp/sail-lab', '/tmp/sail-lab-evil')).toBe(false)
  })

  it('rejects parent escape', () => {
    expect(isPathInside('/tmp/sail-lab', '/tmp/sail-lab/../other')).toBe(false)
    expect(isPathInside('/tmp/sail-lab', '/etc/passwd')).toBe(false)
  })
})
