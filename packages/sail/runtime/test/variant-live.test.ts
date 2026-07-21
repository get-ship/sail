// variantLive — variant-gated live region (connecting → status/polite).
//
// Ports the former use:emptyState contract onto the reusable capability
// the L1 <empty-state> stamps. Attached directly (runAction lifecycle is
// covered elsewhere).
import { describe, expect, it, afterEach } from 'vitest'
import { variantLive } from '../src/actions/variant-live'

function makeHost(attrs: Record<string, string> = {}): HTMLElement {
  const el = document.createElement('div')
  el.className = 'sail-empty-state'
  el.setAttribute('role', 'note')
  el.setAttribute('aria-live', 'off')
  for (const [k, v] of Object.entries(attrs)) {
    el.setAttribute(k, v)
  }
  document.body.append(el)
  return el
}

describe('variantLive (use: action)', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('default variant stays inert: role=note, aria-live=off', () => {
    const el = makeHost()
    variantLive(el)
    expect(el.getAttribute('role')).toBe('note')
    expect(el.getAttribute('aria-live')).toBe('off')
  })

  it('variant=connecting upgrades to status / polite', () => {
    const el = makeHost({ variant: 'connecting' })
    variantLive(el)
    expect(el.getAttribute('role')).toBe('status')
    expect(el.getAttribute('aria-live')).toBe('polite')
  })

  it('non-connecting variants (inline, placeholder) stay inert', () => {
    for (const variant of ['inline', 'placeholder', 'default'] as const) {
      const el = makeHost({ variant })
      variantLive(el)
      expect(el.getAttribute('role')).toBe('note')
      expect(el.getAttribute('aria-live')).toBe('off')
      el.remove()
    }
  })

  it('variant mutation connecting ↔ default re-syncs live attrs', async () => {
    const el = makeHost({ variant: 'default' })
    const handle = variantLive(el)
    expect(el.getAttribute('role')).toBe('note')
    expect(el.getAttribute('aria-live')).toBe('off')

    el.setAttribute('variant', 'connecting')
    // MutationObserver is microtask-async in jsdom
    await Promise.resolve()
    expect(el.getAttribute('role')).toBe('status')
    expect(el.getAttribute('aria-live')).toBe('polite')

    el.setAttribute('variant', 'inline')
    await Promise.resolve()
    expect(el.getAttribute('role')).toBe('note')
    expect(el.getAttribute('aria-live')).toBe('off')

    handle?.destroy?.()
  })

  it('destroy disconnects the observer (no further sync)', async () => {
    const el = makeHost({ variant: 'default' })
    const handle = variantLive(el)
    handle?.destroy?.()

    el.setAttribute('variant', 'connecting')
    await Promise.resolve()
    // frozen at pre-destroy values
    expect(el.getAttribute('role')).toBe('note')
    expect(el.getAttribute('aria-live')).toBe('off')
  })
})
