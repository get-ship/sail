import { describe, expect, it } from 'vitest'
import { sanitizeJson, sanitizeNote, sanitizePath, sanitizeText } from '../src/sanitize.ts'

describe('sanitize', () => {
  it('rewrites paths under workdir to relative', () => {
    expect(sanitizePath('/tmp/work/routes/page.sail', '/tmp/work')).toBe(
      'routes/page.sail',
    )
  })

  it('redacts absolute home/tmp paths in free text', () => {
    const s = sanitizeText('failed at /home/madvib/dev/Ship/foo.sail and /tmp/x/y')
    expect(s).not.toContain('/home/madvib')
    expect(s).toContain('<path>')
  })

  it('redacts workspaces/root/mnt/nix and Windows paths', () => {
    const s = sanitizeText(
      [
        '/workspaces/orch/eval/runs/a',
        '/root/.cache/foo',
        '/mnt/data/proj/file.sail',
        '/nix/store/abc-def/bin/sail',
        'C:\\Users\\me\\proj\\a.sail',
      ].join(' '),
    )
    expect(s).not.toContain('/workspaces')
    expect(s).not.toContain('/root/')
    expect(s).not.toContain('/mnt/')
    expect(s).not.toContain('/nix/')
    expect(s).not.toContain('C:\\Users')
    expect(s).toContain('<path>')
  })

  it('sanitizeJson deep-redacts path strings', () => {
    const out = sanitizeJson(
      { file: '/Users/me/proj/a.sail', nested: { p: '/var/tmp/z/x' } },
      undefined,
    )
    expect(out.file).toBe('<path>')
    expect(out.nested.p).toBe('<path>')
  })

  it('replaces workdir substring in text', () => {
    expect(sanitizeText('wrote /tmp/eval-xyz/routes/page.sail', '/tmp/eval-xyz')).toBe(
      'wrote <workdir>/routes/page.sail',
    )
  })

  it('sanitizeNote shares redaction and strips pipes', () => {
    const n = sanitizeNote('err at /data/proj/x | more')
    expect(n).not.toContain('/data/')
    expect(n).not.toContain('|')
  })

  it('preserves URL paths (does not redact scheme:// hosts)', () => {
    const s = sanitizeText(
      'fetch https://example.com/api/v1 and http://127.0.0.1:7878/docs/guide see /home/me/secret/file',
    )
    expect(s).toContain('https://example.com/api/v1')
    expect(s).toContain('http://127.0.0.1:7878/docs/guide')
    expect(s).not.toContain('/home/me')
    expect(s).toContain('<path>')
  })
})
