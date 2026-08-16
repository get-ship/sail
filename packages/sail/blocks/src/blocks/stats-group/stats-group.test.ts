import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'
import { BLOCKS } from '../../catalog'

const DIR = resolve(process.cwd(), 'src/blocks/stats-group')
const SOURCE_PATH = resolve(DIR, 'stats-group.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(resolve(DIR, 'meta.json'), 'utf8')) as {
  id: string
  kind: string
  primitives: string[]
  recorded: {
    headings: { role: string; name: string; level: number }[]
    values: string[]
    descriptions: string[]
    buttons: unknown[]
    links: unknown[]
    fields: unknown[]
  }
}

describe('block: StatsGroup', () => {
  it('records the recorded figures in meta.json', () => {
    expect(META.id).toBe('StatsGroup')
    expect(META.kind).toBe('block')
    expect(META.primitives).toEqual(expect.arrayContaining(['card']))
    expect(META.recorded.headings.map((h) => h.name)).toEqual([
      'Page views',
      'New users',
      'Completed orders',
    ])
    expect(META.recorded.values).toEqual(['456,133', '2,175', '1,994'])
    expect(META.recorded.buttons).toEqual([])
    expect(META.recorded.links).toEqual([])
    expect(META.recorded.fields).toEqual([])
  })

  it('is listed in the product catalog as a block', () => {
    const row = BLOCKS.find((b) => b.id === 'StatsGroup')
    expect(row?.kind).toBe('block')
    expect(row?.primitives).toEqual(expect.arrayContaining(['card']))
  })

  it('exposes the recorded headings, values, and descriptions by role / name', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    for (const heading of META.recorded.headings) {
      expect(getByRole(v.root, 'heading', { name: heading.name, level: heading.level })).toBeTruthy()
    }
    for (const value of META.recorded.values) {
      expect(getByText(v.root, value)).toBeTruthy()
    }
    for (const description of META.recorded.descriptions) {
      expect(getByText(v.root, description)).toBeTruthy()
    }
    v.unmount()
  })

  it('does not paint chrome on the band', () => {
    expect(SOURCE).not.toMatch(/<(card|grid|row)\b[^>]*\bstyle\(/)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
