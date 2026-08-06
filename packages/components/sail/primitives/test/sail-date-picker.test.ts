import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-date-picker/sail-date-picker.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)
const inputOf = (el: HTMLElement) => shadow(el).querySelector('input') as HTMLInputElement
const panelOf = (el: HTMLElement) => shadow(el).querySelector('[role="dialog"]') as HTMLElement | null
const days = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll("button[data-iso]")) as HTMLButtonElement[]

describe('<sail-date-picker>', () => {

  it('renders the trigger input, panel closed', async () => {
    const { el } = mount(reg, 'sail-date-picker')
    await flush()
    expect(inputOf(el)).not.toBeNull()
    expect(panelOf(el)).toBeNull()
  })

  it('opens the calendar panel on input click', async () => {
    const { el } = mount(reg, 'sail-date-picker', { value: '2026-05-18' })
    await flush()
    inputOf(el).click()
    await flush()
    const panel = panelOf(el)
    expect(panel).not.toBeNull()
    expect(panel!.getAttribute('role')).toBe('dialog')
    const ds = days(el)
    expect(ds.length).toBeGreaterThan(27)
    expect(ds.some((d) => d.dataset.iso === '2026-05-18')).toBe(true)
  })

  it('reflects value attribute round-trip', async () => {
    const { el } = mount(reg, 'sail-date-picker', { value: '2026-01-01' })
    await flush()
    expect(el.getAttribute('value')).toBe('2026-01-01')
    el.setAttribute('value', '2026-12-31')
    await flush()
    expect(el.getAttribute('value')).toBe('2026-12-31')
  })

  it('clicking a day commits the ISO value and invokes onChange', async () => {
    const { el } = mount(reg, 'sail-date-picker', { value: '2026-05-18' })
    await flush()
    inputOf(el).click()
    await flush()
    const events: CustomEvent[] = []
    ;(el as any).onChange = (d: any) => { events.push({ detail: d } as any) }
    const target = days(el).find((d) => d.dataset.iso === '2026-05-20') as HTMLButtonElement
    target.click()
    await flush()
    const last = events.at(-1)!
    expect(last.detail).toEqual({ value: '2026-05-20' })
    expect(inputOf(el).value).toBe('2026-05-20')
  })

  it('respects min/max — dates outside the window are disabled', async () => {
    const { el } = mount(reg, 'sail-date-picker', {
      value: '2026-05-15',
      min: '2026-05-10',
      max: '2026-05-20',
    })
    await flush()
    inputOf(el).click()
    await flush()
    const ds = days(el)
    const before = ds.find((d) => d.dataset.iso === '2026-05-09')
    const after = ds.find((d) => d.dataset.iso === '2026-05-21')
    expect(before?.disabled).toBe(true)
    expect(after?.disabled).toBe(true)
    const inside = ds.find((d) => d.dataset.iso === '2026-05-15')
    expect(inside?.disabled).toBe(false)
  })
})
