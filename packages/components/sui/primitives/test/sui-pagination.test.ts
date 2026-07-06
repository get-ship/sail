// <sui-pagination> — behavioral specs (TDD-first).
//
// The pagination bar composes bare <button>s (first / prev / page-numbers /
// next / last — dissolved from <sui-button>) + sui-select (page-size) and
// renders an "N–M of T" range label. It is a CONTROLLED, presentational
// primitive: it never owns the page — it reflects the `page` / `page-size` /
// `total` props it is given and EMITS the requested transition as
// `sui-page-change { page, pageSize }` (bubbles + composed). The consumer
// (e.g. the sui-table footer) applies the change and feeds the new props back.
//
// The child compiled modules are imported so the real `<select>` chrome
// renders — the change assertions read that.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-pagination/sui-pagination.compiled.js'
import '../src/sui-select/sui-select.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

/** All nav control buttons keyed by their `data-page-action`. */
const action = (el: HTMLElement, name: string): HTMLElement | null =>
  shadow(el).querySelector(`[data-page-action="${name}"]`)

/** The numbered page buttons (each carries `data-page="<n>"`). */
const pageButtons = (el: HTMLElement): HTMLElement[] =>
  Array.from(shadow(el).querySelectorAll('[data-page]'))

/** A control reads as disabled when the bare <button> carries the reflected
 *  `disabled` attribute the controller drives via `disabled={…}`. */
const isDisabled = (btn: HTMLElement | null): boolean =>
  btn != null &&
  (btn.hasAttribute('disabled') || (btn as { disabled?: boolean }).disabled === true)

describe('<sui-pagination>', () => {

  it('renders the four edge controls (first / prev / next / last)', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 50, 'page-size': 10 })
    await flush()
    expect(action(el, 'first')).not.toBeNull()
    expect(action(el, 'prev')).not.toBeNull()
    expect(action(el, 'next')).not.toBeNull()
    expect(action(el, 'last')).not.toBeNull()
  })

  // ── Range label "N–M of T" ────────────────────────────────────────────────
  it('renders an "N–M of T" range label for the current page', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 50, 'page-size': 10 })
    await flush()
    const label = shadow(el).querySelector('[part~="range"]')!
    // Page 1 of 10/page over 50 → items 1–10 of 50.
    expect(label.textContent).toMatch(/1\s*[–-]\s*10\s+of\s+50/)
  })

  it('clamps the range end to the total on the final, partial page', async () => {
    // 7 items, 3/page → page 3 shows item 7 only: "7–7 of 7".
    const { el } = mount(reg, 'sui-pagination', { page: 3, total: 7, 'page-size': 3 })
    await flush()
    const label = shadow(el).querySelector('[part~="range"]')!
    expect(label.textContent).toMatch(/7\s*[–-]\s*7\s+of\s+7/)
  })

  it('reads zero total as an empty range ("0 of 0")', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 0, 'page-size': 10 })
    await flush()
    const label = shadow(el).querySelector('[part~="range"]')!
    expect(label.textContent).toMatch(/0\s+of\s+0/)
  })

  // ── Edge disabling: first/prev on page 1, next/last on the last page ───────
  it('disables first + prev on page 1', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 50, 'page-size': 10 })
    await flush()
    expect(isDisabled(action(el, 'first'))).toBe(true)
    expect(isDisabled(action(el, 'prev'))).toBe(true)
    expect(isDisabled(action(el, 'next'))).toBe(false)
    expect(isDisabled(action(el, 'last'))).toBe(false)
  })

  it('disables next + last on the final page', async () => {
    // 50 items, 10/page → 5 pages; page 5 is the last.
    const { el } = mount(reg, 'sui-pagination', { page: 5, total: 50, 'page-size': 10 })
    await flush()
    expect(isDisabled(action(el, 'next'))).toBe(true)
    expect(isDisabled(action(el, 'last'))).toBe(true)
    expect(isDisabled(action(el, 'first'))).toBe(false)
    expect(isDisabled(action(el, 'prev'))).toBe(false)
  })

  it('disables all four edges when there is a single page', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 4, 'page-size': 10 })
    await flush()
    expect(isDisabled(action(el, 'first'))).toBe(true)
    expect(isDisabled(action(el, 'prev'))).toBe(true)
    expect(isDisabled(action(el, 'next'))).toBe(true)
    expect(isDisabled(action(el, 'last'))).toBe(true)
  })

  // ── Page-number buttons + aria-current ────────────────────────────────────
  it('renders one numbered button per page (small page counts)', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 2, total: 50, 'page-size': 10 })
    await flush()
    const nums = pageButtons(el).map((b) => b.getAttribute('data-page'))
    expect(nums).toEqual(['1', '2', '3', '4', '5'])
  })

  it('marks the active page button with aria-current="page"', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 3, total: 50, 'page-size': 10 })
    await flush()
    const current = pageButtons(el).filter(
      (b) => b.getAttribute('aria-current') === 'page',
    )
    expect(current.length).toBe(1)
    expect(current[0]!.getAttribute('data-page')).toBe('3')
  })

  // ── Emission: sui-page-change { page, pageSize } ──────────────────────────
  it('emits sui-page-change { page, pageSize } when next is clicked (bubbles + composed)', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 50, 'page-size': 10 })
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-page-change', (e) => { evt = e as CustomEvent })
    action(el, 'next')!.click()
    expect(evt).not.toBeNull()
    expect(evt!.detail).toEqual({ page: 2, pageSize: 10 })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('emits page 1 on first and the last page on last', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 3, total: 50, 'page-size': 10 })
    await flush()
    const seen: number[] = []
    el.addEventListener('sui-page-change', (e) => {
      seen.push((e as CustomEvent).detail.page)
    })
    action(el, 'first')!.click()
    action(el, 'last')!.click()
    expect(seen).toEqual([1, 5]) // 5 pages over 50/10
  })

  it('emits the clicked page number from a numbered button', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 50, 'page-size': 10 })
    await flush()
    let detail: { page: number; pageSize: number } | null = null
    el.addEventListener('sui-page-change', (e) => {
      detail = (e as CustomEvent).detail
    })
    const four = pageButtons(el).find((b) => b.getAttribute('data-page') === '4')!
    four.click()
    expect(detail!).toEqual({ page: 4, pageSize: 10 })
  })

  it('does not emit when a disabled edge is clicked (prev on page 1)', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 50, 'page-size': 10 })
    await flush()
    let fired = false
    el.addEventListener('sui-page-change', () => { fired = true })
    action(el, 'prev')!.click()
    expect(fired).toBe(false)
  })

  it('sui-page-change bubbles + is composed (escapes the host)', async () => {
    const { el, container } = mount(reg, 'sui-pagination', {
      page: 1,
      total: 50,
      'page-size': 10,
    })
    await flush()
    let outside = false
    container.addEventListener('sui-page-change', () => { outside = true })
    action(el, 'next')!.click()
    expect(outside).toBe(true)
  })

  // ── Page-size select ──────────────────────────────────────────────────────
  it('composes a sui-select seeded from page-size-options with the active size', async () => {
    const { el } = mount(reg, 'sui-pagination', {
      page: 1,
      total: 200,
      'page-size': 25,
      'page-size-options': '10,25,50,100',
    })
    await flush()
    const select = shadow(el).querySelector('sui-select') as
      | (HTMLElement & { value: string })
      | null
    expect(select).not.toBeNull()
    const opts = Array.from(select!.querySelectorAll('option')).map((o) => o.value)
    expect(opts).toEqual(['10', '25', '50', '100'])
    // sui-select's `value` is a form-value property (not a reflected attribute);
    // prop:value drives the property, which is what we assert.
    expect(select!.value).toBe('25')
  })

  it('emits sui-page-change with the new pageSize (and resets to page 1) on size change', async () => {
    const { el } = mount(reg, 'sui-pagination', {
      page: 3,
      total: 200,
      'page-size': 25,
      'page-size-options': '10,25,50,100',
    })
    await flush()
    let detail: { page: number; pageSize: number } | null = null
    el.addEventListener('sui-page-change', (e) => {
      detail = (e as CustomEvent).detail
    })
    const select = shadow(el).querySelector('sui-select') as HTMLElement & { value: string }
    // Drive the sui-select's emitted sui-change (the public select event).
    select.dispatchEvent(
      new CustomEvent('sui-change', { detail: { value: '50' }, bubbles: true, composed: true }),
    )
    expect(detail).not.toBeNull()
    // A page-size change resets to page 1 so the user is never stranded past
    // the new last page.
    expect(detail!).toEqual({ page: 1, pageSize: 50 })
  })

  it('hides the page-size select when no options are given', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: 1, total: 50, 'page-size': 10 })
    await flush()
    expect(shadow(el).querySelector('sui-select')).toBeNull()
  })
})
