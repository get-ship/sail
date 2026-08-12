// <sail-component-viewer> — declarative chrome only. No DOM fishing.
// Specimens are host-owned; variant chips report up via onVariantChange.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-component-viewer/sail-component-viewer.compiled.js'

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
  container.innerHTML = `<sail-component-viewer ${attrHtml}>${inner}</sail-component-viewer>`
  return container.querySelector('sail-component-viewer') as HTMLElement
}

describe('<sail-component-viewer>', () => {
  it('registers and renders the name + stage layout', async () => {
    const el = mount({ name: 'badge', layout: 'stage' }, '<badge slot="stage">Live</badge>')
    await flush()
    expect(customElements.get('sail-component-viewer')).toBeDefined()
    expect(el.querySelector('h2')!.textContent).toBe('badge')
    expect(el.querySelector('[slot="stage"]')).not.toBeNull()
  })

  it('split layout exposes specimen + knobs slots', async () => {
    const el = mount(
      { name: 'button', layout: 'split' },
      '<div slot="specimen">S</div><div slot="knobs">K</div>',
    )
    await flush()
    expect(el.querySelector('[slot="specimen"]')?.textContent).toBe('S')
    expect(el.querySelector('[slot="knobs"]')?.textContent).toBe('K')
    expect(el.querySelector('[slot="stage"]')).toBeNull()
  })

  it('the specimen is the host-supplied component, not a re-render', async () => {
    const el = mount(
      { name: 'badge', layout: 'stage' },
      '<badge slot="stage" id="specimen">Live</badge>',
    )
    await flush()
    const specimen = el.querySelector('#specimen')
    expect(specimen).not.toBeNull()
    expect(specimen!.tagName.toLowerCase()).toBe('badge')
  })

  it('variant chips report via onVariantChange — no attribute stamping', async () => {
    const el = mount(
      { name: 'badge', layout: 'stage', 'variant-attr': 'variant' },
      '<badge slot="stage" id="specimen">Live</badge>',
    )
    const seen: string[] = []
    ;(el as unknown as { variants: string[]; onVariantChange: (d: { variant: string }) => void }).variants =
      ['solid', 'soft', 'outline']
    ;(el as unknown as { onVariantChange: (d: { variant: string }) => void }).onVariantChange = (d) => {
      seen.push(d.variant)
    }
    await flush()
    const chips = [...el.querySelectorAll('button')].filter((b) =>
      ['solid', 'soft', 'outline'].includes(b.textContent?.trim() ?? ''),
    ) as HTMLElement[]
    expect(chips.map((c) => c.textContent?.trim())).toEqual(['solid', 'soft', 'outline'])

    // Viewer does not write onto the slotted specimen.
    const specimen = el.querySelector('#specimen')!
    expect(specimen.getAttribute('variant')).toBeNull()

    chips[2]!.click()
    await flush()
    expect(seen).toContain('outline')
    expect(specimen.getAttribute('variant')).toBeNull()
  })

  it('renders the generated prop rows', async () => {
    const el = mount({ name: 'badge', layout: 'stage' }, '<badge slot="stage">Live</badge>')
    ;(el as unknown as { rows: Array<{ name: string; type: string; desc: string }> }).rows = [
      { name: 'tone', type: 'default · primary', desc: 'The tone channel.' },
      { name: 'variant', type: 'solid · soft', desc: '' },
    ]
    await flush()
    const codes = [...el.querySelectorAll('code')].map((c) => c.textContent)
    expect(codes).toEqual(['tone', 'variant'])
  })

  it('no variants, no props, no usage — the stage still renders', async () => {
    const el = mount({ name: 'icon', layout: 'stage' }, '<icon slot="stage" name="check"></icon>')
    await flush()
    const chipLabels = [...el.querySelectorAll('button')].map((b) => b.textContent?.trim())
    expect(chipLabels.every((t) => !['solid', 'soft', 'outline'].includes(t ?? ''))).toBe(true)
    expect(el.querySelector('code')).toBeNull()
    expect(el.querySelector('code-block')).toBeNull()
    expect(el.querySelector('[slot="stage"]')).not.toBeNull()
  })
})
