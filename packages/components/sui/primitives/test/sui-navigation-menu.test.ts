import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-navigation-menu/sui-navigation-item.compiled.js'
import '../src/sui-navigation-menu/sui-navigation-menu.compiled.js'
import { createContainerRegistry, flush } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// ── sui-navigation-item ────────────────────────────────────────────────────

describe('<sui-navigation-item>', () => {

  it('renders a button by default (no href)', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="home">Home</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    expect(el.shadowRoot!.querySelector('button')).not.toBeNull()
    expect(el.shadowRoot!.querySelector('a')).toBeNull()
  })

  it('button aria-current is "false" by default', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="home">Home</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    expect(el.shadowRoot!.querySelector('button')!.getAttribute('aria-current')).toBe('false')
  })

  it('renders an anchor when href is set', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="docs" href="/docs">Docs</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    expect(el.shadowRoot!.querySelector('a')).not.toBeNull()
    expect(el.shadowRoot!.querySelector('a')!.getAttribute('href')).toBe('/docs')
    expect(el.shadowRoot!.querySelector('button')).toBeNull()
  })

  it('active attribute sets aria-current="true" on button', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="x" active>X</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    expect(el.hasAttribute('active')).toBe(true)
    expect(el.shadowRoot!.querySelector('button')!.getAttribute('aria-current')).toBe('true')
  })

  it('active attribute sets aria-current="page" on anchor', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="x" href="/x" active>X</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    expect(el.shadowRoot!.querySelector('a')!.getAttribute('aria-current')).toBe('page')
  })

  it('disabled attribute disables the button natively', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="x" disabled>X</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    expect(el.shadowRoot!.querySelector('button')!.disabled).toBe(true)
  })

  it('click dispatches sui-nav-select with value (bubbles + composed)', async () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="home">Home</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-nav-select', (e) => { evt = e as CustomEvent })
    el.shadowRoot!.querySelector('button')!.click()
    expect(evt!.detail).toEqual({ value: 'home' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('disabled item does not dispatch sui-nav-select on click', async () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="x" disabled>X</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    await flush()
    let fired = false
    el.addEventListener('sui-nav-select', () => { fired = true })
    el.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
    expect(fired).toBe(false)
  })

  it('part="link" is present on button element', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="x">X</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    expect(el.shadowRoot!.querySelector('[part~="link"]')).not.toBeNull()
  })

  it('part="indicator" is present', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-navigation-item value="x">X</sui-navigation-item>'
    const el = container.querySelector('sui-navigation-item') as HTMLElement
    expect(el.shadowRoot!.querySelector('[part~="indicator"]')).not.toBeNull()
  })
})

// ── sui-navigation-menu ────────────────────────────────────────────────────

describe('<sui-navigation-menu>', () => {
  const mountMenu = (markup: string) => {
    const container = reg.mk()
    container.innerHTML = markup
    return container.querySelector('sui-navigation-menu') as HTMLElement
  }


  it('sets role="navigation" on host', async () => {
    const el = mountMenu(`
      <sui-navigation-menu value="a">
        <sui-navigation-item value="a">A</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    expect(el.getAttribute('role')).toBe('navigation')
  })

  it('syncs active state to items from initial value', async () => {
    const el = mountMenu(`
      <sui-navigation-menu value="b">
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b">B</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    const items = el.querySelectorAll('sui-navigation-item') as NodeListOf<HTMLElement>
    expect(items[0].hasAttribute('active')).toBe(false)
    expect(items[1].hasAttribute('active')).toBe(true)
  })

  it('fires sui-change and updates value on item click', async () => {
    const el = mountMenu(`
      <sui-navigation-menu value="a">
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b">B</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    const items = el.querySelectorAll('sui-navigation-item') as NodeListOf<HTMLElement>
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => { evt = e as CustomEvent })
    items[1].shadowRoot!.querySelector('button')!.click()
    await flush()
    expect(evt!.detail).toEqual({ value: 'b' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(el.getAttribute('value')).toBe('b')
    expect(items[0].hasAttribute('active')).toBe(false)
    expect(items[1].hasAttribute('active')).toBe(true)
  })

  it('ArrowRight moves focus to next item', async () => {
    const el = mountMenu(`
      <sui-navigation-menu>
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b">B</sui-navigation-item>
        <sui-navigation-item value="c">C</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    const items = el.querySelectorAll('sui-navigation-item') as NodeListOf<HTMLElement>
    items[0].dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(items[1])
  })

  it('ArrowLeft moves focus to previous item', async () => {
    const el = mountMenu(`
      <sui-navigation-menu>
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b">B</sui-navigation-item>
        <sui-navigation-item value="c">C</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    const items = el.querySelectorAll('sui-navigation-item') as NodeListOf<HTMLElement>
    items[2].dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(items[1])
  })

  it('Home moves focus to first item', async () => {
    const el = mountMenu(`
      <sui-navigation-menu>
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b">B</sui-navigation-item>
        <sui-navigation-item value="c">C</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    const items = el.querySelectorAll('sui-navigation-item') as NodeListOf<HTMLElement>
    items[2].dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(items[0])
  })

  it('End moves focus to last item', async () => {
    const el = mountMenu(`
      <sui-navigation-menu>
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b">B</sui-navigation-item>
        <sui-navigation-item value="c">C</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    const items = el.querySelectorAll('sui-navigation-item') as NodeListOf<HTMLElement>
    items[0].dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(items[2])
  })

  it('ArrowRight wraps from last to first item', async () => {
    const el = mountMenu(`
      <sui-navigation-menu>
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b">B</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    const items = el.querySelectorAll('sui-navigation-item') as NodeListOf<HTMLElement>
    items[1].dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(items[0])
  })

  it('disabled items are skipped during keyboard navigation', async () => {
    const el = mountMenu(`
      <sui-navigation-menu>
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b" disabled>B</sui-navigation-item>
        <sui-navigation-item value="c">C</sui-navigation-item>
      </sui-navigation-menu>
    `)
    await flush()
    const items = el.querySelectorAll('sui-navigation-item') as NodeListOf<HTMLElement>
    items[0].dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(items[2])
  })

  it('slot projects items into the parent', () => {
    const el = mountMenu(`
      <sui-navigation-menu>
        <sui-navigation-item value="a">A</sui-navigation-item>
        <sui-navigation-item value="b">B</sui-navigation-item>
      </sui-navigation-menu>
    `)
    const items = el.querySelectorAll(':scope > sui-navigation-item')
    expect(items.length).toBe(2)
  })
})
