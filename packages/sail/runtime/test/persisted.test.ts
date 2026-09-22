import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  createScope,
  withScope,
  mount as mountAmbient,
} from '../src/effect-scope'
import { persisted } from '../src/persisted'

describe('persisted', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    localStorage.clear()
    sessionStorage.clear()
  })
  afterEach(() => {
    vi.useRealTimers()
    localStorage.clear()
    sessionStorage.clear()
  })

  it('starts at initial and does not require storage on create', () => {
    const s = persisted('k', 1)
    expect(s()).toBe(1)
  })

  it('hydrates from localStorage on mount', () => {
    localStorage.setItem('pref', JSON.stringify('dark'))
    const scope = createScope()
    const s = withScope(scope, () => {
      const p = persisted('pref', 'light')
      return p
    })
    expect(s()).toBe('light')
    const host = document.createElement('div')
    document.body.appendChild(host)
    withScope(scope, () => {
      /* mounts already registered */
    })
    scope.runMounts(host)
    expect(s()).toBe('dark')
    host.remove()
    scope.dispose()
  })

  it('debounces writes to storage', () => {
    const scope = createScope()
    const s = withScope(scope, () => persisted('count', 0, { debounceMs: 50 }))
    const host = document.createElement('div')
    scope.runMounts(host)
    s.set(1)
    s.set(2)
    s.set(3)
    expect(localStorage.getItem('count')).toBeNull()
    vi.advanceTimersByTime(50)
    expect(localStorage.getItem('count')).toBe('3')
    scope.dispose()
  })

  it('supports session storage', () => {
    sessionStorage.setItem('tok', JSON.stringify({ a: 1 }))
    const scope = createScope()
    const s = withScope(scope, () =>
      persisted('tok', { a: 0 }, { storage: 'session' }),
    )
    scope.runMounts(document.createElement('div'))
    expect(s()).toEqual({ a: 1 })
    scope.dispose()
  })

  it('SSR path: no throw without window storage APIs during create', () => {
    const scope = createScope()
    const s = withScope(scope, () => persisted('ssr-only', 'ok'))
    expect(s()).toBe('ok')
    // dispose without runMounts — clean
    scope.dispose()
  })
})
