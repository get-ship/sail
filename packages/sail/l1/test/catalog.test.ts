import { describe, expect, it } from 'vitest'
import { FAMILIES, TAG_BY_ID, concernsOf, copyOf, familyOf, hosts } from '../src/index'

describe('@ship/sail-l1 catalog', () => {
  it('is a projection: families from dirs, not a test package', () => {
    const tabs = FAMILIES.find((f) => f.id === 'tabs')
    expect(tabs?.host).toBe('tabs')
    expect(tabs?.parts).toEqual(expect.arrayContaining(['tab', 'tab-list', 'tab-panel']))
    expect(familyOf('tab')).toBe('tabs')
    expect(familyOf('legend')).toBe('fieldset')
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

  it('page / radio / toggle are parts; table has caption/td/th', () => {
    expect(FAMILIES.find((f) => f.id === 'page')).toBeUndefined()
    expect(FAMILIES.find((f) => f.id === 'radio')).toBeUndefined()
    expect(FAMILIES.find((f) => f.id === 'toggle')).toBeUndefined()
    expect(familyOf('page')).toBe('pagination')
    expect(familyOf('radio')).toBe('radio-group')
    expect(familyOf('toggle')).toBe('toggle-group')
    expect(TAG_BY_ID.page?.role).toBe('part')
    expect(TAG_BY_ID.radio?.role).toBe('part')
    expect(TAG_BY_ID.toggle?.role).toBe('part')
    expect(FAMILIES.find((f) => f.id === 'table')?.parts).toEqual([
      'caption',
      'td',
      'th',
    ])
  })
})
