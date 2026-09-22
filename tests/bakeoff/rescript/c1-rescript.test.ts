// Bakeoff ReScript track — DOM integration test.
//
// Mounts the registered `bakeoff-counter` element (the pinned-head compiler
// emit backed by the ReScript-compiled controller) and asserts the full
// loop: a click increments through Table-free counter scope, the write
// crosses into @ship/sail-runtime's signal graph via SailSignal.res, and
// the rendered text updates through a real effect flush.

import { beforeEach, describe, expect, it } from 'vitest'

// Module side effects: registers `<bakeoff-counter>` via component().
import './render/counter.js'

describe('bakeoff: ReScript controller × compiled sail render', () => {
  let host: HTMLElement

  beforeEach(() => {
    host = document.createElement('div')
    document.body.appendChild(host)
    host.innerHTML = '<bakeoff-counter></bakeoff-counter>'
  })

  it('renders 0, then a click drives the runtime signal graph into the text binding', async () => {
    const el = host.querySelector('bakeoff-counter')!
    // Light-DOM architecture: the template renders into the host element.
    const strong = el.querySelector('strong')!
    expect(strong.textContent).toBe('0')

    const button = el.querySelector('button')!
    button.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    // The signal graph runs effects synchronously on write; one macrotask
    // turn also absorbs any queued microtasks before we assert.
    await new Promise<void>((r) => queueMicrotask(r))

    expect(strong.textContent).toBe('1')

    // Second click proves the binding keeps flowing (2 after one more).
    button.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await new Promise<void>((r) => queueMicrotask(r))
    expect(strong.textContent).toBe('2')
  })
})
