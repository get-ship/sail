// Constructor wrappers + counter helpers for the resource-symmetry
// integration test.
//
// Wraps the runtime constructors that real routes allocate during
// mount: EventSource, MutationObserver, ResizeObserver,
// IntersectionObserver, and setInterval/clearInterval. Each
// allocation increments a `created` counter; each release/close
// increments a `closed` counter. The integration test asserts the
// pair is symmetric across `mount() → dispose()`.
//
// We deliberately do NOT wrap addEventListener: too noisy, and the
// observer/EventSource wrappers above already catch every leak class
// the symmetry test cares about (the bugs in 87cf3223 and 0efea6d7).

export interface CounterPair {
  created: number
  closed: number
}

export interface ResourceCounters {
  EventSource: CounterPair
  MutationObserver: CounterPair
  ResizeObserver: CounterPair
  IntersectionObserver: CounterPair
  setInterval: CounterPair
}

export interface InstalledWrappers {
  counters: ResourceCounters
  /** Restore the originals. Call after the test (or in afterAll). */
  restore: () => void
}

export function snapshot(c: ResourceCounters): ResourceCounters {
  return {
    EventSource: { ...c.EventSource },
    MutationObserver: { ...c.MutationObserver },
    ResizeObserver: { ...c.ResizeObserver },
    IntersectionObserver: { ...c.IntersectionObserver },
    setInterval: { ...c.setInterval },
  }
}

/** Open allocations between two snapshots, by kind. Empty when symmetric. */
export function openSince(
  before: ResourceCounters,
  after: ResourceCounters,
): Record<string, number> {
  const out: Record<string, number> = {}
  for (const k of Object.keys(after) as Array<keyof ResourceCounters>) {
    const created = after[k].created - before[k].created
    const closed = after[k].closed - before[k].closed
    if (created !== closed) out[k] = created - closed
  }
  return out
}

/**
 * Install wrapped constructors on `globalThis`. Idempotent in the
 * sense that calling restore() puts the originals back.
 */
export function installWrappers(): InstalledWrappers {
  const counters: ResourceCounters = {
    EventSource: { created: 0, closed: 0 },
    MutationObserver: { created: 0, closed: 0 },
    ResizeObserver: { created: 0, closed: 0 },
    IntersectionObserver: { created: 0, closed: 0 },
    setInterval: { created: 0, closed: 0 },
  }

  type AnyCtor = new (...args: unknown[]) => unknown
  const g = globalThis as unknown as Record<string, unknown>

  const orig: Record<string, unknown> = {}

  // ── EventSource ──
  // jsdom doesn't ship EventSource at all. The test harness installs
  // a minimal stub when the original is absent; some other tests in
  // this monorepo use a similar pattern (see ui-runtime/test/mount.test.ts).
  const origEventSource = g.EventSource as AnyCtor | undefined
  orig.EventSource = origEventSource
  class WrappedEventSource {
    static readonly CONNECTING = 0
    static readonly OPEN = 1
    static readonly CLOSED = 2
    readyState = 1
    url: string
    listeners = new Map<string, Set<(e: MessageEvent) => void>>()
    onmessage: ((e: MessageEvent) => void) | null = null
    onerror: ((e: Event) => void) | null = null
    onopen: ((e: Event) => void) | null = null
    constructor(url: string) {
      this.url = url
      counters.EventSource.created++
    }
    addEventListener(name: string, fn: (e: MessageEvent) => void): void {
      let set = this.listeners.get(name)
      if (!set) {
        set = new Set()
        this.listeners.set(name, set)
      }
      set.add(fn)
    }
    removeEventListener(name: string, fn: (e: MessageEvent) => void): void {
      this.listeners.get(name)?.delete(fn)
    }
    close(): void {
      if (this.readyState === 2) return
      this.readyState = 2
      counters.EventSource.closed++
    }
    /** Test helper — synchronously dispatch an event to listeners. */
    emit(name: string, data: unknown): void {
      const set = this.listeners.get(name)
      if (!set) return
      const ev = { data: typeof data === 'string' ? data : JSON.stringify(data) } as MessageEvent
      for (const fn of set) fn(ev)
    }
  }
  g.EventSource = WrappedEventSource

  // ── Observers ──
  // jsdom ships no-op stubs for these; we wrap whatever is present.
  function wrapObserver(name: 'MutationObserver' | 'ResizeObserver' | 'IntersectionObserver') {
    const Original = g[name] as AnyCtor | undefined
    orig[name] = Original
    const slot = counters[name]
    class Wrapped {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      _inner: any
      constructor(...args: unknown[]) {
        slot.created++
        if (Original) {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          this._inner = new (Original as any)(...args)
        }
      }
      observe(...args: unknown[]): void {
        this._inner?.observe?.(...args)
      }
      disconnect(): void {
        slot.closed++
        this._inner?.disconnect?.()
      }
      takeRecords(): unknown[] {
        return this._inner?.takeRecords?.() ?? []
      }
      unobserve(...args: unknown[]): void {
        this._inner?.unobserve?.(...args)
      }
    }
    g[name] = Wrapped as unknown as AnyCtor
  }
  wrapObserver('MutationObserver')
  wrapObserver('ResizeObserver')
  wrapObserver('IntersectionObserver')

  // ── setInterval / clearInterval ──
  const origSetInterval = g.setInterval as ((...a: unknown[]) => unknown) | undefined
  const origClearInterval = g.clearInterval as ((id: unknown) => void) | undefined
  orig.setInterval = origSetInterval
  orig.clearInterval = origClearInterval
  const liveIntervals = new Set<unknown>()
  g.setInterval = (...args: unknown[]) => {
    counters.setInterval.created++
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const id = (origSetInterval as any)(...args)
    liveIntervals.add(id)
    return id
  }
  g.clearInterval = (id: unknown) => {
    if (liveIntervals.has(id)) {
      counters.setInterval.closed++
      liveIntervals.delete(id)
    }
    origClearInterval?.(id)
  }

  return {
    counters,
    restore: () => {
      g.EventSource = orig.EventSource
      g.MutationObserver = orig.MutationObserver
      g.ResizeObserver = orig.ResizeObserver
      g.IntersectionObserver = orig.IntersectionObserver
      g.setInterval = orig.setInterval
      g.clearInterval = orig.clearInterval
    },
  }
}
