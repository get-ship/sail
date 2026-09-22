import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const DIR = resolve(process.cwd(), 'src/blocks/navbar-minimal')
const SOURCE_PATH = resolve(DIR, 'navbar-minimal.sail')
const META_PATH = resolve(DIR, 'meta.json')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')
const META = JSON.parse(readFileSync(META_PATH, 'utf8')) as {
  id: string
  kind: string
  status: string
  primitives: string[]
  roles: {
    buttons: string[]
    current: string
    icons: string[]
  }
}

const CORE_ROLES = new Set([
  'home',
  'menu',
  'close',
  'search',
  'settings',
  'edit',
  'trash',
  'add',
  'check',
  'download',
  'plus',
  'send',
  'attach',
  'arrow-left',
  'arrow-right',
  'chevron-down',
  'chevron-up',
  'chevron-right',
  'chevron-left',
  'info',
  'warning',
  'success',
  'error',
  'image',
  'file',
  'activity',
  'alarm-clock',
  'alert-triangle',
  'check-circle',
  'circle',
  'compass',
  'ellipsis',
  'file-code',
  'file-code-2',
  'folder',
  'git',
  'git-branch',
  'heart',
  'inbox',
  'log-out',
  'mail',
  'moon',
  'more-vertical',
  'radio',
  'redo',
  'redo-2',
  'refresh',
  'refresh-cw',
  'rocket',
  'ship',
  'sparkles',
  'star',
  'stop',
  'sun',
  'trash-2',
  'undo',
  'undo-2',
  'user',
  'bell',
  'x',
  'x-circle',
  'zap',
])

describe('block: NavbarMinimal', () => {
  it('records an authored block in colocated meta (catalog stays planned)', () => {
    expect(META.id).toBe('NavbarMinimal')
    expect(META.kind).toBe('block')
    expect(META.status).toBe('authored')
    expect(META.primitives).toEqual(expect.arrayContaining(['navbar', 'button', 'icon']))
    expect(META.roles.buttons).toEqual([
      'Home',
      'Dashboard',
      'Analytics',
      'Releases',
      'Account',
      'Security',
      'Settings',
      'Change account',
      'Logout',
    ])
    expect(META.roles.current).toBe('Analytics')
  })

  it('composes the recorded icon-rail names, locatable by role / name only', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    for (const name of META.roles.buttons) {
      expect(getByRole(v.root, 'button', { name }), name).toBeTruthy()
    }
    expect(getByRole(v.root, 'button', { name: META.roles.current, current: true })).toBeTruthy()
    v.unmount()
  })

  it('fails if a recorded recorded name is deleted', () => {
    for (const name of META.roles.buttons) {
      expect(SOURCE).toContain(`label="${name}"`)
    }
  })

  it('uses CORE_ROLES icon names only', () => {
    const used = [...SOURCE.matchAll(/<icon\b[^>]*\bname="([^"]+)"/g)].map((m) => m[1])
    expect(used).toEqual(META.roles.icons)
    for (const name of used) {
      expect(CORE_ROLES.has(name), name).toBe(true)
    }
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(navbar|button|icon)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
    expect(SOURCE).not.toMatch(/\b(fill|ink|paint|component\.)\b/)
  })

  it('is a rail block, not a template page, and has no data-* hooks', () => {
    expect(SOURCE).toMatch(/<navbar\b/)
    expect(SOURCE).not.toMatch(/<(footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
