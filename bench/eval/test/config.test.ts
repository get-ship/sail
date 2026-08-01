import { describe, expect, it } from 'vitest'
import { loadTask, loadProfile, parseJsonc, resolveArm } from '../src/config/load.ts'
import { contentHash } from '../src/config/hash.ts'
import { standardsDir, repoRoot } from '../src/naming/paths.ts'

describe('config loaders', () => {
  const standards = standardsDir(repoRoot())

  it('loads parallax-landing task', () => {
    const t = loadTask('parallax-landing', standards)
    expect(t.id).toBe('parallax-landing')
    expect(t.prompt.length).toBeGreaterThan(20)
  })

  it('loads p0 profile with stable hash', () => {
    const a = loadProfile('p0', standards)
    const b = loadProfile('p0', standards)
    expect(a.hash).toBe(b.hash)
    expect(a.hash.length).toBe(64)
  })

  it('parseJsonc strips comments', () => {
    const v = parseJsonc<{ x: number }>('// hi\n{ "x": 1, }\n')
    expect(v.x).toBe(1)
  })

  it('contentHash is order-independent for keys', () => {
    expect(contentHash({ a: 1, b: 2 })).toBe(contentHash({ b: 2, a: 1 }))
  })

  it('resolveArm', () => {
    expect(resolveArm('sail').gate).toBe('sail_check')
    expect(() => resolveArm('nope')).toThrow(/unknown arm/)
  })
})
