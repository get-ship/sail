import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'
import { BLOCKS } from '../src/catalog'

const SOURCE_PATH = resolve(process.cwd(), 'routes/pricing/page.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')

describe('block: pricing', () => {
  it('is authored', () => {
    expect(BLOCKS.find((b) => b.id === 'pricing')?.status).toBe('authored')
  })

  it('has three named plans and a featured CTA', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'heading', { name: 'Pricing that scales with you', level: 1 })).toBeTruthy()
    expect(getByText(v.root, 'Hobby')).toBeTruthy()
    expect(getByText(v.root, 'Studio')).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Choose Studio' })).toBeTruthy()
    v.unmount()
  })
})
