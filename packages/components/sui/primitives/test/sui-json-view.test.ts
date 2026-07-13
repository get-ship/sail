import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-json-view/sui-json-view.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)

describe('<sui-json-view>', () => {

  it('renders string, number, boolean, and null primitives', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '{"a":1,"b":true,"c":null,"d":"hi"}' })
    await flush()
    const txt = shadow(el).textContent!
    expect(txt).toContain('"a"')
    expect(txt).toContain('1')
    expect(txt).toContain('true')
    expect(txt).toContain('null')
    expect(txt).toContain('"hi"')
  })

  it('shows error message for invalid JSON', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '{bad' })
    await flush()
    expect(shadow(el).textContent).toContain('Invalid JSON')
  })

  it('error element has role=alert', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: 'bad' })
    await flush()
    expect(shadow(el).querySelector('[role="alert"]')).not.toBeNull()
  })

  it('renders nested objects as expandable treeitem rows', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '{"outer":{"inner":1}}', depth: '2' })
    await flush()
    expect(shadow(el).querySelector('[role="treeitem"][aria-expanded]')).not.toBeNull()
  })

  it('respects depth — branches at initial depth are open', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '{"a":{"b":1}}', depth: '1' })
    await flush()
    expect(shadow(el).querySelectorAll('[aria-expanded="true"]').length).toBeGreaterThan(0)
  })

  it('expanded=true opens every branch', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '{"a":{"b":{"c":1}}}', expanded: true, depth: '0' })
    await flush()
    expect(shadow(el).querySelectorAll('[aria-expanded]').length).toBeGreaterThan(0)
    expect(shadow(el).querySelectorAll('[aria-expanded="false"]').length).toBe(0)
  })

  it('renders empty object as a leaf, not a branch', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '{}' })
    await flush()
    const txt = shadow(el).textContent!
    expect(txt).toContain('{}')
    expect(shadow(el).querySelector('[aria-expanded]')).toBeNull()
  })

  it('reacts to data attribute change', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '"first"' })
    await flush()
    expect(shadow(el).textContent).toContain('"first"')
    el.setAttribute('data', '"second"')
    await flush()
    expect(shadow(el).textContent).toContain('"second"')
  })

  it('host has role=tree', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '{"x":1}' })
    await flush()
    expect(el.getAttribute('role')).toBe('tree')
  })

  it('host has aria-label="JSON tree"', async () => {
    const { el } = mount(reg, 'sui-json-view', { data: '{"x":1}' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('JSON tree')
  })

  it('empty data renders nothing', async () => {
    const { el } = mount(reg, 'sui-json-view')
    await flush()
    expect(shadow(el).querySelector('[part="root"]')).toBeNull()
  })
})
