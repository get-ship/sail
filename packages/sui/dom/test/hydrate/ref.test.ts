// @vitest-environment jsdom

import { describe, expect, it } from 'vitest'
import { signal, type Signal } from '@ship/sui-runtime'
import { hydrateRef } from '../../src/hydrate/ref'

describe('hydrate/ref', () => {
  it('reuses an existing signal in scope and clears on dispose', () => {
    const el = document.createElement('div')
    const ref = signal<Element | null>(null)
    const scope: Record<string, unknown> = { btnEl: ref }
    const dispose = hydrateRef(el, scope, 'btnEl')
    expect(ref()).toBe(el)
    expect(scope.btnEl).toBe(ref)
    dispose()
    expect(ref()).toBe(null)
  })

  it('creates a fresh signal when scope has no entry', () => {
    const el = document.createElement('div')
    const scope: Record<string, unknown> = {}
    const dispose = hydrateRef(el, scope, 'btnEl')
    const ref = scope.btnEl as Signal<Element | null>
    expect(typeof ref).toBe('function')
    expect(ref()).toBe(el)
    dispose()
    expect(ref()).toBe(null)
  })
})
