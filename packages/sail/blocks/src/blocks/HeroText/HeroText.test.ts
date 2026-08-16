import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const DIR = resolve(process.cwd(), 'src/blocks/HeroText')
const SOURCE_PATH = resolve(DIR, 'HeroText.sail')
const META_PATH = resolve(DIR, 'meta.json')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(META_PATH, 'utf8')) as {
  id: string
  kind: string
  status: string
  mantineUrl: string
  primitives: string[]
}

describe('block: HeroText', () => {
  it('records authored meta without touching the shared catalog', () => {
    expect(META.id).toBe('HeroText')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/hero-text/')
    expect(META.primitives).toEqual(expect.arrayContaining(['button']))
  })

  it('exposes the Mantine heading and both actions by role / name', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(
      getByRole(v.root, 'heading', { name: 'Automated AI code reviews for any stack' }),
    ).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Book a demo' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Purchase a license' })).toBeTruthy()
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(button|h1|p)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
