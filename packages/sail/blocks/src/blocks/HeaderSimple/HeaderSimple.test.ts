import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText, queryByRole } from '@ship/sail-test'
import { BLOCKS } from '../../catalog'

const DIR = resolve(process.cwd(), 'src/blocks/HeaderSimple')
const SOURCE_PATH = resolve(DIR, 'HeaderSimple.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(resolve(DIR, 'meta.json'), 'utf8')) as {
  id: string
  kind: string
  status: string
  mantineUrl: string
  primitives: string[]
  brand: string
  roles: Array<{ role: string; name: string; href?: string }>
}

describe('block: HeaderSimple', () => {
  it('records an authored block in colocated meta.json (catalog stays planned)', () => {
    expect(META.id).toBe('HeaderSimple')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/header-simple/')
    expect(META.primitives).toEqual(expect.arrayContaining(['navbar', 'a', 'button']))
    expect(META.brand).toBe('Mantine')
    expect(META.roles.map((r) => r.name)).toEqual([
      'Features',
      'Pricing',
      'Learn',
      'Community',
      'Toggle navigation',
    ])
  })

  it('is listed in the product catalog as a block', () => {
    const row = BLOCKS.find((b) => b.id === 'HeaderSimple')
    expect(row?.kind).toBe('block')
    expect(row?.mantineUrl).toBe('https://ui.mantine.dev/component/header-simple/')
    expect(row?.primitives).toEqual(expect.arrayContaining(['navbar', 'a', 'button']))
  })

  it('composes the Mantine brand, links, and burger, locatable by role / name only', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByText(v.root, META.brand)).toBeTruthy()
    for (const row of META.roles) {
      const el = getByRole(v.root, row.role, { name: row.name })
      expect(el, `${row.role}:${row.name}`).toBeTruthy()
      if (row.href) {
        expect((el as HTMLAnchorElement).getAttribute('href')).toBe(row.href)
      }
    }
    expect(queryByRole(v.root, 'textbox')).toBeNull()
    expect(queryByRole(v.root, 'heading')).toBeNull()
    v.unmount()
  })

  it('fails if a recorded link or button is deleted', () => {
    for (const row of META.roles) {
      expect(SOURCE, row.name).toContain(row.name)
      if (row.href) expect(SOURCE, row.href).toContain(row.href)
    }
    expect(SOURCE).toContain(META.brand)
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(navbar|a|button|icon)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
    expect(SOURCE).not.toMatch(/\b(fill|ink|paint|component\.)\b/)
  })

  it('has no page footer and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<footer\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
