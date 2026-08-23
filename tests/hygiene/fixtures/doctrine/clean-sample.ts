import { describe, expect, it } from 'bun:test'

// Doctrine-clean specimen — no detector may fire on this file. Never
// import this fixture.
describe('live surface specimen', () => {
  it('reads tokens through the runtime bridge', () => {
    const style = 'width: calc(2 * var(--sail-gap))'
    expect(typeof style).toBe('string')
  })

  it('renders markup without retired bindings', () => {
    expect('<div>ok</div>').toContain('<div>ok</div>')
  })
})
