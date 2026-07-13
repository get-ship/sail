import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-error-boundary/sui-error-boundary.compiled.js'
import { findNearestBoundary } from '@ship/sui-runtime'
import { createContainerRegistry, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)
const fallbackBox = (el: HTMLElement) =>
  ([...shadow(el).querySelectorAll('.fallback')].find(
    (f) => f.closest('[data-component]') === el,
  ) ?? null) as HTMLElement | null
const fallbackMessage = (el: HTMLElement) =>
  shadow(el).querySelector('.message') as HTMLElement | null
const retryBtn = (el: HTMLElement) =>
  shadow(el).querySelector('.retry') as HTMLButtonElement | null
const visibleSlotContent = (el: HTMLElement, name?: string): Element[] => {
  const slot = (name
    ? shadow(el).querySelector(`slot[name="${name}"]`)
    : shadow(el).querySelector('slot:not([name])')) as HTMLSlotElement | null
  if (!slot) return []
  // Shadow slots assign; the light-DOM slot wrapper renders its children.
  const assigned = slot.assignedElements()
  return assigned.length > 0 ? assigned : [...slot.children]
}

describe('<sui-error-boundary>', () => {

  it('healthy state: renders children via default slot, no fallback visible', () => {
    const { el } = mount(
      reg,
      'sui-error-boundary',
      {},
      '<span data-test="child">protected</span>',
    )
    expect(fallbackBox(el)).toBeNull()
    const assigned = visibleSlotContent(el)
    expect(assigned).toHaveLength(1)
    expect(assigned[0]!.getAttribute('data-test')).toBe('child')
  })

  it('registers a boundary handle on its host', () => {
    const { el } = mount(reg, 'sui-error-boundary', {}, '')
    expect(findNearestBoundary(el)).not.toBeNull()
  })

  it('caught state: default fallback shows "Something went wrong" + retry', () => {
    const { el } = mount(reg, 'sui-error-boundary', {}, '<span>x</span>')
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'render')
    const box = fallbackBox(el)
    expect(box).not.toBeNull()
    expect(fallbackMessage(el)!.textContent).toBe('Something went wrong')
    expect(retryBtn(el)).not.toBeNull()
    expect(el.getAttribute('data-error-message')).toBe('boom')
  })

  it('fallback-text attribute is rendered in the default fallback', () => {
    const { el } = mount(
      reg,
      'sui-error-boundary',
      { 'fallback-text': 'Custom oh-no' },
      '<span>x</span>',
    )
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'effect')
    expect(fallbackMessage(el)!.textContent).toBe('Custom oh-no')
  })

  it('<template slot="fallback"> renders custom fallback markup on catch', () => {
    const { el } = mount(
      reg,
      'sui-error-boundary',
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
    const { el } = mount(reg, 'sui-error-boundary', {}, '<span data-test="child">x</span>')
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'render')
    expect(fallbackBox(el)).not.toBeNull()

    retryBtn(el)!.click()
    expect(fallbackBox(el)).toBeNull()
    expect(visibleSlotContent(el)[0]!.getAttribute('data-test')).toBe('child')
    expect(el.getAttribute('data-error-message')).toBeNull()
  })

  it('host.reset() recovers programmatically', () => {
    const { el } = mount(reg, 'sui-error-boundary', {}, '<span>x</span>')
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'effect')
    expect(fallbackBox(el)).not.toBeNull()
    ;(el as HTMLElement & { reset(): void }).reset()
    expect(fallbackBox(el)).toBeNull()
  })

  it('dispatches sui-error-boundary:caught with { error, source } (bubbles + composed)', () => {
    const { el } = mount(reg, 'sui-error-boundary', {}, '<span>x</span>')
    let evt: CustomEvent<{ error: Error; source: string }> | null = null
    el.addEventListener('sui-error-boundary:caught', (ev) => {
      evt = ev as CustomEvent<{ error: Error; source: string }>
    })
    const handle = findNearestBoundary(el)!
    handle.catch(new Error('boom'), 'effect')
    expect(evt).not.toBeNull()
    expect(evt!.detail.error.message).toBe('boom')
    expect(evt!.detail.source).toBe('effect')
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('inner boundary catches first when nested', () => {
    const { container } = mount(reg, 'sui-error-boundary', {}, '<span>outer</span>')
    container.innerHTML =
      '<sui-error-boundary id="outer"><sui-error-boundary id="inner"><span>x</span></sui-error-boundary></sui-error-boundary>'
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
    const { el } = mount(reg, 'sui-error-boundary', {}, '<span>x</span>')
    expect(findNearestBoundary(el)).not.toBeNull()
    el.remove()
    expect(findNearestBoundary(el)).toBeNull()
  })
})
