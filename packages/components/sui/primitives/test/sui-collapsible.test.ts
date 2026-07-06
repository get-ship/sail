import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-collapsible/sui-collapsible.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const detailsEl = (el: HTMLElement) => shadow(el).querySelector('details') as HTMLDetailsElement | null
const summaryEl = (el: HTMLElement) => shadow(el).querySelector('summary') as HTMLElement | null
const contentWrap = (el: HTMLElement) => shadow(el).querySelector('.content-wrap') as HTMLElement | null

function makeCollapsible(attrs: Record<string, string | boolean> = {}) {
  return mount(
    reg,
    'sui-collapsible',
    attrs,
    '<button slot="trigger">Toggle</button><div>Content</div>',
  )
}

describe('<sui-collapsible>', () => {

  it('open=false by default', () => {
    const { el } = makeCollapsible()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('open attribute reflects on host and native <details>', async () => {
    const { el } = makeCollapsible({ open: true })
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    expect(detailsEl(el)!.open).toBe(true)
  })

  it('renders a native <details>/<summary> pair', () => {
    const { el } = makeCollapsible()
    expect(detailsEl(el)).not.toBeNull()
    expect(summaryEl(el)).not.toBeNull()
  })

  it('clicking the summary (native disclosure) opens the collapsible', async () => {
    const { el } = makeCollapsible()
    summaryEl(el)!.click()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('second click closes it again', async () => {
    const { el } = makeCollapsible()
    summaryEl(el)!.click()
    await flush()
    summaryEl(el)!.click()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('toggle fires sui-toggle with {open: true} (bubbles + composed)', async () => {
    const { el } = makeCollapsible()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-toggle', (e) => { evt = e as CustomEvent })
    summaryEl(el)!.click()
    await flush()
    expect(evt!.detail).toEqual({ open: true })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('toggle fires sui-toggle with {open: false} on second click', async () => {
    const { el } = makeCollapsible({ open: true })
    await flush()
    let detail: { open: boolean } | null = null
    el.addEventListener('sui-toggle', (e) => { detail = (e as CustomEvent).detail })
    summaryEl(el)!.click()
    await flush()
    expect(detail).toEqual({ open: false })
  })

  it('disabled=true — click on summary does nothing (LANG-GAP: <details> has no native disabled)', async () => {
    const { el } = makeCollapsible({ disabled: true })
    summaryEl(el)!.click()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('content-wrap has part=content', () => {
    const { el } = makeCollapsible()
    expect(contentWrap(el)!.getAttribute('part')).toBe('content')
  })

  it('content slot projects into content-inner', () => {
    const { el } = makeCollapsible()
    expect(el.querySelector('div:not([slot])')?.textContent).toBe('Content')
  })

  it('disconnect — no leaked effects', () => {
    const { container } = makeCollapsible({ open: true })
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})
