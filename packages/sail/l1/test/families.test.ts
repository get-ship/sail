import { describe, expect, it } from 'vitest'
import { FAMILIES, TAGS, TAG_BY_ID, familyOf } from '../src/index'

function isBoolOrUnionProp(type: string, values: string[] | undefined): boolean {
  if (type === 'bool') return true
  if (values && values.length > 0) return true
  return type.includes('|')
}

describe('@ship/sail-l1 families', () => {
  it.each([...FAMILIES])('$id has a host whose tag equals the family id', (family) => {
    const host = TAG_BY_ID[family.host]
    expect(host).toBeDefined()
    expect(host!.tag).toBe(family.id)
    expect(host!.role).toBe('host')
    expect(host!.family).toBe(family.id)
  })

  it.each(
    FAMILIES.flatMap((family) => family.parts.map((part) => ({ family: family.id, part }))),
  )('part $part belongs to $family with role part', ({ family, part }) => {
    expect(familyOf(part)).toBe(family)
    expect(TAG_BY_ID[part]?.role).toBe('part')
    expect(TAG_BY_ID[part]?.family).toBe(family)
  })

  it.each([...TAGS])('$tag appears in exactly one family', (tag) => {
    const homes = FAMILIES.filter((family) => family.host === tag.tag || family.parts.includes(tag.tag))
    expect(homes).toHaveLength(1)
    expect(homes[0]?.id).toBe(tag.family)
    expect(familyOf(tag.tag)).toBe(tag.family)
  })

  it.each([...TAGS])('$tag concerns never start with component', (tag) => {
    expect(tag.concerns.some((c) => c.startsWith('component'))).toBe(false)
  })

  it.each([...TAGS])('$tag axis names are a subset of bool or union prop names', (tag) => {
    const allowed = new Set(
      tag.props.filter((prop) => isBoolOrUnionProp(prop.type, prop.values)).map((prop) => prop.name),
    )
    expect(tag.axes.every((axis) => allowed.has(axis.name))).toBe(true)
  })
})
