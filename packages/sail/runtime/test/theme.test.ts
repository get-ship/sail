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
import {
  theme,
  setThemePersist,
  hydrateTheme,
  clearThemeOverrides,
  clearThemeScope,
  lowerThemeDoc,
} from '../src/theme'

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
  root().removeAttribute('theme')
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
    root().setAttribute('theme', 'console')
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

// ─── applyScope — host-scoped DTCG draft paint (T1) ─────────────────────────

/** Minimal modes-in-file DTCG sample (same leaf shape as model/themes). */
function sampleDoc(overrides?: {
  lightPrimary?: string
  darkPrimary?: string
  themeId?: string
  defaultMode?: string
}) {
  return {
    $description: 'test theme',
    $extensions: {
      ship: {
        layer: 'theme',
        themeId: overrides?.themeId ?? 'draft-theme',
        defaultMode: overrides?.defaultMode ?? 'light',
      },
    },
    modes: {
      light: {
        color: {
          primary: {
            default: {
              $value: overrides?.lightPrimary ?? 'oklch(0.5 0.1 40)',
              $type: 'color',
              $extensions: { ship: { cssName: 'primary' } },
            },
          },
          background: {
            $value: 'oklch(0.98 0 0)',
            $type: 'color',
            $extensions: { ship: { cssName: 'background' } },
          },
        },
        typography: {
          body: {
            font: {
              $value: '{font-body}',
              $type: 'fontFamily',
              $extensions: { ship: { cssName: 'body-font' } },
            },
          },
        },
      },
      dark: {
        color: {
          primary: {
            default: {
              $value: overrides?.darkPrimary ?? 'oklch(0.7 0.12 40)',
              $type: 'color',
              $extensions: { ship: { cssName: 'primary' } },
            },
          },
          background: {
            $value: 'oklch(0.15 0 0)',
            $type: 'color',
            $extensions: { ship: { cssName: 'background' } },
          },
        },
        typography: {
          body: {
            font: {
              $value: '{font-body}',
              $type: 'fontFamily',
              $extensions: { ship: { cssName: 'body-font' } },
            },
          },
        },
      },
    },
  }
}

describe('lowerThemeDoc — DTCG → CSS vars (themes.css twin)', () => {
  it('flattens mode leaves with ship.cssName and resolves {alias} → var(--x)', () => {
    const r = lowerThemeDoc(sampleDoc(), 'light')
    expect(r.mode).toBe('light')
    expect(r.themeId).toBe('draft-theme')
    expect(r.patch['--primary']).toBe('oklch(0.5 0.1 40)')
    expect(r.patch['--background']).toBe('oklch(0.98 0 0)')
    // Alias whole-value `{font-body}` → var(--font-body) (compiler resolve_value)
    expect(r.patch['--body-font']).toBe('var(--font-body)')
    expect(r.tokens.find((t) => t.cssName === 'primary')?.path).toBe('color.primary.default')
  })

  it('selects dark mode values when mode=dark', () => {
    const r = lowerThemeDoc(sampleDoc({ darkPrimary: 'oklch(0.8 0.2 10)' }), 'dark')
    expect(r.mode).toBe('dark')
    expect(r.patch['--primary']).toBe('oklch(0.8 0.2 10)')
    expect(r.patch['--background']).toBe('oklch(0.15 0 0)')
  })

  it('defaults mode from $extensions.ship.defaultMode when mode omitted', () => {
    const r = lowerThemeDoc(sampleDoc({ defaultMode: 'dark' }))
    expect(r.mode).toBe('dark')
    expect(r.patch['--primary']).toBe('oklch(0.7 0.12 40)')
  })
})

describe('theme.applyScope — host only + teardown', () => {
  let host: HTMLElement
  let sibling: HTMLElement

  beforeEach(() => {
    host = document.createElement('div')
    sibling = document.createElement('div')
    document.body.append(host, sibling)
  })

  afterEach(() => {
    clearThemeScope(host)
    host.remove()
    sibling.remove()
  })

  it('writes lowered tokens on host only (not :root, not sibling)', () => {
    theme.applyScope(host, { doc: sampleDoc(), mode: 'light' })
    expect(host.style.getPropertyValue('--primary')).toBe('oklch(0.5 0.1 40)')
    expect(host.style.getPropertyValue('--background')).toBe('oklch(0.98 0 0)')
    expect(host.style.getPropertyValue('--body-font')).toBe('var(--font-body)')
    // Scope isolation
    expect(root().style.getPropertyValue('--primary')).toBe('')
    expect(sibling.style.getPropertyValue('--primary')).toBe('')
  })

  it('sets framework theme + data-mode only (never data-theme / data-theme-preview)', () => {
    theme.applyScope(host, { doc: sampleDoc({ themeId: 'paper' }), mode: 'dark' })
    expect(host.getAttribute('theme')).toBe('paper')
    expect(host.getAttribute('data-mode')).toBe('dark')
    expect(host.hasAttribute('data-theme')).toBe(false)
    expect(host.hasAttribute('data-theme-preview')).toBe(false)
  })

  it('teardown via disposer clears props and attrs', () => {
    const dispose = theme.applyScope(host, { doc: sampleDoc(), mode: 'light' })
    expect(host.style.getPropertyValue('--primary')).toBe('oklch(0.5 0.1 40)')
    dispose()
    expect(host.style.getPropertyValue('--primary')).toBe('')
    expect(host.style.getPropertyValue('--background')).toBe('')
    expect(host.hasAttribute('theme')).toBe(false)
    expect(host.hasAttribute('data-mode')).toBe(false)
    expect(host.hasAttribute('data-theme')).toBe(false)
  })

  it('clearScope / clearThemeScope is idempotent', () => {
    theme.applyScope(host, { doc: sampleDoc(), mode: 'light' })
    clearThemeScope(host)
    expect(() => clearThemeScope(host)).not.toThrow()
    expect(() => theme.clearScope(host)).not.toThrow()
  })

  it('re-apply replaces prior props (no sticky tokens across mode switch)', () => {
    // First apply a doc that sets --primary; second mode differs in value.
    theme.applyScope(host, { doc: sampleDoc({ lightPrimary: 'oklch(0.4 0 0)' }), mode: 'light' })
    expect(host.style.getPropertyValue('--primary')).toBe('oklch(0.4 0 0)')
    theme.applyScope(host, {
      doc: sampleDoc({ darkPrimary: 'oklch(0.9 0.1 200)' }),
      mode: 'dark',
    })
    expect(host.style.getPropertyValue('--primary')).toBe('oklch(0.9 0.1 200)')
    expect(host.getAttribute('data-mode')).toBe('dark')
  })

  it('does not use :root session maps (applyScope ≠ preview)', () => {
    register('--primary')
    theme.applyScope(host, { doc: sampleDoc(), mode: 'light' })
    // Session hatch still empty on :root
    expect(root().style.getPropertyValue('--primary')).toBe('')
    theme.reset() // would clear session previews — host scope untouched
    expect(host.style.getPropertyValue('--primary')).toBe('oklch(0.5 0.1 40)')
  })

  it('refuses a non-element host', () => {
    expect(() =>
      theme.applyScope(null as unknown as HTMLElement, { doc: sampleDoc() }),
    ).toThrow(/host must be an HTMLElement/)
  })
})
