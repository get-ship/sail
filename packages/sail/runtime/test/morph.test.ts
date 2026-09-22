import { describe, expect, it } from 'vitest'
import { morph } from '../src/morph'

const fromHtml = (html: string): Element => {
  const tpl = document.createElement('template')
  tpl.innerHTML = html
  return tpl.content.firstElementChild as Element
}

describe('morph', () => {
  it('updates text content in place', () => {
    const t = fromHtml('<div>old</div>')
    const ref = t.firstChild!
    morph(t, fromHtml('<div>new</div>'))
    expect(t.textContent).toBe('new')
    expect(t.firstChild).toBe(ref) // preserved identity
  })

  it('adds new attributes', () => {
    const t = fromHtml('<div></div>')
    morph(t, fromHtml('<div class="x" id="y"></div>'))
    expect(t.getAttribute('class')).toBe('x')
    expect(t.getAttribute('id')).toBe('y')
  })

  it('removes gone attributes', () => {
    const t = fromHtml('<div class="x" id="y"></div>')
    morph(t, fromHtml('<div></div>'))
    expect(t.hasAttribute('class')).toBe(false)
    expect(t.hasAttribute('id')).toBe(false)
  })

  it('updates changed attributes', () => {
    const t = fromHtml('<a href="/old">x</a>')
    morph(t, fromHtml('<a href="/new">x</a>'))
    expect(t.getAttribute('href')).toBe('/new')
  })

  it('replaces element when tag differs', () => {
    const parent = document.createElement('div')
    const t = fromHtml('<span>x</span>')
    parent.appendChild(t)
    morph(parent.firstElementChild as Element, fromHtml('<b>x</b>'))
    expect(parent.firstElementChild!.tagName).toBe('B')
  })

  it('appends new children', () => {
    const t = fromHtml('<ul><li>a</li></ul>')
    morph(t, fromHtml('<ul><li>a</li><li>b</li></ul>'))
    expect(t.children.length).toBe(2)
    expect(t.children[1]!.textContent).toBe('b')
  })

  it('removes surplus children', () => {
    const t = fromHtml('<ul><li>a</li><li>b</li><li>c</li></ul>')
    morph(t, fromHtml('<ul><li>a</li></ul>'))
    expect(t.children.length).toBe(1)
  })

  it('preserves element identity across structural updates', () => {
    const t = fromHtml('<ul><li>a</li><li>b</li></ul>')
    const liA = t.children[0]!
    morph(t, fromHtml('<ul><li>A</li><li>B</li></ul>'))
    expect(t.children[0]).toBe(liA)
    expect(t.children[0]!.textContent).toBe('A')
  })

  it('input value sync', () => {
    const t = fromHtml('<input type="text" />') as HTMLInputElement
    t.value = 'typed'
    morph(t, fromHtml('<input type="text" value="server" />'))
    expect(t.value).toBe('server')
  })

  it('checkbox checked sync', () => {
    const t = fromHtml('<input type="checkbox" />') as HTMLInputElement
    morph(t, fromHtml('<input type="checkbox" checked />'))
    expect(t.checked).toBe(true)
  })

  it('skip option opts subtree out', () => {
    const t = fromHtml('<div><span data-keep>orig</span></div>')
    const span = t.firstElementChild!
    morph(t, fromHtml('<div><span data-keep>changed</span></div>'), {
      skip: (el) => el.hasAttribute('data-keep'),
    })
    expect(span.textContent).toBe('orig')
  })

  it('accepts string input as innerHTML of target', () => {
    const t = document.createElement('div')
    t.innerHTML = '<span>old</span>'
    const span = t.firstElementChild!
    morph(t, '<span>new</span>')
    expect(t.firstElementChild).toBe(span) // reused
    expect(span.textContent).toBe('new')
  })

  it('zero DOM mutations for structurally identical trees', async () => {
    const t = fromHtml('<ul><li id="a">A</li><li id="b">B</li></ul>')
    document.body.appendChild(t)
    const records: MutationRecord[] = []
    const obs = new MutationObserver((rs) => records.push(...rs))
    obs.observe(t, { childList: true, attributes: true, subtree: true, characterData: true })
    morph(t, fromHtml('<ul><li id="a">A</li><li id="b">B</li></ul>'))
    await Promise.resolve()
    records.push(...obs.takeRecords())
    obs.disconnect()
    t.remove()
    expect(records).toEqual([])
  })

  it('single attribute change records only that attribute', async () => {
    const t = fromHtml('<div class="a" data-x="1"></div>')
    document.body.appendChild(t)
    const records: MutationRecord[] = []
    const obs = new MutationObserver((rs) => records.push(...rs))
    obs.observe(t, { attributes: true, attributeOldValue: true })
    morph(t, fromHtml('<div class="a" data-x="2"></div>'))
    await Promise.resolve()
    records.push(...obs.takeRecords())
    obs.disconnect()
    t.remove()
    expect(records.length).toBe(1)
    expect(records[0]!.attributeName).toBe('data-x')
  })

  it('preserves focus on a matched input', () => {
    const t = fromHtml('<form><label>Name</label><input id="name" type="text" /></form>')
    document.body.appendChild(t)
    const input = t.querySelector('#name') as HTMLInputElement
    input.focus()
    expect(document.activeElement).toBe(input)
    morph(t, fromHtml('<form><label>Full name</label><input id="name" type="text" /></form>'))
    expect(document.activeElement).toBe(input)
    expect(t.querySelector('#name')).toBe(input)
    t.remove()
  })

  it('preserves scroll position on a matched scroll container', () => {
    const t = fromHtml('<div id="scroller" style="height: 80px; overflow: auto"><p>lorem</p></div>')
    document.body.appendChild(t)
    t.scrollTop = 42
    morph(t, fromHtml('<div id="scroller" style="height: 80px; overflow: auto"><p>ipsum</p></div>'))
    expect(t.scrollTop).toBe(42)
    t.remove()
  })

  it('reorder by id reuses element identities', () => {
    const t = fromHtml('<ul><li id="a">A</li><li id="b">B</li><li id="c">C</li></ul>')
    const a = t.querySelector('#a')!
    const b = t.querySelector('#b')!
    const c = t.querySelector('#c')!
    morph(t, fromHtml('<ul><li id="b">B</li><li id="c">C</li><li id="a">A</li></ul>'))
    expect(Array.from(t.children)).toEqual([b, c, a])
  })

  it('reorder by id produces minimal mutation records', async () => {
    const t = fromHtml('<ul><li id="a">A</li><li id="b">B</li><li id="c">C</li></ul>')
    document.body.appendChild(t)
    const records: MutationRecord[] = []
    const obs = new MutationObserver((rs) => records.push(...rs))
    obs.observe(t, { childList: true, subtree: true, characterData: true })
    morph(t, fromHtml('<ul><li id="b">B</li><li id="c">C</li><li id="a">A</li></ul>'))
    await Promise.resolve()
    records.push(...obs.takeRecords())
    obs.disconnect()
    // Sanity check: an innerHTML replace would record 3 removes + 3 adds.
    // The id-reorder path moves a single node — at most 1 add + 1 remove.
    const adds = records.flatMap((r) => Array.from(r.addedNodes))
    const removes = records.flatMap((r) => Array.from(r.removedNodes))
    expect(adds.length + removes.length).toBeLessThanOrEqual(2)
    t.remove()
  })

  it('compound: deeply nested mixed changes apply minimally', () => {
    const t = fromHtml(
      '<section><header id="h"><h1>Title</h1></header><main><p id="p1">x</p><p id="p2">y</p></main></section>',
    )
    const h = t.querySelector('#h')!
    const h1 = h.firstElementChild!
    const p1 = t.querySelector('#p1')!
    const p2 = t.querySelector('#p2')!
    morph(
      t,
      fromHtml(
        '<section><header id="h"><h1>New Title</h1></header><main><p id="p2">y2</p><p id="p1">x</p></main></section>',
      ),
    )
    expect(t.querySelector('#h')).toBe(h)
    expect(h.firstElementChild).toBe(h1)
    expect(h1.textContent).toBe('New Title')
    const main = t.querySelector('main')!
    expect(Array.from(main.children)).toEqual([p2, p1])
    expect(p2.textContent).toBe('y2')
    expect(p1.textContent).toBe('x')
  })
})
