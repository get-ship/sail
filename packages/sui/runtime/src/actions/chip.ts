import type { Action } from '../action'

/**
 * `use:chip` — the L1 `<chip>` element's behavior. The chip IS the interactive
 * surface. Visuals (tone/variant/size/selected/disabled) are pure CSS; this
 * action supplies the interaction:
 *  - role=button + tabindex; aria-pressed (from [selected]) and aria-disabled
 *    (from [disabled]) kept in sync, tabindex dropped to -1 while disabled;
 *  - keyboard: Enter/Space activate (host.click()); Backspace/Delete on a
 *    [removable] chip fire `sui-remove`;
 *  - the × button's click fires `sui-remove` (and stops the activate).
 * Written once in the toolchain — no per-instance controller, compiles away.
 */
export const chip: Action<void> = (node) => {
  const el = node as HTMLElement

  if (!el.hasAttribute('role')) el.setAttribute('role', 'button')

  const sync = (): void => {
    const disabled = el.hasAttribute('disabled')
    el.setAttribute('aria-pressed', el.hasAttribute('selected') ? 'true' : 'false')
    if (disabled) {
      el.setAttribute('aria-disabled', 'true')
      el.setAttribute('tabindex', '-1')
    } else {
      el.removeAttribute('aria-disabled')
      if (el.getAttribute('tabindex') == null || el.getAttribute('tabindex') === '-1') {
        el.setAttribute('tabindex', '0')
      }
    }
  }
  sync()
  const obs = new MutationObserver(sync)
  obs.observe(el, { attributes: true, attributeFilter: ['selected', 'disabled'] })

  const emitRemove = (): void => {
    el.dispatchEvent(new CustomEvent('sui-remove', { detail: {}, bubbles: true, composed: true }))
  }

  const onKeydown = (e: KeyboardEvent): void => {
    if (el.hasAttribute('disabled')) return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      el.click()
    } else if (el.hasAttribute('removable') && (e.key === 'Backspace' || e.key === 'Delete')) {
      e.preventDefault()
      emitRemove()
    }
  }
  el.addEventListener('keydown', onKeydown)

  const remove = el.querySelector('.remove') as HTMLElement | null
  const onRemoveClick = (e: MouseEvent): void => {
    e.stopPropagation()
    if (el.hasAttribute('disabled')) return
    emitRemove()
  }
  remove?.addEventListener('click', onRemoveClick)

  return {
    destroy: () => {
      obs.disconnect()
      el.removeEventListener('keydown', onKeydown)
      remove?.removeEventListener('click', onRemoveClick)
    },
  }
}
