// register-properties.test.ts — the `@property` token registry.
//
// `registerCustomProperties` installs the compiler-generated `@property`
// declarations AND records their names into the cross-bundle registry the
// runtime's `theme.*` hatch reads to validate a TokenName. Recording is what
// makes a compiler-reflected `(--x)` a writable theme token.

import { beforeEach, describe, expect, it } from 'vitest'
import { registerCustomProperties, isRegisteredThemeToken } from '../src/register-properties'

beforeEach(() => {
  ;(globalThis as { __sailRegisteredTokens?: Set<string> }).__sailRegisteredTokens = new Set()
  for (const el of Array.from(document.querySelectorAll('style'))) el.remove()
})

describe('token registry', () => {
  it('records every @property name the install declares', () => {
    registerCustomProperties(
      'sail-props-a',
      "@property --brand-hue { syntax: '<number>'; inherits: true; initial-value: 0; }" +
        "@property --row-y { syntax: '*'; inherits: true; }",
    )
    expect(isRegisteredThemeToken('--brand-hue')).toBe(true)
    expect(isRegisteredThemeToken('--row-y')).toBe(true)
    expect(isRegisteredThemeToken('--never')).toBe(false)
  })

  it('records even when the <style> is already installed (dedupe path)', () => {
    const css = "@property --dup { syntax: '*'; inherits: true; }"
    registerCustomProperties('sail-props-dup', css)
    // Wipe just the registry, keep the installed <style>, re-run: the early
    // dedupe return must still record.
    ;(globalThis as { __sailRegisteredTokens?: Set<string> }).__sailRegisteredTokens = new Set()
    registerCustomProperties('sail-props-dup', css)
    expect(isRegisteredThemeToken('--dup')).toBe(true)
  })

  it('actually installs the @property CSS into <head>', () => {
    registerCustomProperties('sail-props-install', "@property --x { syntax: '*'; inherits: true; }")
    expect(document.getElementById('sail-props-install')?.textContent).toContain('@property --x')
  })

  it('dedupes per property NAME across overlapping blocks', () => {
    registerCustomProperties(
      'sail-props-one',
      "@property --a { syntax: '*'; inherits: true; }@property --b { syntax: '*'; inherits: true; }",
    )
    registerCustomProperties(
      'sail-props-two',
      "@property --b { syntax: '<number>'; inherits: true; initial-value: 0; }" +
        "@property --c { syntax: '*'; inherits: true; }",
    )
    const two = document.getElementById('sail-props-two')
    // --b was already registered by block one: only the fresh --c rule lands.
    expect(two?.textContent).not.toContain('@property --b')
    expect(two?.textContent).toContain('@property --c')
    expect(isRegisteredThemeToken('--b')).toBe(true)
    expect(isRegisteredThemeToken('--c')).toBe(true)
  })

  it('installs no <style> when every property in the block is already registered', () => {
    registerCustomProperties('sail-props-first', "@property --seen { syntax: '*'; inherits: true; }")
    registerCustomProperties(
      'sail-props-second',
      "@property --seen { syntax: '<number>'; inherits: true; initial-value: 0; }",
    )
    expect(document.getElementById('sail-props-second')).toBeNull()
    expect(document.querySelectorAll('style')).toHaveLength(1)
  })
})
