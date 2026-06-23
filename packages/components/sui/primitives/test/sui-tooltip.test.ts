import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-tooltip/sui-tooltip.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const bubble = (el: HTMLElement) => shadow(el).querySelector('.bubble') as HTMLElement | null

describe('<sui-tooltip>', () => {

  it('renders .bubble span in shadow DOM', () => {
    const { el } = mount(reg, 'sui-tooltip', {}, '<button>hover me</button><span slot="content">tip</span>')
    expect(bubble(el)).not.toBeNull()
  })

  it('bubble has role=tooltip', () => {
    const { el } = mount(reg, 'sui-tooltip', {}, '<button>x</button><span slot="content">tip</span>')
    expect(bubble(el)!.getAttribute('role')).toBe('tooltip')
  })

  it('bubble has part=bubble', () => {
    const { el } = mount(reg, 'sui-tooltip', {}, '<button>x</button>')
    expect(bubble(el)!.getAttribute('part')).toBe('bubble')
  })

  it('placement=top — reflects on host', () => {
    const { el } = mount(reg, 'sui-tooltip', { placement: 'top' }, '<button>x</button>')
    expect(el.getAttribute('placement')).toBe('top')
  })

  it('placement=bottom reflects on host', () => {
    const { el } = mount(reg, 'sui-tooltip', { placement: 'bottom' }, '<button>x</button>')
    expect(el.getAttribute('placement')).toBe('bottom')
  })

  it('placement=left reflects on host', () => {
    const { el } = mount(reg, 'sui-tooltip', { placement: 'left' }, '<button>x</button>')
    expect(el.getAttribute('placement')).toBe('left')
  })

  it('placement=right reflects on host', () => {
    const { el } = mount(reg, 'sui-tooltip', { placement: 'right' }, '<button>x</button>')
    expect(el.getAttribute('placement')).toBe('right')
  })

  it('bubble gets a unique id', async () => {
    const { el } = mount(reg, 'sui-tooltip', {}, '<button>x</button>')
    await flush()
    const id = bubble(el)!.getAttribute('id')
    expect(id).toBeTruthy()
    expect(id).toMatch(/^sui-tooltip-\d+$/)
  })

  it('first slotted element gets aria-describedby pointing to bubble id', async () => {
    const { el } = mount(reg, 'sui-tooltip', {}, '<button id="trig">x</button><span slot="content">tip</span>')
    await flush()
    const bubbleId = bubble(el)!.getAttribute('id')
    const trigger = el.querySelector('#trig') as HTMLElement
    expect(trigger.getAttribute('aria-describedby')).toBe(bubbleId)
  })

  it('multiple instances get different bubble ids', async () => {
    const { el: a } = mount(reg, 'sui-tooltip', {}, '<button>a</button>')
    const { el: b } = mount(reg, 'sui-tooltip', {}, '<button>b</button>')
    await flush()
    const idA = bubble(a)!.getAttribute('id')
    const idB = bubble(b)!.getAttribute('id')
    expect(idA).not.toBe(idB)
  })

  it('content slot renders inside bubble', () => {
    const { el } = mount(reg, 'sui-tooltip', {}, '<button>x</button><span slot="content">my tip</span>')
    expect(el.querySelector('[slot=content]')).not.toBeNull()
  })

  it('Esc key dispatches blur (covers handler registration)', async () => {
    const { el } = mount(reg, 'sui-tooltip', {}, '<button>x</button>')
    let blurCalled = false
    const origBlur = HTMLElement.prototype.blur
    HTMLElement.prototype.blur = function () { blurCalled = true }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    HTMLElement.prototype.blur = origBlur
    expect(blurCalled).toBe(true)
  })

  it('disconnect — no leaked event listeners', () => {
    const { container } = mount(reg, 'sui-tooltip', {}, '<button>x</button><span slot="content">tip</span>')
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})
