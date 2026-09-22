import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  createScope,
  withScope,
} from '../src/effect-scope'
import { debounce, interval } from '../src/timing'

describe('timing.debounce', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('delays the call until ms of quiet', () => {
    const fn = vi.fn()
    const scope = createScope()
    const d = withScope(scope, () => debounce(fn, 100))
    d('a')
    d('b')
    d('c')
    expect(fn).not.toHaveBeenCalled()
    vi.advanceTimersByTime(99)
    expect(fn).not.toHaveBeenCalled()
    vi.advanceTimersByTime(1)
    expect(fn).toHaveBeenCalledOnce()
    expect(fn).toHaveBeenCalledWith('c')
  })

  it('clears pending timer on scope dispose', () => {
    const fn = vi.fn()
    const scope = createScope()
    const d = withScope(scope, () => debounce(fn, 100))
    d('x')
    scope.dispose()
    vi.advanceTimersByTime(200)
    expect(fn).not.toHaveBeenCalled()
  })
})

describe('timing.interval', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('fires on the interval and stops on dispose', () => {
    const fn = vi.fn()
    const scope = createScope()
    withScope(scope, () => interval(fn, 50))
    expect(fn).not.toHaveBeenCalled()
    vi.advanceTimersByTime(50)
    expect(fn).toHaveBeenCalledTimes(1)
    vi.advanceTimersByTime(100)
    expect(fn).toHaveBeenCalledTimes(3)
    scope.dispose()
    vi.advanceTimersByTime(200)
    expect(fn).toHaveBeenCalledTimes(3)
  })
})
