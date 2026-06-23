// Behavioral suite for <sui-tabs> + <sui-tab> + <sui-tab-panel>. Covers the
// WAI-ARIA tabs contract: tablist/tab/tabpanel roles, roving tabindex, arrow +
// Home/End keyboard navigation, aria-selected, panel visibility following
// selection, and the sui-tab-change event payload. Anchors on roles/aria/
// data-state and emitted events — never styling classes or pixels.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-tabs/sui-tabs.compiled.js'
import '../src/sui-tabs/sui-tab.compiled.js'
import '../src/sui-tab-panel/sui-tab-panel.compiled.js'
import { createContainerRegistry, flush } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

function makeTabs(active = 'one', extra = '') {
  const container = reg.mk()
  container.innerHTML = `
    <sui-tabs ${active ? `active="${active}"` : ''} ${extra}>
      <sui-tab name="one" label="One"></sui-tab>
      <sui-tab name="two" label="Two"></sui-tab>
      <sui-tab name="three" label="Three"></sui-tab>
      <sui-tab-panel name="one">first</sui-tab-panel>
      <sui-tab-panel name="two">second</sui-tab-panel>
      <sui-tab-panel name="three">third</sui-tab-panel>
    </sui-tabs>
  `
  const tabs = container.querySelector('sui-tabs') as HTMLElement
  const tabEls = Array.from(container.querySelectorAll('sui-tab')) as HTMLElement[]
  const panels = Array.from(container.querySelectorAll('sui-tab-panel')) as HTMLElement[]
  const btnOf = (t: HTMLElement) => t.shadowRoot!.querySelector('button[role="tab"]') as HTMLElement
  return { container, tabs, tabEls, panels, btnOf }
}

const key = (el: HTMLElement, k: string) =>
  el.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, composed: true }))

describe('<sui-tabs> — ARIA contract', () => {
  it('registers the custom elements', () => {
    expect(customElements.get('sui-tabs')).toBeDefined()
    expect(customElements.get('sui-tab')).toBeDefined()
    expect(customElements.get('sui-tab-panel')).toBeDefined()
  })

  it('strip is role=tablist with aria-orientation', async () => {
    const { tabs } = makeTabs()
    await flush()
    const list = tabs.shadowRoot!.querySelector('[role="tablist"]')!
    expect(list).not.toBeNull()
    expect(list.getAttribute('aria-orientation')).toBe('horizontal')
  })

  it('each tab renders a button role=tab; selected tab has aria-selected=true', async () => {
    const { tabEls, btnOf } = makeTabs('two')
    await flush()
    const selected = tabEls.map((t) => btnOf(t).getAttribute('aria-selected'))
    expect(selected).toEqual(['false', 'true', 'false'])
  })

  it('roving tabindex: only the active tab button is in the tab order', async () => {
    const { tabEls, btnOf } = makeTabs('two')
    await flush()
    expect(btnOf(tabEls[0]!).tabIndex).toBe(-1)
    expect(btnOf(tabEls[1]!).tabIndex).toBe(0)
    expect(btnOf(tabEls[2]!).tabIndex).toBe(-1)
  })

  it('vertical orientation reflects on the tablist', async () => {
    const { tabs } = makeTabs('one', 'orientation="vertical"')
    await flush()
    const list = tabs.shadowRoot!.querySelector('[role="tablist"]')!
    expect(list.getAttribute('aria-orientation')).toBe('vertical')
  })
})

describe('<sui-tabs> — panel visibility follows selection', () => {
  it('only the active panel is visible (others hidden)', async () => {
    const { panels } = makeTabs('two')
    await flush()
    expect(panels[0]!.hasAttribute('hidden')).toBe(true)
    expect(panels[1]!.hasAttribute('hidden')).toBe(false)
    expect(panels[1]!.hasAttribute('active')).toBe(true)
    expect(panels[2]!.hasAttribute('hidden')).toBe(true)
  })

  it('selecting a different tab swaps which panel is visible', async () => {
    const { tabs, panels } = makeTabs('one')
    await flush()
    expect(panels[0]!.hasAttribute('hidden')).toBe(false)
    tabs.setAttribute('active', 'three')
    await flush()
    expect(panels[0]!.hasAttribute('hidden')).toBe(true)
    expect(panels[2]!.hasAttribute('hidden')).toBe(false)
  })

  it('tabpanel is wired to its tab via id/aria-controls/aria-labelledby', async () => {
    const { tabEls, panels } = makeTabs('one')
    await flush()
    // sui-tabs sets the tab id + aria-controls; the panel labels back to the tab.
    expect(tabEls[0]!.getAttribute('aria-controls')).toBe('sui-tabpanel-one')
    expect(panels[0]!.getAttribute('aria-labelledby')).toBe('sui-tab-one')
    expect(panels[0]!.id).toBe('sui-tabpanel-one')
  })
})

describe('<sui-tabs> — keyboard navigation (automatic activation)', () => {
  it('ArrowRight activates + moves focus to the next tab and emits sui-tab-change (bubbles + composed)', async () => {
    const { tabs, tabEls, btnOf } = makeTabs('one')
    await flush()
    const changes: CustomEvent[] = []
    tabs.addEventListener('sui-tab-change', (e) => changes.push(e as CustomEvent))
    key(btnOf(tabEls[0]!), 'ArrowRight')
    await flush()
    expect(tabs.getAttribute('active')).toBe('two')
    expect(changes).toHaveLength(1)
    expect(changes[0]!.detail).toEqual({ name: 'two' })
    expect(changes[0]!.bubbles).toBe(true)
    expect(changes[0]!.composed).toBe(true)
    expect(tabEls[1]!.shadowRoot!.activeElement).toBe(btnOf(tabEls[1]!))
  })

  it('ArrowLeft wraps from the first tab to the last', async () => {
    const { tabs, tabEls, btnOf } = makeTabs('one')
    await flush()
    key(btnOf(tabEls[0]!), 'ArrowLeft')
    await flush()
    expect(tabs.getAttribute('active')).toBe('three')
  })

  it('Home selects the first tab, End selects the last', async () => {
    const { tabs, tabEls, btnOf } = makeTabs('two')
    await flush()
    key(btnOf(tabEls[1]!), 'Home')
    await flush()
    expect(tabs.getAttribute('active')).toBe('one')
    key(btnOf(tabEls[0]!), 'End')
    await flush()
    expect(tabs.getAttribute('active')).toBe('three')
  })

  it('aria-selected + roving tabindex track the keyboard selection', async () => {
    const { tabs, tabEls, btnOf } = makeTabs('one')
    await flush()
    key(btnOf(tabEls[0]!), 'ArrowRight')
    await flush()
    expect(btnOf(tabEls[1]!).getAttribute('aria-selected')).toBe('true')
    expect(btnOf(tabEls[1]!).tabIndex).toBe(0)
    expect(btnOf(tabEls[0]!).tabIndex).toBe(-1)
    expect(tabs.querySelectorAll('sui-tab-panel')[1]!.hasAttribute('hidden')).toBe(false)
  })
})

describe('<sui-tabs> — pointer + default selection', () => {
  it('clicking a tab activates it and emits sui-tab-change', async () => {
    const { tabs, tabEls, btnOf } = makeTabs('one')
    await flush()
    const changes: string[] = []
    tabs.addEventListener('sui-tab-change', (e) => changes.push((e as CustomEvent).detail.name))
    btnOf(tabEls[2]!).click()
    await flush()
    expect(tabs.getAttribute('active')).toBe('three')
    expect(changes).toEqual(['three'])
  })

  it('with no active set, the first tab is selected by default', async () => {
    const { tabs, tabEls, btnOf } = makeTabs('')
    await flush()
    expect(tabs.getAttribute('active')).toBe('one')
    expect(btnOf(tabEls[0]!).getAttribute('aria-selected')).toBe('true')
  })
})
