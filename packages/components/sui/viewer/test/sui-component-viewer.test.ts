// <sui-component-viewer> — the shared viewer. The contract is that it drives a
// REAL slotted specimen: picking a variant writes the named attribute onto the
// component the host actually shipped, and the prop table comes from the
// generated registry.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-component-viewer/sui-component-viewer.compiled.js'

const containers: HTMLElement[] = []
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

const flush = (): Promise<void> =>
  new Promise((r) => setTimeout(() => queueMicrotask(() => r()), 0))

function mount(attrs: Record<string, string> = {}, inner = ''): HTMLElement {
  const container = document.createElement('div')
  document.body.appendChild(container)
  containers.push(container)
  const attrHtml = Object.entries(attrs)
    .map(([k, v]) => `${k}="${v}"`)
    .join(' ')
  container.innerHTML = `<sui-component-viewer ${attrHtml}>${inner}</sui-component-viewer>`
  return container.querySelector('sui-component-viewer') as HTMLElement
}

describe('<sui-component-viewer>', () => {
  it('registers and renders the name + stage', async () => {
    const el = mount({ name: 'badge' }, '<badge slot="stage">Live</badge>')
    await flush()
    expect(customElements.get('sui-component-viewer')).toBeDefined()
    expect(el.querySelector('.name')!.textContent).toBe('badge')
    expect(el.querySelector('.stage')).not.toBeNull()
  })

  it('the specimen is the host-supplied component, not a re-render', async () => {
    const el = mount({ name: 'badge' }, '<badge slot="stage" id="specimen">Live</badge>')
    await flush()
    const specimen = el.querySelector('#specimen')
    expect(specimen).not.toBeNull()
    expect(specimen!.tagName.toLowerCase()).toBe('badge')
  })

  it('renders a chip per variant and drives the real attribute', async () => {
    const el = mount(
      { name: 'badge', 'variant-attr': 'variant' },
      '<badge slot="stage">Live</badge>',
    )
    ;(el as unknown as { variants: string[] }).variants = ['solid', 'soft', 'outline']
    await flush()
    const chips = [...el.querySelectorAll('[data-variant]')] as HTMLElement[]
    expect(chips.map((c) => c.getAttribute('data-variant'))).toEqual(['solid', 'soft', 'outline'])

    // The first variant applies on mount…
    const specimen = el.querySelector('[slot="stage"]')!
    expect(specimen.getAttribute('variant')).toBe('solid')

    // …and picking another writes it onto the specimen itself.
    chips[2]!.click()
    await flush()
    expect(specimen.getAttribute('variant')).toBe('outline')
  })

  it('emits sui-variant-change (bubbles + composed)', async () => {
    const el = mount(
      { name: 'badge', 'variant-attr': 'variant' },
      '<badge slot="stage">Live</badge>',
    )
    ;(el as unknown as { variants: string[] }).variants = ['solid', 'soft']
    await flush()
    let seen = ''
    document.addEventListener('sui-variant-change', (e) => {
      seen = (e as CustomEvent<{ variant: string }>).detail.variant
    })
    ;(el.querySelector('[data-variant="soft"]') as HTMLElement).click()
    await flush()
    expect(seen).toBe('soft')
  })

  it('renders the generated prop rows', async () => {
    const el = mount({ name: 'badge' }, '<badge slot="stage">Live</badge>')
    ;(el as unknown as { rows: Array<{ name: string; type: string; desc: string }> }).rows = [
      { name: 'tone', type: 'default · primary', desc: 'The tone channel.' },
      { name: 'variant', type: 'solid · soft', desc: '' },
    ]
    await flush()
    const codes = [...el.querySelectorAll('.props code')].map((c) => c.textContent)
    expect(codes).toEqual(['tone', 'variant'])
  })

  it('no variants, no props, no usage — the stage still renders', async () => {
    const el = mount({ name: 'icon' }, '<icon slot="stage" name="check"></icon>')
    await flush()
    expect(el.querySelector('[data-variant]')).toBeNull()
    expect(el.querySelector('.props')).toBeNull()
    expect(el.querySelector('.usage')).toBeNull()
    expect(el.querySelector('.stage')).not.toBeNull()
  })
})
