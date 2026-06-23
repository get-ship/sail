// Declarative form-association tests for component({ formValue, formDisabled }).
//
// jsdom's ElementInternals.setFormValue exists but the FormData
// round-trip lives in Playwright; here we assert the wiring: the
// effect calls setFormValue with the stringified prop value, reset
// restores the prop's declared default, restore decodes through the
// prop's decoder, and disabled drives the bool signal.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { component, prop } from '../src/component-factory'
import type { Signal } from '../src/signal'

let tagCounter = 0
const nextTag = () => `form-prop-${++tagCounter}`

// jsdom's ElementInternals.setFormValue is partial — the spy/round-trip needs
// the real webview tier (proof owed in #164). Gate the suite's setFormValue
// assertions on it; do NOT silently early-return past the only assert.
const setFormValueAbsent =
  typeof (window as unknown as { ElementInternals?: { prototype: ElementInternals } })
    .ElementInternals?.prototype?.setFormValue !== 'function'

const mounted: HTMLElement[] = []
afterEach(() => {
  while (mounted.length) mounted.pop()?.remove()
})

const mount = (tag: string, attrs: Record<string, string> = {}): HTMLElement => {
  const el = document.createElement(tag)
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v)
  document.body.appendChild(el)
  mounted.push(el)
  return el
}

describe('component() with formValue + formDisabled', () => {
  it('registers the element with static formAssociated = true', () => {
    const tag = nextTag()
    component(tag, {
      props: { value: prop.string({ default: '' }) },
      formValue: 'value',
    })
    const ctor = customElements.get(tag) as unknown as { formAssociated?: boolean }
    expect(ctor.formAssociated).toBe(true)
  })

  it.skipIf(setFormValueAbsent)('calls internals.setFormValue with the stringified signal value', () => {
    const tag = nextTag()
    let signalRef: Signal<unknown> | undefined
    component(tag, {
      props: { value: prop.string({ default: 'init' }) },
      formValue: 'value',
      setup: ({ props }) => { signalRef = props.value },
    })
    const el = mount(tag)
    const internals = (el as unknown as { internals?: ElementInternals })
      .internals
    // Access via the runtime's stored prop. We don't have a public
    // reference to internals from the test surface; spy through the
    // host element's attachInternals — already called by defineComponent.
    // Instead, assert that the value reactively flows via FormData
    // listener-friendly path: we monkey-patch setFormValue on the
    // prototype to record calls.
    void internals
    const calls: Array<string | null> = []
    // Re-mount with a spy on ElementInternals.setFormValue. jsdom's
    // implementation exists but we can swap to a spy via a wrapper.
    el.remove()
    const proto = (window as unknown as { ElementInternals: { prototype: ElementInternals } }).ElementInternals.prototype
    const orig = proto.setFormValue
    proto.setFormValue = function (v: unknown) {
      calls.push(v as string | null)
    }
    try {
      const el2 = mount(tag)
      // Initial setFormValue from effect run.
      expect(calls.length).toBeGreaterThanOrEqual(1)
      expect(calls[calls.length - 1]).toBe('init')
      signalRef!.set('next')
      expect(calls[calls.length - 1]).toBe('next')
      void el2
    } finally {
      proto.setFormValue = orig
    }
  })

  it('formResetCallback restores the prop signal to its declared default', () => {
    const tag = nextTag()
    let signalRef: Signal<unknown> | undefined
    component(tag, {
      props: { value: prop.string({ default: 'home' }) },
      formValue: 'value',
      setup: ({ props }) => { signalRef = props.value },
    })
    const el = mount(tag)
    signalRef!.set('away')
    expect(signalRef!()).toBe('away')
    ;(el as unknown as { formResetCallback?: () => void }).formResetCallback!()
    expect(signalRef!()).toBe('home')
  })

  it('formStateRestoreCallback decodes through the prop decoder', () => {
    const tag = nextTag()
    let signalRef: Signal<unknown> | undefined
    component(tag, {
      props: { active: prop.bool({ default: false }) },
      formValue: 'active',
      formDisabled: 'active',
    })
    // formDisabled requires a bool prop. Register a second component
    // that uses string for formValue + bool for formDisabled.
    const tag2 = nextTag()
    component(tag2, {
      props: {
        value: prop.string({ default: '' }),
        disabled: prop.bool({ default: false }),
      },
      formValue: 'value',
      formDisabled: 'disabled',
      setup: ({ props }) => { signalRef = props.value },
    })
    const el = mount(tag2)
    ;(el as unknown as {
      formStateRestoreCallback?: (s: string | null, m: string) => void
    }).formStateRestoreCallback!('restored', 'restore')
    expect(signalRef!()).toBe('restored')
  })

  it('formDisabledCallback drives the bool prop signal', () => {
    const tag = nextTag()
    let disabledSig: Signal<unknown> | undefined
    component(tag, {
      props: { disabled: prop.bool({ default: false }) },
      formDisabled: 'disabled',
      setup: ({ props }) => { disabledSig = props.disabled },
    })
    const el = mount(tag)
    expect(disabledSig!()).toBe(false)
    ;(el as unknown as { formDisabledCallback?: (d: boolean) => void })
      .formDisabledCallback!(true)
    expect(disabledSig!()).toBe(true)
    ;(el as unknown as { formDisabledCallback?: (d: boolean) => void })
      .formDisabledCallback!(false)
    expect(disabledSig!()).toBe(false)
  })

  it('author setup can still reach internals for setValidity', () => {
    const tag = nextTag()
    const seen = vi.fn()
    component(tag, {
      props: { value: prop.string({ default: '' }) },
      formValue: 'value',
      setup: ({ internals }) => {
        seen(internals !== undefined)
      },
    })
    mount(tag)
    expect(seen).toHaveBeenCalledWith(true)
  })

  it('clears form hooks on disconnect', () => {
    const tag = nextTag()
    component(tag, {
      props: { value: prop.string({ default: '' }) },
      formValue: 'value',
    })
    const el = mount(tag)
    expect(
      (el as unknown as { __shipSetFormHooks?: unknown }).__shipSetFormHooks,
    ).toBeDefined()
    el.remove()
    expect(
      (el as unknown as { __shipSetFormHooks?: unknown }).__shipSetFormHooks,
    ).toBeUndefined()
  })
})

// #307: form-associated components must expose `el.value` (and any other
// declared prop) as a prototype getter that reads the prop signal. Without
// this, plain DOM code that does `formEl.value` reads `HTMLElement.value`
// (undefined) instead of the form value — silently invisible to JS.
describe('form-associated component — prop accessors (#307)', () => {
  it('exposes the form-value prop on the prototype as a getter', () => {
    const tag = nextTag()
    component(tag, {
      props: { value: prop.string({ default: '' }) },
      formValue: 'value',
    })
    const el = mount(tag, { value: 'hello' })
    expect((el as unknown as { value: unknown }).value).toBe('hello')
  })

  it('exposes form-disabled as a getter that reflects the attr', () => {
    const tag = nextTag()
    component(tag, {
      props: {
        value: prop.string({ default: '' }),
        disabled: prop.bool({ default: false }),
      },
      formValue: 'value',
      formDisabled: 'disabled',
    })
    const el = mount(tag, { value: 'x', disabled: '' })
    expect((el as unknown as { disabled: unknown }).disabled).toBe(true)
  })

  it('non-form prop accessors coexist on the form class', () => {
    const tag = nextTag()
    component(tag, {
      props: {
        value: prop.string({ default: '' }),
        label: prop.string({ default: '' }),
      },
      formValue: 'value',
    })
    const el = mount(tag, { value: 'v1', label: 'l1' })
    const e = el as unknown as { value: unknown; label: unknown }
    expect(e.value).toBe('v1')
    expect(e.label).toBe('l1')
  })
})