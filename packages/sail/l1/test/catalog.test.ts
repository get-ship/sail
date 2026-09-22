import { describe, expect, it } from 'vitest'
import { COMPONENTS, TAG_BY_ID, concernsOf, copyOf, componentOf, hosts } from '../src/index'

describe('@ship/sail-l1 catalog', () => {
  it('is a projection: components from dirs, not a test package', () => {
    const tabs = COMPONENTS.find((c) => c.id === 'tabs')
    expect(tabs?.host).toBe('tabs')
    expect(tabs?.parts).toEqual(expect.arrayContaining(['tab', 'tab-list', 'tab-panel']))
    expect(componentOf('tab')).toBe('tabs')
    expect(componentOf('legend')).toBe('fieldset')
    expect(TAG_BY_ID.tab?.role).toBe('part')
    expect(TAG_BY_ID.tabs?.role).toBe('host')
  })

  it('lists style concerns, not component.* tokens', () => {
    const card = concernsOf('card')
    expect(card).toEqual(expect.arrayContaining(['border', 'fg', 'bg']))
    expect(card.some((c) => c.startsWith('component'))).toBe(false)
  })

  it('copy is sidecar only', () => {
    expect(copyOf('tabs')?.category).toBe('navigation')
    expect(copyOf('tab')?.category).toBe('navigation')
    expect(copyOf('button')?.category).toBe('inputs')
    expect(copyOf('dialog')?.category).toBe('overlay')
    expect(hosts().length).toBeGreaterThan(1)
  })

  it('page-item / radio / toggle are parts; page and table are hosts', () => {
    expect(COMPONENTS.find((c) => c.id === 'page')?.host).toBe('page')
    expect(COMPONENTS.find((c) => c.id === 'radio')).toBeUndefined()
    expect(COMPONENTS.find((c) => c.id === 'toggle')).toBeUndefined()
    expect(componentOf('page')).toBe('page')
    expect(componentOf('page-item')).toBe('pagination')
    expect(componentOf('radio')).toBe('radio-group')
    expect(componentOf('toggle')).toBe('toggle-group')
    expect(TAG_BY_ID.page?.role).toBe('host')
    expect(TAG_BY_ID['page-item']?.role).toBe('part')
    expect(TAG_BY_ID.radio?.role).toBe('part')
    expect(TAG_BY_ID.toggle?.role).toBe('part')
    expect(COMPONENTS.find((c) => c.id === 'table')?.parts).toEqual([
      'caption',
      'td',
      'th',
    ])
  })
})
