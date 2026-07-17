import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import {
  createScope,
  withScope,
  onCleanup,
} from '../src/effect-scope'
import { enableLeakWarn } from '../src/scope-resource-warn'

// FakeEventSource — jsdom doesn't ship one. Replicates only the shape
// the wrapper needs (constructor + `close`).
class FakeEventSource {
  closed = false
  constructor(public url: string | URL, public init?: EventSourceInit) {}
  close(): void {
    this.closed = true
  }
  addEventListener(): void {}
  removeEventListener(): void {}
}

describe('scope-resource-warn', () => {
  let restoreLeakWarn: (() => void) | null = null
  let warnSpy: ReturnType<typeof vi.spyOn>
  const originalES = globalThis.EventSource

  beforeEach(() => {
    // Install a fake before enabling the wrapper so wrapping works.
    ;(globalThis as { EventSource: typeof EventSource }).EventSource =
      FakeEventSource as unknown as typeof EventSource
    restoreLeakWarn = enableLeakWarn()
    warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})
  })

  afterEach(() => {
    restoreLeakWarn?.()
    restoreLeakWarn = null
    ;(globalThis as { EventSource: typeof EventSource }).EventSource = originalES
    warnSpy.mockRestore()
  })

  it('warns when a scope disposes with an EventSource still open', () => {
    const scope = createScope()
    withScope(scope, () => {
      // Allocate without registering cleanup — simulates the live() bug.
      new globalThis.EventSource('/_deck/data?expr=fake')
    })
    scope.dispose()

    expect(warnSpy).toHaveBeenCalled()
    const text = warnSpy.mock.calls.map(c => c.join(' ')).join('\n')
    expect(text).toMatch(/open resources/)
    expect(text).toMatch(/EventSource/)
  })

  it('does NOT warn when the EventSource is properly closed via cleanup', () => {
    const scope = createScope()
    withScope(scope, () => {
      const es = new globalThis.EventSource('/_deck/data?expr=fake')
      onCleanup(() => es.close())
    })
    scope.dispose()
    // No 'open resources' warning. The wrapper might still emit other
    // warnings (none expected) but specifically not the leak summary.
    const leakWarn = warnSpy.mock.calls.find(c => /open resources/.test(c.join(' ')))
    expect(leakWarn).toBeUndefined()
  })

  it('warns about setInterval that is never cleared', () => {
    const scope = createScope()
    withScope(scope, () => {
      const id = setInterval(() => {}, 1000)
      // Forgot to clear.
      void id
    })
    scope.dispose()
    const text = warnSpy.mock.calls.map(c => c.join(' ')).join('\n')
    expect(text).toMatch(/setInterval/)
  })

  it('does NOT warn when setInterval is properly cleared', () => {
    const scope = createScope()
    withScope(scope, () => {
      const id = setInterval(() => {}, 1000)
      onCleanup(() => clearInterval(id))
    })
    scope.dispose()
    const leakWarn = warnSpy.mock.calls.find(c => /open resources/.test(c.join(' ')))
    expect(leakWarn).toBeUndefined()
  })
})
