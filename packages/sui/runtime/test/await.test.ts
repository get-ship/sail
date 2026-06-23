import { describe, expect, it } from 'vitest'
import { signal } from '../src/signal'
import { bindAwait } from '../src/await'

function setup() {
  const host = document.createElement('div')
  document.body.appendChild(host)
  const anchor = document.createComment('await')
  host.appendChild(anchor)
  return { host, anchor }
}

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('bindAwait', () => {
  it('shows loading then resolves to then', async () => {
    const { host, anchor } = setup()
    bindAwait({
      anchor,
      promise: () => Promise.resolve('OK'),
      loading: () => ({ nodes: [document.createTextNode('LOAD')] }),
      then_: (v) => ({ nodes: [document.createTextNode(String(v))] }),
    })
    expect(host.textContent).toBe('LOAD')
    await tick()
    await tick()
    expect(host.textContent).toBe('OK')
  })

  it('renders catch on rejection', async () => {
    const { host, anchor } = setup()
    bindAwait({
      anchor,
      promise: () => Promise.reject(new Error('boom')),
      loading: () => ({ nodes: [document.createTextNode('LOAD')] }),
      catch_: (e) => ({ nodes: [document.createTextNode((e as Error).message)] }),
    })
    expect(host.textContent).toBe('LOAD')
    await tick()
    await tick()
    expect(host.textContent).toBe('boom')
  })

  it('resets to loading when promise dep changes', async () => {
    const { host, anchor } = setup()
    const id = signal(1)
    let resolveSecond: (v: string) => void = () => {}
    bindAwait({
      anchor,
      promise: () => {
        const cur = id()
        if (cur === 1) return Promise.resolve('first')
        return new Promise((r) => { resolveSecond = r })
      },
      loading: () => ({ nodes: [document.createTextNode('LOAD')] }),
      then_: (v) => ({ nodes: [document.createTextNode(String(v))] }),
    })
    await tick()
    expect(host.textContent).toBe('first')
    id.set(2)
    expect(host.textContent).toBe('LOAD')
    resolveSecond('second')
    await tick()
    await tick()
    expect(host.textContent).toBe('second')
  })

  it('disposer prevents stale resolution from inserting', async () => {
    const { host, anchor } = setup()
    let resolve: (v: string) => void = () => {}
    const dispose = bindAwait({
      anchor,
      promise: () => new Promise<string>((r) => { resolve = r }),
      loading: () => ({ nodes: [document.createTextNode('LOAD')] }),
      then_: (v) => ({ nodes: [document.createTextNode(String(v))] }),
    })
    expect(host.textContent).toBe('LOAD')
    dispose()
    expect(host.textContent).toBe('')
    resolve('LATE')
    await tick()
    await tick()
    expect(host.textContent).toBe('') // stale resolution suppressed
  })
})
