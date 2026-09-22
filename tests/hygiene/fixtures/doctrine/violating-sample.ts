import { describe, expect, it } from 'bun:test'

// Doctrine-violating specimen — every ts-side detector must fire on this
// file when fed through the engine's self-test. Never import this fixture.
describe('retired surface specimen', () => {
  it('asserts a retired binding instead of deleting the test', () => {
    const html = render('<div sx="color:red">x</div>')
    expect(html).not.toContain('sx=')
  })

  it.skip('pending rewrite of the retired sweep', () => {
    expect(true).toBe(true)
  })

  it('refreshes goldens from inside the test body', () => {
    // fs.writeFileSync('golden/atoms.json', rendered)
    process.env.SAIL_GOLDEN_UPDATE = '1'
  })

  it('mints class atoms inside the test file', () => {
    const el = document.createElement('div')
    el.setAttribute('data-sail-host', 'card')
    el.className = 's-card-row'
    expect(el.className).toBe('s-card-row')
  })

  it('asserts a token against itself', () => {
    expect('width: var(--sail-gap)').toContain('var(--sail-gap)')
  })
})

it('patched out of existence', () => {
  return; // patched
})
