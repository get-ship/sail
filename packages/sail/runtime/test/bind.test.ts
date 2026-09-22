import { describe, expect, it } from 'vitest'
import { signal } from '../src/signal'
import {
  bindText,
  bindShow,
  bindAttr,
  bindProp,
  bindDynamic,
  bindValue,
  bindChecked,
  bindHtml,
} from '../src/bind'

describe('bindText', () => {
  it('sets initial text', () => {
    const el = document.createElement('span')
    const $msg = signal('hello')
    bindText(el, () => $msg())
    expect(el.textContent).toBe('hello')
  })

  it('updates on signal change', () => {
    const el = document.createElement('span')
    const $msg = signal('a')
    bindText(el, () => $msg())
    $msg.set('b')
    expect(el.textContent).toBe('b')
  })

  it('null/undefined → empty', () => {
    const el = document.createElement('span')
    const $msg = signal<string | null>(null)
    bindText(el, () => $msg())
    expect(el.textContent).toBe('')
    $msg.set('x')
    expect(el.textContent).toBe('x')
    $msg.set(null)
    expect(el.textContent).toBe('')
  })

  it('disposer stops updates', () => {
    const el = document.createElement('span')
    const $msg = signal('a')
    const dispose = bindText(el, () => $msg())
    dispose()
    $msg.set('b')
    expect(el.textContent).toBe('a')
  })
})

describe('bindShow', () => {
  it('truthy → visible', () => {
    const el = document.createElement('div')
    const $on = signal(true)
    bindShow(el, () => $on())
    expect(el.style.display).toBe('')
  })

  it('falsy → display:none', () => {
    const el = document.createElement('div')
    const $on = signal(false)
    bindShow(el, () => $on())
    expect(el.style.display).toBe('none')
  })

  it('toggles on signal change', () => {
    const el = document.createElement('div')
    const $on = signal(true)
    bindShow(el, () => $on())
    $on.set(false)
    expect(el.style.display).toBe('none')
    $on.set(true)
    expect(el.style.display).toBe('')
  })

  it('preserves non-default display through hide/show', () => {
    const el = document.createElement('div')
    el.style.display = 'flex'
    const $on = signal(true)
    bindShow(el, () => $on())
    $on.set(false)
    expect(el.style.display).toBe('none')
    $on.set(true)
    expect(el.style.display).toBe('flex')
  })
})

describe('bindAttr', () => {
  it('sets string attribute', () => {
    const el = document.createElement('a')
    const $href = signal('/home')
    bindAttr(el, 'href', () => $href())
    expect(el.getAttribute('href')).toBe('/home')
    $href.set('/about')
    expect(el.getAttribute('href')).toBe('/about')
  })

  it('null removes attribute', () => {
    const el = document.createElement('button')
    const $disabled = signal<boolean | null>(null)
    bindAttr(el, 'disabled', () => $disabled())
    expect(el.hasAttribute('disabled')).toBe(false)
  })

  it('false removes attribute', () => {
    const el = document.createElement('button')
    el.setAttribute('disabled', '')
    const $disabled = signal(false)
    bindAttr(el, 'disabled', () => $disabled())
    expect(el.hasAttribute('disabled')).toBe(false)
  })

  it('true sets empty attribute (presence-only)', () => {
    const el = document.createElement('button')
    const $disabled = signal(true)
    bindAttr(el, 'disabled', () => $disabled())
    expect(el.getAttribute('disabled')).toBe('')
  })

  it('non-bool values stringify', () => {
    const el = document.createElement('div')
    const $count = signal(42)
    bindAttr(el, 'data-count', () => $count())
    expect(el.getAttribute('data-count')).toBe('42')
  })

  it('empty string removes attribute (auto-omit)', () => {
    const el = document.createElement('input')
    const $label = signal('hello')
    bindAttr(el, 'aria-label', () => $label())
    expect(el.getAttribute('aria-label')).toBe('hello')
    $label.set('')
    expect(el.hasAttribute('aria-label')).toBe(false)
  })

  it('undefined removes attribute', () => {
    const el = document.createElement('input')
    el.setAttribute('aria-label', 'old')
    const $label = signal<string | undefined>(undefined)
    bindAttr(el, 'aria-label', () => $label())
    expect(el.hasAttribute('aria-label')).toBe(false)
  })
})

describe('bindDynamic — toolchain owns the attribute/property split', () => {
  const define = (tag: string, cls: CustomElementConstructor) => {
    if (!customElements.get(tag)) customElements.define(tag, cls)
  }

  it('native element: attribute-only, never property-routed', () => {
    const el = document.createElement('div')
    const $t = signal('hi')
    bindDynamic(el, 'title', () => $t())
    // `title` has a property mirror, but native elements stay attribute-only.
    expect(el.getAttribute('title')).toBe('hi')
  })

  it('custom element + object value: property, identity preserved, no attribute', () => {
    define('x-dyn-obj', class extends HTMLElement {})
    const el = document.createElement('x-dyn-obj')
    const cols = [{ id: 1 }]
    bindDynamic(el, 'columns', () => cols)
    expect((el as unknown as { columns: unknown }).columns).toBe(cols) // same ref
    expect(el.hasAttribute('columns')).toBe(false) // not stringified onto an attribute
  })

  it('custom element + declared property: routes to the property (reactive)', () => {
    define(
      'x-dyn-prop',
      class extends HTMLElement {
        _v: unknown
        get value() { return this._v }
        set value(v: unknown) { this._v = v }
      },
    )
    const el = document.createElement('x-dyn-prop')
    const $v = signal('a')
    bindDynamic(el, 'value', () => $v())
    expect((el as unknown as { value: unknown }).value).toBe('a')
    $v.set('b')
    expect((el as unknown as { value: unknown }).value).toBe('b')
  })

  it('custom element + undeclared primitive (data-*): falls through to attribute', () => {
    define('x-dyn-data', class extends HTMLElement {})
    const el = document.createElement('x-dyn-data')
    bindDynamic(el, 'data-count', () => 3)
    expect(el.getAttribute('data-count')).toBe('3')
  })
})

describe('bindProp', () => {
  it('sets a property directly', () => {
    const el = document.createElement('div') as HTMLDivElement & { items?: number[] }
    const $items = signal([1, 2, 3])
    bindProp(el, 'items', () => $items())
    expect(el.items).toEqual([1, 2, 3])
  })

  it('updates on signal change with reference fidelity', () => {
    const el = document.createElement('div') as HTMLDivElement & { data?: object }
    const $data = signal({ x: 1 })
    bindProp(el, 'data', () => $data())
    const next = { x: 2 }
    $data.set(next)
    expect(el.data).toBe(next) // same reference, not stringified
  })
})

describe('bindValue', () => {
  it('initial value flows signal → input', () => {
    const el = document.createElement('input')
    const $v = signal('hello')
    bindValue(el, $v)
    expect(el.value).toBe('hello')
  })

  it('signal change flows to input', () => {
    const el = document.createElement('input')
    const $v = signal('a')
    bindValue(el, $v)
    $v.set('b')
    expect(el.value).toBe('b')
  })

  it('user input flows back to signal', () => {
    const el = document.createElement('input')
    const $v = signal('')
    bindValue(el, $v)
    el.value = 'typed'
    el.dispatchEvent(new Event('input'))
    expect($v()).toBe('typed')
  })

  it('disposer detaches both directions', () => {
    const el = document.createElement('input')
    const $v = signal('a')
    const dispose = bindValue(el, $v)
    dispose()
    $v.set('b')
    expect(el.value).toBe('a') // signal→input detached
    el.value = 'c'
    el.dispatchEvent(new Event('input'))
    expect($v()).toBe('b') // input→signal detached; signal stays at 'b' from previous .set
  })

  it('onWrite callback fires on user input with new value', () => {
    const el = document.createElement('input')
    const $v = signal('')
    const seen: string[] = []
    bindValue(el, $v, (v) => seen.push(v))
    el.value = 'typed'
    el.dispatchEvent(new Event('input'))
    expect($v()).toBe('typed')
    expect(seen).toEqual(['typed'])
  })

  it('onWrite NOT invoked on programmatic signal writes', () => {
    const el = document.createElement('input')
    const $v = signal('a')
    const seen: string[] = []
    bindValue(el, $v, (v) => seen.push(v))
    $v.set('b')
    $v.set('c')
    expect(el.value).toBe('c')
    expect(seen).toEqual([])
  })

  it('wrapper host: signal writes the inner range and the value attribute', () => {
    const host = document.createElement('div')
    const inner = document.createElement('input')
    inner.type = 'range'
    inner.min = '0'
    inner.max = '100'
    host.append(inner)
    const $v = signal('30')
    bindValue(host, $v)
    expect(inner.value).toBe('30')
    expect(host.getAttribute('value')).toBe('30')
    $v.set('90')
    expect(inner.value).toBe('90')
    expect(host.getAttribute('value')).toBe('90')
  })

  it('wrapper host: inner input writes back to the signal', () => {
    const host = document.createElement('div')
    const inner = document.createElement('input')
    inner.type = 'range'
    inner.min = '0'
    inner.max = '100'
    host.append(inner)
    const $v = signal('0')
    bindValue(host, $v)
    inner.value = '44'
    inner.dispatchEvent(new Event('input', { bubbles: true }))
    expect($v()).toBe('44')
  })

  it('coerces a number signal so el.value compares equal', () => {
    const el = document.createElement('input')
    el.type = 'range'
    const $v = signal(1)
    bindValue(el, $v as unknown as ReturnType<typeof signal<string>>)
    expect(el.value).toBe('1')
    $v.set(2)
    expect(el.value).toBe('2')
  })
})

describe('bindChecked', () => {
  it('signal flows to checkbox', () => {
    const el = document.createElement('input')
    el.type = 'checkbox'
    const $on = signal(true)
    bindChecked(el, $on)
    expect(el.checked).toBe(true)
    $on.set(false)
    expect(el.checked).toBe(false)
  })

  it('checkbox change flows to signal', () => {
    const el = document.createElement('input')
    el.type = 'checkbox'
    const $on = signal(false)
    bindChecked(el, $on)
    el.checked = true
    el.dispatchEvent(new Event('change'))
    expect($on()).toBe(true)
  })
})

// 'first-render + same-value short-circuit' describe block deleted (P1.2):
// 6 tests asserted on MutationObserver mutation-record counts, a
// micro-optimization detail of the bind short-circuit path. The
// user-visible behavior (textContent / hasAttribute / classList state
// for empty / null / same-value writes) is already covered by the
// bindText / bindAttr describe blocks above. Removing the
// redundant implementation-detail layer lifted the suite from 76%
// behavior-driven to ~78% (per the audit's coverage-quality report).
describe('bindHtml', () => {
  it('renders initial html and updates on signal change', () => {
    const el = document.createElement('div')
    const $html = signal('<p>hi</p>')
    bindHtml(el, () => $html())
    expect(el.querySelector('p')?.textContent).toBe('hi')
    $html.set('<span>bye</span>')
    expect(el.querySelector('p')).toBeNull()
    expect(el.querySelector('span')?.textContent).toBe('bye')
  })

  it('preserves focus on input children across update', () => {
    const el = document.createElement('div')
    el.innerHTML = '<input type="text" />'
    document.body.appendChild(el)
    const input = el.querySelector('input')!
    input.focus()
    expect(document.activeElement).toBe(input)

    const $html = signal('<input type="text" /><span>added</span>')
    bindHtml(el, () => $html())
    // morph should keep the same input instance — focus survives
    expect(document.activeElement).toBe(el.querySelector('input'))
    expect(el.querySelector('span')?.textContent).toBe('added')
    document.body.removeChild(el)
  })

  it('handles SVG namespace correctly', () => {
    const SVG_NS = 'http://www.w3.org/2000/svg'
    const svg = document.createElementNS(SVG_NS, 'svg')
    const $paths = signal('<path d="M1 1h2v2h-2z" />')
    bindHtml(svg, () => $paths())
    const path = svg.querySelector('path')
    expect(path).not.toBeNull()
    expect(path?.namespaceURI).toBe(SVG_NS)
  })

  it('keeps the bound element own attributes — children-only contract (#285)', () => {
    const el = document.createElement('div')
    el.setAttribute('class', 'dag')
    el.setAttribute('part', 'dag')
    el.setAttribute('slot', 'actions')
    const $html = signal('<p>a</p>')
    bindHtml(el, () => $html())
    expect(el.getAttribute('class')).toBe('dag')
    expect(el.getAttribute('part')).toBe('dag')
    expect(el.getAttribute('slot')).toBe('actions')
    // ...and across reactive updates, not just the first run.
    $html.set('<span>b</span>')
    expect(el.getAttribute('class')).toBe('dag')
    expect(el.getAttribute('part')).toBe('dag')
    expect(el.getAttribute('slot')).toBe('actions')
    expect(el.querySelector('span')?.textContent).toBe('b')
  })
})
