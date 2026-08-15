import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const DIR = resolve(process.cwd(), 'src/blocks/AuthenticationTitle')
const SOURCE_PATH = resolve(DIR, 'AuthenticationTitle.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(resolve(DIR, 'meta.json'), 'utf8')) as {
  id: string
  kind: string
  status: string
  mantineUrl: string
  primitives: string[]
}

describe('block: AuthenticationTitle', () => {
  it('records an authored block in colocated meta.json', () => {
    expect(META.id).toBe('AuthenticationTitle')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.mantineUrl).toBe('https://ui.mantine.dev/component/authentication-title/')
    expect(META.primitives).toEqual(
      expect.arrayContaining(['field', 'input', 'button', 'checkbox']),
    )
  })

  it('composes the Mantine auth roles, locatable by role / name only', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'heading', { name: 'Welcome back!', level: 1 })).toBeTruthy()
    expect(getByRole(v.root, 'link', { name: 'Create account' })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Email' })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Password' })).toBeTruthy()
    expect(getByRole(v.root, 'checkbox', { name: 'Remember me' })).toBeTruthy()
    expect(getByRole(v.root, 'link', { name: 'Forgot password?' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Sign in' })).toBeTruthy()
    v.unmount()
  })

  it('keeps Sign in when the identity fields are empty', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    const email = getByRole(v.root, 'textbox', { name: 'Email' }) as HTMLInputElement
    const password = getByRole(v.root, 'textbox', { name: 'Password' }) as HTMLInputElement
    expect(email.value).toBe('')
    expect(password.value).toBe('')
    expect(getByRole(v.root, 'button', { name: 'Sign in' })).toBeTruthy()
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(card|field|input|label|button|checkbox|a)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
