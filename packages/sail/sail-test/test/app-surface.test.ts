// App-test pattern (SPEC-TEST-CONVENTION / AUDIT thread C).
// Locate UI via the published contract — role / name / state — never
// data-sail-host, class atoms, or testid. Site/studio suites are a later
// operator-coordinated pass; this file is the owned living example.

import { describe, expect, it } from 'vitest'
import { queryContract, type TestMap } from '../src/contract'

const map: TestMap = {
  version: 1,
  file: 'routes/page.sail',
  entries: [{ role: 'button', name: 'Save' }],
}

describe('app surface — query only the published contract', () => {
  it('finds the host by role and name', () => {
    document.body.innerHTML = '<main><button type="button">Save</button></main>'
    const btn = queryContract(document.body, map.entries[0]!)
    expect(btn.tagName).toBe('BUTTON')
    expect(btn.textContent).toBe('Save')
  })
})
