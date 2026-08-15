import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const DIR = resolve(process.cwd(), 'src/blocks/UserButton')
const SOURCE_PATH = resolve(DIR, 'UserButton.sail')
const META_PATH = resolve(DIR, 'meta.json')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(META_PATH, 'utf8')) as {
  id: string
  kind: string
  status: string
  mantineUrl: string
  primitives: string[]
  recorded: { name: string; email: string }
}

describe('block: UserButton', () => {
  it('records the Mantine user as authored meta (catalog.ts stays planned)', () => {
    expect(META.id).toBe('UserButton')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/user-button/')
    expect(META.primitives).toEqual(expect.arrayContaining(['button', 'avatar']))
    expect(META.recorded.name).toBe('Harriette Spoonlicker')
    expect(META.recorded.email).toBe('hspoonlicker@outlook.com')
  })

  it('composes name and email, locatable by role / name only', async () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    await v.flush()
    expect(getByRole(v.root, 'button', { name: /Harriette Spoonlicker/ })).toBeTruthy()
    expect(getByText(v.root, 'Harriette Spoonlicker')).toBeTruthy()
    expect(getByText(v.root, 'hspoonlicker@outlook.com')).toBeTruthy()
    v.unmount()
  })

  it('keeps the recorded email next to the name', async () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    await v.flush()
    const btn = getByRole(v.root, 'button', { name: /Harriette Spoonlicker/ })
    expect(btn.textContent).toMatch(/hspoonlicker@outlook\.com/)
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(button|avatar|icon)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
