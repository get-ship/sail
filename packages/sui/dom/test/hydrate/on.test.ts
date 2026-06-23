// @vitest-environment jsdom

import { describe, expect, it, vi } from 'vitest'
import { hydrateOn } from '../../src/hydrate/on'

describe('hydrate/on', () => {
  it('forwards events to the supplied handler', () => {
    const el = document.createElement('button')
    const handler = vi.fn()
    const dispose = hydrateOn(el, 'click', handler)
    el.dispatchEvent(new Event('click'))
    expect(handler).toHaveBeenCalledTimes(1)
    dispose()
    el.dispatchEvent(new Event('click'))
    expect(handler).toHaveBeenCalledTimes(1)
  })

  it('threads modifiers into the runtime on() pipeline', () => {
    const el = document.createElement('button')
    const handler = vi.fn()
    hydrateOn(el, 'click', handler, [{ kind: 'preventDefault' }])
    const ev = new Event('click', { cancelable: true })
    el.dispatchEvent(ev)
    expect(handler).toHaveBeenCalledTimes(1)
    expect(ev.defaultPrevented).toBe(true)
  })
})
