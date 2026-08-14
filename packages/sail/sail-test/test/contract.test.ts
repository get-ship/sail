import { describe, expect, it } from 'vitest'
import { getByContract, isLeanTestMap, queryContract, type TestMap } from '../src/contract'

describe('test-map contract', () => {
  const map: TestMap = {
    version: 1,
    file: 'apps/demo/routes/page.sail',
    entries: [
      { role: 'button', name: 'Save', states: ['pressed'] },
      { role: 'status', name: 'Loading' },
    ],
  }

  it('accepts a lean compiler map and rejects extra keys', () => {
    expect(isLeanTestMap(map)).toBe(true)
    expect(
      isLeanTestMap({
        version: 1,
        file: 'x.sail',
        entries: [{ role: 'button' }],
      }),
    ).toBe(true)
    expect(
      isLeanTestMap({
        version: 1,
        file: 'x.sail',
        entries: [{ role: 'button', class: 'sail-btn' }],
      }),
    ).toBe(false)
    // Accessible-name text may contain the letters `data-`; that is not a selector key.
    expect(
      isLeanTestMap({
        version: 1,
        file: 'x.sail',
        entries: [{ role: 'button', name: 'data-test' }],
      }),
    ).toBe(true)
  })

  it('queryContract uses role + name only', () => {
    document.body.innerHTML =
      '<button aria-pressed="true">Save</button><div role="status" aria-label="Loading"></div>'
    const btn = queryContract(document.body, map.entries[0]!)
    expect(btn.tagName).toBe('BUTTON')
    expect(btn.textContent).toBe('Save')
    const status = queryContract(document.body, map.entries[1]!)
    expect(status.getAttribute('role')).toBe('status')
  })

  it('getByContract talks to a Playwright-shaped host', () => {
    const calls: Array<{ role: string; options: unknown }> = []
    const loc = { kind: 'locator' }
    const host = {
      getByRole(role: string, options?: unknown) {
        calls.push({ role, options })
        return loc
      },
    }
    expect(getByContract(host, { role: 'button', name: 'Save', states: ['pressed'] })).toBe(loc)
    expect(calls).toEqual([
      { role: 'button', options: { name: 'Save', pressed: true } },
    ])
  })
})
