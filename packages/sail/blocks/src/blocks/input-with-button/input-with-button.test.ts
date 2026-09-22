import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'
import { BLOCKS } from '../../catalog'

const SOURCE_PATH = resolve(process.cwd(), 'src/blocks/input-with-button/input-with-button.sail')
const SOURCE = readFileSync(SOURCE_PATH, 'utf8')

describe('block: InputWithButton', () => {
  it('is in the product catalog as an authored block', () => {
    const row = BLOCKS.find((b) => b.id === 'InputWithButton')
    expect(row?.kind).toBe('block')
    expect(row?.status).toBe('authored')
    expect(row?.primitives).toEqual(expect.arrayContaining(['field', 'input', 'button']))
  })

  it('composes a named field and submit, locatable by role / name only', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    expect(getByRole(v.root, 'textbox', { name: 'Search questions' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Search' })).toBeTruthy()
    v.unmount()
  })

  it('keeps the submit when the query is empty', () => {
    const v = renderRoute(SOURCE, { sourcePath: SOURCE_PATH })
    const input = getByRole(v.root, 'textbox', { name: 'Search questions' }) as HTMLInputElement
    expect(input.value).toBe('')
    expect(getByRole(v.root, 'button', { name: 'Search' })).toBeTruthy()
    v.unmount()
  })

  it('does not paint L1 with local style()', () => {
    const l1 = /<(field|input|label|button)\b[^>]*\bstyle\(/
    expect(SOURCE).not.toMatch(l1)
  })

  it('has no page chrome and no data-* hooks', () => {
    expect(SOURCE).not.toMatch(/<(navbar|footer)\b/)
    expect(SOURCE).not.toMatch(/\bdata-testid\b/)
    expect(SOURCE).not.toMatch(/\bdata-[a-z]/)
  })
})
