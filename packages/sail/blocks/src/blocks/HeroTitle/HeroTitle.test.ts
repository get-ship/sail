import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE_PATH = resolve(process.cwd(), 'src/blocks/HeroTitle/HeroTitle.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(
  readFileSync(resolve(process.cwd(), 'src/blocks/HeroTitle/meta.json'), 'utf8'),
) as {
  id: string
  kind: string
  status: string
  mantineUrl: string
  primitives: string[]
}

describe('block: HeroTitle', () => {
  it('records catalog fields locally (isolation — do not touch src/catalog.ts)', () => {
    expect(META.id).toBe('HeroTitle')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/hero-title/')
    expect(META.primitives).toEqual(expect.arrayContaining(['button', 'a']))
  })

  it('exposes the Mantine heading, primary action, and GitHub link by role / name', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(
      getByRole(v.root, 'heading', {
        name: 'A fully featured React components and hooks library',
      }),
    ).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Get started' })).toBeTruthy()
    const github = getByRole(v.root, 'link', { name: 'GitHub' }) as HTMLAnchorElement
    expect(github.href).toBe('https://github.com/mantinedev/mantine')
    v.unmount()
  })

  it('keeps the lead copy the Mantine page shows', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(v.root.textContent).toContain(
      'Build fully functional accessible web applications with ease',
    )
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(button|a|h1|p)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
