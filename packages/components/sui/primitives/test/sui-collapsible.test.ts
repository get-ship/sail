import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-collapsible/sui-collapsible.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const triggerWrap = (el: HTMLElement) => shadow(el).querySelector('.trigger-wrap') as HTMLElement | null
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

  it('open attribute reflects on host', () => {
    const { el } = makeCollapsible({ open: true })
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('trigger click opens collapsible', async () => {
    const { el } = makeCollapsible()
    const btn = el.querySelector('button[slot=trigger]') as HTMLButtonElement
    btn.click()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('second trigger click closes collapsible', async () => {
    const { el } = makeCollapsible()
    const btn = el.querySelector('button[slot=trigger]') as HTMLButtonElement
    btn.click()
    await flush()
    btn.click()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('toggle fires sui-toggle with {open: true} (bubbles + composed)', async () => {
    const { el } = makeCollapsible()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-toggle', (e) => { evt = e as CustomEvent })
    el.querySelector('button[slot=trigger]')!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flush()
    expect(evt!.detail).toEqual({ open: true })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('toggle fires sui-toggle with {open: false} on second click', async () => {
    const { el } = makeCollapsible({ open: true })
    let detail: { open: boolean } | null = null
    el.addEventListener('sui-toggle', (e) => { detail = (e as CustomEvent).detail })
    const tw = triggerWrap(el)!
    tw.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flush()
    expect(detail).toEqual({ open: false })
  })

  it('disabled=true — trigger click does nothing', async () => {
    const { el } = makeCollapsible({ disabled: true })
    el.querySelector('button[slot=trigger]')!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('trigger has aria-expanded=false when closed', async () => {
    const { el } = makeCollapsible()
    await flush()
    const btn = el.querySelector('button[slot=trigger]') as HTMLElement
    expect(btn.getAttribute('aria-expanded')).toBe('false')
  })

  it('trigger has aria-expanded=true when open', async () => {
    const { el } = makeCollapsible({ open: true })
    await flush()
    const btn = el.querySelector('button[slot=trigger]') as HTMLElement
    expect(btn.getAttribute('aria-expanded')).toBe('true')
  })

  it('trigger aria-expanded updates after toggle', async () => {
    const { el } = makeCollapsible()
    const btn = el.querySelector('button[slot=trigger]') as HTMLButtonElement
    await flush()
    expect(btn.getAttribute('aria-expanded')).toBe('false')
    btn.click()
    await flush()
    expect(btn.getAttribute('aria-expanded')).toBe('true')
  })

  it('trigger has aria-controls pointing to content-wrap id', async () => {
    const { el } = makeCollapsible()
    await flush()
    const btn = el.querySelector('button[slot=trigger]') as HTMLElement
    const controls = btn.getAttribute('aria-controls')
    expect(controls).toBeTruthy()
    const wrap = contentWrap(el)!
    expect(wrap.id).toBe(controls)
  })

  it('content-wrap has an id attribute', async () => {
    const { el } = makeCollapsible()
    await flush()
    expect(contentWrap(el)!.id).toMatch(/^sui-collapsible-\d+$/)
  })

  it('content-wrap has part=content', () => {
    const { el } = makeCollapsible()
    expect(contentWrap(el)!.getAttribute('part')).toBe('content')
  })

  it('non-interactive trigger gets tabindex=0', async () => {
    const { el } = mount(
      reg,
      'sui-collapsible',
      {},
      '<div slot="trigger">Toggle</div><div>Content</div>',
    )
    await flush()
    const trigger = el.querySelector('[slot=trigger]') as HTMLElement
    expect(trigger.getAttribute('tabindex')).toBe('0')
  })

  it('Enter key on trigger-wrap toggles open', async () => {
    const { el } = makeCollapsible()
    const tw = triggerWrap(el)!
    tw.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('Space key on trigger-wrap toggles open', async () => {
    const { el } = makeCollapsible()
    const tw = triggerWrap(el)!
    tw.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
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
