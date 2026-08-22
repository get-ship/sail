import type { Action } from '../action'
import { writeCssVar } from '../bind'

let _seq = 0

type Placement = 'top' | 'bottom' | 'left' | 'right'

/**
 * Publish placement geometry as CSS custom properties the bubble sx reads
 * (`top-(--tt-top)`, `translate-x-(--tt-tx)`, arrow after:* insets). Absolute
 * positioning against the host (`relative`) — same visual as the old CSS
 * Anchor Positioning matrix, expressible without contracts host CSS.
 */
const applyPlacement = (el: HTMLElement, raw: string | null): void => {
  const p = (raw === 'bottom' || raw === 'left' || raw === 'right' ? raw : 'top') as Placement
  const gap = 'calc(100% + var(--space-2))'
  // Border-triangle caret (border*Width(4) = 4px ≈ space-1): tuck base under
  // the bubble edge so the filled edge meets the fill (no diamond double seam).
  const arrow = 'calc(-1 * var(--space-1) + 1px)'
  // Half of triangle base (2 × 4px) for left/top 50% centering.
  const shift = 'calc(-1 * var(--space-1))'

  // Reset box + arrow sides.
  for (const side of ['top', 'bottom', 'left', 'right'] as const) {
    writeCssVar(el, `--tt-${side}`, 'auto')
    writeCssVar(el, `--tt-arrow-${side}`, 'auto')
  }
  writeCssVar(el, '--tt-tx', '0')
  writeCssVar(el, '--sx-translate-y', '0')
  writeCssVar(el, '--tt-arrow-ml', '0px')
  writeCssVar(el, '--tt-arrow-mt', '0px')

  if (p === 'bottom') {
    writeCssVar(el, '--tt-top', gap)
    writeCssVar(el, '--tt-left', '50%')
    writeCssVar(el, '--tt-tx', '-50%')
    writeCssVar(el, '--tt-arrow-top', arrow)
    writeCssVar(el, '--tt-arrow-left', '50%')
    writeCssVar(el, '--tt-arrow-ml', shift)
  } else if (p === 'left') {
    writeCssVar(el, '--tt-right', gap)
    writeCssVar(el, '--tt-top', '50%')
    writeCssVar(el, '--sx-translate-y', '-50%')
    writeCssVar(el, '--tt-arrow-right', arrow)
    writeCssVar(el, '--tt-arrow-top', '50%')
    writeCssVar(el, '--tt-arrow-mt', shift)
  } else if (p === 'right') {
    writeCssVar(el, '--tt-left', gap)
    writeCssVar(el, '--tt-top', '50%')
    writeCssVar(el, '--sx-translate-y', '-50%')
    writeCssVar(el, '--tt-arrow-left', arrow)
    writeCssVar(el, '--tt-arrow-top', '50%')
    writeCssVar(el, '--tt-arrow-mt', shift)
  } else {
    // top (default)
    writeCssVar(el, '--tt-bottom', gap)
    writeCssVar(el, '--tt-left', '50%')
    writeCssVar(el, '--tt-tx', '-50%')
    writeCssVar(el, '--tt-arrow-bottom', arrow)
    writeCssVar(el, '--tt-arrow-left', '50%')
    writeCssVar(el, '--tt-arrow-ml', shift)
  }
}

/**
 * `use:tipDescribe` — the L1 `<tooltip>` element's behavior.
 * Show is CSS (group-hover / group-focus-within opacity on the bubble).
 * Hide is forced snappy: bubble Style has no opacity transition, and this
 * action clears residual open paint on mouseleave / focusout / Escape.
 * Also supplies:
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

  /** Instant hide — no linger after pointer leaves the host group. */
  const forceHide = (): void => {
    if (!bubble) return
    writeCssVar(bubble, 'transition', 'none')
    writeCssVar(bubble, 'opacity', '0')
  }

  /** Clear forced hide so CSS group-hover / focus-within can show again. */
  const clearForceHide = (): void => {
    if (!bubble) return
    writeCssVar(bubble, 'transition', null)
    writeCssVar(bubble, 'opacity', null)
  }

  const sync = (): void => {
    const rawPlacement = el.getAttribute('placement')
    applyPlacement(el, rawPlacement)
    if (bubble) {
      if (!bubble.hasAttribute('role')) bubble.setAttribute('role', 'tooltip')
      bubble.id = bubbleId
      bubble.textContent = el.getAttribute('content') ?? ''
      // Border-triangle caret match lives on the bubble node — mirror placement.
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
        writeCssVar(el, prop, null)
      }
    },
  }
}
