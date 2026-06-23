import { describe, expect, it, vi } from 'vitest'
import { component, prop } from '../src/component-factory'
import type { Signal } from '../src/signal'

let tagCounter = 0
const nextTag = () => `ship-set-cf-${++tagCounter}`
const flushMicrotasks = () => new Promise((r) => setTimeout(r, 0))

describe('component() — basic registration', () => {
  it('registers a custom element', () => {
    const tag = nextTag()
    component(tag, { props: {} })
    expect(customElements.get(tag)).toBeDefined()
  })

  it('is idempotent', () => {
    const tag = nextTag()
    component(tag, { props: {} })
    component(tag, { props: {} })
    expect(customElements.get(tag)).toBeDefined()
  })
})

describe('component() — bool prop with reflect', () => {
  it('reflects signal writes back to the host attribute', async () => {
    const tag = nextTag()
    let captured: Signal<boolean> | undefined
    component(tag, {
      props: { open: prop.bool({ default: false, reflect: true }) },
      setup: ({ props }) => {
        captured = props.open as Signal<boolean>
      },
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.hasAttribute('open')).toBe(false)
    captured!.set(true)
    await flushMicrotasks()
    expect(el.hasAttribute('open')).toBe(true)
    captured!.set(false)
    await flushMicrotasks()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('seeds signal from initial host attribute', () => {
    const tag = nextTag()
    let captured: Signal<boolean> | undefined
    component(tag, {
      props: { open: prop.bool({ default: false }) },
      setup: ({ props }) => {
        captured = props.open as Signal<boolean>
      },
    })
    const el = document.createElement(tag)
    el.setAttribute('open', '')
    document.body.appendChild(el)
    expect(captured!()).toBe(true)
    el.remove()
  })
})

describe('component() — setup invocation', () => {
  it('setup gets called with prop signals', () => {
    const tag = nextTag()
    const setup = vi.fn(() => undefined)
    component(tag, {
      props: { name: prop.string({ default: 'x' }) },
      setup,
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(setup).toHaveBeenCalledOnce()
    const ctx = setup.mock.calls[0]![0]
    expect(typeof ctx.props.name).toBe('function')
    expect(ctx.host).toBe(el)
    expect(Array.isArray(ctx.cleanups)).toBe(true)
    el.remove()
  })

  it('cleanups array is drained on disconnect', () => {
    const tag = nextTag()
    const cleanup = vi.fn()
    component(tag, {
      props: {},
      setup: ({ cleanups }) => {
        cleanups.push(cleanup)
      },
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(cleanup).not.toHaveBeenCalled()
    el.remove()
    expect(cleanup).toHaveBeenCalledOnce()
  })
})

describe('component() — union props', () => {
  it('decodes valid values and rejects invalid ones', () => {
    const tag = nextTag()
    let captured: Signal<'sm' | 'md' | 'lg'> | undefined
    component(tag, {
      props: {
        size: prop.union(['sm', 'md', 'lg'] as const, { default: 'md' }),
      },
      setup: ({ props }) => {
        captured = props.size as Signal<'sm' | 'md' | 'lg'>
      },
    })
    const el = document.createElement(tag)
    el.setAttribute('size', 'lg')
    document.body.appendChild(el)
    expect(captured!()).toBe('lg')
    el.setAttribute('size', 'huge')
    expect(captured!()).toBe('md')
    el.remove()
  })
})

describe('component() — formAssociated', () => {
  it('passes ElementInternals to setup when formAssociated is true', () => {
    const tag = nextTag()
    let receivedInternals: ElementInternals | undefined
    component(tag, {
      formAssociated: true,
      props: {},
      setup: ({ internals }) => {
        receivedInternals = internals
      },
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(receivedInternals).toBeDefined()
    // jsdom's ElementInternals is partial — setFormValue isn't always
    // implemented. Verifying the object reaches setup is enough.
    el.remove()
  })

  it('omits internals when formAssociated is false', () => {
    const tag = nextTag()
    let receivedInternals: ElementInternals | undefined = undefined as
      | ElementInternals
      | undefined
    component(tag, {
      props: {},
      setup: ({ internals }) => {
        receivedInternals = internals
      },
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(receivedInternals).toBeUndefined()
    el.remove()
  })
})

describe('component() — number and object props', () => {
  it('decodes number prop', () => {
    const tag = nextTag()
    let captured: Signal<number> | undefined
    component(tag, {
      props: { count: prop.number({ default: 0 }) },
      setup: ({ props }) => { captured = props.count as Signal<number> },
    })
    const el = document.createElement(tag)
    el.setAttribute('count', '7')
    document.body.appendChild(el)
    expect(captured!()).toBe(7)
    el.remove()
  })

  it('decodes object prop with fallback on bad json', () => {
    const tag = nextTag()
    let captured: Signal<{ a: number }> | undefined
    component(tag, {
      props: { data: prop.object<{ a: number }>({ default: { a: 0 } }) },
      setup: ({ props }) => { captured = props.data as Signal<{ a: number }> },
    })
    const el = document.createElement(tag)
    el.setAttribute('data', 'not-json')
    document.body.appendChild(el)
    expect(captured!()).toEqual({ a: 0 })
    el.setAttribute('data', '{"a":3}')
    expect(captured!()).toEqual({ a: 3 })
    el.remove()
  })
})

describe('component() — styles', () => {
  it('injects scoped style for light DOM', () => {
    const tag = nextTag()
    component(tag, {
      props: {},
      styles: 'color: red;',
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const style = document.getElementById(`ship-set-style-${tag}`)
    expect(style).toBeTruthy()
    expect(style!.textContent).toContain(`[data-component="${tag}"]`)
    expect(style!.textContent).toContain('color: red;')
    el.remove()
  })

  it('attaches style to shadow root for shadow DOM', () => {
    const tag = nextTag()
    component(tag, {
      props: {},
      shadow: true,
      styles: 'color: blue;',
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.shadowRoot).toBeTruthy()
    const style = el.shadowRoot!.querySelector('style')
    expect(style).toBeTruthy()
    expect(style!.textContent).toBe('color: blue;')
    el.remove()
  })
})
