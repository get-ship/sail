import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText, queryByRole } from '@ship/sail-test'

const DIR = resolve(process.cwd(), 'src/blocks/FeaturesCards')
const SOURCE_PATH = resolve(DIR, 'FeaturesCards.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(resolve(DIR, 'meta.json'), 'utf8')) as {
  id: string
  kind: string
  status: string
  mantineUrl: string
  primitives: string[]
}

describe('block: FeaturesCards', () => {
  it('is authored in colocated meta.json', () => {
    expect(META.id).toBe('FeaturesCards')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/features-cards/')
    expect(META.primitives).toEqual(expect.arrayContaining(['badge', 'card', 'icon']))
  })

  it('exposes the Mantine heading and feature titles by role / name / text', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(
      getByRole(v.root, 'heading', {
        name: 'Integrate effortlessly with any technology stack',
        level: 2,
      }),
    ).toBeTruthy()
    expect(getByText(v.root, 'Best company ever')).toBeTruthy()
    expect(getByText(v.root, 'Extreme performance')).toBeTruthy()
    expect(getByText(v.root, 'Privacy focused')).toBeTruthy()
    expect(getByText(v.root, 'No third parties')).toBeTruthy()
    expect(queryByRole(v.root, 'button')).toBeNull()
    expect(queryByRole(v.root, 'link')).toBeNull()
    expect(queryByRole(v.root, 'textbox')).toBeNull()
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(badge|card|icon)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
    expect(SOURCE).not.toMatch(/\b(fill|ink|paint|component\.)\b/)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
