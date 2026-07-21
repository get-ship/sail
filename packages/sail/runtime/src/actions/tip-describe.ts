import type { Action } from '../action'

let _seq = 0

/**
 * `use:tipDescribe` — the L1 `<tooltip>` element's behavior. Show/hide and
 * positioning are PURE CSS (hover/focus-within opacity + CSS Anchor
 * Positioning in the skin); this action supplies only what CSS can't:
 *  - a per-instance anchor name (`--sail-tooltip-N`) so multiple tooltips on a
 *    page don't collide on one shared anchor,
 *  - the bubble text (from the `content` attribute) + `aria-describedby` wiring
 *    from the trigger to the bubble,
 *  - Escape blurs the host to dismiss.
 * Written once in the toolchain — no per-instance controller, compiles away.
 */
export const tipDescribe: Action<void> = (node) => {
  const el = node as HTMLElement
  const n = ++_seq
  const bubbleId = `sail-tooltip-${n}`
  el.style.setProperty('--sail-tooltip-anchor', `--sail-tooltip-${n}`)

  const bubble = el.querySelector('.bubble') as HTMLElement | null

  const sync = (): void => {
    if (bubble) {
      bubble.id = bubbleId
      bubble.textContent = el.getAttribute('content') ?? ''
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
  obs.observe(el, { attributes: true, attributeFilter: ['content'] })

  const onKeydown = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') el.blur()
  }
  el.addEventListener('keydown', onKeydown)

  return {
    destroy: () => {
      obs.disconnect()
      el.removeEventListener('keydown', onKeydown)
    },
  }
}
