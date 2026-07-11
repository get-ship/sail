import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-cmdk/sui-cmdk.compiled.js'
import '../src/sui-icon/sui-icon.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'
import type { CmdkItem } from '../src/sui-cmdk/sui-cmdk.controller'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

type Cmdk = HTMLElement & {
  items: CmdkItem[]
  show(): void
  close(): void
  toggle(): void
}

const shadow = (el: HTMLElement) => el.shadowRoot!
const inputOf = (el: HTMLElement) =>
  shadow(el).querySelector('input[role="combobox"]') as HTMLInputElement
const listOf = (el: HTMLElement) =>
  shadow(el).querySelector('ul[role="listbox"]') as HTMLUListElement
const items = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('li.item')) as HTMLLIElement[]
const groups = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('li.group')) as HTMLLIElement[]
const keyOn = (inp: HTMLInputElement, key: string) =>
  inp.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }))

const FRUIT: CmdkItem[] = [
  { id: 'apple', label: 'Apple', keywords: 'red fruit', icon: 'apple' },
  { id: 'banana', label: 'Banana', value: 'banana-value', hint: '⌘B' },
  { id: 'cherry', label: 'Cherry', disabled: true },
]

/** Mount a cmdk, assign live items, open it, and settle. */
async function openWith(list: CmdkItem[]): Promise<Cmdk> {
  const { el } = mount(reg, 'sui-cmdk')
  await flush()
  ;(el as Cmdk).items = list
  ;(el as Cmdk).show()
  await flush()
  return el as Cmdk
}

describe('<sui-cmdk> — structure', () => {
  it('registers and renders the composed dialog + search input + listbox', async () => {
    const { el } = mount(reg, 'sui-cmdk')
    await flush()
    expect(customElements.get('sui-cmdk')).toBeDefined()
    expect(shadow(el).querySelector('dialog')).not.toBeNull()
    const inp = inputOf(el)
    expect(inp).not.toBeNull()
    expect(inp.getAttribute('aria-autocomplete')).toBe('list')
    const list = listOf(el)
    expect(list).not.toBeNull()
    expect(inp.getAttribute('aria-controls')).toBe(list.id)
  })

  it('parses the items="…JSON…" attribute (SSR/declarative fallback)', async () => {
    const json = JSON.stringify([{ id: 'a', label: 'Alpha' }, { id: 'b', label: 'Beta' }])
    const { el } = mount(reg, 'sui-cmdk', { items: json })
    await flush()
    expect(items(el).map((li) => li.dataset.id)).toEqual(['a', 'b'])
  })
})

describe('<sui-cmdk> — items + grouping', () => {
  it('renders one role=option row per item with value defaulting to id', async () => {
    const el = await openWith(FRUIT)
    const rows = items(el)
    expect(rows.map((li) => li.dataset.id)).toEqual(['apple', 'banana', 'cherry'])
    expect(rows.every((li) => li.getAttribute('role') === 'option')).toBe(true)
    // value defaults to id; explicit value is honored
    expect(rows[0]!.dataset.value).toBe('apple')
    expect(rows[1]!.dataset.value).toBe('banana-value')
  })

  it('inserts a heading row when the group changes between items', async () => {
    const el = await openWith([
      { id: '1', label: 'New file', group: 'Create' },
      { id: '2', label: 'New folder', group: 'Create' },
      { id: '3', label: 'Settings', group: 'Go to' },
    ])
    expect(groups(el).map((g) => g.textContent)).toEqual(['Create', 'Go to'])
    expect(items(el)).toHaveLength(3)
  })

  it('marks disabled rows aria-disabled and skips them from selection', async () => {
    const el = await openWith(FRUIT)
    const cherry = items(el).find((li) => li.dataset.id === 'cherry')!
    expect(cherry.getAttribute('aria-disabled')).toBe('true')
  })
})

describe('<sui-cmdk> — filtering', () => {
  it('filters case-insensitively over label and keywords', async () => {
    const el = await openWith(FRUIT)
    const inp = inputOf(el)
    inp.value = 'red'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    // "red" only appears in Apple's keywords
    expect(items(el).map((li) => li.dataset.id)).toEqual(['apple'])
  })

  it('shows the empty-label when nothing matches', async () => {
    const { el } = mount(reg, 'sui-cmdk', { 'empty-label': 'Nada.' })
    await flush()
    ;(el as Cmdk).items = FRUIT
    ;(el as Cmdk).show()
    await flush()
    const inp = inputOf(el)
    inp.value = 'zzzzz'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(items(el)).toHaveLength(0)
    expect(shadow(el).querySelector('li.empty')?.textContent).toBe('Nada.')
  })
})

describe('<sui-cmdk> — keyboard navigation', () => {
  it('ArrowDown/ArrowUp move aria-activedescendant over ENABLED rows with wrap', async () => {
    const el = await openWith(FRUIT) // apple, banana enabled; cherry disabled
    const inp = inputOf(el)
    const activeId = () => inp.getAttribute('aria-activedescendant')
    const idOf = (suffix: string) => items(el).find((li) => li.dataset.id === suffix)!.id
    expect(activeId()).toBe(idOf('apple'))
    keyOn(inp, 'ArrowDown')
    await flush()
    expect(activeId()).toBe(idOf('banana'))
    keyOn(inp, 'ArrowDown') // wraps past disabled cherry back to apple
    await flush()
    expect(activeId()).toBe(idOf('apple'))
    keyOn(inp, 'ArrowUp') // wraps to last enabled = banana
    await flush()
    expect(activeId()).toBe(idOf('banana'))
  })

  it('highlighted row carries aria-selected=true; disabled never selected', async () => {
    const el = await openWith(FRUIT)
    const selected = () =>
      items(el)
        .filter((li) => li.getAttribute('aria-selected') === 'true')
        .map((li) => li.dataset.id)
    expect(selected()).toEqual(['apple'])
    keyOn(inputOf(el), 'End')
    await flush()
    // End jumps to the last ENABLED row (banana), not disabled cherry
    expect(selected()).toEqual(['banana'])
  })
})

describe('<sui-cmdk> — selection + events', () => {
  it('Enter on the highlight emits sui-cmdk-select {id,value} and closes', async () => {
    const el = await openWith(FRUIT)
    let detail: { id: string; value: string } | null = null
    el.addEventListener('sui-cmdk-select', (e) => {
      detail = (e as CustomEvent).detail
    })
    keyOn(inputOf(el), 'ArrowDown') // highlight banana
    await flush()
    keyOn(inputOf(el), 'Enter')
    await flush()
    expect(detail).toEqual({ id: 'banana', value: 'banana-value' })
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('mousedown on a row commits the same payload; disabled rows do not', async () => {
    const el = await openWith(FRUIT)
    let detail: { id: string; value: string } | null = null
    el.addEventListener('sui-cmdk-select', (e) => {
      detail = (e as CustomEvent).detail
    })
    const cherry = items(el).find((li) => li.dataset.id === 'cherry')!
    cherry.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }))
    await flush()
    expect(detail).toBeNull() // disabled — no commit
    const apple = items(el).find((li) => li.dataset.id === 'apple')!
    apple.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }))
    await flush()
    expect(detail).toEqual({ id: 'apple', value: 'apple' })
  })

  it('sui-cmdk-input fires with the query on each keystroke', async () => {
    const el = await openWith(FRUIT)
    const seen: string[] = []
    el.addEventListener('sui-cmdk-input', (e) => seen.push((e as CustomEvent).detail.query))
    const inp = inputOf(el)
    inp.value = 'ba'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(seen).toEqual(['ba'])
  })
})

describe('<sui-cmdk> — open/close API', () => {
  it('open()/close()/toggle() drive the reflected open attribute + lifecycle events', async () => {
    const { el } = mount(reg, 'sui-cmdk')
    await flush()
    const opens: number[] = []
    const closes: number[] = []
    el.addEventListener('sui-cmdk-open', () => opens.push(1))
    el.addEventListener('sui-cmdk-close', () => closes.push(1))
    ;(el as Cmdk).show()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    expect(opens).toHaveLength(1)
    ;(el as Cmdk).close()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(closes).toHaveLength(1)
    ;(el as Cmdk).toggle()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    expect(opens).toHaveLength(2)
  })
})
