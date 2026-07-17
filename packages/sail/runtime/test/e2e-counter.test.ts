// End-to-end: hand-crafted module mirroring the shape the ship-set
// compiler emits for a counter component (see
// crates/ship-set-compiler/tests/fixtures/counter.set (TODO Phase 4)). Proves the
// runtime contract — defineComponent + signal + derived + bindText +
// on — composes into a working custom element.

import { describe, expect, it } from 'vitest'
import {
  defineComponent,
  signal,
  derived,
  bindText,
  on,
} from '../src/index'

describe('e2e: compiled counter', () => {
  it('mounts, updates on click, runs cleanup', () => {
    const tag = 'ship-set-e2e-counter'
    const HTML = `<div class="counter"><p>Count: <span data-st="0"></span></p><p>Doubled: <span data-st="1"></span></p><button data-sa="0">+1</button><button data-sa="1">reset</button></div>`

    defineComponent(
      tag,
      { observedAttrs: [], props: {} },
      (host, _propSignals) => {
        const cleanups: Array<() => void> = []
        const count = signal(0)
        const doubled = derived(() => count() * 2)
        const inc = () => count.set(count() + 1)
        const reset = () => count.set(0)

        host.innerHTML = HTML
        {
          const _n = host.querySelector('[data-st="0"]')
          if (_n) cleanups.push(bindText(_n, () => String(count()), [count]))
        }
        {
          const _n = host.querySelector('[data-st="1"]')
          if (_n) cleanups.push(bindText(_n, () => String(doubled()), [doubled]))
        }
        {
          const _n = host.querySelector('[data-sa="0"]')
          if (_n) cleanups.push(on(_n, 'click', inc))
        }
        {
          const _n = host.querySelector('[data-sa="1"]')
          if (_n) cleanups.push(on(_n, 'click', reset))
        }

        return () => { for (const c of cleanups) try { c() } catch {} }
      },
    )

    const el = document.createElement(tag)
    document.body.appendChild(el)

    const countEl = el.querySelector('[data-st="0"]')!
    const doubledEl = el.querySelector('[data-st="1"]')!
    const incBtn = el.querySelector('[data-sa="0"]') as HTMLButtonElement
    const resetBtn = el.querySelector('[data-sa="1"]') as HTMLButtonElement

    expect(countEl.textContent).toBe('0')
    expect(doubledEl.textContent).toBe('0')

    incBtn.click()
    incBtn.click()
    incBtn.click()
    expect(countEl.textContent).toBe('3')
    expect(doubledEl.textContent).toBe('6')

    resetBtn.click()
    expect(countEl.textContent).toBe('0')
    expect(doubledEl.textContent).toBe('0')

    el.remove()
    // After unmount, further updates do not throw (signals are GC'd
    // implicitly; our cleanup ran).
    incBtn.click()
    expect(countEl.textContent).toBe('0')
  })
})
