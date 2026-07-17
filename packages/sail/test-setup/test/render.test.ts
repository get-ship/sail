// Proves the public author-facing test API (`renderView` / `renderComponent`)
// against the real Rust/WASM compiler + jsdom. This is the exemplar a view
// author copies: compile a `.sail`, mount it, assert the DOM, drive an event,
// assert the reaction.

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { renderView } from '../src/render'

describe('renderView — static template', () => {
  it('renders interpolated text and attributes', () => {
    const v = renderView(`<h1 class="title">Hello {who}</h1>`, {
      controller: () => ({ who: 'world' }),
    })
    expect(v.html()).toContain('Hello world')
    expect(v.root.querySelector('.title')).not.toBeNull()
    v.unmount()
  })
})

describe('renderView — reactive controller', () => {
  it('reflects a signal and updates on a click handler', async () => {
    const v = renderView(
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

describe('renderView — conditional branch', () => {
  it('shows the branch matching the signal', async () => {
    const v = renderView(
      `{#if ready()}<p class="ok">ready</p>{:else}<p class="wait">waiting</p>{/if}`,
      { controller: () => ({ ready: signal(false) }) },
    )
    expect(v.root.querySelector('.wait')).not.toBeNull()
    expect(v.root.querySelector('.ok')).toBeNull()

    ;(v.scope.ready as ReturnType<typeof signal<boolean>>).set(true)
    await v.flush()
    expect(v.root.querySelector('.ok')).not.toBeNull()
    expect(v.root.querySelector('.wait')).toBeNull()
    v.unmount()
  })
})
