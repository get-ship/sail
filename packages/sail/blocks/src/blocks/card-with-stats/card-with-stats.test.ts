import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const DIR = resolve(process.cwd(), 'src/blocks/card-with-stats')
const SOURCE_PATH = resolve(DIR, 'card-with-stats.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(resolve(DIR, 'meta.json'), 'utf8')) as {
  id: string
  kind: string
  status: string
  primitives: string[]
  headings: string[]
  stats: { title: string; value: string }[]
  copy: string[]
}

describe('block: CardWithStats', () => {
  it('records the recorded card as an authored block in meta.json', () => {
    expect(META.id).toBe('CardWithStats')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.primitives).toEqual(expect.arrayContaining(['card', 'badge']))
    expect(META.headings).toEqual(['Running challenge'])
    expect(META.stats).toEqual([
      { title: 'Distance', value: '27.4 km' },
      { title: 'Avg. speed', value: '9.6 km/h' },
      { title: 'Score', value: '88/100' },
    ])
  })

  it('exposes the recorded heading, image, and stats by role / name / text', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'heading', { name: 'Running challenge' })).toBeTruthy()
    expect(getByRole(v.root, 'img', { name: 'Running challenge' })).toBeTruthy()
    expect(getByText(v.root, '80% completed')).toBeTruthy()
    expect(
      getByText(
        v.root,
        '56 km this month • 17% improvement compared to last month • 443 place in global scoreboard',
      ),
    ).toBeTruthy()
    for (const stat of META.stats) {
      expect(getByText(v.root, stat.title)).toBeTruthy()
      expect(getByText(v.root, stat.value)).toBeTruthy()
    }
    v.unmount()
  })

  it('fails if a recorded heading or stat is deleted', () => {
    for (const heading of META.headings) {
      expect(SOURCE).toContain(heading)
    }
    for (const stat of META.stats) {
      expect(SOURCE).toContain(stat.title)
      expect(SOURCE).toContain(stat.value)
    }
    for (const line of META.copy) {
      expect(SOURCE).toContain(line)
    }
  })

  it('does not paint type on the heading or badge', () => {
    expect(SOURCE).not.toMatch(/<(h3|badge)\b[^>]*\bstyle\(/)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
