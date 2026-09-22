// @vitest-environment jsdom

import { describe, expect, it } from 'vitest'
import { walk } from '../../src/hydrate/walk'

describe('hydrate/walk', () => {
  it('returns root for empty path', () => {
    const root = document.createElement('div')
    expect(walk(root, [])).toBe(root)
  })

  it('descends through child indices', () => {
    const root = document.createElement('div')
    root.innerHTML = '<p><span><b>x</b></span></p>'
    const target = root.firstChild!.firstChild!.firstChild!
    expect(walk(root, [0, 0, 0])).toBe(target)
  })

  it('crosses element + text + comment children', () => {
    const root = document.createElement('div')
    root.appendChild(document.createTextNode('A'))
    const c = document.createComment('marker')
    root.appendChild(c)
    const span = document.createElement('span')
    root.appendChild(span)
    expect(walk(root, [0])).toBe(root.firstChild)
    expect(walk(root, [1])).toBe(c)
    expect(walk(root, [2])).toBe(span)
  })

  it('throws if the path falls off', () => {
    const root = document.createElement('div')
    root.appendChild(document.createElement('span'))
    expect(() => walk(root, [0, 5])).toThrow(/walk: path/)
  })
})
