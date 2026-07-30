import { describe, expect, it } from 'vitest'
import { join } from 'node:path'
import { assertSafeSlug, repoRoot } from '../src/paths.ts'
import { toWorkspacePath } from '../src/host.ts'

describe('assertSafeSlug', () => {
  it('accepts lowercase hyphenated slugs', () => {
    expect(assertSafeSlug('login-form')).toBe('login-form')
    expect(assertSafeSlug('a')).toBe('a')
  })

  it('rejects path traversal and odd characters', () => {
    expect(() => assertSafeSlug('../etc')).toThrow(/invalid task slug/)
    expect(() => assertSafeSlug('foo/bar')).toThrow(/invalid task slug/)
    expect(() => assertSafeSlug('Foo')).toThrow(/invalid task slug/)
    expect(() => assertSafeSlug('has space')).toThrow(/invalid task slug/)
    expect(() => assertSafeSlug('')).toThrow(/invalid task slug/)
  })
})

describe('toWorkspacePath', () => {
  it('converts absolute in-repo paths to relative', () => {
    const root = repoRoot()
    expect(toWorkspacePath(join(root, 'eval/tasks/login-form/task.jsonc'), root)).toBe(
      'eval/tasks/login-form/task.jsonc',
    )
  })

  it('rejects paths outside workspace', () => {
    expect(() => toWorkspacePath('/tmp/outside.sail', repoRoot())).toThrow(/escapes workspace/)
  })
})

