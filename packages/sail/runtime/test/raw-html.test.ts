import { describe, expect, it } from 'vitest'
import { signal } from '../src/signal'
import { bindRawHtml } from '../src/raw-html'

describe('bindRawHtml', () => {
  it('sets innerHTML from getter', () => {
    const el = document.createElement('div')
    const sig = signal('<b>Hi</b>')
    bindRawHtml(el, () => sig(), [sig])
    const b = el.querySelector('b')
    expect(b).not.toBeNull()
    expect(b!.tagName).toBe('B')
    expect(b!.textContent).toBe('Hi')
    expect(el.childNodes.length).toBe(1)
  })

  it('updates on dep change', () => {
    const el = document.createElement('div')
    const sig = signal('<i>a</i>')
    bindRawHtml(el, () => sig(), [sig])
    sig.set('<u>b</u>')
    expect(el.querySelector('i')).toBeNull()
    const u = el.querySelector('u')
    expect(u).not.toBeNull()
    expect(u!.textContent).toBe('b')
    expect(el.childNodes.length).toBe(1)
  })

  it('null/undefined renders empty', () => {
    const el = document.createElement('div')
    const sig = signal<string | null>(null)
    bindRawHtml(el, () => sig() as string, [sig])
    expect(el.childNodes.length).toBe(0)
    expect(el.textContent).toBe('')
  })

  it('disposer stops updates', () => {
    const el = document.createElement('div')
    const sig = signal('a')
    const dispose = bindRawHtml(el, () => sig(), [sig])
    dispose()
    sig.set('b')
    expect(el.textContent).toBe('a')
  })

  // Documentation-by-test for the XSS contract: this binding is the
  // ESCAPE hatch. It MUST NOT silently sanitize — callers rely on
  // exact innerHTML semantics. Changing this behavior is a breaking
  // contract change.
  it('does NOT escape — XSS surface is intentional and documented', () => {
    const el = document.createElement('div')
    const sig = signal('<img src=x onerror="window.__xss=true">')
    bindRawHtml(el, () => sig(), [sig])
    // The <img> element is constructed live — the contract is "raw".
    expect(el.querySelector('img')).not.toBeNull()
    expect(el.innerHTML).toContain('onerror')
  })

  it('morph directive routes updates through morph() — preserves identity', () => {
    const el = document.createElement('div')
    el.setAttribute('morph', '')
    document.body.appendChild(el)
    const sig = signal('<p id="a">first</p>')
    bindRawHtml(el, () => sig(), [sig])
    const p = el.querySelector('#a')!
    expect(p.textContent).toBe('first')
    sig.set('<p id="a">second</p>')
    expect(el.querySelector('#a')).toBe(p) // identity preserved
    expect(p.textContent).toBe('second')
    el.remove()
  })

  it('script tags are inert via innerHTML (HTML5 spec) but markup survives', () => {
    const el = document.createElement('div')
    const sig = signal('<script>window.__xss=true</script><b>after</b>')
    bindRawHtml(el, () => sig(), [sig])
    // HTML5: script tags inserted via innerHTML do NOT execute. The
    // markup is preserved but inert. Authors relying on raw HTML
    // should know this — execution requires explicit Element insert.
    expect(el.querySelector('script')).not.toBeNull()
    expect(el.querySelector('b')?.textContent).toBe('after')
    // @ts-expect-error — probe the window for leaked execution
    expect(globalThis.__xss).toBeUndefined()
  })
})
