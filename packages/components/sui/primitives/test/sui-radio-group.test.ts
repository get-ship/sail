import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-radio-group/sui-radio.compiled.js'
import '../src/sui-radio-group/sui-radio-group.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// ── sui-radio ──────────────────────────────────────────────────────────────

describe('<sui-radio>', () => {

  it('renders a native radio input in shadow DOM', () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a' }, 'Option A')
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    expect(input).not.toBeNull()
    expect(input.type).toBe('radio')
  })

  it('checked attribute reflects on host and input', async () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a', checked: true }, 'A')
    expect(el.hasAttribute('checked')).toBe(true)
    await flush()
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    expect(input.checked).toBe(true)
  })

  it('unchecked radio has tabindex=-1', async () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a' }, 'A')
    await flush()
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    expect(input.getAttribute('tabindex')).toBe('-1')
  })

  it('checked radio has tabindex=0', async () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a', checked: true }, 'A')
    await flush()
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    expect(input.getAttribute('tabindex')).toBe('0')
  })

  it('disabled reflects on host and native input', () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a', disabled: true }, 'A')
    expect(el.hasAttribute('disabled')).toBe(true)
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    expect(input.disabled).toBe(true)
  })

  it('click dispatches sui-radio-select (bubbles + composed) with {value}', async () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a' }, 'A')
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-radio-select', (e) => { evt = e as CustomEvent })
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    input.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
    expect(evt!.detail).toEqual({ value: 'a' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('disabled radio does not dispatch sui-radio-select on click', async () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a', disabled: true }, 'A')
    await flush()
    let fired = false
    el.addEventListener('sui-radio-select', () => { fired = true })
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    input.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
    expect(fired).toBe(false)
  })

  it('ArrowDown dispatches sui-radio-nav with direction=next (bubbles + composed)', async () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a', checked: true }, 'A')
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-radio-nav', (e) => { evt = e as CustomEvent })
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, composed: true }))
    expect(evt!.detail).toEqual({ direction: 'next' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('ArrowUp dispatches sui-radio-nav with direction=prev', async () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a', checked: true }, 'A')
    await flush()
    let detail: { direction: string } | null = null
    el.addEventListener('sui-radio-nav', (e) => { detail = (e as CustomEvent).detail })
    const input = el.shadowRoot!.querySelector('input') as HTMLInputElement
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, composed: true }))
    expect(detail).toEqual({ direction: 'prev' })
  })

  it('parts: circle, dot, input present', () => {
    const { el } = mount(reg, 'sui-radio', { value: 'a' }, 'A')
    expect(el.shadowRoot!.querySelector('[part~="circle"]')).not.toBeNull()
    expect(el.shadowRoot!.querySelector('[part~="dot"]')).not.toBeNull()
    expect(el.shadowRoot!.querySelector('[part~="input"]')).not.toBeNull()
  })
})

// ── sui-radio-group ────────────────────────────────────────────────────────

describe('<sui-radio-group>', () => {
  const mountGroup = (attrs = '', radios = '') => {
    const container = reg.mk()
    container.innerHTML = `
      <sui-radio-group ${attrs}>
        ${radios || `
          <sui-radio value="a">Option A</sui-radio>
          <sui-radio value="b">Option B</sui-radio>
          <sui-radio value="c" disabled>Option C</sui-radio>
        `}
      </sui-radio-group>
    `
    return container.querySelector('sui-radio-group') as HTMLElement
  }


  it('sets role=radiogroup on host', async () => {
    const el = mountGroup()
    await flush()
    expect(el.getAttribute('role')).toBe('radiogroup')
  })

  it('selects radio matching initial value attribute', async () => {
    const el = mountGroup('value="b"')
    await flush()
    const radios = el.querySelectorAll('sui-radio')
    expect((radios[0] as HTMLElement).hasAttribute('checked')).toBe(false)
    expect((radios[1] as HTMLElement).hasAttribute('checked')).toBe(true)
  })

  it('clicking a radio updates value and emits sui-change (bubbles + composed)', async () => {
    const el = mountGroup()
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => { evt = e as CustomEvent })

    const radioA = el.querySelector('sui-radio[value="a"]') as HTMLElement
    const input = radioA.shadowRoot!.querySelector('input') as HTMLInputElement
    input.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
    await flush()

    expect(el.getAttribute('value')).toBe('a')
    expect(radioA.hasAttribute('checked')).toBe(true)
    expect(evt!.detail).toEqual({ value: 'a' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('arrow key navigates to next radio and emits sui-change', async () => {
    const el = mountGroup('value="a"')
    await flush()
    let detail: { value: string } | null = null
    el.addEventListener('sui-change', (e) => { detail = (e as CustomEvent).detail })

    const radioA = el.querySelector('sui-radio[value="a"]') as HTMLElement
    const input = radioA.shadowRoot!.querySelector('input') as HTMLInputElement
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, composed: true }))
    await flush()

    expect(el.getAttribute('value')).toBe('b')
    expect(detail).toEqual({ value: 'b' })
  })

  it('navigation skips disabled radio', async () => {
    const el = mountGroup('value="b"')
    await flush()
    const radioB = el.querySelector('sui-radio[value="b"]') as HTMLElement
    const input = radioB.shadowRoot!.querySelector('input') as HTMLInputElement
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, composed: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('a')
  })

  it('group disabled propagates to radios', async () => {
    const el = mountGroup('disabled value="a"')
    await flush()
    const radioB = el.querySelector('sui-radio[value="b"]') as HTMLElement
    expect(radioB.hasAttribute('disabled')).toBe(true)
  })

  it('form participation: FormData contains name=value after selection', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <form id="f">
        <sui-radio-group name="color" value="red">
          <sui-radio value="red">Red</sui-radio>
          <sui-radio value="blue">Blue</sui-radio>
        </sui-radio-group>
      </form>
    `
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    expect(data.get('color')).toBe('red')
  })

  it('form participation: FormData updates after sui-change', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <form id="f2">
        <sui-radio-group name="color" value="red">
          <sui-radio value="red">Red</sui-radio>
          <sui-radio value="blue">Blue</sui-radio>
        </sui-radio-group>
      </form>
    `
    const form = container.querySelector('form') as HTMLFormElement
    const group = container.querySelector('sui-radio-group') as HTMLElement
    await flush()

    const radioBlue = group.querySelector('sui-radio[value="blue"]') as HTMLElement
    const input = radioBlue.shadowRoot!.querySelector('input') as HTMLInputElement
    input.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
    await flush()

    const data = new FormData(form)
    expect(data.get('color')).toBe('blue')
  })

  it('label prop sets aria-label on host', async () => {
    const el = mountGroup('label="Colour"')
    await flush()
    expect(el.getAttribute('aria-label')).toBe('Colour')
  })

  it('tone propagates to child radios', async () => {
    const el = mountGroup('tone="success"')
    await flush()
    const radioA = el.querySelector('sui-radio[value="a"]') as HTMLElement
    expect(radioA.getAttribute('tone')).toBe('success')
  })

  it('size propagates to child radios', async () => {
    const el = mountGroup('size="sm"')
    await flush()
    const radioA = el.querySelector('sui-radio[value="a"]') as HTMLElement
    expect(radioA.getAttribute('size')).toBe('sm')
  })
})
