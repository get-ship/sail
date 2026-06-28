import { describe, expect, it } from 'vitest'
import { component } from '../src/index'
import { mountComponent } from '../src/compose'
import { createScope, withScope, onCleanup } from '../src/effect-scope'
import { LIFECYCLE_KEY } from '../src/controller-resolver'

describe('mountComponent — child composition (mount-form)', () => {
  it('runs setup + __mount into the host and returns a working disposer', () => {
    const mod = component(() => ({
      __mount(host: HTMLElement) {
        const el = document.createElement('span')
        el.textContent = 'child'
        host.appendChild(el)
        return () => {
          host.textContent = ''
        }
      },
    }))

    const host = document.createElement('div')
    const dispose = mountComponent(mod, host, {})
    expect(host.textContent).toBe('child')

    dispose()
    expect(host.textContent).toBe('')
  })

  it("owns the child's scope: parent unmount disposes the child's cleanups", () => {
    let cleaned = 0
    const mod = component(() => {
      // `onCleanup` enrolls on whatever scope is current during setup — inside
      // mountComponent that is the CHILD scope. This is what live()/data()/hooks
      // register, so it's the real ownership contract being tested.
      onCleanup(() => {
        cleaned++
      })
      return { __mount: (_host: HTMLElement) => () => {} }
    })

    const parent = createScope()
    const host = document.createElement('div')
    withScope(parent, () => mountComponent(mod, host, {}))

    expect(cleaned).toBe(0)
    parent.dispose() // parent unmount → child scope auto-disposes (createChildScope)
    expect(cleaned).toBe(1)
  })

  it('disposer is idempotent and independent of the parent', () => {
    let cleaned = 0
    const mod = component(() => {
      onCleanup(() => {
        cleaned++
      })
      return { __mount: (_host: HTMLElement) => () => {} }
    })

    const dispose = mountComponent(mod, document.createElement('div'), {})
    dispose()
    dispose() // second call is a no-op (scope dispose guard)
    expect(cleaned).toBe(1)
  })

  it('wires controller mount()/unmount() from the LIFECYCLE_KEY bundle', () => {
    let mountedHost: HTMLElement | undefined
    let mountCleanup = 0
    let unmounted = 0
    const mod = component(() => {
      const ret: Record<string, unknown> = { __mount: (_h: HTMLElement) => () => {} }
      Object.defineProperty(ret, LIFECYCLE_KEY, {
        value: {
          state: {},
          mount: (_state: Record<string, unknown>, el: HTMLElement) => {
            mountedHost = el
            return () => {
              mountCleanup++
            }
          },
          unmount: () => {
            unmounted++
          },
        },
        enumerable: false,
      })
      return ret
    })

    const host = document.createElement('div')
    const dispose = mountComponent(mod, host, {})
    expect(mountedHost).toBe(host) // mount() received the live host
    expect(unmounted).toBe(0)

    dispose()
    expect(mountCleanup).toBe(1) // mount()'s returned cleanup ran
    expect(unmounted).toBe(1) // unmount() ran
  })
})
