import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

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
  it('records an authored block in colocated meta (catalog stays planned)', () => {
    expect(META.id).toBe('FooterLinks')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/footer-links/')
    expect(META.primitives).toEqual(expect.arrayContaining(['a', 'button', 'separator']))
  })

  it('composes the Mantine headings, links, and socials, locatable by role / name only', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    for (const name of META.roles.headings) {
      expect(getByRole(v.root, 'heading', { name }), name).toBeTruthy()
    }
    for (const name of META.roles.links) {
      expect(getByRole(v.root, 'link', { name }), name).toBeTruthy()
    }
    for (const name of META.roles.buttons) {
      expect(getByRole(v.root, 'button', { name }), name).toBeTruthy()
    }
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
