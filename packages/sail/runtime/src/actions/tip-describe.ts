import type { Action } from '../action'

let _seq = 0

type Placement = 'top' | 'bottom' | 'left' | 'right'

/**
 * Publish placement geometry as CSS custom properties the bubble sx reads
 * (`top-(--tt-top)`, `translate-x-(--tt-tx)`, arrow after:* insets). Absolute
 * positioning against the host (`relative`) — same visual as the old CSS
 * Anchor Positioning matrix, expressible without contracts PE.
 */
const applyPlacement = (el: HTMLElement, raw: string | null): void => {
  const p = (raw === 'bottom' || raw === 'left' || raw === 'right' ? raw : 'top') as Placement
  const gap = 'calc(100% + var(--space-2))'
  const arrow = 'calc(-1 * var(--space-1))'
  const shift = '-3.5px'

  // Reset box + arrow sides.
  for (const side of ['top', 'bottom', 'left', 'right'] as const) {
    el.style.setProperty(`--tt-${side}`, 'auto')
    el.style.setProperty(`--tt-arrow-${side}`, 'auto')
  }
  el.style.setProperty('--tt-tx', '0')
  el.style.setProperty('--sx-translate-y', '0')
  el.style.setProperty('--tt-arrow-ml', '0px')
  el.style.setProperty('--tt-arrow-mt', '0px')

  if (p === 'bottom') {
    el.style.setProperty('--tt-top', gap)
    el.style.setProperty('--tt-left', '50%')
    el.style.setProperty('--tt-tx', '-50%')
    el.style.setProperty('--tt-arrow-top', arrow)
    el.style.setProperty('--tt-arrow-left', '50%')
    el.style.setProperty('--tt-arrow-ml', shift)
  } else if (p === 'left') {
    el.style.setProperty('--tt-right', gap)
    el.style.setProperty('--tt-top', '50%')
    el.style.setProperty('--sx-translate-y', '-50%')
    el.style.setProperty('--tt-arrow-right', arrow)
    el.style.setProperty('--tt-arrow-top', '50%')
    el.style.setProperty('--tt-arrow-mt', shift)
  } else if (p === 'right') {
    el.style.setProperty('--tt-left', gap)
    el.style.setProperty('--tt-top', '50%')
    el.style.setProperty('--sx-translate-y', '-50%')
    el.style.setProperty('--tt-arrow-left', arrow)
    el.style.setProperty('--tt-arrow-top', '50%')
    el.style.setProperty('--tt-arrow-mt', shift)
  } else {
    // top (default)
    el.style.setProperty('--tt-bottom', gap)
    el.style.setProperty('--tt-left', '50%')
    el.style.setProperty('--tt-tx', '-50%')
    el.style.setProperty('--tt-arrow-bottom', arrow)
    el.style.setProperty('--tt-arrow-left', '50%')
    el.style.setProperty('--tt-arrow-ml', shift)
  }
}

/**
 * `use:tipDescribe` — the L1 `<tooltip>` element's behavior. Show/hide is pure
 * CSS (group-hover / group-focus-within opacity on the bubble). This action
 * supplies only what CSS can't alone:
 *  - placement geometry vars the bubble sx reads (box + arrow insets),
 *  - the bubble text (from the `content` attribute) + `aria-describedby`
 *    wiring from the trigger to the bubble,
 *  - Escape blurs the host to dismiss.
 * Written once in the toolchain — no per-instance controller, compiles away.
 */
export const tipDescribe: Action<void> = (node) => {
  const el = node as HTMLElement
  const n = ++_seq
  const bubbleId = `sail-tooltip-${n}`

  // Bubble is the host's tooltip part (data-tooltip-bubble; structure, not a class).
  // Closed action stamps role=tooltip — not free author markup.
  const bubble =
    (el.querySelector('[data-tooltip-bubble]') as HTMLElement | null) ??
    (el.querySelector('[role="tooltip"]') as HTMLElement | null)
  if (bubble && !bubble.hasAttribute('role')) bubble.setAttribute('role', 'tooltip')

  const sync = (): void => {
    applyPlacement(el, el.getAttribute('placement'))
    if (bubble) {
      if (!bubble.hasAttribute('role')) bubble.setAttribute('role', 'tooltip')
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
  obs.observe(el, { attributes: true, attributeFilter: ['content', 'placement'] })

  const onKeydown = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') el.blur()
  }
  el.addEventListener('keydown', onKeydown)

  return {
    destroy: () => {
      obs.disconnect()
      el.removeEventListener('keydown', onKeydown)
      for (const prop of [
        '--tt-top',
        '--tt-bottom',
        '--tt-left',
        '--tt-right',
        '--tt-tx',
        '--sx-translate-y',
        '--tt-arrow-top',
        '--tt-arrow-bottom',
        '--tt-arrow-left',
        '--tt-arrow-right',
        '--tt-arrow-ml',
        '--tt-arrow-mt',
      ]) {
        el.style.removeProperty(prop)
      }
    },
  }
}
