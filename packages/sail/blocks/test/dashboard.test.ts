import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole, getByText } from '@ship/sail-test'
import { BLOCKS } from '../src/catalog'

const SOURCE_PATH = resolve(process.cwd(), 'routes/dashboard/page.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')

describe('block: dashboard', () => {
  it('is in the product catalog as authored', () => {
    const row = BLOCKS.find((b) => b.id === 'dashboard')
    expect(row?.status).toBe('authored')
    expect(row?.primitives).toEqual(
      expect.arrayContaining(['navbar', 'card', 'tabs', 'table', 'field', 'button']),
    )
  })

  it('composes defaults and is locatable by role / name only', async () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'heading', { name: 'Overview', level: 1 })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'New order' })).toBeTruthy()
    expect(getByRole(v.root, 'tab', { name: 'Orders' })).toBeTruthy()
    expect(getByText(v.root, 'Harbor Co')).toBeTruthy()
    click(getByRole(v.root, 'tab', { name: 'Customers' }))
    await v.flush()
    expect(getByRole(v.root, 'tab', { name: 'Customers', selected: true })).toBeTruthy()
    v.unmount()
  })

  it('does not paint every L1 with a local style() — defaults must hold', () => {
    // Page chrome (shell / page) is allowed. Tag-position style() on L1 is not.
    const l1 = /<(navbar|card|tabs|tab-list|tab|tab-panel|table|field|input|button|alert|label)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })
})
