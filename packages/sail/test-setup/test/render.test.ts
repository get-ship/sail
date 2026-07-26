// Proves the public author-facing test API (`renderRoute` / `renderComponent`)
// against the real Rust/WASM compiler + jsdom. This is the exemplar a route
// author copies: compile a `.sail`, mount it, assert the DOM, drive an event,
// assert the reaction.

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { renderRoute } from '../src/render'

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
