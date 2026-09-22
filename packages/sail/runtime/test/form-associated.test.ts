// Form-associated component sanity. Verifies the runtime wires
// ElementInternals correctly and the schema flag triggers the
// form-aware class shape.

import { describe, expect, it, vi } from 'vitest'
import { defineComponent } from '../src/component'
import { signal } from '../src/signal'

let tagCounter = 0
const nextTag = () => `sail-form-${++tagCounter}`

describe('defineComponent (formAssociated)', () => {
  it('sets static formAssociated = true on the class', () => {
    const tag = nextTag()
    defineComponent(
      tag,
      { observedAttrs: [], props: {}, formAssociated: true },
      () => () => {},
    )
    const ctor = customElements.get(tag) as any
    expect(ctor.formAssociated).toBe(true)
  })

  it('passes an internals object as the third factory argument', () => {
    const tag = nextTag()
    let captured: any = 'untouched'
    defineComponent(
      tag,
      { observedAttrs: [], props: {}, formAssociated: true },
      (_host, _propSignals, internals) => {
        captured = internals
        return () => {}
      },
    )
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(captured).toBeDefined()
    expect(captured).not.toBe('untouched')
    // jsdom's ElementInternals is partial — setFormValue/setValidity
    // are missing in 25. The wiring is verified; full method coverage
    // happens in real-browser Playwright tests (TODO).
    el.remove()
  })

  it('does NOT pass internals when formAssociated is unset', () => {
    const tag = nextTag()
    let captured: any = 'untouched'
    defineComponent(
      tag,
      { observedAttrs: [], props: {} },
      (_host, _propSignals, internals) => {
        captured = internals
        return () => {}
      },
    )
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(captured).toBeUndefined()
    el.remove()
  })

  it('formResetCallback dispatches sail:form-reset on the host', () => {
    const tag = nextTag()
    defineComponent(
      tag,
      { observedAttrs: [], props: {}, formAssociated: true },
      () => () => {},
    )
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const onReset = vi.fn()
    el.addEventListener('sail:form-reset', onReset)
    // Invoke manually — jsdom doesn't auto-fire on form reset.
    ;(el as any).formResetCallback()
    expect(onReset).toHaveBeenCalledTimes(1)
    el.remove()
  })

  it('host can be appended inside a <form> without errors', () => {
    // Smoke test: the form-associated class composes with native forms
    // even when jsdom's ElementInternals API is partial. Real
    // setFormValue / FormData round-trip lives in Playwright.
    const tag = nextTag()
    defineComponent(
      tag,
      { observedAttrs: [], props: {}, formAssociated: true },
      () => () => {},
    )
    const form = document.createElement('form')
    const el = document.createElement(tag)
    el.setAttribute('name', 'sample')
    expect(() => form.appendChild(el)).not.toThrow()
    document.body.appendChild(form)
    // The host is a live DOM child of the form (the structural fact jsdom can
    // confirm). jsdom does NOT enroll form-associated custom elements into
    // `form.elements` — that enrollment + the FormData round-trip is proven in
    // the webview tier (#164), so we assert containment, not `form.elements`.
    expect(el.parentElement).toBe(form)
    expect(form.contains(el)).toBe(true)
    form.remove()
  })
})
