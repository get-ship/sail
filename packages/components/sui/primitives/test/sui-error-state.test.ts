import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-error-state/sui-error-state.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-error-state>', () => {

  it('renders a shadow root with a box div', () => {
    const { el } = mount(reg, 'sui-error-state')
    expect(shadow(el).querySelector('[part~="box"]')).not.toBeNull()
  })

  it('box has role=alert', () => {
    const { el } = mount(reg, 'sui-error-state')
    expect(shadow(el).querySelector('[part~="box"]')!.getAttribute('role')).toBe('alert')
  })

  it('box has part=box', () => {
    const { el } = mount(reg, 'sui-error-state')
    expect(shadow(el).querySelector('[part~="box"]')).not.toBeNull()
  })

  it('renders title attr text in .title', async () => {
    const { el } = mount(reg, 'sui-error-state', { title: 'Something broke' })
    await flush()
    expect(shadow(el).querySelector('[part~="title"]')!.textContent).toContain('Something broke')
  })

  it('falls back to "Something went wrong" when title is empty', async () => {
    const { el } = mount(reg, 'sui-error-state')
    await flush()
    expect(shadow(el).querySelector('[part~="title"]')!.textContent).toContain('Something went wrong')
  })

  it('size and variant reflect as attributes', () => {
    const { el } = mount(reg, 'sui-error-state', { size: 'lg', variant: 'inline' })
    expect(el.getAttribute('size')).toBe('lg')
    expect(el.getAttribute('variant')).toBe('inline')
  })

  it('details toggle hidden by default (no slot content)', async () => {
    const { el } = mount(reg, 'sui-error-state', { title: 'Oops' })
    await flush()
    const toggle = shadow(el).querySelector('[part~="details-toggle"]') as HTMLElement | null
    expect(toggle).not.toBeNull()
    // Should be CSS-hidden via :host(:not([data-has-details]))
    expect(el.hasAttribute('data-has-details')).toBe(false)
  })

  it('details toggle shown when details slot has content', async () => {
    const { el } = mount(
      reg,
      'sui-error-state',
      { title: 'Oops' },
      '<pre slot="details">stack trace</pre>',
    )
    await flush()
    expect(el.hasAttribute('data-has-details')).toBe(true)
  })

  it('clicking toggle sets data-details-open', async () => {
    const { el } = mount(
      reg,
      'sui-error-state',
      { title: 'Oops' },
      '<pre slot="details">stack trace</pre>',
    )
    await flush()
    const toggle = shadow(el).querySelector('[part~="details-toggle"]') as HTMLButtonElement
    expect(el.hasAttribute('data-details-open')).toBe(false)
    toggle.click()
    await flush()
    expect(el.hasAttribute('data-details-open')).toBe(true)
  })

  it('second click on toggle removes data-details-open', async () => {
    const { el } = mount(
      reg,
      'sui-error-state',
      { title: 'Oops' },
      '<pre slot="details">stack trace</pre>',
    )
    await flush()
    const toggle = shadow(el).querySelector('[part~="details-toggle"]') as HTMLButtonElement
    toggle.click()
    await flush()
    toggle.click()
    await flush()
    expect(el.hasAttribute('data-details-open')).toBe(false)
  })

  it('toggle button text changes on click', async () => {
    const { el } = mount(
      reg,
      'sui-error-state',
      { title: 'Oops' },
      '<pre slot="details">stack trace</pre>',
    )
    await flush()
    const toggle = shadow(el).querySelector('[part~="details-toggle"]') as HTMLButtonElement
    expect(toggle.textContent).toBe('Show details')
    toggle.click()
    await flush()
    expect(toggle.textContent).toBe('Hide details')
  })

  it('toggle button aria-expanded updates correctly', async () => {
    const { el } = mount(
      reg,
      'sui-error-state',
      { title: 'Oops' },
      '<pre slot="details">error info</pre>',
    )
    await flush()
    const toggle = shadow(el).querySelector('[part~="details-toggle"]') as HTMLButtonElement
    expect(toggle.getAttribute('aria-expanded')).toBe('false')
    toggle.click()
    await flush()
    expect(toggle.getAttribute('aria-expanded')).toBe('true')
  })

  it('renders sui-icon in the icon part', () => {
    const { el } = mount(reg, 'sui-error-state', { icon: 'alert-circle' })
    expect(shadow(el).querySelector('[part~="icon"] sui-icon')).not.toBeNull()
  })
})
