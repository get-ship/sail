// Host route port on CE ctx: registered custom elements must see the same
// live RouteSignals pages get via mount({ route }). Isolated unit tests
// leave the port unset so they do not invent a router.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { component } from '../src/component-factory'
import { mount, type RoutePayload } from '../src/mount'
import { navigate } from '../src/navigate'
import type { RouteSignals } from '../src/route-signals'

let tagCounter = 0
const nextTag = () => `sail-cf-route-${++tagCounter}`

function stubRoute(pathname = '/components/button'): RouteSignals {
  return {
    pathname: () => pathname,
    params: {},
    query: {},
    hash: () => '',
  } as RouteSignals
}

describe('CE ctx — host RouteSignals port', () => {
  afterEach(() => {
    globalThis.__shipRouteSignals = undefined
  })

  it('leaves ctx.route undefined when the host port is unset', () => {
    const tag = nextTag()
    const setup = vi.fn(() => ({}))
    component(tag, { props: {} }, component(setup))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(setup).toHaveBeenCalledOnce()
    const ctx = setup.mock.calls[0]![0] as { route?: unknown; navigate?: unknown }
    expect(ctx.route).toBeUndefined()
    expect(typeof ctx.navigate).toBe('function')
    el.remove()
  })

  it('injects the same host port object onto ctx.route', () => {
    const port = stubRoute('/components/button')
    globalThis.__shipRouteSignals = port
    const tag = nextTag()
    const setup = vi.fn(() => ({}))
    component(tag, { props: {} }, component(setup))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(setup).toHaveBeenCalledOnce()
    const ctx = setup.mock.calls[0]![0] as { route?: RouteSignals; navigate?: unknown }
    expect(ctx.route).toBe(port)
    expect(ctx.route?.pathname()).toBe('/components/button')
    expect(ctx.navigate).toBe(navigate)
    el.remove()
  })

  it('injects route/navigate on the form-associated CE path too', () => {
    const port = stubRoute('/docs/guide')
    globalThis.__shipRouteSignals = port
    const tag = nextTag()
    const setup = vi.fn(() => ({}))
    component(tag, { formAssociated: true, props: {} }, component(setup))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(setup).toHaveBeenCalledOnce()
    const ctx = setup.mock.calls[0]![0] as { route?: RouteSignals; navigate?: unknown }
    expect(ctx.route).toBe(port)
    expect(typeof ctx.navigate).toBe('function')
    el.remove()
  })

  it('mount() without opts.route uses the host port', async () => {
    const port = stubRoute('/components/button')
    globalThis.__shipRouteSignals = port
    const target = document.createElement('div')
    document.body.appendChild(target)
    const payload: RoutePayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    let captured: { route?: unknown; navigate?: unknown } = {}
    const handle = await mount(target, payload, {
      cssScope: 'none',
      module: {
        default: {
          setup: (ctx) => {
            captured = {
              route: (ctx as { route?: unknown }).route,
              navigate: (ctx as { navigate?: unknown }).navigate,
            }
            return {}
          },
        },
      },
    })
    expect(captured.route).toBe(port)
    expect(captured.navigate).toBe(navigate)
    handle.dispose()
    target.remove()
  })

  it('mount() opts.route wins over the host port', async () => {
    const port = stubRoute('/from-port')
    const explicit = stubRoute('/from-opts')
    globalThis.__shipRouteSignals = port
    const target = document.createElement('div')
    document.body.appendChild(target)
    const payload: RoutePayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    let captured: unknown
    const handle = await mount(target, payload, {
      route: explicit,
      cssScope: 'none',
      module: {
        default: {
          setup: (ctx) => {
            captured = (ctx as { route?: unknown }).route
            return {}
          },
        },
      },
    })
    expect(captured).toBe(explicit)
    handle.dispose()
    target.remove()
  })
})
