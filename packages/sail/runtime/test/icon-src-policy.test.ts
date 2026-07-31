import { describe, expect, it } from 'vitest'
import { isAllowedIconSrc } from '../src/actions/icon'

describe('icon src policy (local / same-origin only)', () => {
  it('allows relative and root-relative paths', () => {
    expect(isAllowedIconSrc('/_deck/icons/search.svg')).toBe(true)
    expect(isAllowedIconSrc('./glyph.svg')).toBe(true)
    expect(isAllowedIconSrc('../assets/i.svg')).toBe(true)
  })

  it('blocks protocol-relative and cross-origin https', () => {
    expect(isAllowedIconSrc('//unpkg.com/lucide/search.svg')).toBe(false)
    expect(isAllowedIconSrc('https://unpkg.com/lucide-static/icons/search.svg')).toBe(false)
    expect(isAllowedIconSrc('https://cdn.jsdelivr.net/npm/lucide/search.svg')).toBe(false)
    expect(isAllowedIconSrc('https://example.com/x.svg')).toBe(false)
  })

  it('allows data:image/svg and blob:', () => {
    expect(isAllowedIconSrc('data:image/svg+xml,<svg></svg>')).toBe(true)
    expect(isAllowedIconSrc('blob:http://localhost/abc')).toBe(true)
  })
})
