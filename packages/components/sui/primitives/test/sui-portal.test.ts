import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-portal/sui-portal.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

let warnSpy: ReturnType<typeof vi.spyOn>
beforeEach(() => {
  warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})
})
afterEach(() => {
  warnSpy.mockRestore()
})

const inBody = (sel: string) => document.body.querySelector(sel)
const directBodyChildren = (sel: string) =>
  Array.from(document.body.children).filter((el) => el.matches(sel))

describe('<sui-portal>', () => {

  it('teleports slotted children to body on mount', () => {
    const { el } = mount(
      reg,
      'sui-portal',
      { target: 'body' },
      '<span data-test="child">hi</span>',
    )
    const span = inBody('[data-test="child"]') as HTMLElement | null
    expect(span).not.toBeNull()
    expect(span!.parentElement).toBe(document.body)
    // Not in the portal's light DOM anymore.
    expect(el.querySelector('[data-test="child"]')).toBeNull()
  })

  it('removes teleported children when the portal unmounts', () => {
    mount(reg, 'sui-portal', { target: 'body' }, '<span data-test="bye">bye</span>')
    expect(inBody('[data-test="bye"]')).not.toBeNull()
    reg.cleanup()
    expect(inBody('[data-test="bye"]')).toBeNull()
  })

  it('teleports into a custom selector target', () => {
    const sink = document.createElement('div')
    sink.id = 'portal-root'
    document.body.appendChild(sink)
    try {
      mount(
        reg,
        'sui-portal',
        { target: '#portal-root' },
        '<span data-test="custom">x</span>',
      )
      const span = sink.querySelector('[data-test="custom"]') as HTMLElement | null
      expect(span).not.toBeNull()
      expect(span!.parentElement).toBe(sink)
    } finally {
      sink.remove()
    }
  })

  it('re-targets when the target attribute changes', async () => {
    const sink = document.createElement('div')
    sink.id = 'portal-root-2'
    document.body.appendChild(sink)
    try {
      const { el } = mount(
        reg,
        'sui-portal',
        { target: 'body' },
        '<span data-test="move">m</span>',
      )
      const span = inBody('[data-test="move"]') as HTMLElement
      expect(span.parentElement).toBe(document.body)

      el.setAttribute('target', '#portal-root-2')
      await flush()

      expect(span.parentElement).toBe(sink)
    } finally {
      sink.remove()
    }
  })

  it('multiple portals to body append in registration order', () => {
    mount(reg, 'sui-portal', { target: 'body' }, '<span data-test="one">1</span>')
    mount(reg, 'sui-portal', { target: 'body' }, '<span data-test="two">2</span>')
    const spans = Array.from(
      document.body.querySelectorAll('[data-test="one"], [data-test="two"]'),
    )
    expect(spans).toHaveLength(2)
    const one = spans[0]!
    const two = spans[1]!
    expect(one.getAttribute('data-test')).toBe('one')
    expect(two.getAttribute('data-test')).toBe('two')
    // Document order: "one" appears before "two" in body.
    expect(
      one.compareDocumentPosition(two) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })

  it('missing target — warns and leaves children in light DOM', () => {
    const { el } = mount(
      reg,
      'sui-portal',
      { target: '#does-not-exist' },
      '<span data-test="orphan">o</span>',
    )
    expect(warnSpy).toHaveBeenCalled()
    const callArgs = warnSpy.mock.calls[0]!.join(' ')
    expect(callArgs).toContain('#does-not-exist')
    // Child stays in the portal's light DOM, untouched.
    const orphan = el.querySelector('[data-test="orphan"]') as HTMLElement | null
    expect(orphan).not.toBeNull()
    expect(orphan!.parentElement).toBe(el)
  })

  it('slotchange post-mount teleports newly added children', async () => {
    const { el } = mount(reg, 'sui-portal', { target: 'body' }, '')
    const late = document.createElement('span')
    late.setAttribute('data-test', 'late')
    el.appendChild(late)
    // slotchange dispatches on a microtask; let it settle.
    await flush()
    expect(late.parentElement).toBe(document.body)
    expect(el.querySelector('[data-test="late"]')).toBeNull()
  })

  it('default target is body when no target attribute is set', () => {
    const { el } = mount(reg, 'sui-portal', {}, '<span data-test="default">d</span>')
    void el
    expect(inBody('[data-test="default"]')).not.toBeNull()
  })
})
