// use:tabs — orientation stamp + position pairing (no name contract).

import { afterEach, describe, expect, it } from 'vitest'
import { tabs } from '../src/actions/tabs'

function el(tag: string, host: string, text = ''): HTMLElement {
  const node = document.createElement(tag)
  node.setAttribute('data-sail-host', host)
  if (text) node.textContent = text
  return node
}

function mountTabs(orientation?: 'horizontal' | 'vertical'): {
  root: HTMLElement
  list: HTMLElement
  tabA: HTMLElement
  tabB: HTMLElement
  panelA: HTMLElement
  panelB: HTMLElement
} {
  const root = document.createElement('div')
  root.setAttribute('data-sail-host', 'tabs')
  if (orientation) root.setAttribute('orientation', orientation)
  const list = el('div', 'tab-list')
  const tabA = el('button', 'tab', 'One')
  const tabB = el('button', 'tab', 'Two')
  const panelA = el('div', 'tab-panel', 'Panel one')
  const panelB = el('div', 'tab-panel', 'Panel two')
  list.append(tabA, tabB)
  root.append(list, panelA, panelB)
  document.body.append(root)
  return { root, list, tabA, tabB, panelA, panelB }
}

describe('use:tabs', () => {
  const cleanups: Array<() => void> = []
  afterEach(() => {
    for (const c of cleanups.splice(0)) c()
    document.body.replaceChildren()
  })

  it('pairs nameless tabs to panels by position', () => {
    const { root, tabA, tabB, panelA, panelB } = mountTabs()
    const action = tabs(root)
    cleanups.push(() => action.destroy?.())
    expect(tabA.getAttribute('aria-selected')).toBe('true')
    expect(panelA.hidden).toBe(false)
    expect(panelB.hidden).toBe(true)
    tabB.click()
    expect(tabB.getAttribute('aria-selected')).toBe('true')
    expect(panelB.hidden).toBe(false)
    expect(panelA.hidden).toBe(true)
  })

  it('stamps vertical orientation onto list, tabs, and panels', () => {
    const { root, list, tabA, panelA } = mountTabs('vertical')
    const action = tabs(root)
    cleanups.push(() => action.destroy?.())
    expect(list.getAttribute('orientation')).toBe('vertical')
    expect(list.getAttribute('role')).toBe('tablist')
    expect(list.getAttribute('aria-orientation')).toBe('vertical')
    expect(tabA.getAttribute('orientation')).toBe('vertical')
    expect(panelA.getAttribute('orientation')).toBe('vertical')
  })
})
