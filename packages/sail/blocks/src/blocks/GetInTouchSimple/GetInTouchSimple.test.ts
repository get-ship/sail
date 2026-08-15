import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const DIR = resolve(process.cwd(), 'src/blocks/GetInTouchSimple')
const SOURCE_PATH = resolve(DIR, 'GetInTouchSimple.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(resolve(DIR, 'meta.json'), 'utf8')) as {
  id: string
  kind: string
  status: string
  mantineUrl: string
  primitives: string[]
  roles: Array<{
    role: string
    name: string
    level?: number
    fieldName?: string
    placeholder?: string
  }>
}

describe('block: GetInTouchSimple', () => {
  it('records an authored block in colocated meta.json', () => {
    expect(META.id).toBe('GetInTouchSimple')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/get-in-touch-simple/')
    expect(META.primitives).toEqual(expect.arrayContaining(['field', 'input', 'textarea', 'button']))
  })

  it('composes the Mantine contact roles / names', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'heading', { name: 'Get in touch', level: 2 })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Name' })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Email' })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Subject' })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Message' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Send message' })).toBeTruthy()
    v.unmount()
  })

  it('exposes every recorded meta role, including form field names', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    for (const row of META.roles) {
      const el = getByRole(v.root, row.role, {
        name: row.name,
        ...(row.level !== undefined ? { level: row.level } : {}),
      })
      expect(el, `${row.role}:${row.name}`).toBeTruthy()
      if (row.fieldName) {
        expect((el as HTMLInputElement | HTMLTextAreaElement).name).toBe(row.fieldName)
      }
      if (row.placeholder) {
        expect((el as HTMLInputElement | HTMLTextAreaElement).placeholder).toBe(row.placeholder)
      }
    }
    v.unmount()
  })

  it('keeps send when the fields are empty', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    for (const name of ['Name', 'Email', 'Subject', 'Message'] as const) {
      const field = getByRole(v.root, 'textbox', { name }) as HTMLInputElement | HTMLTextAreaElement
      expect(field.value).toBe('')
    }
    expect(getByRole(v.root, 'button', { name: 'Send message' })).toBeTruthy()
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(field|input|label|textarea|button)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
