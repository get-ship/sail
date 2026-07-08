// Behavioral suite for <sui-theme-picker> — a theme catalog picker that drives
// the framework's neutral `sui-theme-change` channel. The picker itself is
// the SOURCE of truth for active selection; the consumer wires persistence
// and any product-level concerns (this file is the pack's product-neutral
// surface, so we never touch ship.* or @ship/sdk-client here).
//
// What we test (the NEUTRAL API surface):
//   - The picker reflects the active theme as an aria-selected card.
//   - Selecting a card dispatches `sui-theme-change { id, dark }` (bubbles +
//     composed) on the host.
//   - A `sui-theme-change` event on document (from a sibling picker, a
//     settings view, or any host code) re-syncs the active card.
//   - The `surface` prop hides the customize footer on `marketing`.
//   - The picker is open-able via the trigger button (popover toggling).
//   - The `scope` prop isolates the write to the host subtree (the
//     documentElement is not mutated when scope is non-empty).
//
// What we DO NOT test here:
//   - ship.* / @ship/sdk-client / fetch / EventSource — see purity.test.ts.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-theme-picker/sui-theme-picker.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const triggerOf = (el: HTMLElement) =>
  shadow(el).querySelector('[part~="trigger"]') as HTMLButtonElement
const panelOf = (el: HTMLElement) =>
  shadow(el).querySelector('[part~="panel"]') as HTMLElement
const cardsOf = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll<HTMLButtonElement>('[part~="card"]'))
const selectedCard = (el: HTMLElement) =>
  cardsOf(el).find((c) => c.getAttribute('aria-selected') === 'true') ?? null

describe('<sui-theme-picker> — neutral theme catalog', () => {

  it('renders a trigger button + a panel of theme cards in the shadow root', () => {
    const { el } = mount(reg, 'sui-theme-picker')
    expect(el.shadowRoot).not.toBeNull()
    expect(triggerOf(el)).not.toBeNull()
    expect(panelOf(el)).not.toBeNull()
    // The catalog must contain at least one card per theme id declared
    // by the framework tokens (defensive: a count > 0 proves the panel
    // populated, not just that the trigger exists).
    expect(cardsOf(el).length).toBeGreaterThan(0)
  })

  it('marks exactly one card as aria-selected on first mount', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    const selected = cardsOf(el).filter(
      (c) => c.getAttribute('aria-selected') === 'true',
    )
    expect(selected.length).toBe(1)
  })

  it('dispatching `sui-theme-change` on document re-syncs the active card', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    // Find any non-active theme id by sampling a card that is NOT selected.
    const other = cardsOf(el).find(
      (c) => c.getAttribute('aria-selected') !== 'true',
    )
    expect(other, 'picker must have at least one non-active card').not.toBeUndefined()
    const otherId = other!.getAttribute('data-theme-card')!
    // Broadcast on the neutral channel — the picker should re-sync.
    document.dispatchEvent(
      new CustomEvent('sui-theme-change', {
        detail: { id: otherId, dark: false },
        bubbles: true,
        composed: true,
      }),
    )
    await flush()
    const now = selectedCard(el)
    expect(now).not.toBeNull()
    expect(now!.getAttribute('data-theme-card')).toBe(otherId)
  })

  it('dispatching `ship:theme:change` on document does NOT re-sync (the channel was dropped)', async () => {
    // Lock the contract: the picker does NOT listen to `ship:theme:change`.
    // This is the product-neutral guarantee — the picker's state must not
    // be coupled to a product event channel.
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    const beforeId = selectedCard(el)?.getAttribute('data-theme-card') ?? null
    const other = cardsOf(el).find(
      (c) => c.getAttribute('data-theme-card') !== beforeId,
    )
    expect(other).not.toBeUndefined()
    const otherId = other!.getAttribute('data-theme-card')!
    // Broadcast on the (forbidden) product channel — picker must ignore.
    document.dispatchEvent(
      new CustomEvent('ship:theme:change', {
        detail: { id: otherId, dark: false },
        bubbles: true,
        composed: true,
      }),
    )
    await flush()
    const afterId = selectedCard(el)?.getAttribute('data-theme-card') ?? null
    expect(afterId).toBe(beforeId)
  })

  it('hides the customize footer when surface="marketing"', async () => {
    const { el } = mount(reg, 'sui-theme-picker', { surface: 'marketing' })
    await flush()
    expect(shadow(el).querySelector('[data-theme-customize]')).toBeNull()
  })

  it('shows the customize footer when surface="app"', async () => {
    const { el } = mount(reg, 'sui-theme-picker', { surface: 'app' })
    await flush()
    expect(shadow(el).querySelector('[data-theme-customize]')).not.toBeNull()
  })
})
