import { describe, expect, it, vi } from 'vitest'
import { component, prop } from '../src/component-factory'
import { onCleanup } from '../src/effect-scope'
import type { Signal } from '../src/signal'
import { setHostRoute } from '../src/host-route'
import { signal } from '../src/signal'

let tagCounter = 0
const nextTag = () => `sail-cf-${++tagCounter}`
const flushMicrotasks = () => new Promise((r) => setTimeout(r, 0))

// A no-op module — for registration tests that don't inspect the mount ctx.
const noopModule = () => component(() => ({}))
// A module whose setup captures the mount ctx (the prop signals + internals the
// custom-element wrapper passes to `mountComponent`). This is the unified
// contract: a registered component is `component(tag, def, module)`; the module
// receives `ctx.props` (signals) and, when form-associated, `ctx.internals`.
const capturing = (capture: (ctx: { props: Record<string, Signal<unknown>>; internals?: ElementInternals }) => void) =>
  component((ctx) => {
    capture(ctx as { props: Record<string, Signal<unknown>>; internals?: ElementInternals })
    return {}
  })

describe('component() — basic registration', () => {
  it('registers a custom element', () => {
    const tag = nextTag()
    component(tag, { props: {} }, noopModule())
    expect(customElements.get(tag)).toBeDefined()
  })

  it('is idempotent', () => {
    const tag = nextTag()
    component(tag, { props: {} }, noopModule())
    component(tag, { props: {} }, noopModule())
    expect(customElements.get(tag)).toBeDefined()
  })
})

describe('component() — bool prop with reflect', () => {
  it('reflects signal writes back to the host attribute', async () => {
    const tag = nextTag()
    let captured: Signal<boolean> | undefined
    component(
      tag,
      { props: { open: prop.bool({ default: false, reflect: true }) } },
      capturing((ctx) => {
        captured = ctx.props.open as Signal<boolean>
      }),
    )
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
    component(
      tag,
      { props: { open: prop.bool({ default: false }) } },
      capturing((ctx) => {
        captured = ctx.props.open as Signal<boolean>
      }),
    )
    const el = document.createElement(tag)
    el.setAttribute('open', '')
    document.body.appendChild(el)
    expect(captured!()).toBe(true)
    el.remove()
  })
})

describe('component() — host route inject for layout CEs', () => {
  it('passes registered ctx.route into the module setup', () => {
    const tag = nextTag()
    const route = {
      params: {},
      query: {},
      pathname: signal('/components/button'),
      hash: signal(''),
    }
    const navigate = vi.fn()
    setHostRoute({ route, navigate })
    const setup = vi.fn(() => ({}))
    component(tag, { props: {} }, component(setup))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const ctx = setup.mock.calls[0]![0] as { route?: typeof route; navigate?: typeof navigate }
    expect(ctx.route).toBe(route)
    expect(ctx.navigate).toBe(navigate)
    expect(ctx.route?.pathname()).toBe('/components/button')
    el.remove()
    setHostRoute(null)
  })
})

describe('component() — module mount', () => {
  it('mounts the module with the prop signals on ctx.props', () => {
    const tag = nextTag()
    const setup = vi.fn(() => ({}))
    component(tag, { props: { name: prop.string({ default: 'x' }) } }, component(setup))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(setup).toHaveBeenCalledOnce()
    const ctx = setup.mock.calls[0]![0] as { props: Record<string, Signal<unknown>> }
    expect(typeof ctx.props.name).toBe('function')
    el.remove()
  })

  it('disposes the mounted child on disconnect (onCleanup fires)', () => {
    const tag = nextTag()
    const cleanup = vi.fn()
    component(
      tag,
      { props: {} },
      component(() => {
        // onCleanup enrolls on the child scope mountComponent owns; the
        // custom-element disconnect tears that scope down.
        onCleanup(cleanup)
        return {}
      }),
    )
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
    component(
      tag,
      { props: { size: prop.union(['sm', 'md', 'lg'] as const, { default: 'md' }) } },
      capturing((ctx) => {
        captured = ctx.props.size as Signal<'sm' | 'md' | 'lg'>
      }),
    )
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
  it('passes ElementInternals to the module ctx when formAssociated is true', () => {
    const tag = nextTag()
    let receivedInternals: ElementInternals | undefined
    component(
      tag,
      { formAssociated: true, props: {} },
      capturing((ctx) => {
        receivedInternals = ctx.internals
      }),
    )
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(receivedInternals).toBeDefined()
    el.remove()
  })

  it('omits internals when formAssociated is false', () => {
    const tag = nextTag()
    let receivedInternals: ElementInternals | undefined
    component(
      tag,
      { props: {} },
      capturing((ctx) => {
        receivedInternals = ctx.internals
      }),
    )
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
    component(
      tag,
      { props: { count: prop.number({ default: 0 }) } },
      capturing((ctx) => {
        captured = ctx.props.count as Signal<number>
      }),
    )
    const el = document.createElement(tag)
    el.setAttribute('count', '7')
    document.body.appendChild(el)
    expect(captured!()).toBe(7)
    el.remove()
  })

  it('property prop: set via el.prop (identity), never observed as an attribute', () => {
    const tag = nextTag()
    let captured: Signal<{ a: number }> | undefined
    component(
      tag,
      { props: { data: prop.property<{ a: number }>({ default: { a: 0 } }) } },
      capturing((ctx) => {
        captured = ctx.props.data as Signal<{ a: number }>
      }),
    )
    const el = document.createElement(tag)
    document.body.appendChild(el)
    // Seeds from the declared default.
    expect(captured!()).toEqual({ a: 0 })
    // Property assignment bridges to the signal — identity preserved, no JSON.
    const next = { a: 3 }
    ;(el as unknown as { data: { a: number } }).data = next
    expect(captured!()).toBe(next)
    // Property-only: not an observed attribute, so setAttribute is inert.
    el.setAttribute('data', '{"a":9}')
    expect(captured!()).toBe(next)
    el.remove()
  })
})

describe('component() — styles', () => {
  it('injects scoped style for light DOM', () => {
    const tag = nextTag()
    component(tag, { props: {}, styles: 'color: red;' }, noopModule())
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const style = document.getElementById(`sail-style-${tag}`)
    expect(style).toBeTruthy()
    expect(style!.textContent).toContain(`[data-component="${tag}"]`)
    expect(style!.textContent).toContain('color: red;')
    el.remove()
  })

  it('attaches style to shadow root for shadow DOM', () => {
    const tag = nextTag()
    component(tag, { props: {}, shadow: true, styles: 'color: blue;' }, noopModule())
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.shadowRoot).toBeTruthy()
    const style = el.shadowRoot!.querySelector('style')
    expect(style).toBeTruthy()
    expect(style!.textContent).toBe('color: blue;')
    el.remove()
  })
})

describe('component() — light-DOM capture across reconnect', () => {
  // The /components filter crash: {#if} detaches and re-attaches the SAME
  // host node. The connect handler re-runs; re-capturing host.childNodes at
  // that point grabs the previous render's own wrapper as "authored
  // content", and projection then places the wrapper inside its own <slot>
  // — HierarchyRequestError, page replaced by the route error card.
  it('re-attaching the same host reuses the original capture ({#if} round-trip)', async () => {
    const tag = nextTag()
    // The compiled __mount ADOPTS existing markup on a non-empty host (the
    // hydration/morph path) instead of rebuilding — so on reconnect the
    // previously-rendered wrapper stays live, and a stale capture of it is
    // an ancestor of the very slot projection targets.
    const slotModule = () =>
      component(() => ({
        __mount: (host: HTMLElement) => {
          if (host.querySelector(':scope > div.rendered-root')) return
          const wrapper = document.createElement('div')
          wrapper.className = 'rendered-root'
          wrapper.appendChild(document.createElement('slot'))
          host.replaceChildren(wrapper)
        },
      }))
    component(tag, { props: {} }, slotModule())

    const errSpy = vi.spyOn(console, 'error')
    const el = document.createElement(tag)
    const authored = document.createElement('span')
    authored.textContent = 'authored'
    el.appendChild(authored)
    document.body.appendChild(el)
    await flushMicrotasks()
    expect(el.querySelector('slot')?.contains(authored)).toBe(true)

    el.remove()
    document.body.appendChild(el)
    await flushMicrotasks()

    const hierarchyErrs = errSpy.mock.calls.filter((c) =>
      c.map(String).join(' ').includes('HierarchyRequestError'),
    )
    expect(hierarchyErrs).toEqual([])
    expect(el.querySelector('slot')?.contains(authored)).toBe(true)
    errSpy.mockRestore()
    el.remove()
  })
})

describe('component() — re-setup after disconnect', () => {
  // The <if> swap re-inserts branch nodes; a component's reconnect re-runs
  // runSetup, and without clearing, the factory stamps a second template
  // copy (the studio element-tree double-mount).
  it('clears the previous stamp on reconnect (no double-mount)', async () => {
    const tag = nextTag()
    component(tag, { props: {} }, component(() => ({
      __mount(container: HTMLElement | ShadowRoot) {
        const d = document.createElement('div')
        d.className = 'stamp'
        container.appendChild(d)
        return () => {}
      },
    })))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    await flushMicrotasks()
    expect(el.querySelectorAll('.stamp').length).toBe(1)

    el.remove()
    document.body.appendChild(el)
    await flushMicrotasks()
    expect(el.querySelectorAll('.stamp').length).toBe(1)
    el.remove()
  })
})
