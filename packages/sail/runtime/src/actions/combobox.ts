import type { Action } from '../action'

/**
 * Internal: L1 `<combobox>` closed behavior (not a public `use:`).
 * Authors use `<combobox>`, not `use:combobox`.
 *
 * The platform ALMOST has this: `<input list>` + `<datalist>` gives typeahead
 * with zero JS, but its popup is drawn by the UA and cannot be styled or themed
 * at all. That gap — and only that gap — is why this is a primitive.
 *
 * Everything the ARIA combobox pattern demands lives here, so the author writes a
 * tag and some options and nothing else: id generation, the `popovertarget`
 * wiring, aria-expanded / aria-controls / aria-activedescendant, filtering as you
 * type, Up/Down/Home/End roving, Enter to commit, Escape to close, click to
 * commit, and `sail-combobox-select` on commit. Open/close, light-dismiss and the
 * top layer come from the native popover; position comes from `use:anchor` — the
 * same two mechanisms the whole overlay family already runs on.
 *
 * Options are plain `<option>`s in the light DOM. The author's markup stays the
 * thing they'd write anyway.
 */

let uid = 0

export const combobox: Action<void> = (node) => {
  const root = node as HTMLElement
  const input = root.querySelector('input')
  const list = root.querySelector<HTMLElement & { showPopover?: () => void; hidePopover?: () => void }>(
    '[popover]',
  )
  if (!input || !list) return

  // Element mode lowers to STATIC structure, so the author's attributes ride on
  // the root, not on the inner input. Forward the ones that belong to the field —
  // same contract as use:anchor reading `placement` off its own element.
  const ph = root.getAttribute('placeholder')
  if (ph) input.placeholder = ph
  const val = root.getAttribute('value')
  if (val) {
    const match = [...root.querySelectorAll('option')].find((o) => o.getAttribute('value') === val)
    if (match) input.value = match.textContent?.trim() ?? ''
  }

  const id = `sail-cb-${++uid}`
  list.id ||= `${id}-list`
  input.id ||= `${id}-input`

  // The trigger is the input itself — the platform opens the popover for us.
  input.setAttribute('popovertarget', list.id)
  input.setAttribute('role', 'combobox')
  input.setAttribute('aria-controls', list.id)
  input.setAttribute('aria-autocomplete', 'list')
  input.setAttribute('aria-expanded', 'false')
  if (!input.hasAttribute('autocomplete')) input.setAttribute('autocomplete', 'off')
  list.setAttribute('role', 'listbox')

  const options = (): HTMLElement[] => [...list.querySelectorAll<HTMLElement>('option')]
  const shown = (): HTMLElement[] => options().filter((o) => !o.hidden && !o.hasAttribute('disabled'))

  options().forEach((o, i) => {
    o.id ||= `${id}-opt-${i}`
    o.setAttribute('role', 'option')
    o.setAttribute('aria-selected', 'false')
  })

  let active: HTMLElement | null = null

  const setActive = (o: HTMLElement | null): void => {
    if (active) {
      active.removeAttribute('data-active')
      active.setAttribute('aria-selected', 'false')
    }
    active = o
    if (o) {
      o.setAttribute('data-active', '')
      o.setAttribute('aria-selected', 'true')
      input.setAttribute('aria-activedescendant', o.id)
      o.scrollIntoView({ block: 'nearest' })
    } else {
      input.removeAttribute('aria-activedescendant')
    }
  }

  const isOpen = (): boolean => list.matches(':popover-open')

  const open = (): void => {
    if (isOpen()) return
    try {
      list.showPopover?.()
    } catch {
      /* no popover support */
    }
  }
  const close = (): void => {
    if (!isOpen()) return
    try {
      list.hidePopover?.()
    } catch {
      /* already closed */
    }
  }

  // Filter as you type. An empty query shows everything — the listbox is a
  // picker first and a search second.
  const filter = (): void => {
    const q = input.value.trim().toLowerCase()
    for (const o of options()) {
      const text = (o.textContent ?? '').toLowerCase()
      o.hidden = q !== '' && !text.includes(q)
    }
    const vis = shown()
    if (!vis.length) {
      close()
      setActive(null)
      return
    }
    if (!active || active.hidden) setActive(vis[0] ?? null)
    open()
  }

  const commit = (o: HTMLElement | null): void => {
    if (!o) return
    const value = o.getAttribute('value') ?? o.textContent ?? ''
    input.value = o.textContent?.trim() ?? ''
    root.setAttribute('value', value)
    // The committed label is a SELECTION, not a query. Leaving the old filter in
    // place would mean reopening the picker shows only whatever the last search
    // happened to match — so drop it, and reopen on the full list with the
    // selection as the cursor. (An ABANDONED query is different: light-dismiss
    // leaves the text in the field, so its filter correctly survives.)
    for (const opt of options()) opt.hidden = false
    close()
    root.dispatchEvent(
      new CustomEvent('sail-combobox-select', {
        detail: { value, label: input.value },
        bubbles: true,
        composed: true,
      }),
    )
  }

  const move = (delta: number): void => {
    const vis = shown()
    if (!vis.length) return
    if (!isOpen()) open()
    const i = active ? vis.indexOf(active) : -1
    const next = (i + delta + vis.length) % vis.length
    setActive(vis[next] ?? null)
  }

  const onInput = (): void => filter()

  const onKeydown = (e: KeyboardEvent): void => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        isOpen() ? move(1) : (open(), setActive(shown()[0] ?? null))
        break
      case 'ArrowUp':
        e.preventDefault()
        move(-1)
        break
      case 'Home':
        if (isOpen()) {
          e.preventDefault()
          setActive(shown()[0] ?? null)
        }
        break
      case 'End':
        if (isOpen()) {
          e.preventDefault()
          const vis = shown()
          setActive(vis[vis.length - 1] ?? null)
        }
        break
      case 'Enter':
        if (isOpen() && active) {
          e.preventDefault()
          commit(active)
        }
        break
      case 'Escape':
        if (isOpen()) {
          e.preventDefault()
          close()
        }
        break
      default:
        break
    }
  }

  const onPointerdown = (e: PointerEvent): void => {
    const o = (e.target as Element | null)?.closest('option') as HTMLElement | null
    // pointerdown, not click: the input must not lose focus to the option first,
    // and the native popover's light-dismiss must not race us.
    if (o && !o.hasAttribute('disabled')) {
      e.preventDefault()
      commit(o)
    }
  }

  // The popover owns open/close (trigger, Escape, light-dismiss) — we only mirror
  // its state into ARIA, so there is exactly one source of truth for "is it open".
  const onToggle = (e: Event): void => {
    const open_ = (e as ToggleEvent).newState === 'open'
    input.setAttribute('aria-expanded', String(open_))
    if (!open_) return setActive(null)
    if (active) return
    // Reopening lands the cursor on the current selection when there is one —
    // otherwise on the first option.
    const sel = root.getAttribute('value')
    const cur = sel ? (options().find((o) => o.getAttribute('value') === sel) ?? null) : null
    setActive(cur && !cur.hidden ? cur : (shown()[0] ?? null))
  }

  input.addEventListener('input', onInput)
  input.addEventListener('keydown', onKeydown)
  list.addEventListener('pointerdown', onPointerdown as EventListener)
  list.addEventListener('toggle', onToggle)

  return {
    destroy: () => {
      input.removeEventListener('input', onInput)
      input.removeEventListener('keydown', onKeydown)
      list.removeEventListener('pointerdown', onPointerdown as EventListener)
      list.removeEventListener('toggle', onToggle)
    },
  }
}
