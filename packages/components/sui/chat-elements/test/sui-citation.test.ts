// Behavioral suite for <sui-citation>. AOT registration on import.
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-citation/sui-citation.compiled.js'

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
  return c.querySelector('sui-citation')!
}
// The pill is the public interactive surface — the component's only <button>.
const pill = (el: HTMLElement): HTMLButtonElement =>
  el.querySelector<HTMLButtonElement>('button')!

describe('<sui-citation>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sui-citation')).toBeDefined()
  })

  it('renders the source label + numbered index for the reader', () => {
    const el = mount('<sui-citation source="docs/ship.md" index="2"></sui-citation>')
    expect(el.textContent).toContain('docs/ship.md')
    expect(el.textContent).toContain('2')
  })

  it('exposes an accessible label naming the source + index', () => {
    const el = mount('<sui-citation source="docs/ship.md" index="2"></sui-citation>')
    const label = pill(el).getAttribute('aria-label')!
    expect(label).toContain('docs/ship.md')
    expect(label).toContain('2')
  })

  it('emits sui-citation-open with { source, index } on click', async () => {
    const el = mount('<sui-citation source="docs/ship.md" index="2"></sui-citation>')
    await flush()
    let detail: { source: string; index: number } | null = null
    el.addEventListener('sui-citation-open', (e) => {
      detail = (e as CustomEvent).detail
    })
    pill(el).click()
    expect(detail).toEqual({ source: 'docs/ship.md', index: 2 })
  })

  it('the open event reaches the document (bubbles + composed)', async () => {
    const el = mount('<sui-citation source="s" index="1"></sui-citation>')
    await flush()
    let seenAtDocument = false
    document.addEventListener('sui-citation-open', () => { seenAtDocument = true }, {
      once: true,
    })
    pill(el).click()
    expect(seenAtDocument).toBe(true)
  })
})
