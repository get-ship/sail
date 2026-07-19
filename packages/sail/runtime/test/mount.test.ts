// mount() — F1 unified entry point.
//
// Tests pass `opts.module` directly so the dynamic `import(payload.moduleUrl)`
// path doesn't run. JSDOM has no URL.createObjectURL and no actual ESM
// dynamic-import resolution, so the injected-module pattern is the
// idiomatic test seam (mirrors how `attach()` tests inject modules).

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import { mount, mountAmbient, onCleanup, signal } from '../src/index'
import { queryKeyed } from '../src/query'
import type { ViewPayload } from '../src/index'

// JSDOM doesn't implement EventSource — stub minimally for live-channel paths.
class FakeEventSource {
  url: string
  closed = false
  // 1 = OPEN. Mount blocks on `readyState === OPEN` before resolving so the
  // stub starts already open — none of the existing tests care about the
  // OPEN handshake, only the patch-signals delivery.
  readyState = 1
  listeners = new Map<string, Set<(e: MessageEvent) => void>>()
  constructor(url: string) {
    this.url = url
  }
  addEventListener(name: string, fn: (e: MessageEvent) => void) {
    let set = this.listeners.get(name)
    if (!set) {
      set = new Set()
      this.listeners.set(name, set)
    }
    set.add(fn)
  }
  removeEventListener(name: string, fn: (e: MessageEvent) => void) {
    this.listeners.get(name)?.delete(fn)
  }
  close() {
    this.closed = true
  }
  emit(name: string, data: unknown) {
    const set = this.listeners.get(name)
    if (!set) return
    const ev = { data: JSON.stringify(data) } as MessageEvent
    for (const fn of set) fn(ev)
  }
}
;(globalThis as unknown as { EventSource: unknown }).EventSource = FakeEventSource

interface FakeModule {
  default: {
    setup: (ctx: {
      ship?: unknown
      route?: unknown
      navigate?: (path: string) => void
      scope?: Record<string, unknown>
      stateKeys?: ReadonlySet<string>
    }) => Record<string, unknown>
  }
}

function emptyModule(): FakeModule {
  return { default: { setup: (_ctx) => ({}) } }
}

describe('mount()', () => {
  let target: HTMLElement

  beforeEach(() => {
    document.body.innerHTML = ''
    target = document.createElement('div')
    document.body.appendChild(target)
  })

  it('mounts a payload and returns a handle', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<p>hello</p>',
      css: '',
      scope: {},
      moduleUrl: '/_deck/views/demo/view.js',
    }
    const handle = await mount(target, payload, { module: emptyModule() })
    expect(handle.element).toBe(target)
    expect(target.children.length).toBe(1)
    const p = target.querySelector('p')
    expect(p).toBeTruthy()
    expect(p!.tagName).toBe('P')
    expect(p!.textContent).toBe('hello')
    handle.dispose()
  })

  it('builds signals from payload.scope', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: { count: 5, label: 'hi' },
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: emptyModule() })
    const scope = handle.scope as Record<string, () => unknown>
    expect(scope.count!()).toBe(5)
    expect(scope.label!()).toBe('hi')
    handle.dispose()
  })

  it('injects css into a <style> adjacent to the target by default', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '.x { color: red }',
      scope: {},
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: emptyModule() })
    const style = target.parentElement?.querySelector('style[data-sail-css]') as HTMLStyleElement
    expect(style).toBeTruthy()
    expect(style.textContent).toBe('.x { color: red }')
    handle.dispose()
    expect(document.querySelector('style[data-sail-css]')).toBeNull()
  })

  it('cssScope: "none" skips css injection', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '.x { color: red }',
      scope: {},
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: emptyModule(), cssScope: 'none' })
    expect(document.querySelector('style[data-sail-css]')).toBeNull()
    handle.dispose()
  })

  it('runs setup() with the provided ship/route/navigate', async () => {
    let captured: { ship?: unknown; navigate?: string } = {}
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    const navigate = (_p: string) => {}
    const handle = await mount(target, payload, {
      ship: { id: 'sdk' },
      navigate,
      module: {
        default: {
          setup: (ctx) => {
            captured = { ship: ctx.ship, navigate: typeof ctx.navigate }
            return {}
          },
        },
      },
    })
    expect(captured).toEqual({ ship: { id: 'sdk' }, navigate: 'function' })
    handle.dispose()
  })

  it('reload preserves serializable signal values', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div>v1</div>',
      css: '',
      scope: { count: 0 },
      moduleUrl: '/x',
    }
    const mod = emptyModule()
    const handle = await mount(target, payload, { module: mod })
    const scope = handle.scope as Record<string, ((v?: unknown) => unknown) & { set: (v: unknown) => void }>
    scope.count!.set(42)
    expect(scope.count!()).toBe(42)

    const next: ViewPayload = {
      ...payload,
      html: '<div>v2</div>',
      // Server reset count to 0; runtime should restore from current signal.
      scope: { count: 0 },
    }
    // reload's options are inherited from the initial mount; the test
    // module is reused so reload doesn't re-import.
    await handle.reload(next)
    const reloaded = target.querySelector('div')
    expect(reloaded).toBeTruthy()
    expect(reloaded!.textContent).toBe('v2')
    const scope2 = handle.scope as Record<string, () => unknown>
    expect(scope2.count!()).toBe(42)
    handle.dispose()
  })

  it('restoreScope overlays values during initial mount', async () => {
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: { count: 0 },
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, {
      module: emptyModule(),
      restoreScope: { count: 99 },
    })
    const scope = handle.scope as Record<string, () => unknown>
    expect(scope.count!()).toBe(99)
    handle.dispose()
  })

  it('dispose runs hydrate-disposer + ambient onCleanup + closes EventSource', async () => {
    const events: string[] = []
    const mod: FakeModule = {
      default: {
        setup: (_ctx) => {
          const ret: Record<string, unknown> = {}
          ret.hydrate = () => () => events.push('hydrate-dispose')
          mountAmbient(() => {
            events.push('mount')
            onCleanup(() => events.push('cleanup'))
          })
          return ret
        },
      },
    }
    const payload: ViewPayload = {
      name: 'demo',
      html: '<div></div>',
      css: '',
      scope: {
        live1: { kind: 'loading' as const },
        __subscribe__: '/_deck/views/demo/sse',
      } as unknown as Record<string, unknown>,
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: mod })
    expect(events).toContain('mount')
    handle.dispose()
    expect(events).toContain('hydrate-dispose')
    expect(events).toContain('cleanup')
    handle.dispose() // idempotent
  })

  it('morphs onto existing SSR markup (preserves node identity)', async () => {
    // Pre-populate target as if SSR had injected content into #sail-root.
    target.innerHTML = '<div id="hero">old</div><p>tail</p>'
    const ssrHero = target.querySelector('#hero') as HTMLElement
    const ssrTail = target.querySelector('p') as HTMLElement

    const payload: ViewPayload = {
      name: 'demo',
      html: '<div id="hero">new</div><p>tail</p>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    const handle = await mount(target, payload, { module: emptyModule() })

    // The original nodes are reused (morphed in place), not replaced.
    expect(target.querySelector('#hero')).toBe(ssrHero)
    expect(target.querySelector('p')).toBe(ssrTail)
    expect(ssrHero.textContent).toBe('new')
    handle.dispose()
  })

  it('clean-mount on empty target avoids the innerHTML setter on target', async () => {
    // Spy on Element.prototype.innerHTML — clean path must not invoke it
    // on `target` (it may still be used on a throwaway <template>).
    const desc = Object.getOwnPropertyDescriptor(Element.prototype, 'innerHTML')!
    let setOnTarget = 0
    Object.defineProperty(Element.prototype, 'innerHTML', {
      configurable: true,
      get: desc.get,
      set(value: string) {
        if (this === target) setOnTarget++
        desc.set!.call(this, value)
      },
    })
    try {
      const payload: ViewPayload = {
        name: 'demo',
        html: '<p>fresh</p>',
        css: '',
        scope: {},
        moduleUrl: '/x',
      }
      const handle = await mount(target, payload, { module: emptyModule() })
      expect(setOnTarget).toBe(0)
      expect(target.querySelector('p')?.textContent).toBe('fresh')
      handle.dispose()
    } finally {
      Object.defineProperty(Element.prototype, 'innerHTML', desc)
    }
  })

  it('mount via name string resolves through the injected loader', async () => {
    // The runtime is host-agnostic: mounting by name delegates payload
    // resolution to `opts.loader.fetchView`. The `/_deck/...` URL scheme lives
    // in the host's loader (shell / deck view-host), not in mount().
    const payload: ViewPayload = {
      name: 'demo-fetch',
      html: '<div>fetched</div>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    const fetchView = vi.fn(async (nameOrPath: string) => {
      expect(nameOrPath).toBe('demo-fetch')
      return payload
    })
    const handle = await mount(target, 'demo-fetch', {
      module: emptyModule(),
      loader: { fetchView, loadComponent: async () => {} },
    })
    expect(fetchView).toHaveBeenCalledWith('demo-fetch')
    expect(target.querySelector('div')?.textContent).toBe('fetched')
    handle.dispose()
  })

  it('mount by name without a loader throws a clear error', async () => {
    await expect(mount(target, 'demo-fetch', { module: emptyModule() })).rejects.toThrow(
      /requires opts\.loader/,
    )
  })

  it('throws if target is missing', async () => {
    await expect(
      mount(null as unknown as HTMLElement, {} as ViewPayload, { module: emptyModule() }),
    ).rejects.toThrow(/target is required/)
  })

  it('throws if module has no default.setup', async () => {
    const payload: ViewPayload = {
      name: 'broken',
      html: '<div></div>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    await expect(
      mount(target, payload, {
        module: { default: {} } as unknown as Parameters<typeof mount>[2] extends infer O
          ? O extends { module?: infer M } ? M : never
          : never,
      }),
    ).rejects.toThrow(/missing default\.setup/)
  })
})

describe('signal sanity in test harness', () => {
  it('signal export is callable', () => {
    const s = signal(1)
    expect(s()).toBe(1)
  })
})

// ── openLiveChannel end-to-end coverage ───────────────────────────────────
//
// Mount creates a FakeEventSource (via the global stub above), registers
// patch-signals handlers on it, and patches the live signal when a frame
// arrives. We intercept the constructed instance, mount a view that has a
// live scope key + __subscribe__, then emit SSE frames and verify the
// LiveSignal walks loading → ok → ok (second value).

describe('openLiveChannel (via mount)', () => {
  let target: HTMLElement
  // Capture the FakeEventSource instance that openLiveChannel constructs.
  let lastFakeES: FakeEventSource | null = null
  const OrigFakeEventSource = FakeEventSource
  class CapturingFakeEventSource extends OrigFakeEventSource {
    constructor(url: string) {
      super(url)
      lastFakeES = this
    }
  }

  beforeEach(() => {
    document.body.innerHTML = ''
    target = document.createElement('div')
    document.body.appendChild(target)
    lastFakeES = null
    ;(globalThis as unknown as { EventSource: unknown }).EventSource = CapturingFakeEventSource
  })

  afterEach(() => {
    ;(globalThis as unknown as { EventSource: unknown }).EventSource = FakeEventSource
  })

  it('patch-signals frame transitions LiveSignal loading → ok, second frame advances value', async () => {
    // Payload: one live key + __subscribe__ triggers openLiveChannel.
    const payload: ViewPayload = {
      name: 'live-test',
      html: '<div></div>',
      css: '',
      scope: {
        items: { kind: 'loading' as const },
        __subscribe__: '/_deck/views/live-test/sse',
      } as unknown as Record<string, unknown>,
      moduleUrl: '/x',
    }

    const handle = await mount(target, payload, { module: emptyModule() })

    // The FakeEventSource was constructed by openLiveChannel.
    expect(lastFakeES).not.toBeNull()
    const es = lastFakeES!

    // Read the live signal from the built scope.
    const scope = handle.scope as Record<string, () => unknown>
    const itemsSig = scope['items']!

    // Before any SSE frame the signal is still loading (server-seeded value).
    expect((itemsSig() as { kind: string }).kind).toBe('loading')

    // Emit the first patch-signals frame: loading → ok.
    es.emit('patch-signals', { name: 'items', value: { kind: 'ok', value: ['a', 'b'] } })
    const afterFirst = itemsSig() as { kind: string; value: unknown }
    expect(afterFirst.kind).toBe('ok')
    expect(afterFirst.value).toEqual(['a', 'b'])

    // Emit a second frame: ok → ok with a new value.
    es.emit('patch-signals', { name: 'items', value: { kind: 'ok', value: ['a', 'b', 'c'] } })
    const afterSecond = itemsSig() as { kind: string; value: unknown }
    expect(afterSecond.kind).toBe('ok')
    expect(afterSecond.value).toEqual(['a', 'b', 'c'])

    handle.dispose()
    // After dispose the EventSource is closed.
    expect(es.closed).toBe(true)
  })

  it('ignores patch-signals frames for keys not in liveKeys', async () => {
    const payload: ViewPayload = {
      name: 'live-test-filter',
      html: '<div></div>',
      css: '',
      scope: {
        items: { kind: 'loading' as const },
        __subscribe__: '/_deck/views/live-test-filter/sse',
      } as unknown as Record<string, unknown>,
      moduleUrl: '/x',
    }

    const handle = await mount(target, payload, { module: emptyModule() })
    const es = lastFakeES!
    const scope = handle.scope as Record<string, () => unknown>
    const itemsSig = scope['items']!

    // Frame for an unknown key — signal must stay loading.
    es.emit('patch-signals', { name: 'unknown-key', value: { kind: 'ok', value: 99 } })
    expect((itemsSig() as { kind: string }).kind).toBe('loading')

    handle.dispose()
  })
})

// ── keyed clients injection (the mount half of the data/ops port) ──
//
// `opts.clients` is THE injection surface at mount: each key resolves from
// `ctx.clients.X` and is spread into scope as bare `X`. The NAMES come from
// the app manifest (there is no `<client>` declaration — see
// `src/clients.ts`); the host supplies the VALUES here. `ship` is just a
// key — aliased both ways with the legacy `opts.ship` so hosts and views
// on either side of the migration interoperate.
describe('mount() keyed clients', () => {
  let target: HTMLElement

  beforeEach(() => {
    document.body.innerHTML = ''
    target = document.createElement('div')
    document.body.appendChild(target)
  })

  const payload = (): ViewPayload => ({
    name: 'clients-demo',
    html: '<div></div>',
    css: '',
    scope: {},
    moduleUrl: '/x',
  })

  type Ctx = {
    ship?: unknown
    clients?: Record<string, unknown>
    scope?: Record<string, unknown>
  }

  it('a non-ship client key resolves a live data source end-to-end', async () => {
    // Fake `api` client — what a non-Ship host injects. The module body
    // mirrors compiled output for a controller declaring a live
    // `queryKeyed` entry over the injected `api` client.
    const api = {
      demo: { clock: async () => ({ now: 42 }) },
    }
    let t: (() => { kind: string; value?: { now: number } }) | undefined
    const mod = {
      default: {
        clients: ['api'],
        setup: (ctx: unknown) => {
          const { api } = (ctx as Ctx).clients as {
            api: { demo: { clock: () => Promise<{ now: number }> } }
          }
          const $scope: Record<string, unknown> = {}
          $scope['t'] = queryKeyed(() => ['api', 'demo', 'clock'], () => api.demo.clock())
          t = $scope['t'] as typeof t
          return Object.assign({}, $scope)
        },
      },
    }
    const handle = await mount(target, payload(), {
      clients: { api },
      module: mod as never,
    })
    // query() resolves the thunk's promise on a microtask.
    await new Promise((r) => setTimeout(r, 0))
    expect(t).toBeTypeOf('function')
    expect(t!()).toEqual({ kind: 'ok', value: { now: 42 } })
    handle.dispose()
  })

  it('clients are spread into setup scope so bare keys resolve', async () => {
    const api = { ping: () => 'pong' }
    let captured: Ctx = {}
    const handle = await mount(target, payload(), {
      clients: { api },
      module: {
        default: {
          setup: (ctx: unknown) => {
            captured = ctx as Ctx
            return {}
          },
        },
      } as never,
    })
    expect(captured.clients).toEqual({ api })
    expect((captured.scope as Record<string, unknown>)['api']).toBe(api)
    handle.dispose()
  })

  it('opts.ship is aliased into clients.ship (legacy host, new view)', async () => {
    const ship = { id: 'sdk' }
    let captured: Ctx = {}
    const handle = await mount(target, payload(), {
      ship,
      module: {
        default: {
          setup: (ctx: unknown) => {
            captured = ctx as Ctx
            return {}
          },
        },
      } as never,
    })
    expect(captured.clients).toEqual({ ship })
    expect(captured.ship).toBe(ship)
    handle.dispose()
  })

  it('clients.ship feeds the legacy ctx.ship alias (new host, old view)', async () => {
    const ship = { id: 'sdk' }
    let captured: Ctx = {}
    const handle = await mount(target, payload(), {
      clients: { ship },
      module: {
        default: {
          setup: (ctx: unknown) => {
            captured = ctx as Ctx
            return {}
          },
        },
      } as never,
    })
    expect(captured.ship).toBe(ship)
    expect(captured.clients).toEqual({ ship })
    handle.dispose()
  })

  it('an explicit clients.ship wins over opts.ship in the map', async () => {
    const legacy = { id: 'legacy' }
    const keyed = { id: 'keyed' }
    let captured: Ctx = {}
    const handle = await mount(target, payload(), {
      ship: legacy,
      clients: { ship: keyed },
      module: {
        default: {
          setup: (ctx: unknown) => {
            captured = ctx as Ctx
            return {}
          },
        },
      } as never,
    })
    expect((captured.clients as Record<string, unknown>)['ship']).toBe(keyed)
    handle.dispose()
  })

})
