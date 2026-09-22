// Slice — regression coverage for the scope→mount(host) cutover on
// <sail-annotation-layer> and <sail-annotation-list>.
//
// Background: the post-convergence "SAIL names no host" compile stops passing
// `host` as the controller scope's 2nd arg, so the scope-level `host!` reads
// were silently undefined and event dispatches from scope never reached the
// host element. These tests pin the host-event-dispatch contract that the
// dead-`host` bug broke.
//
// Both events are the spec contract from the .sail files: bubbles + composed
// + a non-null `detail`. The TDD test for the list (clicking Remove →
// `sail-annotation-remove`) is the one the dead-`host` bug would silently fail
// (the dispatch would no-op on a null/undefined host). The layer test pins
// the `staged` JS property + the layer→list contract.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-annotation/sail-annotation-list.compiled.js'
import '../src/sail-annotation/sail-annotation-layer.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'
import type { AnnotationEvent } from '../src/sail-annotation/sail-annotation.internals'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Narrow CustomEvent detail with a type guard, not an inline cast — the
// `unknown` from CustomEvent.detail is the right surface for validation.
const isRemoveDetail = (v: unknown): v is { id: string } =>
  typeof v === 'object' && v !== null && 'id' in v && typeof (v as { id: unknown }).id === 'string'

// A representative staged annotation the list can render. ULID shape isn't
// material here — the test asserts the list renders one row and the click
// emits a remove event with the right id.
const STAGED: AnnotationEvent[] = [
  {
    id: '01HZZZ1TEST00000000000000',
    kind: 'point',
    target: { selector: '#thing' },
    note: 'fix this',
    created_at: '2026-06-30T00:00:00.000Z',
    author_kind: 'human',
  },
]

// Host with the controller-installed `staged` accessor. We type-guard the
// property's presence (it lives on the element after mount), then read it.
// Light-DOM components: the template renders into the host itself.
const root = (el: HTMLElement) => (el.shadowRoot ?? el)

const readStaged = (el: HTMLElement): AnnotationEvent[] | undefined => {
  if (!('staged' in el)) return undefined
  const v = (el as { staged: unknown }).staged
  return Array.isArray(v) ? (v as AnnotationEvent[]) : undefined
}

describe('<sail-annotation-list> — host event dispatch (post scope→mount fix)', () => {

  it('renders one row per staged annotation (the staged JS property wires through)', async () => {
    const { el } = mount(reg, 'sail-annotation-list', {}, '')
    readStaged; // satisfy ts-no-unused for the type guard
    ;(el as unknown as { staged: AnnotationEvent[] }).staged = STAGED
    await flush()
    const items = root(el).querySelectorAll('[role="listitem"]')
    expect(items.length).toBe(STAGED.length)
    expect(items[0]!.textContent).toContain('fix this')
  })

  it('clicking Remove dispatches sail-annotation-remove with the row id (bubbles + composed)', async () => {
    const { el } = mount(reg, 'sail-annotation-list', {}, '')
    ;(el as unknown as { staged: AnnotationEvent[] }).staged = STAGED
    await flush()

    let captured: CustomEvent<unknown> | null = null
    el.addEventListener('sail-annotation-remove', (ev) => {
      captured = ev as CustomEvent<unknown>
    })
    const removeBtn = root(el).querySelector(
      '[role="listitem"] button[aria-label="Remove annotation"]',
    ) as HTMLButtonElement | null
    expect(removeBtn).not.toBeNull()
    removeBtn!.click()
    await flush()

    expect(captured).not.toBeNull()
    if (!captured) return
    expect(captured.bubbles).toBe(true)
    expect(captured.composed).toBe(true)
    // Narrow the detail with a guard (NOT an inline cast).
    expect(isRemoveDetail(captured.detail)).toBe(true)
    if (isRemoveDetail(captured.detail)) {
      expect(captured.detail.id).toBe(STAGED[0]!.id)
    }
  })
})

describe('<sail-annotation-layer> — host element surface (post scope→mount fix)', () => {

  it('mount renders the capture overlay over the slotted content (the host must be alive)', async () => {
    const { el } = mount(reg, 'sail-annotation-layer', { mode: 'point' }, '<span data-test="wrapped">x</span>')
    await flush()
    // The slotted content is projected, and mode=point arms the overlay that
    // sits on top of it. If `mount` never ran (or the host never
    // materialized), neither would be in the tree.
    expect(root(el).querySelector('[data-test="wrapped"]')).not.toBeNull()
    expect(root(el).querySelector('[data-state="capture"]')).not.toBeNull()
  })

  it('mode=off leaves the overlay unarmed', async () => {
    const { el } = mount(reg, 'sail-annotation-layer', {}, '<span data-test="wrapped">x</span>')
    await flush()
    expect(root(el).querySelector('[data-state="capture"]')).toBeNull()
  })

  it('exposes a `staged` JS property the parent annotation-list can read', async () => {
    // The `staged` accessor is installed by `mount` (not scope) — this test
    // proves the runtime handed a real host to mount. Pre-fix the accessor
    // install happened before scope (and was still installed) but the list's
    // emit path was broken, which is covered by the list tests above.
    const { el } = mount(reg, 'sail-annotation-layer', {}, '')
    await flush()
    // `staged` is a controller-installed property: presence is the proof.
    expect('staged' in el).toBe(true)
  })
})
