import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE_PATH = resolve(process.cwd(), 'src/blocks/hero-title/hero-title.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(
  readFileSync(resolve(process.cwd(), 'src/blocks/hero-title/meta.json'), 'utf8'),
) as {
  id: string
  kind: string
  status: string
  primitives: string[]
}

describe('block: HeroTitle', () => {
  it('records catalog fields locally (isolation — do not touch src/catalog.ts)', () => {
    expect(META.id).toBe('HeroTitle')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.primitives).toEqual(expect.arrayContaining(['button', 'a']))
  })

  it('exposes the recorded heading, primary action, and GitHub link by role / name', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(
      getByRole(v.root, 'heading', {
        name: 'A language for product UI',
      }),
    ).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Get started' })).toBeTruthy()
    const github = getByRole(v.root, 'link', { name: 'GitHub' }) as HTMLAnchorElement
    expect(github.getAttribute('href')).toBe('#')
    v.unmount()
  })

  it('keeps the lead copy the recorded page shows', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(v.root.textContent).toContain(
      'compiler that refuses the wrong Style',
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
