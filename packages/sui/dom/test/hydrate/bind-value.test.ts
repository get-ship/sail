// @vitest-environment jsdom

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { hydrateBindValue, hydrateBindChecked } from '../../src/hydrate/bind-value'

describe('hydrate/bind-value', () => {
  it('mirrors signal -> input.value and input -> signal', () => {
    const el = document.createElement('input')
    const sig = signal('init')
    const dispose = hydrateBindValue(el, sig)
    expect(el.value).toBe('init')
    sig.set('next')
    expect(el.value).toBe('next')
    el.value = 'typed'
    el.dispatchEvent(new Event('input'))
    expect(sig()).toBe('typed')
    dispose()
    sig.set('after')
    expect(el.value).toBe('typed')
  })

  it('renders nullish as empty string', () => {
    const el = document.createElement('input')
    const sig = signal<unknown>(null)
    hydrateBindValue(el, sig)
    expect(el.value).toBe('')
  })
})

describe('hydrate/bind-checked', () => {
  it('mirrors signal -> input.checked and change -> signal', () => {
    const el = document.createElement('input')
    el.type = 'checkbox'
    const sig = signal(false)
    const dispose = hydrateBindChecked(el, sig)
    expect(el.checked).toBe(false)
    sig.set(true)
    expect(el.checked).toBe(true)
    el.checked = false
    el.dispatchEvent(new Event('change'))
    expect(sig()).toBe(false)
    dispose()
  })
})
