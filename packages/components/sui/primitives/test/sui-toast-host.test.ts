import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-toast-host/sui-toast-host.compiled.js'
import '../src/sui-toast/sui-toast-container.compiled.js'
import '../src/sui-toast/sui-toast.compiled.js'
import '../src/sui-portal/sui-portal.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()

afterEach(() => {
  reg.cleanup()
  for (const stack of Array.from(
    document.body.querySelectorAll('[data-sui-toast-stack]'),
  )) {
    stack.parentElement?.remove()
  }
})

const dispatchNotify = (
  target: EventTarget,
  detail: Record<string, unknown> = {},
) => {
  target.dispatchEvent(
    new CustomEvent('ship:notify', {
      detail,
      bubbles: true,
      composed: true,
    }),
  )
}

describe('<sui-toast-host>', () => {

  it('renders an inner <sui-toast-container> with forwarded position', async () => {
    const { el } = mount(reg, 'sui-toast-host', { position: 'top-left' })
    await flush()
    const container = el.shadowRoot!.querySelector('sui-toast-container')
    expect(container).not.toBeNull()
    expect(container!.getAttribute('position')).toBe('top-left')
  })

  it('default position bottom-right propagates to the inner container', async () => {
    const { el } = mount(reg, 'sui-toast-host')
    await flush()
    const container = el.shadowRoot!.querySelector('sui-toast-container')!
    expect(container.getAttribute('position')).toBe('bottom-right')
  })

  it('listens for ship:notify on document.body when no shell ancestor', async () => {
    const { el } = mount(reg, 'sui-toast-host')
    await flush()
    const container = el.shadowRoot!.querySelector(
      'sui-toast-container',
    ) as HTMLElement & {
      show: (opts: { title?: string; tone?: string }) => HTMLElement
    }
    // sanity: programmatic API is available on the container
    expect(typeof container.show).toBe('function')

    dispatchNotify(document.body, {
      message: 'Saved',
      tone: 'success',
      description: 'All changes synced.',
      duration: 0,
    })
    await flush()

    const stack = document.body.querySelector('[data-sui-toast-stack]')
    expect(stack).not.toBeNull()
    const toast = stack!.querySelector('sui-toast')!
    expect(toast.getAttribute('tone')).toBe('success')
    expect(toast.getAttribute('duration')).toBe('0')
    expect(toast.textContent).toContain('Saved')
    expect(toast.textContent).toContain('All changes synced.')
  })

  it('forwards actionHtml into the toast actions slot', async () => {
    const { el } = mount(reg, 'sui-toast-host')
    await flush()
    dispatchNotify(document.body, {
      message: 'Update available',
      actionHtml: '<button id="upd-act">Reload</button>',
    })
    await flush()
    const stack = document.body.querySelector('[data-sui-toast-stack]')!
    const toast = stack.querySelector('sui-toast')!
    // The action markup lands as a real, live element inside the toast — assert
    // the parsed button (id + label), not a raw HTML substring.
    const action = toast.querySelector('button')!
    expect(action).not.toBeNull()
    expect(action.id).toBe('upd-act')
    expect(action.textContent).toBe('Reload')
    // …and it must reach the *actions* slot, not the default-slot fallback.
    // A forwarded action that lost its slot="actions" would still satisfy the
    // querySelector above but render in the body and leave the toast marked
    // data-no-actions. Assert the live slot assignment instead.
    const actionsSlot = toast.shadowRoot!.querySelector(
      'slot[name="actions"]',
    ) as HTMLSlotElement
    const assigned = actionsSlot.assignedElements()
    expect(assigned.some((node) => node === action || node.contains(action))).toBe(
      true,
    )
    expect(toast.hasAttribute('data-no-actions')).toBe(false)
  })

  it('listens on the closest ship-shell ancestor when present', async () => {
    if (!customElements.get('ship-shell')) {
      class StubShell extends HTMLElement {}
      customElements.define('ship-shell', StubShell)
    }
    const container = reg.mk()
    container.innerHTML =
      '<ship-shell><sui-toast-host></sui-toast-host></ship-shell>'
    const shell = container.querySelector('ship-shell')!
    await flush()

    dispatchNotify(shell, { message: 'Hello shell', tone: 'info' })
    await flush()

    const stack = document.body.querySelector('[data-sui-toast-stack]')!
    expect(stack.querySelector('sui-toast')!.textContent).toContain(
      'Hello shell',
    )
  })

  it('removes the ship:notify listener on disconnect', async () => {
    const { el } = mount(reg, 'sui-toast-host')
    await flush()
    el.remove()
    await flush()

    dispatchNotify(document.body, { message: 'after-remove' })
    await flush()

    // The disconnected host's ship:notify listener is gone, so no toast was
    // created after remove — regardless of whether a stack element lingers.
    expect(document.body.querySelector('[data-sui-toast-stack] sui-toast')).toBeNull()
  })
})
