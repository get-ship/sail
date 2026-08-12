import type { Action } from '../action'

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
    el.style.setProperty('--rng-pct', `${pct}%`)
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
    tickRow.style.position = 'relative'
    tickRow.style.width = '100%'
    tickRow.style.height = '1.15rem'
    tickRow.style.marginTop = '0.15rem'
    tickRow.style.pointerEvents = 'none'
    tickRow.style.setProperty('--rng-thumb', thumb)
    tickRow.replaceChildren(
      ...labels.map((label, i) => {
        const span = document.createElement('span')
        span.textContent = label
        const t = labels.length === 1 ? 0.5 : i / (labels.length - 1)
        span.style.position = 'absolute'
        span.style.top = '0'
        span.style.left = `calc(var(--rng-thumb) / 2 + ${t} * (100% - var(--rng-thumb)))`
        span.style.transform = 'translateX(-50%)'
        span.style.fontSize = '0.75rem'
        span.style.fontFamily = 'var(--font-mono, ui-monospace, monospace)'
        span.style.color = 'var(--muted-foreground)'
        span.style.whiteSpace = 'nowrap'
        span.style.lineHeight = '1.15rem'
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
