// @vitest-environment jsdom

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sui-runtime'
import { hydrateAttr } from '../../src/hydrate/attr'

describe('hydrate/attr', () => {
  it('writes the attribute and updates on signal change', () => {
    const el = document.createElement('a')
    const href = signal('/a')
    hydrateAttr(el, 'href', () => href())
    expect(el.getAttribute('href')).toBe('/a')
    href.set('/b')
    expect(el.getAttribute('href')).toBe('/b')
  })

  it('removes the attribute on null / undefined / false', () => {
    const el = document.createElement('a')
    const v = signal<unknown>('x')
    hydrateAttr(el, 'data-x', () => v())
    expect(el.getAttribute('data-x')).toBe('x')
    v.set(null)
    expect(el.hasAttribute('data-x')).toBe(false)
    v.set(false)
    expect(el.hasAttribute('data-x')).toBe(false)
    v.set('y')
    expect(el.getAttribute('data-x')).toBe('y')
  })
})
