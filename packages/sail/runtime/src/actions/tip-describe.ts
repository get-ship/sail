import type { Action } from '../action'

let _seq = 0

/**
 * `use:tipDescribe` — L1 `<tooltip>` behavior without a geometry mailbox.
 * Show is CSS (group-hover / group-focus-within). Hide is `data-hidden`
 * on the bubble (Style). Content + aria-describedby + Escape stay here.
 */
export const tipDescribe: Action<void> = (node) => {
  const el = node as HTMLElement
  const n = ++_seq
  const bubbleId = `sail-tooltip-${n}`

  const bubble =
    (el.querySelector('[data-tooltip-bubble]') as HTMLElement | null) ??
    (el.querySelector('[role="tooltip"]') as HTMLElement | null)
  if (bubble && !bubble.hasAttribute('role')) bubble.setAttribute('role', 'tooltip')

  const forceHide = (): void => {
    bubble?.setAttribute('data-hidden', 'true')
  }

  const clearForceHide = (): void => {
    bubble?.removeAttribute('data-hidden')
  }

  const sync = (): void => {
    const rawPlacement = el.getAttribute('placement')
    if (bubble) {
      if (!bubble.hasAttribute('role')) bubble.setAttribute('role', 'tooltip')
      bubble.id = bubbleId
      bubble.textContent = el.getAttribute('content') ?? ''
      const p =
        rawPlacement === 'bottom' || rawPlacement === 'left' || rawPlacement === 'right'
          ? rawPlacement
          : 'top'
      bubble.setAttribute('placement', p)
    }
    const trigger = Array.from(el.children).find((c) => c !== bubble) as
      | HTMLElement
      | undefined
    if (trigger) {
      const existing = trigger.getAttribute('aria-describedby') ?? ''
      if (!existing.split(/\s+/).filter(Boolean).includes(bubbleId)) {
        trigger.setAttribute(
          'aria-describedby',
          existing ? `${existing} ${bubbleId}` : bubbleId,
        )
      }
    }
  }
  sync()
  const obs = new MutationObserver(sync)
  obs.observe(el, { attributes: true, attributeFilter: ['content', 'placement'] })

  const onKeydown = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') {
      forceHide()
      el.blur()
    }
  }
  const onMouseLeave = (): void => forceHide()
  const onMouseEnter = (): void => clearForceHide()
  const onFocusOut = (e: FocusEvent): void => {
    const next = e.relatedTarget as Node | null
    if (next && el.contains(next)) return
    forceHide()
  }
  const onFocusIn = (): void => clearForceHide()

  el.addEventListener('keydown', onKeydown)
  el.addEventListener('mouseleave', onMouseLeave)
  el.addEventListener('mouseenter', onMouseEnter)
  el.addEventListener('focusout', onFocusOut)
  el.addEventListener('focusin', onFocusIn)

  return {
    destroy: () => {
      obs.disconnect()
      el.removeEventListener('keydown', onKeydown)
      el.removeEventListener('mouseleave', onMouseLeave)
      el.removeEventListener('mouseenter', onMouseEnter)
      el.removeEventListener('focusout', onFocusOut)
      el.removeEventListener('focusin', onFocusIn)
      clearForceHide()
    },
  }
}
