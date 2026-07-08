// P1.4: <sui-nav-top> — top application bar.
//
// A pure layout primitive: the bar chrome (height, border, background) and
// three named slots (`brand`, default, `actions`). No state. The slotted
// children own their own state (theme, account, etc.) — wiring that is the
// consumer's job. The host fires `sui:nav-top:ready` once after first mount
// so the app can wire post-mount concerns.
//
// These tests cover the NEUTRAL API surface that ships with @ship/sui-primitives:
//   - 3 named slots: `brand`, default (no name), `actions`
//   - 1 boolean reflected prop: `sticky`
//   - 1 composed bubbling event: `sui:nav-top:ready`
//   - 3 parts: `brand`, `primary`, `actions`
//   - 0 SDK imports; 0 product-channel reads (see purity.test.ts).

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-nav-top/sui-nav-top.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-nav-top> — neutral layout primitive', () => {

  it('renders a shadow root with three slot wrappers (brand / primary / actions)', () => {
    const { el } = mount(reg, 'sui-nav-top')
    expect(el.shadowRoot).not.toBeNull()
    const root = shadow(el)
    expect(root.querySelector('[part~="brand"]')).not.toBeNull()
    expect(root.querySelector('[part~="primary"]')).not.toBeNull()
    expect(root.querySelector('[part~="actions"]')).not.toBeNull()
  })

  it('exposes a named `brand` slot, a default slot in the primary zone, and a named `actions` slot', () => {
    const { el } = mount(reg, 'sui-nav-top')
    const root = shadow(el)
    expect(root.querySelector('slot[name="brand"]')).not.toBeNull()
    // The default slot is the unnamed one — `querySelector('slot:not([name])')` matches it.
    const defaultSlot = root.querySelector('slot:not([name])')
    expect(defaultSlot).not.toBeNull()
    // The default slot lives INSIDE the primary part.
    expect(defaultSlot!.closest('[part~="primary"]')).not.toBeNull()
    expect(root.querySelector('slot[name="actions"]')).not.toBeNull()
  })

  it('projects slotted children through brand / default / actions zones', () => {
    const { el } = mount(
      reg,
      'sui-nav-top',
      {},
      '<a slot="brand">Home</a><span>middle</span><button slot="actions">Go</button>',
    )
    // The slotted children appear in light DOM (slotted, not cloned) and
    // show up in `el.textContent`.
    expect(el.textContent).toContain('Home')
    expect(el.textContent).toContain('middle')
    expect(el.textContent).toContain('Go')
  })

  it('reflects the `sticky` boolean prop onto the host attribute', async () => {
    const { el } = mount(reg, 'sui-nav-top', { sticky: true })
    await flush()
    expect(el.hasAttribute('sticky')).toBe(true)
  })

  it('does NOT emit `sticky` on the host when the prop is absent', () => {
    const { el } = mount(reg, 'sui-nav-top')
    expect(el.hasAttribute('sticky')).toBe(false)
  })

  it('fires `sui:nav-top:ready` once after first mount (bubbles, composed)', async () => {
    // The event is dispatched in `mount()`, which runs synchronously when
    // the host is connected — the listener MUST be in place before
    // the element is appended. We listen on the container (events bubble)
    // and only then write the innerHTML that appends the host.
    const container = reg.mk()
    let captured: CustomEvent | null = null
    container.addEventListener('sui:nav-top:ready', (e) => {
      captured = e as CustomEvent
    })
    container.innerHTML = '<sui-nav-top></sui-nav-top>'
    await flush()
    expect(captured, 'sui:nav-top:ready must fire once on first mount').not.toBeNull()
    expect(captured!.bubbles).toBe(true)
    expect(captured!.composed).toBe(true)
    expect(captured!.target).toBe(container.querySelector('sui-nav-top'))
  })

  it('ships ZERO product-coupled identifiers in the host markup (the anatomy is generic)', () => {
    // The `brand` slot is a generic layout slot name (left-aligned zone in
    // a top bar), not a product domain reference. Verify no other
    // product-domain nouns leak into the host markup or class names.
    const { el } = mount(
      reg,
      'sui-nav-top',
      {},
      '<a slot="brand">Home</a><span>middle</span><button slot="actions">Go</button>',
    )
    const html = el.outerHTML.toLowerCase()
    expect(html).not.toMatch(/account|project|billing/)
    // The internal class names live in shadow; the public parts are the
    // only consumer-visible identifiers, and they're `brand/primary/actions`.
    const root = shadow(el)
    const classNames = Array.from(root.querySelectorAll('[class]'))
      .map((n) => n.getAttribute('class') ?? '')
      .join(' ')
    expect(classNames).not.toMatch(/account|project|billing|ship-account|ship-project/)
  })
})
