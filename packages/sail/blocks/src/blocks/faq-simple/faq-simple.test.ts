import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText, queryByRole } from '@ship/sail-test'

const DIR = resolve(process.cwd(), 'src/blocks/faq-simple')
const SOURCE_PATH = resolve(DIR, 'faq-simple.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(resolve(DIR, 'meta.json'), 'utf8')) as {
  id: string
  kind: string
  status: string
  primitives: string[]
  headings: Array<{ role: string; name: string; level: number }>
  questions: string[]
}

describe('block: FaqSimple', () => {
  it('records an authored block in colocated meta.json', () => {
    expect(META.id).toBe('FaqSimple')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.primitives).toEqual(expect.arrayContaining(['accordion']))
    expect(META.headings).toEqual([
      { role: 'heading', name: 'Frequently Asked Questions', level: 1 },
    ])
    expect(META.questions).toHaveLength(5)
  })

  it('exposes the recorded heading by role / name', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(
      getByRole(v.root, 'heading', {
        name: 'Frequently Asked Questions',
        level: 1,
      }),
    ).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: META.questions[0] })).toBeTruthy()
    expect(queryByRole(v.root, 'link')).toBeNull()
    expect(queryByRole(v.root, 'textbox')).toBeNull()
    v.unmount()
  })

  it('exposes every recorded question by visible text', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    for (const question of META.questions) {
      expect(getByText(v.root, question), question).toBeTruthy()
    }
    v.unmount()
  })

  it('fails if a recorded heading or question is deleted', () => {
    for (const heading of META.headings) {
      expect(SOURCE).toContain(heading.name)
    }
    for (const question of META.questions) {
      expect(SOURCE).toContain(question)
    }
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(accordion|strong|h1|p)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
    expect(SOURCE).not.toMatch(/\b(fill|ink|paint|component\.)\b/)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
