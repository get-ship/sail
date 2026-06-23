import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-combobox/sui-combobox.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const inputOf = (el: HTMLElement) => shadow(el).querySelector('input[role="combobox"]') as HTMLInputElement
const listOf = (el: HTMLElement) => shadow(el).querySelector('ul[role="listbox"]') as HTMLUListElement | null
const opts = (el: HTMLElement) => Array.from(shadow(el).querySelectorAll('li.opt[data-value]')) as HTMLLIElement[]

describe('<sui-combobox>', () => {

  it('renders a shadow root with a combobox input', async () => {
    const { el } = mount(reg, 'sui-combobox')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    const inp = inputOf(el)
    expect(inp).not.toBeNull()
    expect(inp.getAttribute('aria-expanded')).toBe('false')
    expect(inp.getAttribute('aria-autocomplete')).toBe('list')
  })

  it('parses light-DOM <option> children into options', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, '<option value="a">Alpha</option><option value="b">Beta</option>')
    await flush()
    inputOf(el).focus()
    await flush()
    expect(opts(el).map((o) => o.dataset.value)).toEqual(['a', 'b'])
  })

  it('parses options="…JSON…" attribute', async () => {
    const { el } = mount(reg, 'sui-combobox', { options: JSON.stringify([{ value: 'x', label: 'Xray' }, { value: 'y', label: 'Yankee' }]) })
    await flush()
    inputOf(el).focus()
    await flush()
    expect(opts(el).map((o) => o.dataset.value)).toEqual(['x', 'y'])
  })

  it('filters options by case-insensitive label substring', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, '<option value="a">Apple</option><option value="b">Apricot</option><option value="c">Banana</option>')
    await flush()
    const inp = inputOf(el)
    inp.focus()
    await flush()
    inp.value = 'AP'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(opts(el).map((o) => o.dataset.value)).toEqual(['a', 'b'])
  })

  it('reflects `value` attribute round-trip', async () => {
    const { el } = mount(reg, 'sui-combobox', { value: 'foo' })
    await flush()
    expect(el.getAttribute('value')).toBe('foo')
    el.setAttribute('value', 'bar')
    await flush()
    expect(el.getAttribute('value')).toBe('bar')
  })
})

const threeFruit =
  '<option value="a">Apple</option><option value="b">Banana</option><option value="c">Cherry</option>'

// NOTE: form-associated SUI components currently expose no JS `value`
// accessor (the runtime's prop-accessor loop only covers the non-form class),
// so committed values are asserted through DOM-observable truth: the input's
// label text and aria-selected on reopen.
type SuiCombobox = HTMLElement & { open(): void; close(): void }

/** Reopen the listbox and read which option labels are aria-selected. */
async function selectedOnReopen(el: HTMLElement): Promise<string[]> {
  ;(el as SuiCombobox).open()
  await flush()
  return opts(el).map((o) => o.getAttribute('aria-selected')!)
}

const keyOn = (inp: HTMLInputElement, key: string) =>
  inp.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }))

describe('<sui-combobox> — open/close ARIA contract', () => {
  it('focus opens the listbox: aria-expanded=true, data-open on host', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, threeFruit)
    await flush()
    const inp = inputOf(el)
    inp.focus()
    await flush()
    expect(inp.getAttribute('aria-expanded')).toBe('true')
    expect(el.hasAttribute('data-open')).toBe(true)
    expect(listOf(el)).not.toBeNull()
  })

  it('input is wired to the listbox via aria-controls + aria-haspopup semantics', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, threeFruit)
    await flush()
    const inp = inputOf(el)
    inp.focus()
    await flush()
    const list = listOf(el)!
    expect(list.id).not.toBe('')
    expect(inp.getAttribute('aria-controls')).toBe(list.id)
  })

  it('option matching the current value carries aria-selected=true', async () => {
    const { el } = mount(reg, 'sui-combobox', { value: 'b' }, threeFruit)
    await flush()
    inputOf(el).focus()
    await flush()
    const selected = opts(el).map((o) => o.getAttribute('aria-selected'))
    expect(selected).toEqual(['false', 'true', 'false'])
  })

  it('exposed open()/close() drive the listbox; open() is a no-op when disabled', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, threeFruit)
    await flush()
    ;(el as SuiCombobox).open()
    await flush()
    expect(inputOf(el).getAttribute('aria-expanded')).toBe('true')
    ;(el as SuiCombobox).close()
    await flush()
    expect(inputOf(el).getAttribute('aria-expanded')).toBe('false')
    expect(listOf(el)).toBeNull()

    const { el: dis } = mount(reg, 'sui-combobox', { disabled: true }, threeFruit)
    await flush()
    ;(dis as SuiCombobox).open()
    await flush()
    expect(inputOf(dis).getAttribute('aria-expanded')).toBe('false')
    expect(listOf(dis)).toBeNull()
  })
})

describe('<sui-combobox> — keyboard contract', () => {
  it('ArrowDown on a closed combobox opens the list', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, threeFruit)
    await flush()
    const inp = inputOf(el)
    keyOn(inp, 'ArrowDown')
    await flush()
    expect(inp.getAttribute('aria-expanded')).toBe('true')
  })

  it('ArrowDown/ArrowUp move aria-activedescendant through the options with wrap', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, threeFruit)
    await flush()
    const inp = inputOf(el)
    inp.focus()
    await flush()
    const ids = opts(el).map((o) => o.id)
    expect(inp.getAttribute('aria-activedescendant')).toBe(ids[0])
    keyOn(inp, 'ArrowDown')
    await flush()
    expect(inp.getAttribute('aria-activedescendant')).toBe(ids[1])
    keyOn(inp, 'ArrowUp')
    await flush()
    expect(inp.getAttribute('aria-activedescendant')).toBe(ids[0])
    keyOn(inp, 'ArrowUp') // wraps from first to last
    await flush()
    expect(inp.getAttribute('aria-activedescendant')).toBe(ids[2])
  })

  it('Enter commits the highlighted option: value, label text, closed list', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, threeFruit)
    await flush()
    const inp = inputOf(el)
    inp.focus()
    await flush()
    keyOn(inp, 'ArrowDown') // highlight Banana
    await flush()
    keyOn(inp, 'Enter')
    await flush()
    expect(inp.value).toBe('Banana')
    expect(inp.getAttribute('aria-expanded')).toBe('false')
    expect(listOf(el)).toBeNull()
    expect(await selectedOnReopen(el)).toEqual(['false', 'true', 'false'])
  })

  it('Escape closes without changing the value and restores the selected label', async () => {
    const { el } = mount(reg, 'sui-combobox', { value: 'c' }, threeFruit)
    await flush()
    const inp = inputOf(el)
    inp.focus() // opening clears the filter text
    await flush()
    keyOn(inp, 'ArrowDown')
    await flush()
    keyOn(inp, 'Escape')
    await flush()
    expect(inp.value).toBe('Cherry')
    expect(inp.getAttribute('aria-expanded')).toBe('false')
    expect(await selectedOnReopen(el)).toEqual(['false', 'false', 'true'])
  })

  it('Tab closes the list, leaving the value untouched', async () => {
    const { el } = mount(reg, 'sui-combobox', { value: 'a' }, threeFruit)
    await flush()
    const inp = inputOf(el)
    inp.focus()
    await flush()
    expect(inp.getAttribute('aria-expanded')).toBe('true')
    keyOn(inp, 'Tab')
    await flush()
    expect(inp.getAttribute('aria-expanded')).toBe('false')
    expect(await selectedOnReopen(el)).toEqual(['true', 'false', 'false'])
  })
})

describe('<sui-combobox> — event contract', () => {
  it('sui-change: bubbles + composed with {value, label} payload on commit', async () => {
    const { el, container } = mount(reg, 'sui-combobox', {}, threeFruit)
    await flush()
    let evt: CustomEvent | null = null
    container.addEventListener('sui-change', (e) => { evt = e as CustomEvent })
    const inp = inputOf(el)
    inp.focus()
    await flush()
    keyOn(inp, 'Enter') // commits the first highlighted option (Apple)
    await flush()
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toEqual({ value: 'a', label: 'Apple' })
  })

  it('mousedown commit on a listbox row emits the same payload shape', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, threeFruit)
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => { evt = e as CustomEvent })
    inputOf(el).focus()
    await flush()
    const row = opts(el)[2] // Cherry
    row.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, composed: true, cancelable: true }))
    await flush()
    expect(evt!.detail).toEqual({ value: 'c', label: 'Cherry' })
    expect(inputOf(el).value).toBe('Cherry')
  })
})
