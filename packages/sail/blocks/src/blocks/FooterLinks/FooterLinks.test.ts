import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText, queryByRole } from '@ship/sail-test'
import { BLOCKS } from '../../catalog'

const DIR = resolve(process.cwd(), 'src/blocks/FooterLinks')
const SOURCE_PATH = resolve(DIR, 'FooterLinks.sail')
const META_PATH = resolve(DIR, 'meta.json')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(META_PATH, 'utf8')) as {
  id: string
  kind: string
  status: string
  mantineUrl: string
  primitives: string[]
  roles: {
    headings: string[]
    links: string[]
    buttons: string[]
    text: string[]
  }
}

describe('block: FooterLinks', () => {
  it('records an authored block in colocated meta', () => {
    expect(META.id).toBe('FooterLinks')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/footer-links/')
    expect(META.primitives).toEqual(expect.arrayContaining(['a', 'separator']))
    expect(META.primitives).not.toContain('button')
    const row = BLOCKS.find((b) => b.id === 'FooterLinks')
    expect(row?.primitives).toEqual(expect.arrayContaining(['a', 'separator']))
    expect(row?.primitives).not.toContain('button')
  })

  it('composes the Mantine headings, links, and socials, locatable by role / name only', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    for (const name of META.roles.headings) {
      expect(getByRole(v.root, 'heading', { name }), name).toBeTruthy()
    }
    for (const name of META.roles.links) {
      expect(getByRole(v.root, 'link', { name }), name).toBeTruthy()
    }
    expect(META.roles.buttons).toEqual([])
    expect(queryByRole(v.root, 'button')).toBeNull()
    for (const text of META.roles.text) {
      expect(getByText(v.root, text), text).toBeTruthy()
    }
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(a|button|icon|separator)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
