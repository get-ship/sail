import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getAllByRole, getByRole, getByText } from '@ship/sail-test'
import { BLOCKS } from '../src/catalog'

const SOURCE_PATH = resolve(process.cwd(), 'routes/dashboard/page.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')

describe('template: dashboard', () => {
  it('is in the product catalog as an authored template', () => {
    const row = BLOCKS.find((b) => b.id === 'dashboard')
    expect(row?.kind).toBe('template')
    expect(row?.status).toBe('authored')
    expect(row?.primitives).toEqual(
      expect.arrayContaining(['navbar', 'dialog', 'card', 'table', 'field', 'button']),
    )
  })

  it('composes defaults and is locatable by role / name only', async () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'heading', { name: 'Overview', level: 1 })).toBeTruthy()
    expect(getAllByRole(v.root, 'button', { name: 'New order' })).toHaveLength(2)
    expect(getByText(v.root, 'Harbor Co')).toBeTruthy()
    v.unmount()
  })
})
