import { describe, expect, it } from 'vitest'
import { signal } from '../src/signal'
import type { LiveSignal, LiveState } from '../src/live-state'
import { bindWhileError, bindWhileLoading } from '../src/while'

function setup() {
  const host = document.createElement('div')
  document.body.appendChild(host)
  const anchor = document.createComment('while')
  host.appendChild(anchor)
  return { host, anchor }
}

const makeLive = <T>(initial: LiveState<T>) =>
  signal<LiveState<T>>(initial) as LiveSignal<T>

describe('bindWhileLoading', () => {
  it('renders during loading, unmounts on ok', () => {
    const { host, anchor } = setup()
    const sig = makeLive<number>({ kind: 'loading' })
    bindWhileLoading({
      anchor,
      signal: sig,
      render: () => ({ nodes: [document.createTextNode('LOAD')] }),
    })
    expect(host.textContent).toBe('LOAD')
    sig.set({ kind: 'ok', value: 1 })
    expect(host.textContent).toBe('')
  })

  it('does not re-mount across already-loading writes', () => {
    const { host, anchor } = setup()
    const sig = makeLive<number>({ kind: 'loading' })
    let renders = 0
    bindWhileLoading({
      anchor,
      signal: sig,
      render: () => {
        renders++
        return { nodes: [document.createTextNode('LOAD')] }
      },
    })
    expect(renders).toBe(1)
    sig.set({ kind: 'loading' }) // identical kind; signal coalesces by Object.is on full state
    expect(renders).toBe(1)
    expect(host.textContent).toBe('LOAD')
  })

  it('re-mounts when source flips back to loading', () => {
    const { host, anchor } = setup()
    const sig = makeLive<number>({ kind: 'loading' })
    let renders = 0
    bindWhileLoading({
      anchor,
      signal: sig,
      render: () => {
        renders++
        return { nodes: [document.createTextNode('LOAD')] }
      },
    })
    sig.set({ kind: 'ok', value: 1 })
    expect(host.textContent).toBe('')
    sig.set({ kind: 'loading' })
    expect(host.textContent).toBe('LOAD')
    expect(renders).toBe(2)
  })

  it('disposer fully tears down', () => {
    const { host, anchor } = setup()
    const sig = makeLive<number>({ kind: 'loading' })
    const dispose = bindWhileLoading({
      anchor,
      signal: sig,
      render: () => ({ nodes: [document.createTextNode('LOAD')] }),
    })
    expect(host.textContent).toBe('LOAD')
    dispose()
    expect(host.textContent).toBe('')
    // After dispose, signal updates should not re-mount.
    sig.set({ kind: 'loading' })
    expect(host.textContent).toBe('')
  })
})

describe('bindWhileError', () => {
  it('renders error message, unmounts on transition', () => {
    const { host, anchor } = setup()
    const sig = makeLive<number>({ kind: 'loading' })
    bindWhileError({
      anchor,
      signal: sig,
      render: (err) => ({ nodes: [document.createTextNode(err.message)] }),
    })
    expect(host.textContent).toBe('')
    sig.set({ kind: 'error', error: new Error('boom') })
    expect(host.textContent).toBe('boom')
    sig.set({ kind: 'ok', value: 1 })
    expect(host.textContent).toBe('')
  })

  it('re-renders with new error on subsequent error transition', () => {
    const { host, anchor } = setup()
    const sig = makeLive<number>({ kind: 'loading' })
    bindWhileError({
      anchor,
      signal: sig,
      render: (err) => ({ nodes: [document.createTextNode(err.message)] }),
    })
    sig.set({ kind: 'error', error: new Error('first') })
    expect(host.textContent).toBe('first')
    sig.set({ kind: 'loading' })
    sig.set({ kind: 'error', error: new Error('second') })
    expect(host.textContent).toBe('second')
  })
})

describe('multiple boundaries on the same signal', () => {
  it('each mounts and unmounts independently', () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    const a1 = document.createComment('loading')
    const a2 = document.createComment('error')
    host.appendChild(a1)
    host.appendChild(a2)

    const sig = makeLive<number>({ kind: 'loading' })
    bindWhileLoading({
      anchor: a1,
      signal: sig,
      render: () => ({ nodes: [document.createTextNode('L')] }),
    })
    bindWhileError({
      anchor: a2,
      signal: sig,
      render: (err) => ({ nodes: [document.createTextNode(`E:${err.message}`)] }),
    })
    expect(host.textContent).toBe('L')
    sig.set({ kind: 'error', error: new Error('x') })
    expect(host.textContent).toBe('E:x')
    sig.set({ kind: 'ok', value: 1 })
    expect(host.textContent).toBe('')
  })
})
