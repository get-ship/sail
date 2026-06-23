// @vitest-environment node
//
// The server DOM is the SSR substrate: the SAME render + bindings that run in
// the browser run against it in a V8 isolate and serialize to first-paint
// HTML. These tests pin the broader cases the synthetic foundation flagged —
// SVG-namespace serialization, `innerHTML`, and `bindHtml` driving the real
// runtime `morph` — plus the Node-navigation surface morph depends on. Runs in
// the `node` environment (no jsdom): `installServerDom` IS the DOM here.

import { describe, it, expect, beforeEach } from 'vitest'
import { installServerDom, bindHtml } from '../src/server'

// `document` is installed onto globalThis by installServerDom; cast it to a
// minimal local handle so the test reads the server DOM, not the lib's
// `Document` type (there is no real DOM in the node environment).
interface Doc {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  createElement(tag: string): any
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  createElementNS(ns: string, tag: string): any
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  createTextNode(text: string): any
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  createComment(data: string): any
}

const SVG_NS = 'http://www.w3.org/2000/svg'

describe('server-dom', () => {
  let dom: ReturnType<typeof installServerDom>
  let doc: Doc
  beforeEach(() => {
    dom = installServerDom()
    doc = globalThis.document as unknown as Doc
  })

  it('serializes text (escaped), elements, comments', () => {
    const root = dom.container()
    const el = doc.createElement('div')
    el.setAttribute('id', 'x')
    el.appendChild(doc.createTextNode('a & b < c'))
    el.appendChild(doc.createComment('flag'))
    root.appendChild(el)
    expect(dom.html(root)).toBe('<div id="x">a &amp; b &lt; c<!--flag--></div>')
  })

  it('serializes SVG namespaced, self-closing empty elements', () => {
    const root = dom.container()
    const svg = doc.createElementNS(SVG_NS, 'svg')
    const circle = doc.createElementNS(SVG_NS, 'circle')
    circle.setAttribute('r', '10')
    svg.appendChild(circle)
    root.appendChild(svg)
    const html = dom.html(root)
    expect(html).toContain('<svg>')
    // Empty SVG element self-closes — the namespaced branch (HTML void
    // elements like <br> never emit the slash).
    expect(html).toContain('<circle r="10"/>')
  })

  it('does NOT self-close empty HTML elements (only SVG)', () => {
    const root = dom.container()
    root.appendChild(doc.createElement('div'))
    expect(dom.html(root)).toBe('<div></div>')
  })

  it('round-trips innerHTML verbatim', () => {
    const root = dom.container()
    const el = doc.createElement('section')
    el.innerHTML = '<b>bold</b><i>x</i>'
    root.appendChild(el)
    expect(dom.html(root)).toBe('<section><b>bold</b><i>x</i></section>')
  })

  it('bindHtml first-paints through the real runtime morph', () => {
    const root = dom.container()
    const el = doc.createElement('article')
    root.appendChild(el)
    // The runtime binding, unchanged — it runs effect → morph against the
    // server DOM. First paint must carry the html.
    bindHtml(el as unknown as Element, () => '<p>hello</p>')
    expect(dom.html(root)).toContain('<p>hello</p>')
  })

  it('exposes the Node-navigation surface morph needs', () => {
    const ul = doc.createElement('ul')
    const a = doc.createElement('li')
    const b = doc.createElement('li')
    ul.appendChild(a)
    ul.appendChild(b)
    expect(ul.firstChild).toBe(a)
    expect(a.nextSibling).toBe(b)
    expect(b.nextSibling).toBe(null)
    expect(ul.children.length).toBe(2)

    const clone = ul.cloneNode(true)
    expect(clone).not.toBe(ul)
    expect(clone.children.length).toBe(2)

    // replaceChild swaps in place.
    const c = doc.createElement('li')
    ul.replaceChild(c, a)
    expect(ul.firstChild).toBe(c)
    expect(ul.children.length).toBe(2)
  })
})
