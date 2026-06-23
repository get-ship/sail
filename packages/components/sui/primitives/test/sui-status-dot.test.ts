// First-shipped ported primitive — proves shadow-DOM components
// authored in `.html.set` register, instantiate, observe attrs, and
// reactively update their DOM.

import { afterEach, describe, expect, it } from 'vitest'
// AOT registration: importing the compiled output calls
// `customElements.define()` at module load.
import '../src/sui-status-dot/sui-status-dot.compiled.js'

const containers: HTMLElement[] = []
const mkContainer = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

describe('<sui-status-dot>', () => {
  it('registers the custom element with the global registry', () => {
    expect(customElements.get('sui-status-dot')).toBeDefined()
  })

  it('instantiates with a shadow root containing styles + the dot', () => {
    const container = mkContainer()
    container.innerHTML = '<sui-status-dot></sui-status-dot>'
    const el = container.querySelector('sui-status-dot')!
    // Shadow root is attached.
    expect(el.shadowRoot).not.toBeNull()
    // Styles are inside the shadow root.
    const style = el.shadowRoot!.querySelector('style')
    expect(style).not.toBeNull()
    expect(style!.textContent).toContain('.dot')
    // Rendered span lives in the shadow root.
    const dot = el.shadowRoot!.querySelector('[part="dot"]')
    expect(dot).not.toBeNull()
  })

  it('reflects tone / pulse / size attributes on the host', () => {
    const container = mkContainer()
    container.innerHTML = '<sui-status-dot tone="primary" pulse size="sm"></sui-status-dot>'
    const el = container.querySelector('sui-status-dot')!
    expect(el.getAttribute('tone')).toBe('primary')
    expect(el.hasAttribute('pulse')).toBe(true)
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('exposes role and aria-label when label is set; aria-hidden otherwise', () => {
    const container = mkContainer()
    container.innerHTML = '<sui-status-dot label="Online"></sui-status-dot>'
    const el = container.querySelector('sui-status-dot')!
    const dot = el.shadowRoot!.querySelector('[part="dot"]')!
    // Label set → role=img + aria-label=label, no aria-hidden.
    expect(dot.getAttribute('role')).toBe('img')
    expect(dot.getAttribute('aria-label')).toBe('Online')
    expect(dot.hasAttribute('aria-hidden')).toBe(false)

    // Default (no label) → aria-hidden=true.
    container.innerHTML = '<sui-status-dot></sui-status-dot>'
    const elBare = container.querySelector('sui-status-dot')!
    const dotBare = elBare.shadowRoot!.querySelector('[part="dot"]')!
    expect(dotBare.getAttribute('aria-hidden')).toBe('true')
    expect(dotBare.hasAttribute('role')).toBe(false)
  })

  it('updates aria-label reactively when the label attribute changes', async () => {
    const container = mkContainer()
    container.innerHTML = '<sui-status-dot label="One"></sui-status-dot>'
    const el = container.querySelector('sui-status-dot')!
    const dot = el.shadowRoot!.querySelector('[part="dot"]')!
    expect(dot.getAttribute('aria-label')).toBe('One')
    el.setAttribute('label', 'Two')
    // Allow the attributeChangedCallback → signal-set tick.
    await new Promise((r) => setTimeout(r, 0))
    expect(dot.getAttribute('aria-label')).toBe('Two')
  })
})
