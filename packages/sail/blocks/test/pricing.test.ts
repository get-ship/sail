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

  it('has three named plans, a featured CTA, and a comparison table', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'heading', { name: 'Pricing that grows with you', level: 1 })).toBeTruthy()
    expect(getByRole(v.root, 'heading', { name: 'Hobby' })).toBeTruthy()
    expect(getByRole(v.root, 'heading', { name: 'Studio' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Choose Studio' })).toBeTruthy()
    expect(getByRole(v.root, 'heading', { name: 'Compare plans' })).toBeTruthy()
    expect(getByText(v.root, 'Most popular')).toBeTruthy()
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 =
      /<(navbar|card|button|badge|separator|icon|toggle-group|accordion|alert|table|avatar|theme-picker)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })
})
