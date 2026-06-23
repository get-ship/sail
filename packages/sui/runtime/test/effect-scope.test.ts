import { describe, it, expect, vi } from 'vitest'

import {
  createScope,
  createChildScope,
  withScope,
  onCleanup,
  currentScope,
  type EffectScope,
} from '../src/effect-scope'

describe('effect-scope', () => {
  it('runs registered cleanups in reverse order on dispose', () => {
    const scope = createScope()
    const order: string[] = []
    withScope(scope, () => {
      onCleanup(() => order.push('a'))
      onCleanup(() => order.push('b'))
      onCleanup(() => order.push('c'))
    })
    scope.dispose()
    expect(order).toEqual(['c', 'b', 'a'])
  })

  it('treats onCleanup outside a scope as a no-op', () => {
    // No throws, no side effects — just doesn't register anywhere.
    expect(() => onCleanup(() => {})).not.toThrow()
    expect(currentScope()).toBeNull()
  })

  it('restores the prior scope on return AND on throw', () => {
    const outer = createScope()
    const inner = createScope()
    let observedInside: EffectScope | null = null
    withScope(outer, () => {
      withScope(inner, () => {
        observedInside = currentScope()
      })
      expect(currentScope()).toBe(outer)
    })
    expect(observedInside).toBe(inner)
    expect(currentScope()).toBeNull()

    // Throwing path
    expect(() => {
      withScope(outer, () => {
        throw new Error('boom')
      })
    }).toThrow('boom')
    expect(currentScope()).toBeNull()
  })

  it('dispose is idempotent', () => {
    const scope = createScope()
    const cleanup = vi.fn()
    withScope(scope, () => onCleanup(cleanup))
    scope.dispose()
    scope.dispose()
    scope.dispose()
    expect(cleanup).toHaveBeenCalledTimes(1)
  })

  it('logs and continues when a cleanup throws', () => {
    const scope = createScope()
    const after = vi.fn()
    const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    withScope(scope, () => {
      onCleanup(after) // last registered → first to run
      onCleanup(() => {
        throw new Error('cleanup failed')
      })
    })
    scope.dispose()
    expect(after).toHaveBeenCalledOnce()
    expect(errSpy).toHaveBeenCalledWith(
      expect.stringContaining('[sui/scope] cleanup threw'),
      expect.any(Error),
    )
    errSpy.mockRestore()
  })

  it('disposes child when parent disposes', () => {
    const parent = createScope()
    const childFn = vi.fn()
    let child: EffectScope | undefined
    withScope(parent, () => {
      child = createChildScope()
      withScope(child, () => onCleanup(childFn))
    })
    expect(child!.disposed).toBe(false)
    parent.dispose()
    expect(child!.disposed).toBe(true)
    expect(childFn).toHaveBeenCalledOnce()
  })

  it('child stays alive when its scope finishes but parent is still active', () => {
    const parent = createScope()
    let child: EffectScope | undefined
    const childFn = vi.fn()
    withScope(parent, () => {
      child = createChildScope()
      withScope(child, () => onCleanup(childFn))
      // child WAS the current scope here, but withScope returned —
      // child itself is still alive until its parent disposes.
    })
    expect(child!.disposed).toBe(false)
    expect(childFn).not.toHaveBeenCalled()

    parent.dispose()
    expect(child!.disposed).toBe(true)
    expect(childFn).toHaveBeenCalledOnce()
  })

  it('runs late-registered cleanups immediately if scope already disposed', () => {
    const scope = createScope()
    scope.dispose()
    const fn = vi.fn()
    scope.cleanup(fn)
    expect(fn).toHaveBeenCalledOnce()
  })

  it('createChildScope without explicit parent picks up the current scope', () => {
    const root = createScope()
    let child: EffectScope | undefined
    withScope(root, () => {
      child = createChildScope()
    })
    expect(child!.disposed).toBe(false)
    root.dispose()
    expect(child!.disposed).toBe(true)
  })

  it('disposed flag flips on dispose', () => {
    const scope = createScope()
    expect(scope.disposed).toBe(false)
    scope.dispose()
    expect(scope.disposed).toBe(true)
  })

  it('cleanup that registers another cleanup during dispose runs it immediately', () => {
    const scope = createScope()
    const late = vi.fn()
    withScope(scope, () => {
      onCleanup(() => {
        // During dispose, scope.disposed is true → cleanup() runs late
        // registrations immediately.
        scope.cleanup(late)
      })
    })
    scope.dispose()
    expect(late).toHaveBeenCalledOnce()
  })

  it('cleanup that itself disposes the same scope is idempotent', () => {
    const scope = createScope()
    const order: string[] = []
    withScope(scope, () => {
      onCleanup(() => order.push('a'))
      onCleanup(() => {
        order.push('b')
        // Reentrant dispose is a no-op (already in progress / completed).
        scope.dispose()
      })
      onCleanup(() => order.push('c'))
    })
    scope.dispose()
    // Reverse order: c then b then a. Reentrant scope.dispose() inside b
    // returns immediately because `disposed = true` already.
    expect(order).toEqual(['c', 'b', 'a'])
  })

  it('cleanup that throws AND a downstream cleanup that throws — both errors logged, all siblings run', () => {
    const scope = createScope()
    const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const ran: string[] = []
    withScope(scope, () => {
      onCleanup(() => ran.push('a'))
      onCleanup(() => {
        ran.push('b')
        throw new Error('b-fail')
      })
      onCleanup(() => ran.push('c'))
      onCleanup(() => {
        ran.push('d')
        throw new Error('d-fail')
      })
    })
    scope.dispose()
    // Reverse-order: d (throws), c, b (throws), a — all four ran.
    expect(ran).toEqual(['d', 'c', 'b', 'a'])
    expect(errSpy).toHaveBeenCalledTimes(2)
    errSpy.mockRestore()
  })

  it('grandchild auto-disposes when grandparent disposes', () => {
    const grand = createScope()
    let child: EffectScope | undefined
    let grandchild: EffectScope | undefined
    const grandchildFn = vi.fn()
    withScope(grand, () => {
      child = createChildScope()
      withScope(child!, () => {
        grandchild = createChildScope()
        withScope(grandchild!, () => onCleanup(grandchildFn))
      })
    })
    expect(grandchild!.disposed).toBe(false)
    grand.dispose()
    expect(child!.disposed).toBe(true)
    expect(grandchild!.disposed).toBe(true)
    expect(grandchildFn).toHaveBeenCalledOnce()
  })

  it('onCleanup outside any scope after the current scope has been popped is a no-op', () => {
    const scope = createScope()
    withScope(scope, () => {
      onCleanup(() => {})
    })
    // After withScope returns, currentScope is null again.
    const fn = vi.fn()
    onCleanup(fn) // no-op
    scope.dispose()
    expect(fn).not.toHaveBeenCalled()
  })
})
