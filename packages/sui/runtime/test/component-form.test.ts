// Declarative form-association tests for component({ formValue, formDisabled }).
//
// jsdom's ElementInternals.setFormValue exists but the FormData round-trip
// lives in Playwright; here we assert the wiring: the effect calls setFormValue
// with the stringified prop value, reset restores the prop's declared default,
// restore decodes through the prop's decoder, and disabled drives the bool
// signal. A registered component is now `component(tag, def, module)`; tests
// that need the prop signal capture it via the module's `ctx.props`.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { component, prop } from '../src/component-factory'
import type { Signal } from '../src/signal'

let tagCounter = 0
const nextTag = () => `form-prop-${++tagCounter}`

const noopModule = () => component(() => ({}))

// HOST-CONDITIONAL (intentional, not fossilized) — every `it.skipIf(setFormValueAbsent)`
// below is gated on jsdom's partial ElementInternals.setFormValue; the spy/round-trip
// needs the real webview tier (proof owed in #164). When jsdom ships a real
// setFormValue, the gate flips automatically — no test logic changes.
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
    component(tag, { props: { value: prop.string({ default: '' }) }, formValue: 'value' }, noopModule())
    const ctor = customElements.get(tag) as unknown as { formAssociated?: boolean }
    expect(ctor.formAssociated).toBe(true)
  })

  it.skipIf(setFormValueAbsent)('calls internals.setFormValue with the stringified signal value', () => {
    const tag = nextTag()
    let signalRef: Signal<unknown> | undefined
    component(
      tag,
      { props: { value: prop.string({ default: 'init' }) }, formValue: 'value' },
      component((ctx) => {
        signalRef = ctx.props.value as Signal<unknown>
        return {}
      }),
    )
    const calls: Array<string | null> = []
    const proto = (window as unknown as { ElementInternals: { prototype: ElementInternals } })
      .ElementInternals.prototype
    const orig = proto.setFormValue
    proto.setFormValue = function (v: unknown) {
      calls.push(v as string | null)
    }
    try {
      mount(tag)
      expect(calls.length).toBeGreaterThanOrEqual(1)
      expect(calls[calls.length - 1]).toBe('init')
      signalRef!.set('next')
      expect(calls[calls.length - 1]).toBe('next')
    } finally {
      proto.setFormValue = orig
    }
  })

  it('formResetCallback restores the prop signal to its declared default', () => {
    const tag = nextTag()
    let signalRef: Signal<unknown> | undefined
    component(
      tag,
      { props: { value: prop.string({ default: 'home' }) }, formValue: 'value' },
      component((ctx) => {
        signalRef = ctx.props.value as Signal<unknown>
        return {}
      }),
    )
    const el = mount(tag)
    signalRef!.set('away')
    expect(signalRef!()).toBe('away')
    ;(el as unknown as { formResetCallback?: () => void }).formResetCallback!()
    expect(signalRef!()).toBe('home')
  })

  it('formStateRestoreCallback decodes through the prop decoder', () => {
    const tag = nextTag()
    let signalRef: Signal<unknown> | undefined
    const tag2 = nextTag()
    component(
      tag2,
      {
        props: {
          value: prop.string({ default: '' }),
          disabled: prop.bool({ default: false }),
        },
        formValue: 'value',
        formDisabled: 'disabled',
      },
      component((ctx) => {
        signalRef = ctx.props.value as Signal<unknown>
        return {}
      }),
    )
    const el = mount(tag2)
    ;(el as unknown as {
      formStateRestoreCallback?: (s: string | null, m: string) => void
    }).formStateRestoreCallback!('restored', 'restore')
    expect(signalRef!()).toBe('restored')
  })

  it('formDisabledCallback drives the bool prop signal', () => {
    const tag = nextTag()
    let disabledSig: Signal<unknown> | undefined
    component(
      tag,
      { props: { disabled: prop.bool({ default: false }) }, formDisabled: 'disabled' },
      component((ctx) => {
        disabledSig = ctx.props.disabled as Signal<unknown>
        return {}
      }),
    )
    const el = mount(tag)
    expect(disabledSig!()).toBe(false)
    ;(el as unknown as { formDisabledCallback?: (d: boolean) => void }).formDisabledCallback!(true)
    expect(disabledSig!()).toBe(true)
    ;(el as unknown as { formDisabledCallback?: (d: boolean) => void }).formDisabledCallback!(false)
    expect(disabledSig!()).toBe(false)
  })

  it('the module ctx can still reach internals for setValidity', () => {
    const tag = nextTag()
    const seen = vi.fn()
    component(
      tag,
      { props: { value: prop.string({ default: '' }) }, formValue: 'value' },
      component((ctx) => {
        seen((ctx as { internals?: ElementInternals }).internals !== undefined)
        return {}
      }),
    )
    mount(tag)
    expect(seen).toHaveBeenCalledWith(true)
  })

  it('clears form hooks on disconnect', () => {
    const tag = nextTag()
    component(tag, { props: { value: prop.string({ default: '' }) }, formValue: 'value' }, noopModule())
    const el = mount(tag)
    expect((el as unknown as { __shipSetFormHooks?: unknown }).__shipSetFormHooks).toBeDefined()
    el.remove()
    expect((el as unknown as { __shipSetFormHooks?: unknown }).__shipSetFormHooks).toBeUndefined()
  })
})

// #307: form-associated components must expose `el.value` (and any other
// declared prop) as a prototype getter that reads the prop signal.
describe('form-associated component — prop accessors (#307)', () => {
  it('exposes the form-value prop on the prototype as a getter', () => {
    const tag = nextTag()
    component(tag, { props: { value: prop.string({ default: '' }) }, formValue: 'value' }, noopModule())
    const el = mount(tag, { value: 'hello' })
    expect((el as unknown as { value: unknown }).value).toBe('hello')
  })

  it('exposes form-disabled as a getter that reflects the attr', () => {
    const tag = nextTag()
    component(
      tag,
      {
        props: {
          value: prop.string({ default: '' }),
          disabled: prop.bool({ default: false }),
        },
        formValue: 'value',
        formDisabled: 'disabled',
      },
      noopModule(),
    )
    const el = mount(tag, { value: 'x', disabled: '' })
    expect((el as unknown as { disabled: unknown }).disabled).toBe(true)
  })

  it('non-form prop accessors coexist on the form class', () => {
    const tag = nextTag()
    component(
      tag,
      {
        props: {
          value: prop.string({ default: '' }),
          label: prop.string({ default: '' }),
        },
        formValue: 'value',
      },
      noopModule(),
    )
    const el = mount(tag, { value: 'v1', label: 'l1' })
    const e = el as unknown as { value: unknown; label: unknown }
    expect(e.value).toBe('v1')
    expect(e.label).toBe('l1')
  })
})
