import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { effect } from '../src/signal'
import { bindFor } from '../src/for'
import { bindIf } from '../src/if'
import {
  attachBoundary,
  detachBoundary,
  findNearestBoundary,
  runProtected,
  type BoundaryHandle,
  type BoundarySource,
} from '../src/error-boundary'

interface CapturedError {
  err: unknown
  source: BoundarySource
}

function makeBoundary(host: HTMLElement): { handle: BoundaryHandle; caught: CapturedError[] } {
  const caught: CapturedError[] = []
  let inError = false
  const handle: BoundaryHandle = {
    catch(err, source) {
      caught.push({ err, source })
      inError = true
    },
    reset() {
      inError = false
    },
  }
  attachBoundary(host, handle)
  // expose for tests; not part of the contract.
  ;(handle as { inError?: boolean }).inError = inError
  return { handle, caught }
}

describe('findNearestBoundary', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('returns null when no boundary exists in the ancestor chain', () => {
    const node = document.createElement('div')
    document.body.appendChild(node)
    expect(findNearestBoundary(node)).toBeNull()
  })

  it('finds a boundary on a direct parent', () => {
    const boundary = document.createElement('div')
    const child = document.createElement('span')
    boundary.appendChild(child)
    document.body.appendChild(boundary)
    const { handle } = makeBoundary(boundary)
    expect(findNearestBoundary(child)).toBe(handle)
  })

  it('finds the innermost boundary when multiple are nested', () => {
    const outer = document.createElement('div')
    const inner = document.createElement('div')
    const target = document.createElement('span')
    outer.appendChild(inner)
    inner.appendChild(target)
    document.body.appendChild(outer)
    const { handle: outerHandle } = makeBoundary(outer)
    const { handle: innerHandle } = makeBoundary(inner)
    expect(findNearestBoundary(target)).toBe(innerHandle)
    expect(findNearestBoundary(outer)).toBe(outerHandle)
  })

  it('crosses shadow-root boundaries to find an outer boundary', () => {
    const boundary = document.createElement('div')
    const host = document.createElement('div')
    boundary.appendChild(host)
    document.body.appendChild(boundary)
    const root = host.attachShadow({ mode: 'open' })
    const inner = document.createElement('span')
    root.appendChild(inner)
    const { handle } = makeBoundary(boundary)
    expect(findNearestBoundary(inner)).toBe(handle)
  })

  it('detachBoundary removes the registration', () => {
    const boundary = document.createElement('div')
    document.body.appendChild(boundary)
    const { handle } = makeBoundary(boundary)
    expect(findNearestBoundary(boundary)).toBe(handle)
    detachBoundary(boundary)
    expect(findNearestBoundary(boundary)).toBeNull()
  })
})

describe('runProtected', () => {
  afterEach(() => {
    document.body.innerHTML = ''
    vi.restoreAllMocks()
  })

  it('returns fn result on success', () => {
    expect(runProtected(null, 'render', () => 42)).toBe(42)
  })

  it('routes a throw to the nearest boundary', () => {
    const boundary = document.createElement('div')
    document.body.appendChild(boundary)
    const child = document.createElement('span')
    boundary.appendChild(child)
    const { caught } = makeBoundary(boundary)
    const result = runProtected(child, 'render', () => {
      throw new Error('boom')
    })
    expect(result).toBeUndefined()
    expect(caught).toHaveLength(1)
    expect(caught[0]!.source).toBe('render')
    expect((caught[0]!.err as Error).message).toBe('boom')
  })

  it('logs to console.error and returns undefined when no boundary is in scope', () => {
    const err = vi.spyOn(console, 'error').mockImplementation(() => {})
    const node = document.createElement('div')
    document.body.appendChild(node)
    const result = runProtected(node, 'render', () => {
      throw new Error('uncaught')
    })
    expect(result).toBeUndefined()
    expect(err).toHaveBeenCalled()
  })
})

describe('bindFor — boundary integration', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('routes a render throw to the boundary and continues for healthy items', () => {
    const boundary = document.createElement('div')
    document.body.appendChild(boundary)
    const { caught } = makeBoundary(boundary)
    const anchor = document.createComment('list')
    boundary.appendChild(anchor)

    const items = signal<{ id: string; bad: boolean }[]>([
      { id: 'a', bad: false },
      { id: 'b', bad: true },
      { id: 'c', bad: false },
    ])

    bindFor({
      anchor,
      items: () => items(),
      key: (i) => i.id,
      render: (item) => {
        if (item.bad) throw new Error(`bad ${item.id}`)
        const el = document.createElement('li')
        el.id = `it-${item.id}`
        return { nodes: [el] }
      },
    })

    expect(caught).toHaveLength(1)
    expect((caught[0]!.err as Error).message).toBe('bad b')
    const rendered = Array.from(boundary.querySelectorAll('li')).map((n) => n.id)
    expect(rendered).toEqual(['it-a', 'it-c'])
  })
})

describe('bindIf — boundary integration', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('routes a branch render throw to the nearest boundary', () => {
    const boundary = document.createElement('div')
    document.body.appendChild(boundary)
    const { caught } = makeBoundary(boundary)
    const anchor = document.createComment('if')
    boundary.appendChild(anchor)

    bindIf({
      anchor,
      cond: () => true,
      then_: () => {
        throw new Error('branch boom')
      },
    })

    expect(caught).toHaveLength(1)
    expect((caught[0]!.err as Error).message).toBe('branch boom')
    expect(caught[0]!.source).toBe('render')
  })
})

describe('effect — boundary integration via host capture', () => {
  let errSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
  })

  afterEach(() => {
    document.body.innerHTML = ''
    vi.restoreAllMocks()
  })

  it('routes a re-run throw to the boundary captured at creation time', async () => {
    const { withHost } = await import('../src/signal')
    const boundary = document.createElement('div')
    document.body.appendChild(boundary)
    const host = document.createElement('div')
    boundary.appendChild(host)
    const { caught } = makeBoundary(boundary)

    const trigger = signal(0)
    const dispose = withHost(host, () =>
      effect(() => {
        if (trigger() > 0) throw new Error('reactive boom')
      }),
    )

    trigger.set(1)
    expect(caught).toHaveLength(1)
    expect((caught[0]!.err as Error).message).toBe('reactive boom')
    expect(caught[0]!.source).toBe('effect')

    dispose()
  })

  it('console.errors when no boundary is in scope', async () => {
    const { withHost } = await import('../src/signal')
    const host = document.createElement('div')
    document.body.appendChild(host)
    const trigger = signal(0)
    const dispose = withHost(host, () =>
      effect(() => {
        if (trigger() > 0) throw new Error('lonely throw')
      }),
    )
    trigger.set(1)
    expect(errSpy).toHaveBeenCalled()
    const logged = errSpy.mock.calls.flat().some((c) => String(c).includes('lonely throw'))
    expect(logged).toBe(true)
    dispose()
  })
})

describe('boundary handle reset', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('a reset clears the captured state so further runs are observed', () => {
    const boundary = document.createElement('div')
    document.body.appendChild(boundary)
    const seen: BoundarySource[] = []
    const handle: BoundaryHandle = {
      catch(_err, source) {
        seen.push(source)
      },
      reset() {
        seen.length = 0
      },
    }
    attachBoundary(boundary, handle)
    const child = document.createElement('span')
    boundary.appendChild(child)

    runProtected(child, 'render', () => {
      throw new Error('first')
    })
    expect(seen).toEqual(['render'])

    handle.reset()
    runProtected(child, 'mount', () => {
      throw new Error('second')
    })
    expect(seen).toEqual(['mount'])
  })
})
