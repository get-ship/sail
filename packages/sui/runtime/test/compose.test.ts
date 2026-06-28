import { describe, expect, it } from 'vitest'
import { component } from '../src/index'
import { mountComponent } from '../src/compose'
import { createScope, withScope, onCleanup } from '../src/effect-scope'

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
})
