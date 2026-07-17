// Behavioral suite for <sail-message>. AOT registration on import.
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-message/sail-message.compiled.js'

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
  return c.querySelector('sail-message')!
}
// Light DOM, no class hooks: the anatomy IS the semantics. The bubble is the
// box that carries the default slot — the turn's projection contract.
const bubble = (el: HTMLElement): HTMLElement =>
  el.querySelector<HTMLSlotElement>('slot:not([name])')!.parentElement!

describe('<sail-message>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sail-message')).toBeDefined()
  })

  it('reflects role / content-type / status onto the host for attribute-driven CSS', () => {
    const el = mount(
      '<sail-message role="user" content-type="code" status="error">hi</sail-message>',
    )
    expect(el.getAttribute('role')).toBe('user')
    expect(el.getAttribute('content-type')).toBe('code')
    expect(el.getAttribute('status')).toBe('error')
  })

  it('projects the turn content into the bubble', () => {
    const el = mount('<sail-message>hello there</sail-message>')
    // The bubble carries the content slot; the consumer text renders within it.
    expect(bubble(el)).not.toBeNull()
    expect(bubble(el).textContent).toContain('hello there')
  })

  it('surfaces the role label + timestamp in the meta row', () => {
    const el = mount(
      '<sail-message role="assistant" timestamp="2024-01-01T00:00:00Z">x</sail-message>',
    )
    expect(el.textContent).toContain('assistant')
    const time = el.querySelector<HTMLTimeElement>('time')!
    expect(time.getAttribute('datetime')).toBe('2024-01-01T00:00:00Z')
  })

  it('omits the timestamp element when no timestamp is supplied', () => {
    const el = mount('<sail-message>x</sail-message>')
    expect(el.querySelector('time')).toBeNull()
  })

  it('exposes a trailing slot for per-message actions', () => {
    const el = mount(
      '<sail-message>body<button slot="trailing">Retry</button></sail-message>',
    )
    expect(el.querySelector('slot[name="trailing"]')).not.toBeNull()
    expect(el.querySelector('[slot="trailing"]')!.textContent).toBe('Retry')
  })
})
