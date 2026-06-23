import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-accordion/sui-accordion.compiled.js'
import '../src/sui-accordion/sui-accordion-item.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

function mountAccordion(type = 'single', value = '', extraItems = '') {
  const container = reg.mk()
  container.innerHTML = `
    <sui-accordion type="${type}" ${value ? `value="${value}"` : ''}>
      <sui-accordion-item value="a">
        <span slot="trigger">Item A</span>Content A
      </sui-accordion-item>
      <sui-accordion-item value="b">
        <span slot="trigger">Item B</span>Content B
      </sui-accordion-item>
      <sui-accordion-item value="c" disabled>
        <span slot="trigger">Item C</span>Content C
      </sui-accordion-item>
      ${extraItems}
    </sui-accordion>
  `
  return container.querySelector('sui-accordion') as HTMLElement
}

// ── <sui-accordion> ──────────────────────────────────────────────────────────

describe('<sui-accordion>', () => {

  it('renders a shadow root with a slot', () => {
    const { el } = mount(reg, 'sui-accordion')
    expect(shadow(el).querySelector('slot')).not.toBeNull()
  })

  it('type and value reflect as attributes', () => {
    const { el } = mount(reg, 'sui-accordion', { type: 'multiple', value: 'a,b' })
    expect(el.getAttribute('type')).toBe('multiple')
    expect(el.getAttribute('value')).toBe('a,b')
  })

  it('all items closed by default', async () => {
    const el = mountAccordion()
    await flush()
    const items = el.querySelectorAll('sui-accordion-item')
    for (const item of items) {
      expect(item.hasAttribute('open')).toBe(false)
    }
  })

  it('value attribute opens the matching item', async () => {
    const el = mountAccordion('single', 'a')
    await flush()
    const items = el.querySelectorAll('sui-accordion-item')
    expect(items[0].hasAttribute('open')).toBe(true)
    expect(items[1].hasAttribute('open')).toBe(false)
  })

  it('single mode: clicking opens one item', async () => {
    const el = mountAccordion()
    await flush()
    const itemA = el.querySelector('sui-accordion-item[value="a"]') as HTMLElement
    const btn = shadow(itemA).querySelector('button') as HTMLButtonElement
    btn.click()
    await flush()
    expect(itemA.hasAttribute('open')).toBe(true)
  })

  it('single mode: clicking second item closes first', async () => {
    const el = mountAccordion()
    await flush()
    const itemA = el.querySelector('sui-accordion-item[value="a"]') as HTMLElement
    const itemB = el.querySelector('sui-accordion-item[value="b"]') as HTMLElement
    shadow(itemA).querySelector('button')!.click()
    await flush()
    shadow(itemB).querySelector('button')!.click()
    await flush()
    expect(itemA.hasAttribute('open')).toBe(false)
    expect(itemB.hasAttribute('open')).toBe(true)
  })

  it('single mode: clicking open item closes it', async () => {
    const el = mountAccordion('single', 'a')
    await flush()
    const itemA = el.querySelector('sui-accordion-item[value="a"]') as HTMLElement
    shadow(itemA).querySelector('button')!.click()
    await flush()
    expect(itemA.hasAttribute('open')).toBe(false)
  })

  it('multiple mode: several items can be open simultaneously', async () => {
    const el = mountAccordion('multiple')
    await flush()
    const itemA = el.querySelector('sui-accordion-item[value="a"]') as HTMLElement
    const itemB = el.querySelector('sui-accordion-item[value="b"]') as HTMLElement
    shadow(itemA).querySelector('button')!.click()
    await flush()
    shadow(itemB).querySelector('button')!.click()
    await flush()
    expect(itemA.hasAttribute('open')).toBe(true)
    expect(itemB.hasAttribute('open')).toBe(true)
  })

  it('disabled item does not toggle', async () => {
    const el = mountAccordion()
    await flush()
    const itemC = el.querySelector('sui-accordion-item[value="c"]') as HTMLElement
    shadow(itemC).querySelector('button')!.click()
    await flush()
    expect(itemC.hasAttribute('open')).toBe(false)
  })

  it('fires sui-change with correct value on toggle (bubbles + composed)', async () => {
    const el = mountAccordion()
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', e => { evt = e as CustomEvent })
    const itemA = el.querySelector('sui-accordion-item[value="a"]') as HTMLElement
    shadow(itemA).querySelector('button')!.click()
    await flush()
    expect(evt!.detail).toEqual({ value: 'a' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })
})

// ── <sui-accordion-item> ─────────────────────────────────────────────────────

describe('<sui-accordion-item>', () => {

  it('renders trigger button and content-wrap in shadow root', () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x' })
    expect(shadow(el).querySelector('button.trigger')).not.toBeNull()
    expect(shadow(el).querySelector('.content-wrap')).not.toBeNull()
  })

  it('value, disabled, open reflect as attributes', () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'y', disabled: true, open: true })
    expect(el.getAttribute('value')).toBe('y')
    expect(el.hasAttribute('disabled')).toBe(true)
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('trigger has aria-expanded=false when closed', async () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x' })
    await flush()
    expect(shadow(el).querySelector('button')!.getAttribute('aria-expanded')).toBe('false')
  })

  it('trigger has aria-expanded=true when open', async () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x', open: true })
    await flush()
    expect(shadow(el).querySelector('button')!.getAttribute('aria-expanded')).toBe('true')
  })

  it('trigger aria-controls matches content-wrap id', async () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x' })
    await flush()
    const btn = shadow(el).querySelector('button')!
    const wrap = shadow(el).querySelector('.content-wrap') as HTMLElement
    expect(btn.getAttribute('aria-controls')).toBe(wrap.id)
    expect(wrap.id).toMatch(/^sui-acc-content-\d+$/)
  })

  it('content-wrap has role=region and aria-labelledby pointing to trigger', async () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x' })
    await flush()
    const btn = shadow(el).querySelector('button')!
    const wrap = shadow(el).querySelector('.content-wrap')!
    expect(wrap.getAttribute('role')).toBe('region')
    expect(wrap.getAttribute('aria-labelledby')).toBe(btn.id)
  })

  it('trigger has part="trigger" and chevron has part="chevron"', () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x' })
    expect(shadow(el).querySelector('[part~="trigger"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="chevron"]')).not.toBeNull()
  })

  it('content-wrap has part="content"', () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x' })
    expect(shadow(el).querySelector('[part~="content"]')).not.toBeNull()
  })

  it('trigger button is disabled when disabled attribute set', async () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x', disabled: true })
    await flush()
    expect(shadow(el).querySelector('button')!.hasAttribute('disabled')).toBe(true)
  })

  it('dispatches sui-accordion-item-toggle when trigger is clicked (bubbles + composed)', async () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'abc' })
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-accordion-item-toggle', e => { evt = e as CustomEvent })
    shadow(el).querySelector('button')!.click()
    await flush()
    expect(evt!.detail).toEqual({ value: 'abc' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('does not dispatch toggle when disabled', async () => {
    const { el } = mount(reg, 'sui-accordion-item', { value: 'x', disabled: true })
    await flush()
    let count = 0
    el.addEventListener('sui-accordion-item-toggle', () => count++)
    shadow(el).querySelector('button')!.click()
    await flush()
    expect(count).toBe(0)
  })
})

// ── integration ──────────────────────────────────────────────────────────────

describe('<sui-accordion> + <sui-accordion-item> integration', () => {
  it('ArrowDown on trigger dispatches sui-accordion-nav with next direction (bubbles + composed)', async () => {
    const el = mountAccordion()
    await flush()
    const itemA = el.querySelector('sui-accordion-item[value="a"]') as HTMLElement
    let evt: CustomEvent | null = null
    el.addEventListener('sui-accordion-nav', e => { evt = e as CustomEvent })
    shadow(itemA).querySelector('button')!.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }),
    )
    await flush()
    expect((evt!.detail as { direction: string }).direction).toBe('next')
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('Home key dispatches sui-accordion-nav with first direction', async () => {
    const el = mountAccordion()
    await flush()
    const itemA = el.querySelector('sui-accordion-item[value="a"]') as HTMLElement
    let detail: { direction: string } | null = null
    el.addEventListener('sui-accordion-nav', e => { detail = (e as CustomEvent).detail })
    shadow(itemA).querySelector('button')!.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Home', bubbles: true }),
    )
    await flush()
    expect(detail?.direction).toBe('first')
  })
})
