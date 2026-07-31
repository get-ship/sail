// theme.test.ts — the sanctioned runtime theme-write hatch.
//
// Contract:
//   - preview writes a DECLARED token to `:root`; a non-token key refuses.
//   - reset drops session previews, keeps committed overrides.
//   - commit persists (localStorage default + host hook) and applies live.
//   - current() reports the active theme id + resolved override values.
//   - hydrateTheme rehydrates committed overrides from storage (live boot).
//
// The full `:root`→descendant inheritance proof is the browser gate (jsdom does
// not resolve `var()` inheritance); here we assert the MECHANISM — the token is
// written to `documentElement` and the shape constraint (declared-token-only) holds.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { theme, setThemePersist, hydrateTheme, clearThemeOverrides } from '../src/theme'

const root = () => document.documentElement

/** Register a token in the compiler `@property` registry the runtime reads. */
function register(...names: string[]): void {
  const g = globalThis as { __sailRegisteredTokens?: Set<string> }
  g.__sailRegisteredTokens ??= new Set<string>()
  for (const n of names) g.__sailRegisteredTokens.add(n)
}

beforeEach(() => {
  register('--brand-hue', '--surface-1')
  clearThemeOverrides()
  // Clear any committed state left on :root / storage between tests.
  root().removeAttribute('style')
  root().removeAttribute('data-theme')
  try {
    localStorage.clear()
  } catch {}
  setThemePersist(null)
})

afterEach(() => {
  setThemePersist(null)
})

describe('theme.preview — the live :root write', () => {
  it('writes a declared token onto :root', () => {
    theme.preview({ '--brand-hue': '210' })
    expect(root().style.getPropertyValue('--brand-hue')).toBe('210')
  })

  it('refuses a non-token key (no --) and applies nothing', () => {
    expect(() => theme.preview({ 'not-a-token': 'x' })).toThrow(/not a writable token/)
    expect(root().getAttribute('style')).toBeNull()
  })

  it('refuses an undeclared --custom-property', () => {
    expect(() => theme.preview({ '--never-declared-xyz': '1' })).toThrow(/not a writable token/)
  })

  it('is all-or-nothing: a bad key in the patch reverts the whole write', () => {
    expect(() => theme.preview({ '--brand-hue': '210', 'bad': 'y' })).toThrow()
    // The good key must NOT have been applied — validation is up-front.
    expect(root().style.getPropertyValue('--brand-hue')).toBe('')
  })
})

describe('theme.reset — drop session previews', () => {
  it('removes previewed tokens from :root', () => {
    theme.preview({ '--brand-hue': '123' })
    theme.reset()
    expect(root().style.getPropertyValue('--brand-hue')).toBe('')
  })

  it('keeps committed overrides through a reset', () => {
    theme.commit({ '--brand-hue': '77' })
    theme.preview({ '--surface-1': 'red' })
    theme.reset()
    expect(root().style.getPropertyValue('--brand-hue')).toBe('77') // committed stays
    expect(root().style.getPropertyValue('--surface-1')).toBe('') // preview dropped
  })
})

describe('theme.commit — persist + apply', () => {
  it('applies the token live and write-throughs to localStorage (default)', () => {
    theme.commit({ '--brand-hue': '42' })
    expect(root().style.getPropertyValue('--brand-hue')).toBe('42')
    const stored = JSON.parse(localStorage.getItem('ship-theme-overrides') ?? '{}')
    expect(stored['--brand-hue']).toBe('42')
  })

  it('invokes the host persist hook with the patch', () => {
    const hook = vi.fn()
    setThemePersist(hook)
    theme.commit({ '--brand-hue': '9' })
    expect(hook).toHaveBeenCalledWith({ '--brand-hue': '9' })
  })

  it('refuses a non-token key', () => {
    expect(() => theme.commit({ nope: 'x' })).toThrow(/not a writable token/)
  })
})

describe('theme.current — active id + resolved overrides', () => {
  it('reports the active theme id and the overridden tokens', () => {
    root().dataset.theme = 'console'
    theme.commit({ '--brand-hue': '200' })
    theme.preview({ '--surface-1': 'blue' })
    const cur = theme.current()
    expect(cur.id).toBe('console')
    expect(Object.keys(cur.tokens).sort()).toEqual(['--brand-hue', '--surface-1'])
  })
})

describe('hydrateTheme — live-persistence boot', () => {
  it('re-applies committed overrides from storage to :root', () => {
    localStorage.setItem('ship-theme-overrides', JSON.stringify({ '--brand-hue': '333' }))
    hydrateTheme()
    expect(root().style.getPropertyValue('--brand-hue')).toBe('333')
    // A hydrated override is committed, so reset keeps it.
    theme.reset()
    expect(root().style.getPropertyValue('--brand-hue')).toBe('333')
  })

  it('ignores an unwritable token in storage (defends against a poisoned cache)', () => {
    localStorage.setItem('ship-theme-overrides', JSON.stringify({ 'evil': 'x' }))
    expect(() => hydrateTheme()).not.toThrow()
    expect(root().style.getPropertyValue('evil')).toBe('')
  })
})
