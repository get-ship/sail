// Behavioral suite for <sui-message>. AOT registration on import.
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-message/sui-message.compiled.js'

const containers: HTMLElement[] = []
const mk = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

const mount = (markup: string): HTMLElement => {
  const c = mk()
  c.innerHTML = markup
  return c.querySelector('sui-message')!
}
const bubble = (el: HTMLElement): HTMLElement =>
  el.shadowRoot!.querySelector<HTMLElement>('[part="bubble"]')!

describe('<sui-message>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sui-message')).toBeDefined()
  })

  it('reflects role / content-type / status onto the host for attribute-driven CSS', () => {
    const el = mount(
      '<sui-message role="user" content-type="code" status="error">hi</sui-message>',
    )
    expect(el.getAttribute('role')).toBe('user')
    expect(el.getAttribute('content-type')).toBe('code')
    expect(el.getAttribute('status')).toBe('error')
  })

  it('projects the turn content into the bubble', () => {
    const el = mount('<sui-message>hello there</sui-message>')
    // The bubble carries the content slot; the consumer text renders within it.
    expect(bubble(el)).not.toBeNull()
    expect(el.textContent).toContain('hello there')
  })

  it('surfaces the role label + timestamp in the meta row', () => {
    const el = mount(
      '<sui-message role="assistant" timestamp="2024-01-01T00:00:00Z">x</sui-message>',
    )
    const meta = el.shadowRoot!.querySelector<HTMLElement>('[part="meta"]')!
    expect(meta.textContent).toContain('assistant')
    const time = el.shadowRoot!.querySelector<HTMLTimeElement>('time')!
    expect(time.getAttribute('datetime')).toBe('2024-01-01T00:00:00Z')
  })

  it('omits the timestamp element when no timestamp is supplied', () => {
    const el = mount('<sui-message>x</sui-message>')
    expect(el.shadowRoot!.querySelector('time')).toBeNull()
  })

  it('exposes a trailing slot for per-message actions', () => {
    const el = mount(
      '<sui-message>body<button slot="trailing">Retry</button></sui-message>',
    )
    expect(el.shadowRoot!.querySelector('slot[name="trailing"]')).not.toBeNull()
    expect(el.querySelector('[slot="trailing"]')!.textContent).toBe('Retry')
  })
})
