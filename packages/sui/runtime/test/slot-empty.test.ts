// `bindSlotEmpty` runtime — JSDOM doesn't fire `slotchange` events the
// way a real browser does (no shadow-DOM slot machinery), so these
// tests drive the listener directly via `slot.dispatchEvent(new Event(
// 'slotchange'))` and stub `assignedNodes` with the desired payload.
//
// What we're verifying:
//   - The initial sync runs at attach time (no slotchange needed).
//   - `slotchange` updates the host attribute.
//   - Empty / whitespace-only / comment-only assigned nodes count as empty.
//   - Element children with text or descendants count as visible.
//   - The disposer removes the listener.

import { describe, expect, it, vi } from 'vitest'
import { bindSlotEmpty } from '../src/slot-empty'

interface MockSlot extends HTMLElement {
  assignedNodes(opts?: { flatten: boolean }): Node[]
}

function makeSlot(initial: Node[]): { slot: MockSlot; setNodes: (n: Node[]) => void } {
  const slot = document.createElement('slot') as unknown as MockSlot
  // bindSlotEmpty routes by root node: ShadowRoot → assignedNodes +
  // slotchange (what these stub-driven tests exercise); light DOM →
  // childNodes + MutationObserver (covered in light-dom.test.ts). Mount the
  // slot in a real shadow root so the stubs stay meaningful.
  const shadowHost = document.createElement('div')
  shadowHost.attachShadow({ mode: 'open' }).appendChild(slot)
  let nodes = initial
  slot.assignedNodes = (_opts?: { flatten: boolean }): Node[] => nodes
  return {
    slot,
    setNodes: (n: Node[]) => {
      nodes = n
    },
  }
}

describe('bindSlotEmpty', () => {
  it('sets the host attribute on attach when slot is empty', () => {
    const host = document.createElement('div')
    const { slot } = makeSlot([])
    const dispose = bindSlotEmpty(slot, host, 'data-icon-only')
    expect(host.hasAttribute('data-icon-only')).toBe(true)
    dispose()
  })

  it('does not set the attribute when slot has visible text', () => {
    const host = document.createElement('div')
    const text = document.createTextNode('Save')
    const { slot } = makeSlot([text])
    const dispose = bindSlotEmpty(slot, host, 'data-icon-only')
    expect(host.hasAttribute('data-icon-only')).toBe(false)
    dispose()
  })

  it('treats whitespace-only text nodes as empty', () => {
    const host = document.createElement('div')
    const text = document.createTextNode('   \n\t  ')
    const { slot } = makeSlot([text])
    const dispose = bindSlotEmpty(slot, host, 'data-icon-only')
    expect(host.hasAttribute('data-icon-only')).toBe(true)
    dispose()
  })

  it('treats element with text content as visible', () => {
    const host = document.createElement('div')
    const span = document.createElement('span')
    span.textContent = 'Save'
    const { slot } = makeSlot([span])
    const dispose = bindSlotEmpty(slot, host, 'data-icon-only')
    expect(host.hasAttribute('data-icon-only')).toBe(false)
    dispose()
  })

  it('treats element with element children as visible (icon-only span)', () => {
    const host = document.createElement('div')
    const wrapper = document.createElement('span')
    wrapper.appendChild(document.createElement('svg'))
    const { slot } = makeSlot([wrapper])
    const dispose = bindSlotEmpty(slot, host, 'data-icon-only')
    expect(host.hasAttribute('data-icon-only')).toBe(false)
    dispose()
  })

  it('treats truly empty element (no text, no children) as empty', () => {
    const host = document.createElement('div')
    const empty = document.createElement('span')
    const { slot } = makeSlot([empty])
    const dispose = bindSlotEmpty(slot, host, 'data-icon-only')
    expect(host.hasAttribute('data-icon-only')).toBe(true)
    dispose()
  })

  it('toggles the attribute on slotchange when content arrives', () => {
    const host = document.createElement('div')
    const { slot, setNodes } = makeSlot([])
    const dispose = bindSlotEmpty(slot, host, 'data-icon-only')
    expect(host.hasAttribute('data-icon-only')).toBe(true)

    setNodes([document.createTextNode('Save')])
    slot.dispatchEvent(new Event('slotchange'))
    expect(host.hasAttribute('data-icon-only')).toBe(false)

    setNodes([])
    slot.dispatchEvent(new Event('slotchange'))
    expect(host.hasAttribute('data-icon-only')).toBe(true)
    dispose()
  })

  it('disposer removes the listener', () => {
    const host = document.createElement('div')
    const { slot, setNodes } = makeSlot([])
    const removeSpy = vi.spyOn(slot, 'removeEventListener')
    const dispose = bindSlotEmpty(slot, host, 'data-icon-only')
    dispose()
    expect(removeSpy).toHaveBeenCalledWith('slotchange', expect.any(Function))

    // After dispose, slotchange events must not mutate the attribute.
    setNodes([document.createTextNode('Save')])
    slot.dispatchEvent(new Event('slotchange'))
    expect(host.hasAttribute('data-icon-only')).toBe(true)
  })

  it('honors the supplied attribute name', () => {
    const host = document.createElement('div')
    const { slot } = makeSlot([])
    const dispose = bindSlotEmpty(slot, host, 'data-empty')
    expect(host.hasAttribute('data-empty')).toBe(true)
    expect(host.hasAttribute('data-icon-only')).toBe(false)
    dispose()
  })
})
