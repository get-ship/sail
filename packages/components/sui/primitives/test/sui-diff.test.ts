import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-diff/sui-diff.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)
// Lines are semantic list items carrying a public data-line-kind state +
// part="line"; we anchor on those, never the styling classes (.line.add …).
const lines = (el: HTMLElement) => shadow(el).querySelectorAll('[data-line-kind]')
const linesOf = (el: HTMLElement, kind: string) =>
  shadow(el).querySelectorAll(`[data-line-kind="${kind}"]`)
const marker = (line: Element) => line.querySelector('[aria-hidden="true"]')

describe('<sui-diff>', () => {

  it('computes LCS diff from from/to props', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'a\nb\nc', to: 'a\nB\nc' })
    await flush()
    expect(linesOf(el, 'del').length).toBe(1)
    expect(linesOf(el, 'add').length).toBe(1)
    expect(linesOf(el, 'ctx').length).toBe(2)
  })

  it('parses raw unified-diff value', async () => {
    const { el } = mount(reg, 'sui-diff', { value: '@@ -1,2 +1,2 @@\n-old\n+new\n ctx' })
    await flush()
    expect(linesOf(el, 'hunk').length).toBe(1)
    expect(linesOf(el, 'add')[0]?.textContent).toContain('new')
    expect(linesOf(el, 'del')[0]?.textContent).toContain('old')
  })

  it('value prop takes precedence over from/to', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'x', to: 'y', value: '+only-this' })
    await flush()
    expect(lines(el).length).toBe(1)
    expect(linesOf(el, 'add').length).toBe(1)
  })

  it('add lines carry an added role + a + marker', async () => {
    const { el } = mount(reg, 'sui-diff', { from: '', to: 'new' })
    await flush()
    const addLine = linesOf(el, 'add')[0]!
    expect(addLine.getAttribute('aria-label')).toMatch(/^added:/)
    expect(marker(addLine)?.textContent).toBe('+')
  })

  it('del lines carry a removed role + a - marker', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'old', to: '' })
    await flush()
    const delLine = linesOf(el, 'del')[0]!
    expect(delLine.getAttribute('aria-label')).toMatch(/^removed:/)
    expect(marker(delLine)?.textContent).toBe('-')
  })

  it('hunk lines carry a hunk role + an @ marker', async () => {
    const { el } = mount(reg, 'sui-diff', { value: '@@ -1 +1 @@\n ctx' })
    await flush()
    const hunkLine = linesOf(el, 'hunk')[0]!
    expect(hunkLine.getAttribute('aria-label')).toMatch(/^hunk:/)
    expect(marker(hunkLine)?.textContent).toBe('@')
  })

  it('reacts to prop changes', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'x', to: 'y' })
    await flush()
    expect(lines(el).length).toBe(2)
    el.setAttribute('from', 'same')
    el.setAttribute('to', 'same')
    await flush()
    expect(linesOf(el, 'ctx').length).toBe(1)
    expect(linesOf(el, 'add').length).toBe(0)
  })

  it('empty props render no lines', async () => {
    const { el } = mount(reg, 'sui-diff')
    await flush()
    expect(lines(el).length).toBe(0)
  })

  it('host has role=region', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'a', to: 'b' })
    await flush()
    expect(el.getAttribute('role')).toBe('region')
  })

  it('host has aria-label="diff"', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'a', to: 'b' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('diff')
  })

  it('each line is a listitem carrying its kind', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'x', to: 'y' })
    await flush()
    for (const line of lines(el)) {
      expect(line.getAttribute('data-line-kind')).toBeTruthy()
      expect(line.getAttribute('role')).toBe('listitem')
    }
  })
})
