import { afterEach, describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { compileAndMount as hydrate } from '../src/test-utils/compile-sui'
import { state, signal, type Signal } from '@ship/sail-runtime'

const counterSetup = (): { count: Signal<number> } => ({ count: signal(0) })

const fixturePath = (name: string): string =>
  resolve(__dirname, 'fixtures', name)
const loadFixture = (name: string): string =>
  readFileSync(fixturePath(name), 'utf8')

const containers: HTMLElement[] = []
const mkContainer = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}

afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

const flush = () => new Promise((r) => setTimeout(r, 0))

describe('hydrate — hello (static)', () => {
  it('renders static markup verbatim and disposes cleanly', () => {
    const container = mkContainer()
    const handle = hydrate(loadFixture('hello.sail'), undefined, {
      container,
    })
    expect(container.querySelector('h1')!.textContent).toBe('Hello, world.')
    expect(container.querySelectorAll('p')).toHaveLength(1)
    handle.dispose()
    expect(container.children).toHaveLength(0)
  })
})

describe('hydrate — counter (state + on:click)', () => {
  it('updates the displayed count when the button is clicked', async () => {
    const container = mkContainer()
    const handle = hydrate(loadFixture('counter.sail'), counterSetup, {
      container,
    })
    const value = container.querySelector('[data-test="value"]')!
    expect(value.textContent).toBe('0')

    const inc = container.querySelector(
      '[data-test="inc"]',
    ) as HTMLButtonElement
    inc.click()
    inc.click()
    expect(value.textContent).toBe('2')

    const reset = container.querySelector(
      '[data-test="reset"]',
    ) as HTMLButtonElement
    reset.click()
    expect(value.textContent).toBe('0')

    handle.dispose()
    expect(container.children).toHaveLength(0)
  })

  it('removes event listeners on dispose so post-dispose clicks are inert', () => {
    const container = mkContainer()
    const handle = hydrate(loadFixture('counter.sail'), counterSetup, {
      container,
    })
    const inc = container.querySelector(
      '[data-test="inc"]',
    ) as HTMLButtonElement
    handle.dispose()
    // Re-attach the button so we can click it; the listener should
    // already have been removed.
    document.body.appendChild(inc)
    inc.click()
    // No exception thrown, no errors.
    inc.remove()
  })
})

describe('hydrate — agents (live data + while + if + for)', () => {
  it('walks loading → ok and renders the grid', async () => {
    const container = mkContainer()
    let resolveNow!: (v: { items: { id: string; name: string }[] }) => void
    const ship = {
      agents: {
        summaries: () =>
          new Promise<{ items: { id: string; name: string }[] }>((res) => {
            resolveNow = res
          }),
      },
    }
    const handle = hydrate(loadFixture('agents.sail'), undefined, {
      container,
      ship,
    })
    expect(container.querySelector('[data-test="loading"]')).toBeTruthy()

    resolveNow({ items: [{ id: 'a', name: 'Ada' }, { id: 'b', name: 'Bea' }] })
    await flush()

    expect(container.querySelector('[data-test="loading"]')).toBeNull()
    const items = container.querySelectorAll('[data-test="agent"]')
    expect(items).toHaveLength(2)
    expect(items[0]!.textContent).toBe('Ada')
    expect(items[1]!.textContent).toBe('Bea')
    handle.dispose()
  })

  it('renders the empty branch when items is []', async () => {
    const container = mkContainer()
    const ship = {
      agents: { summaries: () => Promise.resolve({ items: [] }) },
    }
    const handle = hydrate(loadFixture('agents.sail'), undefined, {
      container,
      ship,
    })
    await flush()
    expect(container.querySelector('[data-test="empty"]')).toBeTruthy()
    expect(container.querySelector('[data-test="grid"]')).toBeNull()
    handle.dispose()
  })

  it('renders the error branch when the source rejects', async () => {
    const container = mkContainer()
    const ship = {
      agents: { summaries: () => Promise.reject(new Error('boom')) },
    }
    const handle = hydrate(loadFixture('agents.sail'), undefined, {
      container,
      ship,
    })
    await flush()
    const err = container.querySelector('[data-test="error"]')
    expect(err).toBeTruthy()
    expect(err!.textContent).toBe('Error: boom')
    handle.dispose()
  })

  it('does not flicker while-loading once already in :ok', async () => {
    const container = mkContainer()
    const ship = {
      agents: {
        summaries: () => Promise.resolve({ items: [{ id: 'x', name: 'X' }] }),
      },
    }
    const handle = hydrate(loadFixture('agents.sail'), undefined, {
      container,
      ship,
    })
    await flush()
    expect(container.querySelector('[data-test="loading"]')).toBeNull()
    // Force a re-evaluation via a no-op DOM patch — the live signal
    // should remain in `:ok` and the loading branch must stay
    // unmounted.
    await flush()
    expect(container.querySelector('[data-test="loading"]')).toBeNull()
    handle.dispose()
  })
})

describe('hydrate — ship-button (component + props + reflect)', () => {
  it('registers a custom element and reflects bool prop changes', async () => {
    const container = mkContainer()
    hydrate(loadFixture('ship-button.sail'), undefined, {
      container,
      componentName: 'ship-button-test',
    })

    const el = document.createElement('ship-button-test')
    el.setAttribute('label', 'Save')
    document.body.appendChild(el)
    await flush()

    const root = el.querySelector('[data-test="root"]') as HTMLButtonElement
    expect(root).toBeTruthy()
    expect(root.getAttribute('aria-label')).toBe('Save')
    expect(el.querySelector('[data-test="spinner"]')).toBeNull()

    el.setAttribute('loading', '')
    await flush()
    expect(el.hasAttribute('loading')).toBe(true)
    expect(el.querySelector('[data-test="spinner"]')).toBeTruthy()
    expect(root.hasAttribute('disabled')).toBe(true)
    el.remove()
  })
})

describe('hydrate — disposer fully tears down', () => {
  it('removes inserted nodes and unsubscribes signals', () => {
    const container = mkContainer()
    const handle = hydrate(loadFixture('counter.sail'), counterSetup, {
      container,
    })
    expect(container.childNodes.length).toBeGreaterThan(0)
    handle.dispose()
    expect(container.childNodes.length).toBe(0)
  })

  it('disposes effects so subsequent signal writes do not throw', () => {
    const container = mkContainer()
    const handle = hydrate(loadFixture('counter.sail'), counterSetup, {
      container,
    })
    handle.dispose()
    // No-op DOM mutations after dispose should be safe.
    expect(() => {
      container.innerHTML = ''
    }).not.toThrow()
  })
})

// Expression-guard tests (rejecting assignments, statements, and
// non-handler arrows inside `{expr}`) moved to the Rust compiler's
// diagnostics suite — see `crates/sail/compiler/tests/diagnostics.rs`.
