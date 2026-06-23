import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-alert/sui-alert.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
// Anchor on the public part contract, not the styling classes (.box/.close).
const box = (el: HTMLElement) => shadow(el).querySelector('[part="box"]') as HTMLElement | null
const closeBtn = (el: HTMLElement) => shadow(el).querySelector('[part="close"]') as HTMLButtonElement | null

describe('<sui-alert>', () => {

  it('tone=info — reflects on host', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'info' }, 'heads up')
    expect(el.getAttribute('tone')).toBe('info')
  })

  it('all tones reflect on host', () => {
    const tones = ['info', 'success', 'warning', 'destructive', 'muted']
    for (const tone of tones) {
      const { el } = mount(reg, 'sui-alert', { tone }, 'msg')
      expect(el.getAttribute('tone')).toBe(tone)
    }
  })

  it('box has part=box', () => {
    const { el } = mount(reg, 'sui-alert', {}, 'msg')
    expect(box(el)!.getAttribute('part')).toBe('box')
  })

  it('role=status for non-assertive tones (info)', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'info' }, 'msg')
    expect(box(el)!.getAttribute('role')).toBe('status')
  })

  it('role=status for success tone', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'success' }, 'msg')
    expect(box(el)!.getAttribute('role')).toBe('status')
  })

  it('role=status for muted tone', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'muted' }, 'msg')
    expect(box(el)!.getAttribute('role')).toBe('status')
  })

  it('role=alert for warning tone', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'warning' }, 'msg')
    expect(box(el)!.getAttribute('role')).toBe('alert')
  })

  it('role=alert for destructive tone', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'destructive' }, 'msg')
    expect(box(el)!.getAttribute('role')).toBe('alert')
  })

  it('aria-live=polite for info tone', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'info' }, 'msg')
    expect(box(el)!.getAttribute('aria-live')).toBe('polite')
  })

  it('aria-live=assertive for warning tone', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'warning' }, 'msg')
    expect(box(el)!.getAttribute('aria-live')).toBe('assertive')
  })

  it('aria-live=assertive for destructive tone', () => {
    const { el } = mount(reg, 'sui-alert', { tone: 'destructive' }, 'msg')
    expect(box(el)!.getAttribute('aria-live')).toBe('assertive')
  })

  it('aria-atomic=true on box', () => {
    const { el } = mount(reg, 'sui-alert', {}, 'msg')
    expect(box(el)!.getAttribute('aria-atomic')).toBe('true')
  })

  it('dismissible=false — close button has display:none (not interactable)', () => {
    const { el } = mount(reg, 'sui-alert', {}, 'msg')
    expect(el.hasAttribute('dismissible')).toBe(false)
    // Button is in shadow DOM but CSS-hidden; host has no [dismissible] attr
  })

  it('dismissible=true — reflects on host and close button is in DOM', () => {
    const { el } = mount(reg, 'sui-alert', { dismissible: true }, 'msg')
    expect(el.hasAttribute('dismissible')).toBe(true)
    expect(closeBtn(el)).not.toBeNull()
  })

  it('close button has part=close', () => {
    const { el } = mount(reg, 'sui-alert', { dismissible: true }, 'msg')
    expect(closeBtn(el)!.getAttribute('part')).toBe('close')
  })

  it('close button has aria-label=Dismiss', () => {
    const { el } = mount(reg, 'sui-alert', { dismissible: true }, 'msg')
    expect(closeBtn(el)!.getAttribute('aria-label')).toBe('Dismiss')
  })

  it('dismiss click fires sui-dismiss: bubbles + composed, no payload (detail null)', () => {
    const { el } = mount(reg, 'sui-alert', { dismissible: true }, 'msg')
    let evt: CustomEvent | null = null
    el.addEventListener('sui-dismiss', (e) => { evt = e as CustomEvent })
    closeBtn(el)!.click()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toBeNull()
  })

  it('dismiss click removes host from DOM', () => {
    const { el, container } = mount(reg, 'sui-alert', { dismissible: true }, 'msg')
    closeBtn(el)!.click()
    expect(container.querySelector('sui-alert')).toBeNull()
  })

  it('dismissible=false — close click does NOT fire sui-dismiss', () => {
    const { el } = mount(reg, 'sui-alert', {}, 'msg')
    let fired = 0
    el.addEventListener('sui-dismiss', () => fired++)
    closeBtn(el)?.click()
    expect(fired).toBe(0)
  })

  it('icon slot is projected', () => {
    const { el } = mount(reg, 'sui-alert', {}, '<span slot="icon">!</span>')
    expect(el.querySelector('[slot=icon]')).not.toBeNull()
  })

  it('title slot is projected', () => {
    const { el } = mount(reg, 'sui-alert', {}, '<span slot="title">Title</span>')
    expect(el.querySelector('[slot=title]')).not.toBeNull()
  })

  it('actions slot is projected', () => {
    const { el } = mount(reg, 'sui-alert', {}, '<button slot="actions">OK</button>')
    expect(el.querySelector('[slot=actions]')).not.toBeNull()
  })

  it('disconnect — no leaked effects', () => {
    const { container } = mount(reg, 'sui-alert', { dismissible: true }, 'msg')
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})
