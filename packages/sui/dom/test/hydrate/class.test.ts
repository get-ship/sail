// @vitest-environment jsdom

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sui-runtime'
import { hydrateClass } from '../../src/hydrate/class'

describe('hydrate/class', () => {
  it('toggles the class on truthy / off on falsy', () => {
    const el = document.createElement('div')
    const active = signal(false)
    hydrateClass(el, 'active', () => active())
    expect(el.classList.contains('active')).toBe(false)
    active.set(true)
    expect(el.classList.contains('active')).toBe(true)
    active.set(0)
    expect(el.classList.contains('active')).toBe(false)
  })
})
