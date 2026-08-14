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
})
