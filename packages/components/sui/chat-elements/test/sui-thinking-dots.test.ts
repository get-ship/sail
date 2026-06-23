// Behavioral suite for <sui-thinking-dots>. AOT registration: importing the
// compiled output calls customElements.define() at module load.
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-thinking-dots/sui-thinking-dots.compiled.js'

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
const flush = () => new Promise((r) => setTimeout(r, 0))

const mount = (markup: string): HTMLElement => {
  const c = mk()
  c.innerHTML = markup
  return c.querySelector('sui-thinking-dots')!
}
// The live region is the announced status; anchor on the part/role contract.
const region = (el: HTMLElement): HTMLElement =>
  el.shadowRoot!.querySelector<HTMLElement>('[part="dots"]')!
const dots = (el: HTMLElement): NodeListOf<HTMLElement> =>
  el.shadowRoot!.querySelectorAll<HTMLElement>('[part="dot"]')

describe('<sui-thinking-dots>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sui-thinking-dots')).toBeDefined()
  })

  it('announces a polite live status with a default accessible label', () => {
    const el = mount('<sui-thinking-dots></sui-thinking-dots>')
    const r = region(el)
    expect(r.getAttribute('role')).toBe('status')
    expect(r.getAttribute('aria-live')).toBe('polite')
    expect(r.getAttribute('aria-label')).toBe('Assistant is thinking')
  })

  it('lets the consumer override the announced label', () => {
    const el = mount('<sui-thinking-dots label="Running tool…"></sui-thinking-dots>')
    expect(region(el).getAttribute('aria-label')).toBe('Running tool…')
  })

  it('keeps the three dots decorative (aria-hidden), not announced individually', () => {
    const el = mount('<sui-thinking-dots></sui-thinking-dots>')
    const d = dots(el)
    expect(d.length).toBe(3)
    for (const dot of d) expect(dot.getAttribute('aria-hidden')).toBe('true')
  })

  it('reflects tone + size onto the host so CSS can scale/color the dots', () => {
    const el = mount('<sui-thinking-dots tone="primary" size="lg"></sui-thinking-dots>')
    expect(el.getAttribute('tone')).toBe('primary')
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('mounts and unmounts cleanly across the stream lifecycle', async () => {
    const c = mk()
    c.innerHTML = '<sui-thinking-dots></sui-thinking-dots>'
    await flush()
    expect(c.querySelector('sui-thinking-dots')).not.toBeNull()
    c.querySelector('sui-thinking-dots')!.remove()
    await flush()
    expect(c.querySelector('sui-thinking-dots')).toBeNull()
  })
})
