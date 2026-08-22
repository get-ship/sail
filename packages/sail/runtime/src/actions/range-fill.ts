import type { Action } from '../action'
import { writeCssVar } from '../bind'

/**
 * `use:rangeFill` — fill the WebKit track (`--rng-pct`) and, when `ticks`
 * is set, stamp labels at **thumb centers**.
 *
 * Geometry (the only reliable alignment):
 *   left = thumb/2 + t * (100% - thumb)
 * where t is 0…1 along the discrete range.
 * Docs must not padX a sibling row — that desyncs from thumb size.
 */
export const rangeFill: Action<void> = (node) => {
  const el = node as HTMLInputElement
  let tickRow: HTMLElement | null = null

  const thumbVar = (): string => {
    const raw = getComputedStyle(el).getPropertyValue('--rng-thumb').trim()
    return raw || '1rem'
  }

  const syncFill = (): void => {
    const min = parseFloat(el.min || '0')
    const max = parseFloat(el.max || '100')
    const val = parseFloat(el.value || '0')
    const pct = max > min ? ((val - min) / (max - min)) * 100 : 0
    writeCssVar(el, '--rng-pct', `${pct}%`)
  }

  const syncTicks = (): void => {
    const labels = (el.getAttribute('ticks') ?? '')
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
    if (!labels.length) {
      tickRow?.remove()
      tickRow = null
      return
    }
    if (!tickRow || !tickRow.isConnected) {
      tickRow = document.createElement('div')
      tickRow.setAttribute('data-slider-ticks', '')
      tickRow.setAttribute('aria-hidden', 'true')
      el.after(tickRow)
    }
    const thumb = thumbVar()
    writeCssVar(tickRow, 'position', 'relative')
    writeCssVar(tickRow, 'width', '100%')
    writeCssVar(tickRow, 'height', '1.15rem')
    writeCssVar(tickRow, 'margin-top', '0.15rem')
    writeCssVar(tickRow, 'pointer-events', 'none')
    writeCssVar(tickRow, '--rng-thumb', thumb)
    tickRow.replaceChildren(
      ...labels.map((label, i) => {
        const span = document.createElement('span')
        span.textContent = label
        const t = labels.length === 1 ? 0.5 : i / (labels.length - 1)
        writeCssVar(span, 'position', 'absolute')
        writeCssVar(span, 'top', '0')
        writeCssVar(span, 'left', `calc(var(--rng-thumb) / 2 + ${t} * (100% - var(--rng-thumb)))`)
        writeCssVar(span, 'transform', 'translateX(-50%)')
        writeCssVar(span, 'font-size', '0.75rem')
        writeCssVar(span, 'font-family', 'var(--font-mono, ui-monospace, monospace)')
        writeCssVar(span, 'color', 'var(--muted-foreground)')
        writeCssVar(span, 'white-space', 'nowrap')
        writeCssVar(span, 'line-height', '1.15rem')
        return span
      }),
    )
  }

  const sync = (): void => {
    syncFill()
    syncTicks()
  }

  sync()
  el.addEventListener('input', syncFill)
  const obs = new MutationObserver(sync)
  obs.observe(el, {
    attributes: true,
    attributeFilter: ['value', 'min', 'max', 'ticks', 'size'],
  })

  return {
    destroy: () => {
      el.removeEventListener('input', syncFill)
      obs.disconnect()
      tickRow?.remove()
      tickRow = null
    },
  }
}
