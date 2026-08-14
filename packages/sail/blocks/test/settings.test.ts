import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'
import { BLOCKS } from '../src/catalog'

const SOURCE_PATH = resolve(process.cwd(), 'routes/settings/page.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')

describe('block: settings', () => {
  it('is authored', () => {
    expect(BLOCKS.find((b) => b.id === 'settings')?.status).toBe('authored')
  })

  it('is locatable by role / name', async () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'heading', { name: 'Settings', level: 1 })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Save changes' })).toBeTruthy()
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 =
      /<(navbar|fieldset|legend|field|input|label|switch|button)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })
})
