// Proves the public author-facing test API (`renderRoute` / `renderComponent`)
// against the real Rust/WASM compiler + jsdom. This is the exemplar a route
// author copies: compile a `.sail`, mount it, assert the DOM, drive an event,
// assert the reaction.

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { renderComponent, renderRoute } from '../src/render'

describe('renderRoute — static template', () => {
  it('renders interpolated text and attributes', () => {
    const v = renderRoute(`<h1>Hello {who}</h1>`, {
      controller: () => ({ who: 'world' }),
    })
    expect(v.html()).toContain('Hello world')
    expect(v.root.querySelector('h1')).not.toBeNull()
    v.unmount()
  })
})

describe('renderRoute — reactive controller', () => {
  it('reflects a signal and updates on a click handler', async () => {
    const v = renderRoute(
      `<button on:click={() => count.set(count() + 1)}>{count()}</button>`,
      { controller: () => ({ count: signal(0) }) },
    )
    const btn = v.root.querySelector('button')!
    expect(btn.textContent).toBe('0')

    v.fire('button', 'click')
    await v.flush()
    expect(btn.textContent).toBe('1')

    // Scope is exposed — assert state directly too.
    expect((v.scope.count as () => number)()).toBe(1)
    v.unmount()
  })
})

describe('renderRoute — conditional branch', () => {
  it('shows the branch matching the signal', async () => {
    const v = renderRoute(
      `<if cond={ready()}><p data-testid="ok">ready</p><else><p data-testid="wait">waiting</p></else></if>`,
      { controller: () => ({ ready: signal(false) }) },
    )
    expect(v.root.querySelector('[data-testid="wait"]')).not.toBeNull()
    expect(v.root.querySelector('[data-testid="ok"]')).toBeNull()

    ;(v.scope.ready as ReturnType<typeof signal<boolean>>).set(true)
    await v.flush()
    expect(v.root.querySelector('[data-testid="ok"]')).not.toBeNull()
    expect(v.root.querySelector('[data-testid="wait"]')).toBeNull()
    v.unmount()
  })
})

// App-local component controllers use the same ambient vocabulary as route
// controllers. Bare `signal()` (no import) must mount and react.
describe('renderComponent — ambient signal on app-local CE', () => {
  it('mounts an app-local CE whose controller uses bare signal() with no import', async () => {
    const source = `
<controller>
export default controller(() => {
  const count = signal(0)
  return {
    count,
    inc: () => count.set(count() + 1),
  }
})
</controller>
<scope>
  props: {
    label = 'Probe'
  }
</scope>
<button data-probe on:click={inc}>{label()}:{count()}</button>
`
    const tag = 'sail-g2-ambient-probe'
    const { el, fire, flush, unmount } = renderComponent(source, tag)
    expect(el.isConnected).toBe(true)
    expect(el.textContent).toContain('Probe:0')
    fire('[data-probe]', 'click')
    await flush()
    expect(el.textContent).toContain('Probe:1')
    unmount()
  })
})
