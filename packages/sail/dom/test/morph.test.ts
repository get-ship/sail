// Edge-case tests for the minimal DOM morph used by `patch-elements`.
// Companion to the existing `bindings.test.ts` and `hydrate.test.ts` —
// these specifically target the morph algorithm's known thin spots
// per the documented limitations.

import { describe, it, expect } from 'vitest'
import { morph } from '../src/morph'

function el(html: string): HTMLElement {
  const tpl = document.createElement('template')
  tpl.innerHTML = html.trim()
  const node = tpl.content.firstElementChild as HTMLElement
  if (!node) throw new Error('test fixture parsed to no element')
  document.body.innerHTML = ''
  document.body.appendChild(node)
  return node
}

describe('morph — basic behavior', () => {
  it('updates attributes in place on tag match', () => {
    const a = el('<div class="old" data-x="1">hi</div>')
    morph(a, '<div class="new" data-x="2">hi</div>')
    expect(a.getAttribute('class')).toBe('new')
    expect(a.getAttribute('data-x')).toBe('2')
    expect(a.isConnected).toBe(true)
  })

  it('removes attributes that are absent in the incoming element', () => {
    const a = el('<div class="x" data-y="1" data-z="2">hi</div>')
    morph(a, '<div class="x">hi</div>')
    expect(a.hasAttribute('data-y')).toBe(false)
    expect(a.hasAttribute('data-z')).toBe(false)
    expect(a.getAttribute('class')).toBe('x')
  })

  it('preserves element identity on tag match', () => {
    const a = el('<input type="text" value="hello" />')
    const original = a
    morph(a, '<input type="text" value="hello" data-marker="set" />')
    expect(original.isConnected).toBe(true)
    expect(original.getAttribute('data-marker')).toBe('set')
  })

  it('replaces the element wholesale when tags differ', () => {
    const a = el('<span>hi</span>')
    morph(a, '<div>hi</div>')
    expect(a.isConnected).toBe(false)
    expect(document.body.firstElementChild?.tagName).toBe('DIV')
  })
})

describe('morph — children matching', () => {
  it('keyed children survive reorder', () => {
    const root = el(
      '<ul><li id="a">A</li><li id="b">B</li><li id="c">C</li></ul>',
    )
    const liA = root.querySelector('#a')!
    const liB = root.querySelector('#b')!
    const liC = root.querySelector('#c')!
    morph(
      root,
      '<ul><li id="c">C</li><li id="a">A</li><li id="b">B</li></ul>',
    )
    expect(root.querySelector('#a')).toBe(liA)
    expect(root.querySelector('#b')).toBe(liB)
    expect(root.querySelector('#c')).toBe(liC)
    expect([...root.children].map((c) => c.id)).toEqual(['c', 'a', 'b'])
  })

  it('data-key matches the same as id', () => {
    const root = el(
      '<ul><li data-key="a">A</li><li data-key="b">B</li></ul>',
    )
    const liA = root.querySelector('[data-key="a"]')!
    morph(root, '<ul><li data-key="b">B</li><li data-key="a">A</li></ul>')
    expect(root.querySelector('[data-key="a"]')).toBe(liA)
  })

  it('drops removed keyed children', () => {
    const root = el(
      '<ul><li id="a">A</li><li id="b">B</li><li id="c">C</li></ul>',
    )
    morph(root, '<ul><li id="a">A</li><li id="c">C</li></ul>')
    expect(root.querySelector('#b')).toBeNull()
    expect([...root.children].map((c) => c.id)).toEqual(['a', 'c'])
  })

  it('inserts new keyed children', () => {
    const root = el('<ul><li id="a">A</li></ul>')
    morph(
      root,
      '<ul><li id="a">A</li><li id="b">B</li><li id="c">C</li></ul>',
    )
    expect([...root.children].map((c) => c.id)).toEqual(['a', 'b', 'c'])
  })

  it('position-matches unkeyed children of the same tag', () => {
    const root = el('<div><span>1</span><span>2</span></div>')
    const span1 = root.children[0]!
    const span2 = root.children[1]!
    morph(root, '<div><span>x</span><span>y</span></div>')
    expect(root.children[0]).toBe(span1)
    expect(root.children[1]).toBe(span2)
    expect(root.textContent).toBe('xy')
  })

  it('text nodes morph by value in position', () => {
    const root = el('<p>hello world</p>')
    morph(root, '<p>goodbye world</p>')
    expect(root.textContent).toBe('goodbye world')
  })
})

describe('morph — documented limitations', () => {
  it('drops everything but the first root element when given a multi-root fragment', () => {
    // Documented limitation: parseFragmentRoot returns firstElementChild only.
    const root = el('<div>a</div>')
    morph(root, '<div>b</div><div>c</div>')
    expect(root.textContent).toBe('b')
    // The second <div>c</div> is silently dropped — confirms current behavior.
    expect(document.body.children.length).toBe(1)
  })

  it('returns a no-op on whitespace-only / element-less html', () => {
    const root = el('<div>before</div>')
    morph(root, '   \n   ')
    // Nothing to morph against; target untouched.
    expect(root.textContent).toBe('before')
    expect(root.isConnected).toBe(true)
  })

  it('comment-only html is also a no-op', () => {
    const root = el('<div>before</div>')
    morph(root, '<!-- comment -->')
    expect(root.textContent).toBe('before')
  })
})

describe('morph — input focus preservation', () => {
  it('keeps focus on a same-tag input across attribute updates', () => {
    const input = el('<input type="text" value="abc" />') as HTMLInputElement
    input.focus()
    input.setSelectionRange(1, 2)
    expect(document.activeElement).toBe(input)
    morph(input, '<input type="text" value="abc" data-marker="updated" />')
    // Element identity preserved → focus survives.
    expect(document.activeElement).toBe(input)
    expect(input.getAttribute('data-marker')).toBe('updated')
  })
})
