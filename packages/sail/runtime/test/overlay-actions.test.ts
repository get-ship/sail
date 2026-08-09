// Overlay family: live trigger bind (hover / context-menu) + combobox picker-first.
// Catalog-shaped markup: trigger sibling of panel; panel action may mount first.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { combobox } from '../src/actions/combobox'
import { contextMenu } from '../src/actions/context-menu'
import { hoverSurface } from '../src/actions/hover-surface'
import { anchor } from '../src/actions/anchor'
import { initJsdomPolyfills } from '../../test-setup/src/jsdom-polyfills'

initJsdomPolyfills()

afterEach(() => {
  document.body.replaceChildren()
  vi.useRealTimers()
})

/** Drain MutationObserver + the action's rAF-coalesced rebind. */
async function flushMo(): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
  if (vi.isFakeTimers()) {
    await vi.advanceTimersByTimeAsync(0)
    await vi.advanceTimersByTimeAsync(16)
  } else {
    await new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r())))
  }
}

function pointerEnter(el: EventTarget): void {
  el.dispatchEvent(new PointerEvent('pointerenter', { bubbles: true }))
}

describe('use:hoverSurface — live trigger bind', () => {
  it('opens on pointerenter after a late-mounted invoker (catalog mount order)', async () => {
    vi.useFakeTimers()
    const card = document.createElement('div')
    card.id = 'grid-hc'
    card.setAttribute('popover', '')
    card.setAttribute('open-delay', '0')
    card.setAttribute('close-delay', '0')
    document.body.append(card)

    const handle = hoverSurface(card) as { destroy: () => void }
    // Panel mounted first — no trigger yet (one-shot would stay dead).
    const show = vi.fn()
    ;(card as HTMLElement & { showPopover: () => void }).showPopover = show

    const trig = document.createElement('button')
    trig.setAttribute('popovertarget', 'grid-hc')
    trig.textContent = '@ada'
    document.body.append(trig)
    await flushMo()

    pointerEnter(trig)
    vi.runAllTimers()
    expect(show).toHaveBeenCalled()

    handle.destroy()
  })

  it('rebinds when popovertarget appears on an existing node', async () => {
    vi.useFakeTimers()
    const card = document.createElement('div')
    card.id = 'hc2'
    card.setAttribute('open-delay', '0')
    document.body.append(card)
    const show = vi.fn()
    ;(card as HTMLElement & { showPopover: () => void }).showPopover = show

    const handle = hoverSurface(card) as { destroy: () => void }
    const trig = document.createElement('span')
    document.body.append(trig)
    await flushMo()

    pointerEnter(trig)
    vi.runAllTimers()
    expect(show).not.toHaveBeenCalled()

    trig.setAttribute('popovertarget', 'hc2')
    await flushMo()
    pointerEnter(trig)
    vi.runAllTimers()
    expect(show).toHaveBeenCalled()

    handle.destroy()
  })
})

describe('use:contextMenu — live for-target bind', () => {
  it('opens on contextmenu of a late-mounted for-target', async () => {
    vi.useFakeTimers()
    const menu = document.createElement('div')
    menu.id = 'grid-cm'
    menu.setAttribute('popover', '')
    menu.setAttribute('for', 'grid-cm-target')
    document.body.append(menu)

    const show = vi.fn()
    const hide = vi.fn()
    ;(menu as HTMLElement & { showPopover: () => void; hidePopover: () => void }).showPopover = show
    ;(menu as HTMLElement & { hidePopover: () => void }).hidePopover = hide
    menu.getBoundingClientRect = () =>
      ({
        width: 120,
        height: 80,
        top: 0,
        left: 0,
        right: 120,
        bottom: 80,
        x: 0,
        y: 0,
        toJSON() {},
      }) as DOMRect

    const handle = contextMenu(menu) as { destroy: () => void }

    const canvas = document.createElement('div')
    canvas.id = 'grid-cm-target'
    canvas.textContent = 'right-click me'
    document.body.append(canvas)
    await flushMo()

    canvas.dispatchEvent(
      new MouseEvent('contextmenu', { bubbles: true, clientX: 40, clientY: 50 }),
    )
    vi.runAllTimers()
    expect(show).toHaveBeenCalled()
    expect(menu.style.getPropertyValue('--anchor-left')).toBeTruthy()

    handle.destroy()
  })

  it('rebinds when for attribute is set after mount', async () => {
    vi.useFakeTimers()
    // Sibling layout (not body-parent): without `for`, default target is parent
    // of the menu — keep canvas outside that parent so we only open after rebind.
    const wrap = document.createElement('div')
    document.body.append(wrap)
    const menu = document.createElement('div')
    menu.setAttribute('popover', '')
    wrap.append(menu)
    const show = vi.fn()
    ;(menu as HTMLElement & { showPopover: () => void; hidePopover: () => void }).showPopover = show
    ;(menu as HTMLElement & { hidePopover: () => void }).hidePopover = () => {}
    menu.getBoundingClientRect = () =>
      ({
        width: 80,
        height: 40,
        top: 0,
        left: 0,
        right: 80,
        bottom: 40,
        x: 0,
        y: 0,
        toJSON() {},
      }) as DOMRect

    const handle = contextMenu(menu) as { destroy: () => void }
    const canvas = document.createElement('div')
    canvas.id = 'late-for'
    document.body.append(canvas)

    canvas.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, clientX: 10, clientY: 10 }))
    vi.runAllTimers()
    expect(show).not.toHaveBeenCalled()

    menu.setAttribute('for', 'late-for')
    await flushMo()
    canvas.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, clientX: 10, clientY: 10 }))
    vi.runAllTimers()
    expect(show).toHaveBeenCalled()

    handle.destroy()
  })
})

describe('use:combobox — picker-first open + filter + width', () => {
  function mountCb(opts?: { value?: string }) {
    const host = document.createElement('div')
    host.className = 'sail-combobox'
    if (opts?.value) host.setAttribute('value', opts.value)
    host.style.width = '260px'
    host.innerHTML = `
      <input type="text" />
      <div popover placement="bottom" class="listbox"></div>
    `
    const list = host.querySelector('.listbox') as HTMLElement
    for (const [value, label, disabled] of [
      ['apple', 'Apple', false],
      ['banana', 'Banana', false],
      ['mango', 'Mango', false],
      ['melon', 'Melon', true],
    ] as const) {
      const o = document.createElement('option')
      o.setAttribute('value', value)
      o.textContent = label
      if (disabled) o.setAttribute('disabled', '')
      list.append(o)
    }
    document.body.append(host)
    const input = host.querySelector('input') as HTMLInputElement
    // Popover open state for jsdom polyfill + :popover-open probe.
    let open = false
    list.matches = ((s: string) => (s === ':popover-open' ? open : false)) as typeof list.matches
    ;(list as HTMLElement & { showPopover: () => void }).showPopover = () => {
      if (open) return
      open = true
      list.dispatchEvent(
        new ToggleEvent('toggle', { newState: 'open', oldState: 'closed' }),
      )
    }
    ;(list as HTMLElement & { hidePopover: () => void }).hidePopover = () => {
      if (!open) return
      open = false
      list.dispatchEvent(
        new ToggleEvent('toggle', { newState: 'closed', oldState: 'open' }),
      )
    }
    const destroy = (combobox(host) as { destroy: () => void } | void)?.destroy
    return { host, input, list, destroy, isOpen: () => open }
  }

  it('focus opens the full list without typing', () => {
    const { input, list, destroy, isOpen } = mountCb()
    input.dispatchEvent(new FocusEvent('focus'))
    expect(isOpen()).toBe(true)
    expect(input.getAttribute('aria-expanded')).toBe('true')
    const visible = [...list.querySelectorAll('option')].filter((o) => !o.hidden)
    expect(visible.map((o) => o.getAttribute('value'))).toEqual([
      'apple',
      'banana',
      'mango',
      'melon',
    ])
    destroy?.()
  })

  it('pointerdown on the field opens the full list', () => {
    const { input, destroy, isOpen } = mountCb()
    input.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    expect(isOpen()).toBe(true)
    destroy?.()
  })

  it('typing filters by label; empty query restores all', () => {
    const { input, list, destroy } = mountCb()
    input.dispatchEvent(new FocusEvent('focus'))
    input.value = 'an'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    let visible = [...list.querySelectorAll('option')].filter((o) => !o.hidden)
    expect(visible.map((o) => o.getAttribute('value'))).toEqual(['banana', 'mango'])

    input.value = ''
    input.dispatchEvent(new Event('input', { bubbles: true }))
    visible = [...list.querySelectorAll('option')].filter((o) => !o.hidden)
    expect(visible).toHaveLength(4)
    destroy?.()
  })

  it('filters by option value as well as label', () => {
    const { input, list, destroy } = mountCb()
    input.value = 'ban'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    const visible = [...list.querySelectorAll('option')].filter((o) => !o.hidden)
    expect(visible.map((o) => o.getAttribute('value'))).toEqual(['banana'])
    destroy?.()
  })

  it('ArrowDown opens full list and activates first enabled option', () => {
    const { input, list, destroy, isOpen } = mountCb()
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(isOpen()).toBe(true)
    const active = list.querySelector('option[data-active]')
    expect(active?.getAttribute('value')).toBe('apple')
    destroy?.()
  })

  it('reopen after commit shows full list (selection is not a query)', () => {
    const { host, input, list, destroy, isOpen } = mountCb()
    input.dispatchEvent(new FocusEvent('focus'))
    const mango = list.querySelector('option[value=mango]') as HTMLElement
    mango.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    expect(host.getAttribute('value')).toBe('mango')
    expect(input.value).toBe('Mango')
    expect(isOpen()).toBe(false)

    input.dispatchEvent(new FocusEvent('focus'))
    expect(isOpen()).toBe(true)
    const visible = [...list.querySelectorAll('option')].filter((o) => !o.hidden)
    expect(visible).toHaveLength(4)
    expect(list.querySelector('option[data-active]')?.getAttribute('value')).toBe('mango')
    destroy?.()
  })

  it('use:anchor publishes --anchor-width from the trigger for list width match', () => {
    const { input, list, destroy } = mountCb()
    // Give the input a measurable width; anchor reads getBoundingClientRect.
    input.getBoundingClientRect = () =>
      ({
        width: 260,
        height: 36,
        left: 10,
        top: 20,
        right: 270,
        bottom: 56,
        x: 10,
        y: 20,
        toJSON() {},
      }) as DOMRect
    Object.defineProperty(list, 'offsetWidth', { value: 100, configurable: true })
    Object.defineProperty(list, 'offsetHeight', { value: 120, configurable: true })
    list.matches = ((s: string) => s === ':popover-open') as typeof list.matches

    const h = anchor(list, 'bottom') as { update: (p: string) => void; destroy: () => void }
    // Synchronous place on update (open + measured).
    h.update('bottom')
    expect(list.style.getPropertyValue('--anchor-width')).toBe('260px')
    // Product contract: listbox skin uses both w and minW from this var
    // (see combobox.sail + element-skins .s-wbindanchorWidth).
    h.destroy()
    destroy?.()
  })
})
