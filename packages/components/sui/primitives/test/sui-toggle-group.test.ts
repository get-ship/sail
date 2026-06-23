import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-toggle-group/sui-toggle.compiled.js'
import '../src/sui-toggle-group/sui-toggle-group.compiled.js'
import { createContainerRegistry, flush } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// ── sui-toggle ─────────────────────────────────────────────────────────────

describe('<sui-toggle>', () => {

  it('renders a button in shadow DOM', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle value="bold">B</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    const btn = el.shadowRoot!.querySelector('button')
    expect(btn).not.toBeNull()
  })

  it('aria-pressed is false by default', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle value="bold">B</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    expect(el.shadowRoot!.querySelector('button')!.getAttribute('aria-pressed')).toBe('false')
  })

  it('toggles pressed on click and emits sui-toggle (bubbles + composed)', async () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle value="bold">B</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-toggle', (e) => { evt = e as CustomEvent })
    el.shadowRoot!.querySelector('button')!.click()
    await flush()
    expect(el.hasAttribute('pressed')).toBe(true)
    expect(evt!.detail).toEqual({ pressed: true, value: 'bold' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('second click un-toggles pressed', async () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle value="bold" pressed>B</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    await flush()
    el.shadowRoot!.querySelector('button')!.click()
    await flush()
    expect(el.hasAttribute('pressed')).toBe(false)
  })

  it('disabled attribute disables button', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle disabled value="x">X</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    expect(el.shadowRoot!.querySelector('button')!.disabled).toBe(true)
  })

  it('disabled toggle does not dispatch sui-toggle on click', async () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle disabled value="x">X</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    await flush()
    let fired = false
    el.addEventListener('sui-toggle', () => { fired = true })
    el.shadowRoot!.querySelector('button')!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(fired).toBe(false)
  })

  it('size reflects on host', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle size="sm" value="x">X</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('variant reflects on host', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle variant="ghost" value="x">X</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    expect(el.getAttribute('variant')).toBe('ghost')
  })

  it('aria-pressed reflects pressed attribute', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle pressed value="x">X</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    expect(el.shadowRoot!.querySelector('button')!.getAttribute('aria-pressed')).toBe('true')
  })

  it('part="button" present on button', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-toggle value="x">X</sui-toggle>'
    const el = container.querySelector('sui-toggle') as HTMLElement
    expect(el.shadowRoot!.querySelector('[part~="button"]')).not.toBeNull()
  })
})

// ── sui-toggle-group ───────────────────────────────────────────────────────

describe('<sui-toggle-group>', () => {
  const mountGroup = (markup: string) => {
    const container = reg.mk()
    container.innerHTML = markup
    return container.querySelector('sui-toggle-group') as HTMLElement
  }


  it('sets role=group on host', async () => {
    const el = mountGroup(`
      <sui-toggle-group type="single">
        <sui-toggle value="a">A</sui-toggle>
      </sui-toggle-group>
    `)
    await flush()
    expect(el.getAttribute('role')).toBe('group')
  })

  it('single mode: selecting one deselects others and emits sui-change', async () => {
    const el = mountGroup(`
      <sui-toggle-group type="single">
        <sui-toggle value="a">A</sui-toggle>
        <sui-toggle value="b">B</sui-toggle>
      </sui-toggle-group>
    `)
    await flush()
    const toggles = el.querySelectorAll('sui-toggle') as NodeListOf<HTMLElement>
    let lastValue = ''
    let lastEvt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => {
      lastEvt = e as CustomEvent
      lastValue = (e as CustomEvent).detail.value
    })

    toggles[0].shadowRoot!.querySelector('button')!.click()
    await flush()
    expect(lastValue).toBe('a')
    expect(lastEvt!.detail).toEqual({ value: 'a' })
    expect(lastEvt!.bubbles).toBe(true)
    expect(lastEvt!.composed).toBe(true)
    expect(toggles[0].hasAttribute('pressed')).toBe(true)
    expect(toggles[1].hasAttribute('pressed')).toBe(false)

    toggles[1].shadowRoot!.querySelector('button')!.click()
    await flush()
    expect(lastValue).toBe('b')
    expect(toggles[0].hasAttribute('pressed')).toBe(false)
    expect(toggles[1].hasAttribute('pressed')).toBe(true)
  })

  it('single mode: clicking selected toggle deselects it', async () => {
    const el = mountGroup(`
      <sui-toggle-group type="single" value="a">
        <sui-toggle value="a">A</sui-toggle>
        <sui-toggle value="b">B</sui-toggle>
      </sui-toggle-group>
    `)
    await flush()
    const toggleA = el.querySelector('sui-toggle[value="a"]') as HTMLElement
    toggleA.shadowRoot!.querySelector('button')!.click()
    await flush()
    // Empty value reflects as absent attribute (encodeAttr returns null for '').
    expect(el.getAttribute('value') ?? '').toBe('')
    expect(toggleA.hasAttribute('pressed')).toBe(false)
  })

  it('multiple mode: allows multiple selections', async () => {
    const el = mountGroup(`
      <sui-toggle-group type="multiple">
        <sui-toggle value="a">A</sui-toggle>
        <sui-toggle value="b">B</sui-toggle>
      </sui-toggle-group>
    `)
    await flush()
    const toggles = el.querySelectorAll('sui-toggle') as NodeListOf<HTMLElement>
    toggles[0].shadowRoot!.querySelector('button')!.click()
    await flush()
    toggles[1].shadowRoot!.querySelector('button')!.click()
    await flush()
    expect(el.getAttribute('value')).toBe('a,b')
    expect(toggles[0].hasAttribute('pressed')).toBe(true)
    expect(toggles[1].hasAttribute('pressed')).toBe(true)
  })

  it('multiple mode: second click deselects', async () => {
    const el = mountGroup(`
      <sui-toggle-group type="multiple" value="a,b">
        <sui-toggle value="a">A</sui-toggle>
        <sui-toggle value="b">B</sui-toggle>
      </sui-toggle-group>
    `)
    await flush()
    const toggleA = el.querySelector('sui-toggle[value="a"]') as HTMLElement
    toggleA.shadowRoot!.querySelector('button')!.click()
    await flush()
    expect(el.getAttribute('value')).toBe('b')
  })

  it('initial value=a sets correct pressed state', async () => {
    const el = mountGroup(`
      <sui-toggle-group type="single" value="a">
        <sui-toggle value="a">A</sui-toggle>
        <sui-toggle value="b">B</sui-toggle>
      </sui-toggle-group>
    `)
    await flush()
    const toggles = el.querySelectorAll('sui-toggle') as NodeListOf<HTMLElement>
    expect(toggles[0].hasAttribute('pressed')).toBe(true)
    expect(toggles[1].hasAttribute('pressed')).toBe(false)
  })

  it('size propagates to child toggles', async () => {
    const el = mountGroup(`
      <sui-toggle-group size="sm">
        <sui-toggle value="a">A</sui-toggle>
      </sui-toggle-group>
    `)
    await flush()
    const toggle = el.querySelector('sui-toggle') as HTMLElement
    expect(toggle.getAttribute('size')).toBe('sm')
  })

  it('variant propagates to child toggles', async () => {
    const el = mountGroup(`
      <sui-toggle-group variant="ghost">
        <sui-toggle value="a">A</sui-toggle>
      </sui-toggle-group>
    `)
    await flush()
    const toggle = el.querySelector('sui-toggle') as HTMLElement
    expect(toggle.getAttribute('variant')).toBe('ghost')
  })

  it('form participation: FormData contains name=value', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <form>
        <sui-toggle-group name="fmt" type="multiple" value="bold,italic">
          <sui-toggle value="bold">B</sui-toggle>
          <sui-toggle value="italic">I</sui-toggle>
        </sui-toggle-group>
      </form>
    `
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    expect(data.get('fmt')).toBe('bold,italic')
  })

  it('form participation: FormData updates after toggle', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <form>
        <sui-toggle-group name="fmt" type="single">
          <sui-toggle value="bold">B</sui-toggle>
          <sui-toggle value="italic">I</sui-toggle>
        </sui-toggle-group>
      </form>
    `
    const form = container.querySelector('form') as HTMLFormElement
    const group = container.querySelector('sui-toggle-group') as HTMLElement
    await flush()
    const toggleBold = group.querySelector('sui-toggle[value="bold"]') as HTMLElement
    toggleBold.shadowRoot!.querySelector('button')!.click()
    await flush()
    const data = new FormData(form)
    expect(data.get('fmt')).toBe('bold')
  })
})
