import { describe, expect, it } from 'vitest'
import { resolveSlots, snapshotChildren } from '../src/slot'

function setupHost(html: string) {
  const host = document.createElement('div')
  host.innerHTML = html
  document.body.appendChild(host)
  return host
}

describe('resolveSlots', () => {
  it('default slot collects unattributed children', () => {
    const host = setupHost('<p>one</p><p>two</p>')
    const originals = snapshotChildren(host)
    host.innerHTML = '<header>hi</header><!--default--><footer>bye</footer>'
    const anchor = host.childNodes[1] as Comment
    resolveSlots(host, [{ name: 'default', anchor }], originals)
    expect(host.children.length).toBe(4)
    expect(host.children[0]!.tagName).toBe('HEADER')
    expect(host.children[1]!.textContent).toBe('one')
    expect(host.children[2]!.textContent).toBe('two')
    expect(host.children[3]!.tagName).toBe('FOOTER')
  })

  it('named slot collects children with matching slot attr', () => {
    const host = setupHost('<span slot="title">Title</span><p>body</p>')
    const originals = snapshotChildren(host)
    host.innerHTML = '<h1><!--title--></h1><div><!--default--></div>'
    const titleAnchor = host.querySelector('h1')!.firstChild as Comment
    const defaultAnchor = host.querySelector('div')!.firstChild as Comment
    resolveSlots(
      host,
      [
        { name: 'title', anchor: titleAnchor },
        { name: 'default', anchor: defaultAnchor },
      ],
      originals,
    )
    expect(host.querySelector('h1')!.textContent).toBe('Title')
    expect(host.querySelector('div')!.textContent).toBe('body')
  })

  it('fallback renders when no fill provided', () => {
    const host = setupHost('')
    const originals = snapshotChildren(host)
    host.innerHTML = '<div><!--default--></div>'
    const anchor = host.querySelector('div')!.firstChild as Comment
    resolveSlots(
      host,
      [
        {
          name: 'default',
          anchor,
          fallback: () => {
            const span = document.createElement('span')
            span.textContent = 'fallback'
            return [span]
          },
        },
      ],
      originals,
    )
    expect(host.querySelector('span')!.textContent).toBe('fallback')
  })

  it('fallback ignored when fill present', () => {
    const host = setupHost('<p>real</p>')
    const originals = snapshotChildren(host)
    host.innerHTML = '<div><!--default--></div>'
    const anchor = host.querySelector('div')!.firstChild as Comment
    resolveSlots(
      host,
      [
        {
          name: 'default',
          anchor,
          fallback: () => {
            const span = document.createElement('span')
            span.textContent = 'fallback'
            return [span]
          },
        },
      ],
      originals,
    )
    expect(host.querySelector('span')).toBeNull()
    expect(host.querySelector('p')!.textContent).toBe('real')
  })

  it('strips slot attribute from distributed children', () => {
    const host = setupHost('<span slot="x">hi</span>')
    const originals = snapshotChildren(host)
    host.innerHTML = '<div><!--x--></div>'
    const anchor = host.querySelector('div')!.firstChild as Comment
    resolveSlots(host, [{ name: 'x', anchor }], originals)
    expect(host.querySelector('span')!.hasAttribute('slot')).toBe(false)
  })

  it('multiple originals for same slot all get distributed', () => {
    const host = setupHost('<p slot="items">a</p><p slot="items">b</p><p slot="items">c</p>')
    const originals = snapshotChildren(host)
    host.innerHTML = '<ul><!--items--></ul>'
    const anchor = host.querySelector('ul')!.firstChild as Comment
    resolveSlots(host, [{ name: 'items', anchor }], originals)
    expect(host.querySelectorAll('ul > p').length).toBe(3)
    expect(host.querySelectorAll('ul > p')[0]!.textContent).toBe('a')
    expect(host.querySelectorAll('ul > p')[2]!.textContent).toBe('c')
  })

  it('removes anchor placeholders after resolution', () => {
    const host = setupHost('<p>x</p>')
    const originals = snapshotChildren(host)
    host.innerHTML = '<div><!--default--></div>'
    const anchor = host.querySelector('div')!.firstChild as Comment
    resolveSlots(host, [{ name: 'default', anchor }], originals)
    // Anchor comment is gone
    expect(
      Array.from(host.querySelector('div')!.childNodes).filter(
        (n) => n.nodeType === Node.COMMENT_NODE,
      ).length,
    ).toBe(0)
  })
})
