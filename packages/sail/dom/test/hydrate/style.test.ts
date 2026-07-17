// @vitest-environment jsdom

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { hydrateStyle } from '../../src/hydrate/style'

describe('hydrate/style', () => {
  it('sets and clears an inline-style property', () => {
    const el = document.createElement('div')
    const color = signal<unknown>('red')
    hydrateStyle(el, 'color', () => color())
    expect(el.style.color).toBe('red')
    color.set('blue')
    expect(el.style.color).toBe('blue')
    color.set('')
    expect(el.style.color).toBe('')
    color.set(null)
    expect(el.style.color).toBe('')
    color.set('green')
    expect(el.style.color).toBe('green')
  })
})
