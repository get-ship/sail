// Light-DOM component registration — the default architecture's runtime
// path: no shadow root, template renders into the element, original
// children project into <slot> wrappers, styles inject as an @scope donut,
// and the document-level contracts sheet installs once.

import { afterEach, describe, expect, it } from 'vitest'
import { component } from '../src/component-factory'
import type { MountModule } from '../src/mount'

let n = 0
const freshTag = (): string => `light-probe-${++n}`

const moduleWithTemplate = (html: string): MountModule =>
  ({
    setup: () => ({
      // Mirrors the compiled module shape: render into the shadow root when
      // one exists, else into the element (`__root = shadowRoot ?? container`).
      __mount: (host: HTMLElement) => {
        const root = host.shadowRoot ?? host
        const tmpl = document.createElement('template')
        tmpl.innerHTML = html
        root.append(...Array.from(tmpl.content.childNodes))
      },
    }),
  }) as unknown as MountModule

afterEach(() => {
  document.body.innerHTML = ''
})

describe('light-DOM component registration', () => {
  it('renders into the element — no shadow root, internals directly queryable', () => {
    const tag = freshTag()
    component(tag, { styles: ':scope { color: red; }' }, moduleWithTemplate('<span class="badge">x</span>'))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.shadowRoot).toBeNull()
    expect(el.querySelector('.badge')).not.toBeNull()
  })

  it('projects original children into named and default slots', () => {
    const tag = freshTag()
    component(
      tag,
      {},
      moduleWithTemplate('<span class="wrap"><slot name="leading"></slot><slot></slot></span>'),
    )
    const el = document.createElement(tag)
    el.innerHTML = '<b slot="leading">L</b>body text'
    document.body.appendChild(el)
    const wrap = el.querySelector('.wrap')!
    const leading = wrap.querySelector('slot[name="leading"]')!
    const dflt = wrap.querySelector('slot:not([name])')!
    expect(leading.querySelector('b')?.textContent).toBe('L')
    expect(dflt.textContent).toContain('body text')
    // The original children moved — they are no longer direct host children.
    expect(Array.from(el.childNodes[0]!.childNodes).length).toBeGreaterThan(0)
    expect(el.querySelector(':scope > b')).toBeNull()
  })

  it('drops unslotted children when the template has no slots (shadow parity)', () => {
    const tag = freshTag()
    component(tag, {}, moduleWithTemplate('<span class="only"></span>'))
    const el = document.createElement(tag)
    el.innerHTML = '<i>gone</i>'
    document.body.appendChild(el)
    expect(el.querySelector('i')).toBeNull()
    expect(el.querySelector('.only')).not.toBeNull()
  })

  it('keeps slot fallback content when nothing is assigned', () => {
    const tag = freshTag()
    component(tag, {}, moduleWithTemplate('<slot name="icon"><em>fallback</em></slot>'))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('em')?.textContent).toBe('fallback')
  })

  it('injects styles as an @scope donut bounded at nested components', () => {
    const tag = freshTag()
    component(tag, { styles: ':scope { color: red; }' }, moduleWithTemplate('<i>s</i>'))
    document.body.appendChild(document.createElement(tag))
    const style = document.getElementById(`ship-set-style-${tag}`)!
    expect(style.textContent).toContain(`@scope ([data-component="${tag}"]) to ([data-component])`)
    expect(style.textContent).toContain(':scope { color: red; }')
  })

  it('installs the document-level light contracts exactly once', () => {
    const t1 = freshTag()
    const t2 = freshTag()
    component(t1, {}, moduleWithTemplate('<i>a</i>'))
    component(t2, {}, moduleWithTemplate('<i>b</i>'))
    document.body.appendChild(document.createElement(t1))
    document.body.appendChild(document.createElement(t2))
    const sheets = document.querySelectorAll('#sui-light-contracts')
    expect(sheets.length).toBe(1)
    const css = sheets[0]!.textContent ?? ''
    expect(css).toContain("[tone='primary']")
    expect(css).toContain("[size='sm']")
    expect(css).not.toContain(':host')
    expect(css).toContain('slot { display: contents; }')
    // The interactive-surface contracts: baseline, focus, disabled,
    // transitions — framework decisions no component re-makes.
    expect(css).toContain(':focus-visible')
    expect(css).toContain(':disabled')
    expect(css).toContain('forced-colors: active')
    expect(css).toMatch(/:where\(button\)[^}]*user-select: none/)
  })

  it('shadow: true still attaches a shadow root (the containment feature)', () => {
    const tag = freshTag()
    component(tag, { shadow: true }, moduleWithTemplate('<span class="inner">x</span>'))
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.shadowRoot).not.toBeNull()
    expect(el.shadowRoot!.querySelector('.inner')).not.toBeNull()
    expect(el.querySelector('.inner')).toBeNull()
  })
})
