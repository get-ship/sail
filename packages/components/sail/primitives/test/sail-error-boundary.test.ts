import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-error-boundary/sail-error-boundary.compiled.js'
import { findNearestBoundary } from '@ship/sail-runtime'
import { createContainerRegistry, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)
const fallbackBox = (el: HTMLElement) =>
  ([...shadow(el).querySelectorAll('[role="alert"]')].find(
    (f) => f.closest('[data-component]') === el,
  ) ?? null) as HTMLElement | null
const fallbackText = (el: HTMLElement) => fallbackBox(el)?.textContent ?? ''
const retryBtn = (el: HTMLElement) =>
  shadow(el).querySelector('[role="alert"] button') as HTMLButtonElement | null
const visibleSlotContent = (el: HTMLElement, name?: string): Element[] => {
  const slot = (name
    ? shadow(el).querySelector(`slot[name="${name}"]`)
    : shadow(el).querySelector('slot:not([name])')) as HTMLSlotElement | null
  if (!slot) return []
  // Shadow slots assign; the light-DOM slot wrapper renders its children.
  const assigned = slot.assignedElements()
  return assigned.length > 0 ? assigned : [...slot.children]
}

describe('<sail-error-boundary>', () => {

  it('healthy state: renders children via default slot, no fallback visible', () => {
    const { el } = mount(
      reg,
      'sail-error-boundary',
      {},
      '<span data-test="child">protected</span>',
    )
    expect(fallbackBox(el)).toBeNull()
    const assigned = visibleSlotContent(el)
    expect(assigned).toHaveLength(1)
    expect(assigned[0]!.getAttribute('data-test')).toBe('child')
  })

  it('registers a boundary handle on its host', () => {
    const { el } = mount(reg, 'sail-error-boundary', {}, '')
    expect(findNearestBoundary(el)).not.toBeNull()
  })

  it('caught state: default fallback shows "Something went wrong" + retry', () => {
    const { el } = mount(reg, 'sail-error-boundary', {}, '<span>x</span>')
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'render')
    const box = fallbackBox(el)
    expect(box).not.toBeNull()
    expect(fallbackText(el)).toContain('Something went wrong')
    expect(retryBtn(el)).not.toBeNull()
    expect(el.getAttribute('data-error-message')).toBe('boom')
  })

  it('fallback-text attribute is rendered in the default fallback', () => {
    const { el } = mount(
      reg,
      'sail-error-boundary',
      { 'fallback-text': 'Custom oh-no' },
      '<span>x</span>',
    )
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'effect')
    expect(fallbackText(el)).toContain('Custom oh-no')
  })

  it('<template slot="fallback"> renders custom fallback markup on catch', () => {
    const { el } = mount(
      reg,
      'sail-error-boundary',
      {},
      '<span data-test="child">x</span><template slot="fallback"><div data-test="custom-fallback">my fallback</div></template>',
    )
    // Pre-catch: the synthesized wrapper exists but the fallback slot
    // isn't rendered yet (subtree is in healthy branch).
    expect(shadow(el).querySelector('slot[name="fallback"]')).toBeNull()

    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'mount')

    const fallbackSlot = shadow(el).querySelector('slot[name="fallback"]') as HTMLSlotElement
    expect(fallbackSlot).not.toBeNull()
    const a = fallbackSlot.assignedElements()
    const assigned = a.length > 0 ? a : [...fallbackSlot.children]
    expect(assigned).toHaveLength(1)
    expect(assigned[0]!.querySelector('[data-test="custom-fallback"]')).not.toBeNull()
  })

  it('clicking the retry button clears the fallback and re-renders children', () => {
    const { el } = mount(reg, 'sail-error-boundary', {}, '<span data-test="child">x</span>')
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'render')
    expect(fallbackBox(el)).not.toBeNull()

    retryBtn(el)!.click()
    expect(fallbackBox(el)).toBeNull()
    expect(visibleSlotContent(el)[0]!.getAttribute('data-test')).toBe('child')
    expect(el.getAttribute('data-error-message')).toBeNull()
  })

  it('host.reset() recovers programmatically', () => {
    const { el } = mount(reg, 'sail-error-boundary', {}, '<span>x</span>')
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'effect')
    expect(fallbackBox(el)).not.toBeNull()
    ;(el as HTMLElement & { reset(): void }).reset()
    expect(fallbackBox(el)).toBeNull()
  })

  it('invokes onErrorCaught with { error, source }', () => {
    const { el } = mount(reg, 'sail-error-boundary', {}, '<span>x</span>')
    let detail: { error: Error; source: string } | null = null
    ;(el as any).onErrorCaught = (d: { error: Error; source: string }) => {
      detail = d
    }
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'effect')
    expect(detail).not.toBeNull()
    expect(detail!.error.message).toBe('boom')
    expect(detail!.source).toBe('effect')
  })

  it('inner boundary catches first when nested', () => {
    const { container } = mount(reg, 'sail-error-boundary', {}, '<span>outer</span>')
    container.innerHTML =
      '<sail-error-boundary id="outer"><sail-error-boundary id="inner"><span>x</span></sail-error-boundary></sail-error-boundary>'
    const outer = container.querySelector('#outer') as HTMLElement
    const inner = container.querySelector('#inner') as HTMLElement
    const target = inner.querySelector('span')!
    const innerHandle = findNearestBoundary(inner)!
    const found = findNearestBoundary(target)
    expect(found).toBe(innerHandle)
    found!.catch(new Error('inner-boom'), 'render')
    expect(fallbackBox(inner)).not.toBeNull()
    expect(fallbackBox(outer)).toBeNull()
  })

  it('disconnecting the boundary removes its handle', () => {
    const { el } = mount(reg, 'sail-error-boundary', {}, '<span>x</span>')
    expect(findNearestBoundary(el)).not.toBeNull()
    el.remove()
    expect(findNearestBoundary(el)).toBeNull()
  })
})
