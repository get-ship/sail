import { describe, expect, it, vi } from 'vitest'
import { data } from '../src/data'

class FakeEventSource {
  static instances: FakeEventSource[] = []
  url: string
  closed = false
  listeners = new Map<string, Set<(ev: any) => void>>()
  constructor(url: string) {
    this.url = url
    FakeEventSource.instances.push(this)
  }
  addEventListener(type: string, fn: (ev: any) => void) {
    let s = this.listeners.get(type)
    if (!s) {
      s = new Set()
      this.listeners.set(type, s)
    }
    s.add(fn)
  }
  removeEventListener(type: string, fn: (ev: any) => void) {
    this.listeners.get(type)?.delete(fn)
  }
  close() {
    this.closed = true
  }
  emit(type: string, ev: any) {
    this.listeners.get(type)?.forEach((fn) => fn(ev))
  }
}

const ES = FakeEventSource as unknown as typeof globalThis.EventSource

describe('data', () => {
  it('seeds the signal with initial value', () => {
    const { signal: sig } = data({ initial: { count: 0 } })
    expect(sig()).toEqual({ count: 0 })
  })

  it('skips subscription when no URL', () => {
    FakeEventSource.instances = []
    data({ initial: 0, EventSource: ES })
    expect(FakeEventSource.instances).toHaveLength(0)
  })

  it('opens SSE when subscribe URL given', () => {
    FakeEventSource.instances = []
    data({ initial: 0, subscribe: '/sse', EventSource: ES })
    expect(FakeEventSource.instances).toHaveLength(1)
    expect(FakeEventSource.instances[0]!.url).toBe('/sse')
  })

  it('applies patches into the signal', () => {
    FakeEventSource.instances = []
    const { signal: sig } = data<{ count: number }>({
      initial: { count: 0 },
      subscribe: '/sse',
      EventSource: ES,
    })
    const es = FakeEventSource.instances[0]!
    es.emit('patch', { data: JSON.stringify({ op: 'set', path: 'count', value: 5 }) })
    expect(sig()).toEqual({ count: 5 })
  })

  it('routes through onPatch when provided', () => {
    FakeEventSource.instances = []
    const onPatch = vi.fn()
    data({
      initial: 0,
      subscribe: '/sse',
      onPatch,
      EventSource: ES,
    })
    const es = FakeEventSource.instances[0]!
    es.emit('patch', { data: '{"op":"set","value":1}' })
    expect(onPatch).toHaveBeenCalledTimes(1)
  })

  it('onError catches JSON parse failures', () => {
    FakeEventSource.instances = []
    const onError = vi.fn()
    data({ initial: 0, subscribe: '/sse', onError, EventSource: ES })
    const es = FakeEventSource.instances[0]!
    es.emit('patch', { data: 'not json' })
    expect(onError).toHaveBeenCalledTimes(1)
  })

  it('dispose closes the stream', () => {
    FakeEventSource.instances = []
    const { dispose } = data({ initial: 0, subscribe: '/sse', EventSource: ES })
    const es = FakeEventSource.instances[0]!
    expect(es.closed).toBe(false)
    dispose()
    expect(es.closed).toBe(true)
  })

  it('dispose is a noop when no subscription', () => {
    const { dispose } = data({ initial: 0 })
    expect(() => dispose()).not.toThrow()
  })
})
