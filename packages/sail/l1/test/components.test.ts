import { describe, expect, it } from 'vitest'
import { COMPONENTS, TAGS, TAG_BY_ID, componentOf } from '../src/index'

function isBoolOrUnionProp(type: string, values: string[] | undefined): boolean {
  if (type === 'bool') return true
  if (values && values.length > 0) return true
  return type.includes('|')
}

describe('@ship/sail-l1 components', () => {
  it.each([...COMPONENTS])('$id has a host whose tag equals the component id', (component) => {
    const host = TAG_BY_ID[component.host]
    expect(host).toBeDefined()
    expect(host!.tag).toBe(component.id)
    expect(host!.role).toBe('host')
    expect(host!.component).toBe(component.id)
  })

  it.each(
    COMPONENTS.flatMap((component) =>
      component.parts.map((part) => ({ component: component.id, part })),
    ),
  )('part $part belongs to $component with role part', ({ component, part }) => {
    expect(componentOf(part)).toBe(component)
    expect(TAG_BY_ID[part]?.role).toBe('part')
    expect(TAG_BY_ID[part]?.component).toBe(component)
  })

  it.each([...TAGS])('$tag appears in exactly one component', (tag) => {
    const homes = COMPONENTS.filter(
      (component) => component.host === tag.tag || component.parts.includes(tag.tag),
    )
    expect(homes).toHaveLength(1)
    expect(homes[0]?.id).toBe(tag.component)
    expect(componentOf(tag.tag)).toBe(tag.component)
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
