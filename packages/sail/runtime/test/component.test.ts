import { describe, expect, it, vi } from 'vitest'
import { defineComponent, camelOf } from '../src/component'
import { signal } from '../src/signal'
import type { Signal } from '../src/signal'

let tagCounter = 0
const nextTag = () => `sail-test-${++tagCounter}`

describe('camelOf', () => {
  it('single word', () => expect(camelOf('count')).toBe('count'))
  it('kebab two', () => expect(camelOf('fetch-url')).toBe('fetchUrl'))
  it('kebab three', () => expect(camelOf('a-b-c')).toBe('aBC'))
})

describe('defineComponent', () => {
  it('registers a custom element', () => {
    const tag = nextTag()
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    expect(customElements.get(tag)).toBeDefined()
  })

  it('re-defining our own tag is a silent HMR hot-swap (no warn)', () => {
    // Pre-#40 this was a warned no-op. Now a re-define of OUR tag swaps the impl
    // (component hot-swap), so it must NOT warn — that would spam every edit.
    const tag = nextTag()
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    expect(warn).not.toHaveBeenCalled()
    warn.mockRestore()
  })

  it('warns once on a FOREIGN tag collision (a tag we did not define)', () => {
    // The collision warn is preserved for the genuine case: a tag registered
    // OUTSIDE defineComponent (another module reaching customElements.define).
    const tag = nextTag()
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    customElements.define(tag, class extends HTMLElement {})
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    expect(warn).toHaveBeenCalledTimes(1)
    expect(warn.mock.calls[0]![0]).toMatch(/called twice/)
    warn.mockRestore()
  })

  it('re-define hot-swaps live instances (HMR), clearing old DOM', () => {
    const tag = nextTag()
    const renderInto = (label: string) => (host: HTMLElement) => {
      const s = document.createElement('span')
      s.textContent = label
      host.append(s) // APPEND (not replace) — proves __suiReRun clears old DOM
      return () => {}
    }
    defineComponent(tag, { observedAttrs: [], props: {} }, renderInto('A'))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelectorAll('span').length).toBe(1)
    expect(el.textContent).toBe('A')

    // Re-define (what an HMR re-import does) with a NEW factory. The LIVE
    // instance must re-render with the new code — and NOT duplicate the DOM.
    defineComponent(tag, { observedAttrs: [], props: {} }, renderInto('B'))
    expect(el.textContent).toBe('B')
    expect(el.querySelectorAll('span').length).toBe(1)

    // A fresh instance mounted after the swap uses the new factory too.
    const el2 = document.createElement(tag)
    document.body.appendChild(el2)
    expect(el2.textContent).toBe('B')
    el.remove()
    el2.remove()
  })

  it('runs factory on connect, cleanup on disconnect', () => {
    const tag = nextTag()
    const cleanup = vi.fn()
    const factory = vi.fn(() => cleanup)
    defineComponent(tag, { observedAttrs: [], props: {} }, factory)
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(factory).toHaveBeenCalledTimes(1)
    expect(cleanup).not.toHaveBeenCalled()
    el.remove()
    expect(cleanup).toHaveBeenCalledTimes(1)
  })

  it('sets data-component for CSS scoping', () => {
    const tag = nextTag()
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.dataset.component).toBe(tag)
    el.remove()
  })

  it('routes attribute changes through prop signals', () => {
    const tag = nextTag()
    let captured: Signal<unknown> | undefined
    defineComponent(
      tag,
      {
        observedAttrs: ['count'],
        props: { count: { camel: 'count', decode: (r) => Number(r) } },
      },
      (_host, propSignals) => {
        propSignals.count = signal(0)
        captured = propSignals.count
        return () => {}
      },
    )
    const el = document.createElement(tag)
    document.body.appendChild(el)
    el.setAttribute('count', '7')
    expect(captured!()).toBe(7)
    el.remove()
  })

  it('routes a JS property write through the prop signal (prop:X path)', () => {
    // The compiler's `prop:X={v}` binding sets `el.X = v` as a JS property.
    // Without an accessor that delegates to the signal, this writes a dead
    // expando and the component never reacts — the bug behind dialogs that
    // wouldn't open. The accessor must forward to `propSignals[camel]`.
    const tag = nextTag()
    let captured: Signal<unknown> | undefined
    defineComponent(
      tag,
      {
        observedAttrs: ['open'],
        props: { open: { camel: 'open', decode: (r) => r !== null } },
      },
      (_host, propSignals) => {
        propSignals.open = signal(false)
        captured = propSignals.open
        return () => {}
      },
    )
    const el = document.createElement(tag) as HTMLElement & { open: boolean }
    document.body.appendChild(el)
    el.open = true
    expect(captured!()).toBe(true)
    // Reading the property reflects the signal value (untracked point-read).
    expect(el.open).toBe(true)
    el.remove()
  })

  it('buffers a pre-connect property write and flushes on connect', () => {
    // `prop:X` can fire before the element is connected (signals not created
    // yet). The accessor buffers into pendingProps; connectedCallback flushes
    // it once the factory has populated propSignals.
    const tag = nextTag()
    let captured: Signal<unknown> | undefined
    defineComponent(
      tag,
      {
        observedAttrs: ['count'],
        props: { count: { camel: 'count', decode: (r) => Number(r) } },
      },
      (_host, propSignals) => {
        propSignals.count = signal(0)
        captured = propSignals.count
        return () => {}
      },
    )
    const el = document.createElement(tag) as HTMLElement & { count: number }
    el.count = 42 // before connect — signal does not exist yet
    document.body.appendChild(el)
    expect(captured!()).toBe(42)
    el.remove()
  })

  it('does not shadow native HTMLElement props (e.g. title)', () => {
    // Names already on the prototype chain keep the native attribute path —
    // the accessor loop skips them so we never break built-in behavior.
    const tag = nextTag()
    defineComponent(
      tag,
      {
        observedAttrs: ['title'],
        props: { title: { camel: 'title', decode: (r) => r } },
      },
      (_host, propSignals) => {
        propSignals.title = signal('')
        return () => {}
      },
    )
    const el = document.createElement(tag)
    document.body.appendChild(el)
    el.title = 'native'
    // Native title reflects to the attribute — proof we didn't override it.
    expect(el.getAttribute('title')).toBe('native')
    el.remove()
  })

  it('cleanup error does not throw', () => {
    const tag = nextTag()
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {
      throw new Error('boom')
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(() => el.remove()).not.toThrow()
    expect(spy).toHaveBeenCalled()
    spy.mockRestore()
  })
})
